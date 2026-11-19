//! The crawl: the job that turns what a closure says it is owed into
//! AniList requests, one node at a time, and writes the edges back.
//!
//! One crawl at a time, a rate limit parks a node rather than losing it, a
//! node whose fetch fails for any other reason is stamped as having no
//! edges (a benign failure and an empty list look the same from outside,
//! and nothing must ask again for ever), and a read only re-crawls a root
//! once a minute.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use rusqlite::{Connection, params};

use super::closure::{self, Closure};
use super::{as_i64, as_u64};
use crate::Metadata;
use crate::apply::{is_rate_limited, provider_unreachable};
use crate::fetch::message_of;
use crate::record::{self, StubWrite};
use anibeam_contract::*;
use anibeam_foundation::jobs::{Finished, JobCtx};
use anibeam_foundation::services::Services;
use anibeam_foundation::sync::recover;
use anibeam_foundation::time;
use anibeam_net::anilist::{Enrichment, RelatedNode};
use anibeam_net::images;

/// How long a rate-limited node sits out, and the floor under whatever
/// AniList asked for: a `Retry-After: 0` still costs a full minute, so a
/// provider saying stop is never answered with a burst.
const DEFERRAL: i64 = 60;

/// A franchise opened twice inside a minute is crawled once. The limiter
/// paces the requests either way; this stops repeated opens from queueing
/// the same component over and over.
const REFRESH_THROTTLE: Duration = Duration::from_secs(60);

/// Starts the Crawl job over these seeds, taking each seed again even
/// though its edges are already known, which is what a series page's own
/// read asks for: AniList may have added a relation since.
///
/// Crawl runs one at a time, so a second call while one is running is
/// handed the running job's id and starts nothing.
pub fn start(metadata: &Arc<Metadata>, seeds: Vec<u64>) -> u64 {
    let owner = metadata.clone();
    metadata
        .services
        .jobs
        .clone()
        .start(JobKind::Crawl, move |ctx| async move {
            run(&owner, &ctx, seeds, true).await
        })
}

/// Starts the Crawl job over every owned series whose node still owes its
/// edges. A seed whose component an earlier seed already closed no longer
/// owes anything by the time its turn comes, so it costs nothing.
pub fn start_gap_crawl(metadata: &Arc<Metadata>) -> u64 {
    let owner = metadata.clone();
    metadata
        .services
        .jobs
        .clone()
        .start(JobKind::Crawl, move |ctx| async move {
            let seeds = owner.services.store.write_async(gap_seeds).await?;
            run(&owner, &ctx, seeds, false).await
        })
}

/// The read path's crawl: a series page asks for its franchise, and this
/// is what goes and gets whatever is missing behind it. At most once a
/// minute per root, so opening the same page again changes nothing.
pub fn maybe_crawl_for_read(metadata: &Arc<Metadata>, seed: u64, root: u64) {
    let mut recent = recover(&metadata.crawl_recent);
    if !record_crawl(&mut recent, root, Instant::now()) {
        return;
    }
    // Dropped before the job starts: nothing the job does should be able
    // to wait on this gate.
    drop(recent);
    start(metadata, vec![seed]);
}

/// Whether this root is due a read-driven crawl, recording the instant when
/// it is. False means the last one was inside the minute.
fn record_crawl(recent: &mut HashMap<u64, Instant>, root: u64, at: Instant) -> bool {
    if recent
        .get(&root)
        .is_some_and(|last| at.duration_since(*last) < REFRESH_THROTTLE)
    {
        return false;
    }
    recent.insert(root, at);
    true
}

/// The body both entry points share. Every seed is closed, the closure says
/// what it is owed, one node is fetched, and the closure is taken again so
/// a member the fetch turned up joins the same walk.
async fn run(
    metadata: &Metadata,
    ctx: &Arc<JobCtx>,
    seeds: Vec<u64>,
    refetch_seeds: bool,
) -> Result<Finished, CoreError> {
    let (mut fetched, mut deferred) = (0u64, 0u64);
    // Every node this run has already parked. The database deferral is
    // what governs the next job, but reading it back is not enough to end
    // this one: a walk that takes a minute of backoff per node would find
    // the first deferral spent by the time it came round again, and two
    // owed nodes would hand the job back and forth for ever.
    let mut parked: HashSet<u64> = HashSet::new();
    // The nodes that answered with something other than a rate limit,
    // reported as one line at the end rather than one line each: a
    // transport outage across a 150 node walk is one state change, not
    // 150 of them.
    let (mut failed, mut first_failure) = (0u64, None::<String>);
    for seed in seeds {
        ctx.checkpoint()?;
        // Consumed on the first time round: the seed is taken again once,
        // and after that it is owed nothing, which is what ends the loop.
        let mut force = refetch_seeds;
        loop {
            ctx.checkpoint()?;
            let now = time::now_secs();
            let graph = metadata
                .services
                .store
                .write_async(move |c| closure::close(c, seed, closure::CAP, now))
                .await?;
            let next = if force {
                force = false;
                Some(seed)
            } else {
                next_to_fetch(&graph, &parked)
            };
            let Some(id) = next else { break };

            let (done, total) = metadata.services.store.write_async(progress).await?;
            ctx.progress(done, Some(total), &format!("AniList {id}"));

            let failure = match metadata.net.anilist.enrichment_raw(id).await {
                // The reply and the JSON behind it. The raw half is not
                // written: a crawl asks for the series page alone, and
                // that partial bundle must never displace the four-part
                // one a full fetch left in `raw`.
                Ok((Some(enrichment), _raw)) => {
                    write_node(&metadata.services, id, &enrichment, time::now_secs()).await?;
                    fetch_covers(metadata, &enrichment).await;
                    fetched += 1;
                    ctx.emit(
                        Level::Debug,
                        format!("franchise {} changed", graph.root),
                        EventBody::GraphChanged { root: graph.root },
                    );
                    continue;
                }
                // AniList answered, and the answer was that it carries no
                // entry. Nothing more will ever come of asking again.
                Ok((None, _raw)) => format!("AniList has no entry for {id}"),
                // AniList saying stop. The node keeps owing its edges and
                // is left alone for a minute rather than counted as done.
                // The terminal line's `deferred` is what says this
                // happened: one activity log line per parked node would be
                // per-node chatter in a log meant to carry state changes.
                Err(e) if is_rate_limited(&e) => {
                    let wait = retry_after(&e);
                    let until = time::now_secs() + wait;
                    mark(&metadata.services, id, DEFER, until).await?;
                    parked.insert(id);
                    deferred += 1;
                    tracing::debug!("the crawl deferred {id} for {wait}s after a rate limit");
                    continue;
                }
                // Nothing answered at all. The node never had its turn,
                // and stamping it here would mark it edgeless for good on
                // the strength of a dead socket, so the crawl ends instead
                // and the next one starts where this stopped.
                Err(e) if provider_unreachable(&e) => return Err(e),
                // Anything else had its turn.
                Err(e) => message_of(&e),
            };
            // A node that had its turn and produced nothing is stamped with
            // no edges, which is exactly what a node that genuinely has
            // none looks like, so the walk moves on instead of circling;
            // the next full refresh is the recovery.
            tracing::debug!("the crawl stamped {id} with no edges: {failure}");
            failed += 1;
            first_failure.get_or_insert(failure);
            mark(&metadata.services, id, STAMP, time::now_secs()).await?;
        }
    }

    images::sweep_after(&metadata.net, "a crawl").await;

    // One line for every node that failed, ahead of the terminal event, so
    // an outage says so once and names what it looked like.
    if let Some(first) = first_failure {
        ctx.emit(
            Level::Warn,
            format!("Crawl: {failed} nodes failed, first: {first}"),
            EventBody::Notice,
        );
    }

    Ok(Finished {
        plain: None,
        level: Level::Info,
        message: format!("Crawl finished: {fetched} fetched, {deferred} deferred"),
        body: EventBody::CrawlFinished { fetched, deferred },
    })
}

/// The next node worth a request: a member whose edges are not in the
/// table, which is not sitting out a rate limit, and which this run has not
/// already parked. A boundary node is never one: the walk stopped at it.
///
/// `parked` is why the walk ends. The column alone is not enough: with the
/// real backoff schedule a node takes about a minute to give up, so by the
/// time a long walk came round again the first deferral would have expired
/// and the same two nodes would trade places for ever.
fn next_to_fetch(graph: &Closure, parked: &HashSet<u64>) -> Option<u64> {
    graph
        .nodes
        .iter()
        .find(|n| {
            graph.members.contains(&n.anilist_id)
                && !n.relations_fetched
                && n.deferred_until.is_none()
                && !parked.contains(&n.anilist_id)
        })
        .map(|n| n.anilist_id)
}

/// How long the node waits. AniList's own number when it sent one, never
/// less than a minute either way. The upper clamp is a cast guard rather
/// than a policy: `Retry-After` is a header a provider writes, and a
/// nonsense value must not become a nonsense instant.
fn retry_after(e: &CoreError) -> i64 {
    let asked = match e {
        CoreError::Provider {
            retry_after: Some(seconds),
            ..
        } => *seconds,
        _ => 0.0,
    };
    let seconds = asked.clamp(0.0, f64::from(u32::MAX));
    (seconds.ceil() as i64).max(DEFERRAL)
}

/// One node's edges, in one transaction. The node's own row is filled in
/// from the reply (blanks filled, values kept, `fetched_at` untouched: a
/// crawl is not a metadata fetch), its outgoing edges are replaced, and
/// every neighbour gets the stub row the edge needs to point at.
async fn write_node(
    services: &Services,
    id: u64,
    enrichment: &Enrichment,
    now: i64,
) -> Result<(), CoreError> {
    let own = own_stub(id, enrichment);
    let edges: Vec<(String, RelatedNode)> = enrichment
        .relations
        .as_ref()
        .map(|r| {
            r.edges
                .iter()
                .filter(|e| e.node.id != 0)
                .map(|e| (e.relation_type.clone(), e.node.clone()))
                .collect()
        })
        .unwrap_or_default();
    services.store
        .tx_async(move |tx| {
            record::write_stub(tx, &own)?;
            tx.execute("DELETE FROM relations WHERE from_id = ?1", params![as_i64(id)])?;
            for (relation, node) in &edges {
                if node.id != id {
                    record::write_stub(tx, &record::stub_from_node(node))?;
                }
                tx.execute(
                    "INSERT OR REPLACE INTO relations (from_id, to_id, relation) VALUES (?1, ?2, ?3)",
                    params![as_i64(id), as_i64(node.id), relation],
                )?;
            }
            tx.execute(STAMP, params![as_i64(id), now])?;
            Ok(())
        })
        .await
}

/// What the series page reply says about the node itself. The same shape a
/// relation's other end writes, so one upsert rule covers both.
fn own_stub(id: u64, e: &Enrichment) -> StubWrite {
    StubWrite {
        id,
        mal_id: e.id_mal,
        media_type: e.type_.clone(),
        title_romaji: e.title.as_ref().and_then(|t| t.romaji.clone()),
        title_english: e.title.as_ref().and_then(|t| t.english.clone()),
        title_native: e.title.as_ref().and_then(|t| t.native.clone()),
        format: e.format.clone(),
        status: e.status.clone(),
        year: e
            .season_year
            .or_else(|| e.start_date.as_ref().and_then(|d| d.year)),
        cover_url: e.cover_image.as_ref().and_then(|c| c.large.clone()),
        site_url: e.site_url.clone(),
        episodes: None,
        average_score: None,
    }
}

/// The node's poster and its neighbours', so the graph draws with pictures
/// rather than gaps. The neighbours matter as much as the node: the
/// library's own image fill only ever reaches a cover some series owns,
/// and a boundary node owns none, so a stub the crawl wrote would draw
/// blank for ever. A failure is that one url's own and is bookkeeping,
/// not a state change.
async fn fetch_covers(metadata: &Metadata, enrichment: &Enrichment) {
    let mut urls: Vec<String> = enrichment
        .cover_image
        .as_ref()
        .and_then(|c| c.large.clone())
        .into_iter()
        .collect();
    for edge in enrichment.relations.iter().flat_map(|r| r.edges.iter()) {
        if let Some(url) = edge.node.cover_image.as_ref().and_then(|c| c.large.clone()) {
            urls.push(url);
        }
    }
    urls.sort();
    urls.dedup();
    if urls.is_empty() {
        return;
    }
    for (url, outcome) in metadata.net.images.ensure(&urls).await {
        if let Err(e) = outcome {
            tracing::debug!("the crawl could not fetch {url}: {e}");
        }
    }
}

/// The node's edges are in: stamped at `?2`, and any deferral cleared.
const STAMP: &str =
    "UPDATE anilist_media SET relations_fetched_at = ?2, crawl_deferred_until = NULL WHERE id = ?1";

/// The node is rate limited: it keeps owing its edges and is left out of
/// the fetch list until `?2` passes.
const DEFER: &str = "UPDATE anilist_media SET crawl_deferred_until = ?2 WHERE id = ?1";

/// One of the two updates above on a node with nothing else to write. The
/// bare stub goes first because a seed can be an id with no media row at
/// all, and an `UPDATE` that matches nothing would leave the node owing its
/// edges for ever, which is a loop rather than a missing stamp.
async fn mark(services: &Services, id: u64, sql: &'static str, at: i64) -> Result<(), CoreError> {
    services
        .store
        .tx_async(move |tx| {
            record::write_stub(
                tx,
                &StubWrite {
                    id,
                    ..StubWrite::default()
                },
            )?;
            tx.execute(sql, params![as_i64(id), at])?;
            Ok(())
        })
        .await
}

/// Every owned series whose node still owes its edges, smallest id first.
fn gap_seeds(conn: &mut Connection) -> Result<Vec<u64>, CoreError> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT s.anilist_id FROM series s JOIN anilist_media m ON m.id = s.anilist_id
         WHERE m.relations_fetched_at IS NULL
         ORDER BY s.anilist_id",
    )?;
    let rows = stmt.query_map([], |r| Ok(as_u64(r.get::<_, i64>(0)?)))?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

/// The crawl bar's two numbers: how many owned AniList-matched series have
/// their node's edges, out of how many there are.
fn progress(conn: &mut Connection) -> Result<(u64, u64), CoreError> {
    let (done, total): (i64, i64) = conn.query_row(
        "SELECT count(m.relations_fetched_at), count(*)
         FROM series s LEFT JOIN anilist_media m ON m.id = s.anilist_id
         WHERE s.anilist_id IS NOT NULL",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    Ok((as_u64(done), as_u64(total)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::franchise::closure::Node;
    use anibeam_contract::Provider;

    fn owing(id: u64) -> Node {
        Node {
            anilist_id: id,
            ..Node::default()
        }
    }

    /// The three reasons a node is not the next request: it is a boundary
    /// node, its edges are already known, or it is out of reach for now,
    /// whether the column says so or this run already parked it.
    #[test]
    fn the_fetch_list_skips_a_node_this_run_has_already_parked() {
        let members: HashSet<u64> = [1, 2, 3].into_iter().collect();
        let graph = Closure {
            root: 1,
            nodes: vec![
                Node {
                    relations_fetched: true,
                    ..owing(1)
                },
                Node {
                    deferred_until: Some(9_999),
                    ..owing(2)
                },
                owing(3),
                owing(50),
            ],
            edges: Vec::new(),
            members,
            complete: false,
            owed: vec![2, 3],
        };

        assert_eq!(
            next_to_fetch(&graph, &HashSet::new()),
            Some(3),
            "1 is done, 2 is deferred, 50 is a boundary node"
        );

        // The same closure a moment later, with node 3's deferral written
        // but already expired: the column no longer holds it back, and only
        // the run's own record of having parked it does.
        let parked: HashSet<u64> = [3].into_iter().collect();
        assert_eq!(next_to_fetch(&graph, &parked), None);
    }

    /// A read crawls a root at most once a minute; another root has its
    /// own minute.
    #[test]
    fn a_read_crawls_one_root_at_most_once_a_minute() {
        let mut recent: HashMap<u64, Instant> = HashMap::new();
        let t0 = Instant::now();
        assert!(record_crawl(&mut recent, 1, t0));
        assert!(!record_crawl(&mut recent, 1, t0 + Duration::from_secs(59)));
        assert!(
            record_crawl(&mut recent, 2, t0),
            "another root has its own minute"
        );
        assert!(record_crawl(&mut recent, 1, t0 + Duration::from_secs(61)));
        assert!(
            !record_crawl(&mut recent, 1, t0 + Duration::from_secs(70)),
            "the minute runs from the last crawl"
        );
    }

    /// A minute is the floor, whatever AniList asked for, and its own
    /// number wins when it is longer.
    #[test]
    fn a_deferral_is_never_shorter_than_a_minute() {
        let limited = |seconds: Option<f64>| {
            CoreError::provider_retry(Provider::Anilist, Some(429), "rate limited", seconds)
        };
        assert_eq!(retry_after(&limited(None)), 60);
        assert_eq!(retry_after(&limited(Some(0.0))), 60);
        assert_eq!(retry_after(&limited(Some(30.0))), 60);
        assert_eq!(retry_after(&limited(Some(90.5))), 91);
        assert_eq!(
            retry_after(&limited(Some(f64::NAN))),
            60,
            "a header that makes no sense costs the floor"
        );
    }
}
