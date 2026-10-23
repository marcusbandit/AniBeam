//! Refinement logic with no UI: token groups, query rebuilding, and one stateless refine
//! step. Ported from anirss's `refinement.py`.
//!
//! Nothing here prints, prompts or touches a terminal. `apply` takes the refetch (and the
//! optional enrich) as callbacks, so the caller decides how a feed is read.

use super::bestfit;
use super::titles::{RES_RE, poster_of, title_tokens};
use super::{BestfitConfig, Endpoint};
use crate::contract::{BestFit, CoreError, RefineOp, Release, ReleaseGroup};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::future::Future;
use std::pin::Pin;
use std::sync::LazyLock;

/// A boxed, sendable future, what the refetch and enrich callbacks hand back.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Reads a feed: the endpoint and the query to ask it.
pub type FetchFn<'f> = dyn for<'a> Fn(&'a Endpoint, &'a str) -> BoxFuture<'a, Result<Vec<Release>, CoreError>>
    + Send
    + Sync
    + 'f;

/// Fills in what a feed left out (seeders and the like) on freshly fetched releases.
pub type EnrichFn<'f> = dyn Fn(Vec<Release>) -> BoxFuture<'static, Vec<Release>> + Send + Sync + 'f;

// TODO(port): use endpoints::FIELD_RE once it is public there.
/// A query field: a quoted run (with its optional leading `-`) or a bare word.
static FIELD_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"-?"[^"]*"|\S+"#).expect("field pattern compiles"));

// TODO(port): use erai::metadata_hit once it is public there.
/// Does `token` match the item's structured erai fields? Resolution, category and
/// linktype by equality, infomkv by substring. False for items without erai metadata (all
/// fields empty), so nyaa and generic feeds are unaffected.
fn metadata_hit(item: &Release, token: &str) -> bool {
    if item.resolution.is_empty() && item.infomkv.is_empty() && item.linktype.is_empty() {
        return false;
    }
    let lc = token.to_lowercase();
    lc == item.resolution.to_lowercase()
        || item.infomkv.to_lowercase().contains(&lc)
        || lc == item.linktype.to_lowercase()
        || lc == item.category.to_lowercase()
}

/// What `auto_resolution` decided.
#[derive(Clone, Debug, PartialEq)]
pub struct AutoResolution {
    pub query: String,
    pub items: Vec<Release>,
    /// The `<n>p` appended, empty when nothing was.
    pub token: String,
    /// How many titles lacked a `<n>p`, which is why it was skipped.
    pub skipped_missing: usize,
}

/// If `query` lacks a `<n>p` token and every item has one, append the highest and keep
/// only the items at that resolution.
pub fn auto_resolution(query: &str, items: &[Release]) -> AutoResolution {
    let unchanged = |skipped_missing| AutoResolution {
        query: query.to_string(),
        items: items.to_vec(),
        token: String::new(),
        skipped_missing,
    };
    if items.is_empty() || RES_RE.is_match(query) {
        return unchanged(0);
    }
    let per_max: Vec<Option<i64>> = items
        .iter()
        .map(|item| {
            RES_RE
                .captures_iter(&item.title)
                .filter_map(|c| c[1].parse::<i64>().ok())
                .max()
        })
        .collect();
    let missing = per_max.iter().filter(|res| res.is_none()).count();
    if missing > 0 {
        return unchanged(missing);
    }
    let highest = per_max.iter().flatten().copied().max().unwrap_or(0);
    let token = format!("{highest}p");
    let kept = items
        .iter()
        .zip(&per_max)
        .filter(|(_, res)| **res == Some(highest))
        .map(|(item, _)| item.clone())
        .collect();
    AutoResolution {
        query: format!("{query} {token}"),
        items: kept,
        token,
        skipped_missing: 0,
    }
}

/// A release's poster and tokens, read once.
struct Parsed {
    poster: Option<String>,
    tokens: Vec<String>,
}

fn parse(item: &Release) -> Parsed {
    Parsed {
        poster: poster_of(&item.title),
        tokens: title_tokens(&item.title),
    }
}

fn members_of(selected: &[Release], parsed: &[Parsed], token: &str) -> Vec<usize> {
    let is_poster = token.starts_with('[');
    (0..selected.len())
        .filter(|&i| {
            if is_poster {
                parsed[i].poster.as_deref() == Some(token)
            } else {
                parsed[i].tokens.iter().any(|t| t == token) || metadata_hit(&selected[i], token)
            }
        })
        .collect()
}

fn matches_all_tokens(item: &Release, tokens: &[String]) -> bool {
    let poster = poster_of(&item.title);
    let toks = title_tokens(&item.title);
    tokens.iter().all(|t| {
        if t.starts_with('[') {
            poster.as_deref() == Some(t.as_str())
        } else {
            toks.contains(t) || metadata_hit(item, t)
        }
    })
}

/// Custom-term match: title substring, plus erai's structured fields (the polished title
/// drops the linktype, so "Torrent" only exists there; items without erai metadata match
/// on the title alone).
fn custom_hit(item: &Release, term_lc: &str) -> bool {
    item.title.to_lowercase().contains(term_lc) || metadata_hit(item, term_lc)
}

/// Find tokens that partition `selected` into refinable subsets, grouped by membership.
pub fn compute_groups(selected: &[Release]) -> Vec<ReleaseGroup> {
    let total = selected.len();
    let parsed: Vec<Parsed> = selected.iter().map(parse).collect();
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for p in &parsed {
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        if let Some(poster) = &p.poster {
            seen.insert(poster);
        }
        for token in &p.tokens {
            seen.insert(token);
        }
        for token in seen {
            *counts.entry(token).or_insert(0) += 1;
        }
    }

    let refinable = counts
        .iter()
        .filter(|&(_, &count)| 1 < count && count < total)
        .map(|(t, _)| *t);

    let mut slot: HashMap<Vec<usize>, usize> = HashMap::new();
    let mut by_members: Vec<(Vec<usize>, Vec<&str>)> = Vec::new();
    for token in refinable {
        let members = members_of(selected, &parsed, token);
        match slot.get(&members) {
            Some(&at) => by_members[at].1.push(token),
            None => {
                slot.insert(members.clone(), by_members.len());
                by_members.push((members, vec![token]));
            }
        }
    }

    let mut groups: Vec<ReleaseGroup> = by_members
        .into_iter()
        .map(|(members, tokens)| {
            let mut tokens_sorted: Vec<String> = tokens.iter().map(|t| t.to_string()).collect();
            tokens_sorted.sort_by(|a, b| {
                (!a.starts_with('['), std::cmp::Reverse(a.chars().count()), a).cmp(&(
                    !b.starts_with('['),
                    std::cmp::Reverse(b.chars().count()),
                    b,
                ))
            });
            let has_poster = tokens_sorted.iter().any(|t| t.starts_with('['));
            ReleaseGroup {
                label: tokens_sorted.join("+"),
                member_count: members.len() as u64,
                has_poster,
                members: members.iter().map(|&i| i as u64).collect(),
                tokens: tokens_sorted,
            }
        })
        .collect();

    groups.sort_by(|a, b| {
        (!a.has_poster, std::cmp::Reverse(a.member_count), &a.label).cmp(&(
            !b.has_poster,
            std::cmp::Reverse(b.member_count),
            &b.label,
        ))
    });
    groups
}

/// The query the best-fit step would refetch with, or "" when running it would be a no-op
/// (nothing to rank, empty rebuild, or the current query already matching the top
/// release's profile).
fn best_fit_query_for(query: &str, selected: &[Release], cfg: &BestfitConfig) -> String {
    let Some(best) = bestfit::best_item(selected, cfg) else {
        return String::new();
    };
    let new_query = bestfit::best_fit_query(best);
    if new_query.is_empty() || new_query.to_lowercase() == query.to_lowercase() {
        return String::new();
    }
    new_query
}

/// The matchable core of a query field: drop a leading `-` and any quotes.
fn field_text(field: &str) -> String {
    let core = field.strip_prefix('-').unwrap_or(field);
    let chars: Vec<char> = core.chars().collect();
    let core = if chars.len() >= 2 && chars[0] == '"' && chars[chars.len() - 1] == '"' {
        chars[1..chars.len() - 1].iter().collect::<String>()
    } else {
        core.to_string()
    };
    core.to_lowercase()
}

/// Median index (in characters) at which `text` appears across the selected titles.
///
/// Infinite when no title contains it. Using the median (rather than one sample) keeps the
/// ordering stable when release groups format titles differently. An even count gives the
/// mean of the two middle values, as Python's `statistics.median` does.
fn title_position(text: &str, selected: &[Release]) -> f64 {
    let mut spots: Vec<usize> = Vec::new();
    if !text.is_empty() {
        for item in selected {
            let lowered = item.title.to_lowercase();
            if let Some(at) = lowered.find(text) {
                spots.push(lowered[..at].chars().count());
            }
        }
    }
    if spots.is_empty() {
        return f64::INFINITY;
    }
    spots.sort_unstable();
    let mid = spots.len() / 2;
    if spots.len() % 2 == 1 {
        spots[mid] as f64
    } else {
        (spots[mid - 1] + spots[mid]) as f64 / 2.0
    }
}

/// Slot `token` into `query` ahead of the first existing field that occurs later in the
/// titles, so the query mirrors real release-name order (1080p before multisub) instead
/// of being blindly appended. Posters are never crossed; exclusions (`-tag`) always stay
/// at the tail.
fn insert_positional(query: &str, token: &str, selected: &[Release]) -> String {
    let mut fields: Vec<&str> = FIELD_RE.find_iter(query).map(|m| m.as_str()).collect();
    let token_pos = title_position(&token.to_lowercase(), selected);
    let mut insert_at = fields.len();
    for (i, field) in fields.iter().enumerate() {
        if field.starts_with('[') {
            continue; // never slot in front of a poster
        }
        if field.starts_with('-') {
            insert_at = i; // exclusions stay at the end
            break;
        }
        if title_position(&field_text(field), selected) > token_pos {
            insert_at = i;
            break;
        }
    }
    fields.insert(insert_at, token);
    fields.join(" ")
}

/// Extend `query` with `tokens`, each placed where it belongs.
///
/// A poster (`[...]`) leads the title, so it goes to the front unless the query already
/// starts with one. Every other token is positioned by where it occurs in the titles. The
/// caller refetches nyaa with the rebuilt query rather than filtering locally, so
/// episodes beyond the first RSS page can surface.
pub fn build_refined_query(query: &str, tokens: &[String], selected: &[Release]) -> String {
    let mut new_query = query.to_string();
    for token in tokens {
        if token.starts_with('[') {
            if !new_query.trim_start().starts_with('[') {
                new_query = format!("{token} {new_query}");
            }
            continue;
        }
        new_query = insert_positional(&new_query, token, selected);
    }
    new_query
}

/// Append a nyaa-style `-tag` to `query`. The query comes back unchanged when the term is
/// empty after stripping.
pub fn add_exclude_to_query(query: &str, term: &str) -> String {
    let term = term.trim().trim_start_matches('-').trim();
    if term.is_empty() {
        return query.to_string();
    }
    let flag = if term.contains(' ') {
        format!("-\"{term}\"")
    } else {
        format!("-{term}")
    };
    format!("{query} {flag}")
}

/// Append a positive search term to `query`, quoting it if it contains spaces.
pub fn add_term_to_query(query: &str, term: &str) -> String {
    let term = term.trim();
    if term.is_empty() {
        return query.to_string();
    }
    let flag = if term.contains(' ') {
        format!("\"{term}\"")
    } else {
        term.to_string()
    };
    format!("{query} {flag}")
}

// One stateless refine step.

/// Generic RSS endpoints (Erai-raws, AniRena, and the like) are word-AND over post text,
/// so refinements filter the fetched items locally; nyaa refetches server-side so a
/// narrower query can surface older episodes.
pub fn is_client_side(ep: &Endpoint) -> bool {
    ep.kind != "nyaa"
}

/// Result of `apply`. When `applied` is false, `query` and `items` are the inputs
/// unchanged and `reason` says why: `noop`, `zero_results`, `no_match` or `empty_term`.
#[derive(Clone, Debug, PartialEq)]
pub struct Outcome {
    pub applied: bool,
    pub query: String,
    pub items: Vec<Release>,
    /// `refetch` or `client`.
    pub mode: String,
    pub previous_count: usize,
    pub reason: Option<String>,
}

impl Outcome {
    pub fn delta(&self) -> i64 {
        self.items.len() as i64 - self.previous_count as i64
    }
}

/// Ask the endpoint for `new_query`; None when it came back empty, else the releases,
/// enriched when a hook is given.
async fn refetch(
    ep: &Endpoint,
    new_query: &str,
    fetch: &FetchFn<'_>,
    enrich: Option<&EnrichFn<'_>>,
) -> Result<Option<Vec<Release>>, CoreError> {
    let fetched = fetch(ep, new_query).await?;
    if fetched.is_empty() {
        return Ok(None);
    }
    Ok(Some(match enrich {
        Some(enrich) => enrich(fetched).await,
        None => fetched,
    }))
}

/// Apply one refine action the way anirss's refine loop does.
///
/// `Tokens` narrows to a `compute_groups` group's tokens, `Term` is a custom substring,
/// `Exclude` drops a term, `BestFit` pins the top release's profile. nyaa endpoints
/// refetch with the rebuilt query; rss endpoints filter `items` client-side. A failed
/// refetch is the fetch callback's error.
pub async fn apply(
    ep: &Endpoint,
    query: &str,
    items: &[Release],
    op: &RefineOp,
    cfg: &BestfitConfig,
    fetch: &FetchFn<'_>,
    enrich: Option<&EnrichFn<'_>>,
) -> Result<Outcome, CoreError> {
    let prev = items.len();
    let client = is_client_side(ep);
    let mode = if client { "client" } else { "refetch" };

    let unchanged = |reason: &str| Outcome {
        applied: false,
        query: query.to_string(),
        items: items.to_vec(),
        mode: mode.to_string(),
        previous_count: prev,
        reason: Some(reason.to_string()),
    };
    let changed = |new_query: String, new_items: Vec<Release>| Outcome {
        applied: true,
        query: new_query,
        items: new_items,
        mode: mode.to_string(),
        previous_count: prev,
        reason: None,
    };

    match op {
        RefineOp::Tokens { tokens } => {
            let tokens: Vec<String> = tokens.iter().filter(|t| !t.is_empty()).cloned().collect();
            if tokens.is_empty() {
                return Err(CoreError::invalid(
                    "tokens",
                    "op 'tokens' needs a non-empty `tokens` list",
                ));
            }
            let new_query = build_refined_query(query, &tokens, items);
            if !client {
                return Ok(match refetch(ep, &new_query, fetch, enrich).await? {
                    None => unchanged("zero_results"),
                    Some(new_items) => changed(new_query, new_items),
                });
            }
            let kept: Vec<Release> = items
                .iter()
                .filter(|it| matches_all_tokens(it, &tokens))
                .cloned()
                .collect();
            if kept.is_empty() || kept == items {
                return Ok(unchanged("noop"));
            }
            Ok(changed(new_query, kept))
        }
        RefineOp::Term { term } => {
            let term = term.trim();
            if term.is_empty() {
                return Ok(unchanged("empty_term"));
            }
            let term_lc = term.to_lowercase();
            if !items.iter().any(|it| custom_hit(it, &term_lc)) {
                return Ok(unchanged("no_match"));
            }
            let new_query = add_term_to_query(query, term);
            if !client {
                return Ok(match refetch(ep, &new_query, fetch, enrich).await? {
                    None => unchanged("zero_results"),
                    Some(new_items) => changed(new_query, new_items),
                });
            }
            let kept = items
                .iter()
                .filter(|it| custom_hit(it, &term_lc))
                .cloned()
                .collect();
            Ok(changed(new_query, kept))
        }
        RefineOp::Exclude { term } => {
            let new_query = add_exclude_to_query(query, term);
            if new_query == query {
                return Ok(unchanged("empty_term"));
            }
            if !client {
                return Ok(match refetch(ep, &new_query, fetch, enrich).await? {
                    None => unchanged("zero_results"),
                    Some(new_items) => changed(new_query, new_items),
                });
            }
            let term_lc = term.trim().trim_start_matches('-').trim().to_lowercase();
            let kept: Vec<Release> = items
                .iter()
                .filter(|it| !it.title.to_lowercase().contains(&term_lc))
                .cloned()
                .collect();
            if kept.is_empty() {
                return Ok(unchanged("zero_results"));
            }
            Ok(changed(new_query, kept))
        }
        RefineOp::BestFit => {
            let best_query = best_fit_query_for(query, items, cfg);
            if best_query.is_empty() {
                return Ok(unchanged("noop"));
            }
            if !client {
                return Ok(match refetch(ep, &best_query, fetch, enrich).await? {
                    None => unchanged("zero_results"),
                    Some(new_items) => changed(best_query, new_items),
                });
            }
            let Some(best) = bestfit::best_item(items, cfg) else {
                return Ok(unchanged("noop"));
            };
            let parts = bestfit::profile_parts(best);
            let kept: Vec<Release> = items
                .iter()
                .filter(|it| bestfit::matches_profile(&it.title, &parts))
                .cloned()
                .collect();
            if kept.is_empty() || kept.len() == items.len() {
                return Ok(unchanged("noop"));
            }
            Ok(changed(best_query, kept))
        }
    }
}

/// What the best-fit step would do: whether it is available, the query it would refetch
/// with, the release it pins, and that release's profile.
pub fn best_fit_info(query: &str, items: &[Release], cfg: &BestfitConfig) -> BestFit {
    let best = bestfit::best_item(items, cfg);
    let best_query = best_fit_query_for(query, items, cfg);
    BestFit {
        available: !best_query.is_empty(),
        query: best_query,
        item: best.cloned(),
        profile: best.map(bestfit::profile_parts).unwrap_or_default(),
    }
}

/// Indexes into `items` of the releases a group's tokens select.
pub fn group_members(items: &[Release], group: &ReleaseGroup) -> Vec<u64> {
    let Some(first) = group.tokens.first() else {
        return Vec::new();
    };
    let parsed: Vec<Parsed> = items.iter().map(parse).collect();
    members_of(items, &parsed, first)
        .into_iter()
        .map(|i| i as u64)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    fn item(title: &str, link: &str) -> Release {
        Release {
            title: title.to_string(),
            link: link.to_string(),
            seeders: None,
            leechers: None,
            downloads: None,
            size: String::new(),
            category: String::new(),
            page: String::new(),
            source: String::new(),
            resolution: String::new(),
            infomkv: String::new(),
            linktype: String::new(),
        }
    }

    fn frieren(n: usize, group: &str, res: &str) -> Vec<Release> {
        (1..=n)
            .map(|i| {
                item(
                    &format!("[{group}] Frieren - {i:02} ({res}) [ABCD{i:04}].mkv"),
                    &format!("magnet:{i}"),
                )
            })
            .collect()
    }

    fn erai_ep() -> Endpoint {
        Endpoint::new("erai", "rss", "https://erai.example/feed/?s={query}")
    }
    fn nyaa() -> Endpoint {
        Endpoint::new("nyaa", "nyaa", "https://nyaa.example/")
    }
    fn generic_rss() -> Endpoint {
        Endpoint::new("rss1", "rss", "https://rss1.example/f?q={query}")
    }

    fn tui_items() -> Vec<Release> {
        vec![
            item("Show - 01 [1080p][Hevc]", "l1"),
            item("Show - 02 [1080p][Hevc]", "l2"),
            item("Show - 03 [1080p][Avc]", "l3"),
        ]
    }

    fn links(items: &[Release]) -> Vec<&str> {
        items.iter().map(|i| i.link.as_str()).collect()
    }

    type Calls = Arc<Mutex<Vec<(String, String)>>>;
    type Fetched = Result<Vec<Release>, CoreError>;

    /// A fetch callback answering `result` and recording every call.
    fn fetcher(
        result: Fetched,
    ) -> (
        Calls,
        impl for<'a> Fn(&'a Endpoint, &'a str) -> BoxFuture<'a, Fetched>,
    ) {
        let calls: Calls = Arc::default();
        let log = calls.clone();
        let result = Mutex::new(Some(result));
        (calls, move |ep: &Endpoint, q: &str| {
            log.lock().unwrap().push((ep.name.clone(), q.to_string()));
            let answer = match &*result.lock().unwrap() {
                Some(Ok(items)) => Ok(items.clone()),
                Some(Err(e)) => Err(CoreError::internal(e.to_string())),
                None => Ok(Vec::new()),
            };
            Box::pin(async move { answer })
        })
    }

    /// A fetch callback that fails the test when called.
    fn never()
    -> impl for<'a> Fn(&'a Endpoint, &'a str) -> BoxFuture<'a, Result<Vec<Release>, CoreError>>
    {
        |_: &Endpoint, _: &str| panic!("this step must not fetch")
    }

    async fn run(
        ep: &Endpoint,
        query: &str,
        items: &[Release],
        op: RefineOp,
        fetch: &FetchFn<'_>,
    ) -> Outcome {
        apply(
            ep,
            query,
            items,
            &op,
            &BestfitConfig::default(),
            fetch,
            None,
        )
        .await
        .unwrap()
    }

    fn tokens_op(tokens: &[&str]) -> RefineOp {
        RefineOp::Tokens {
            tokens: tokens.iter().map(|t| t.to_string()).collect(),
        }
    }
    fn term_op(term: &str) -> RefineOp {
        RefineOp::Term {
            term: term.to_string(),
        }
    }
    fn exclude_op(term: &str) -> RefineOp {
        RefineOp::Exclude {
            term: term.to_string(),
        }
    }

    // auto resolution

    #[test]
    fn auto_resolution_appends_the_highest_and_keeps_those_items() {
        let mut items = frieren(2, "SubsPlease", "720p");
        items.extend(frieren(2, "SubsPlease", "1080p"));
        let out = auto_resolution("Frieren", &items);
        assert_eq!(out.query, "Frieren 1080p");
        assert_eq!(out.token, "1080p");
        assert_eq!(out.skipped_missing, 0);
        assert_eq!(out.items, items[2..].to_vec());
    }

    #[test]
    fn auto_resolution_skips_when_a_title_lacks_one() {
        let mut items = frieren(2, "SubsPlease", "1080p");
        items.push(item("[Group] Frieren - 03", "l3"));
        let out = auto_resolution("Frieren", &items);
        assert_eq!(out.query, "Frieren");
        assert_eq!(out.items, items);
        assert_eq!((out.token.as_str(), out.skipped_missing), ("", 1));
    }

    #[test]
    fn auto_resolution_leaves_a_query_with_a_resolution_alone() {
        let mut items = frieren(1, "SubsPlease", "720p");
        items.extend(frieren(1, "SubsPlease", "1080p"));
        let out = auto_resolution("Frieren 720p", &items);
        assert_eq!(out.query, "Frieren 720p");
        assert_eq!(out.items, items);
        assert_eq!((out.token.as_str(), out.skipped_missing), ("", 0));
        assert_eq!(auto_resolution("Frieren", &[]).query, "Frieren");
    }

    #[test]
    fn auto_resolution_reads_the_highest_resolution_in_each_title() {
        let items = vec![
            item("Show [720p][1080p]", "a"),
            item("Show [1080p]", "b"),
            item("Show [720p]", "c"),
        ];
        let out = auto_resolution("Show", &items);
        assert_eq!(out.items, items[..2].to_vec());
    }

    // groups

    #[test]
    fn groups_carry_member_indexes() {
        let items = vec![
            item("[SubsPlease] Frieren - 01 (1080p) [A1].mkv", "l1"),
            item("[SubsPlease] Frieren - 02 (1080p) [A2].mkv", "l2"),
            item("[Erai-raws] Frieren - 02 [1080p][HEVC].mkv", "l3"),
            item("[Erai-raws] Frieren - 03 [1080p][HEVC].mkv", "l4"),
            item("[Judas] Frieren - 04 [1080p].mkv", "l5"),
        ];
        let groups = compute_groups(&items);
        assert!(!groups.is_empty());
        let by_label: HashMap<&str, &ReleaseGroup> =
            groups.iter().map(|g| (g.label.as_str(), g)).collect();
        assert_eq!(by_label["[SubsPlease]"].members, vec![0, 1]);
        assert_eq!(by_label["[Erai-raws]+HEVC"].members, vec![2, 3]);
        assert_eq!(
            by_label["[Erai-raws]+HEVC"].tokens,
            vec!["[Erai-raws]", "HEVC"]
        );
        for g in &groups {
            let mut sorted = g.members.clone();
            sorted.sort_unstable();
            assert_eq!(g.members, sorted);
            assert_eq!(g.members.len() as u64, g.member_count);
            assert!(g.members.iter().all(|&i| (i as usize) < items.len()));
            assert!(1 < g.member_count && (g.member_count as usize) < items.len());
        }
        assert!(by_label["[SubsPlease]"].has_poster);
    }

    #[test]
    fn groups_sort_posters_first_then_larger_then_label() {
        let items = vec![
            item("[A] Show - 01 [xx]", "1"),
            item("[A] Show - 02 [xx]", "2"),
            item("[B] Show - 03 [yy]", "3"),
            item("[B] Show - 04 [yy]", "4"),
            item("[C] Show - 05 [yy]", "5"),
        ];
        let labels: Vec<String> = compute_groups(&items)
            .into_iter()
            .map(|g| g.label)
            .collect();
        // Poster groups first; [A]+xx and [B] tie on two members and fall to label order;
        // the bare token yy (three members, no poster) comes last.
        assert_eq!(labels, vec!["[A]+xx", "[B]", "yy"]);
    }

    #[test]
    fn group_tokens_sort_poster_first_then_longest_then_alphabetical() {
        let items = vec![
            item("[G] Show Alpha Beta Gamma", "1"),
            item("[G] Show Alpha Beta Gamma", "2"),
            item("[H] Other", "3"),
        ];
        let groups = compute_groups(&items);
        let g = groups.iter().find(|g| g.has_poster).unwrap();
        assert_eq!(g.tokens, vec!["[G]", "Alpha", "Gamma", "Beta", "Show"]);
        assert_eq!(g.label, "[G]+Alpha+Gamma+Beta+Show");
    }

    #[test]
    fn member_indexes_follow_the_filtered_items() {
        let items = vec![
            item("[A] Show - 01 (720p)", "x0"),
            item("[A] Show - 02 (1080p)", "x1"),
            item("[A] Show - 03 (1080p) [v2]", "x2"),
            item("[B] Show - 04 (1080p) [v2]", "x3"),
        ];
        let kept = auto_resolution("Show", &items);
        assert_eq!(links(&kept.items), vec!["x1", "x2", "x3"]);
        let groups = compute_groups(&kept.items);
        let members: HashMap<&str, &Vec<u64>> = groups
            .iter()
            .map(|g| (g.label.as_str(), &g.members))
            .collect();
        assert_eq!(members["[A]"], &vec![0, 1]);
    }

    #[test]
    fn group_members_helper() {
        let items = frieren(3, "SubsPlease", "1080p");
        for g in compute_groups(&items) {
            let mut sorted = group_members(&items, &g);
            let again = sorted.clone();
            sorted.sort_unstable();
            assert_eq!(again, sorted);
        }
        let empty = ReleaseGroup {
            label: String::new(),
            tokens: vec![],
            member_count: 0,
            has_poster: false,
            members: vec![],
        };
        assert!(group_members(&items, &empty).is_empty());
    }

    #[test]
    fn groups_use_erai_metadata_for_membership() {
        let mut items = vec![
            item("Show - 01 [1080p]", "l1"),
            item("Show - 02 [1080p]", "l2"),
            item("Show - 03 [1080p]", "l3"),
        ];
        items[0].linktype = "Torrent".into();
        items[1].linktype = "Torrent".into();
        items[2].linktype = "Magnet".into();
        let group = ReleaseGroup {
            label: "Torrent".into(),
            tokens: vec!["Torrent".into()],
            member_count: 2,
            has_poster: false,
            members: vec![],
        };
        assert_eq!(group_members(&items, &group), vec![0, 1]);
    }

    // query building

    #[test]
    fn positional_insert_mirrors_title_order() {
        let items = vec![
            item("[G] Show 1080p MultiSub", "1"),
            item("[G] Show 1080p MultiSub", "2"),
        ];
        let tokens = vec!["1080p".to_string()];
        assert_eq!(
            build_refined_query("[G] Show MultiSub", &tokens, &items),
            "[G] Show 1080p MultiSub"
        );
        // A token no title carries goes to the tail, ahead of exclusions.
        let tokens = vec!["Nope".to_string()];
        assert_eq!(
            build_refined_query("Show -Hevc", &tokens, &items),
            "Show Nope -Hevc"
        );
        assert_eq!(build_refined_query("Show", &tokens, &items), "Show Nope");
    }

    #[test]
    fn a_poster_token_leads_unless_the_query_has_one() {
        let items = tui_items();
        let tokens = vec!["[Grp]".to_string()];
        assert_eq!(build_refined_query("Show", &tokens, &items), "[Grp] Show");
        assert_eq!(
            build_refined_query("  [Old] Show", &tokens, &items),
            "  [Old] Show"
        );
    }

    #[test]
    fn quoted_fields_stay_whole_when_positioned() {
        let items = vec![
            item("Show Two Words 1080p", "1"),
            item("Show Two Words 1080p", "2"),
        ];
        let tokens = vec!["1080p".to_string()];
        assert_eq!(
            build_refined_query("Show \"Two Words\"", &tokens, &items),
            "Show \"Two Words\" 1080p"
        );
        let tokens = vec!["Show".to_string()];
        assert_eq!(
            build_refined_query("\"Two Words\"", &tokens, &items),
            "Show \"Two Words\""
        );
    }

    #[test]
    fn title_position_is_the_median_and_even_counts_average() {
        let items = vec![item("ab x", "1"), item("abcd x", "2"), item("zz", "3")];
        assert_eq!(title_position("x", &items), 4.0); // spots 3 and 5
        assert_eq!(title_position("zz", &items), 0.0);
        assert!(title_position("nope", &items).is_infinite());
        assert!(title_position("", &items).is_infinite());
        let odd = vec![item("x", "1"), item("..x", "2"), item("......x", "3")];
        assert_eq!(title_position("x", &odd), 2.0);
    }

    #[test]
    fn exclude_and_term_query_text() {
        assert_eq!(add_exclude_to_query("Show", " -Hevc"), "Show -Hevc");
        assert_eq!(add_exclude_to_query("Show", "a b"), "Show -\"a b\"");
        assert_eq!(add_exclude_to_query("Show", "---"), "Show");
        assert_eq!(add_exclude_to_query("Show", " - "), "Show");
        assert_eq!(
            add_term_to_query("Show", "  two words "),
            "Show \"two words\""
        );
        assert_eq!(add_term_to_query("Show", "   "), "Show");
    }

    // apply: nyaa

    #[tokio::test]
    async fn nyaa_tokens_refetches_with_the_rebuilt_query() {
        let items = tui_items();
        let (calls, fetch) = fetcher(Ok(items[..2].to_vec()));
        let out = run(&nyaa(), "Show 1080p", &items, tokens_op(&["Hevc"]), &fetch).await;
        assert!(out.applied && out.mode == "refetch" && out.reason.is_none());
        assert_eq!(out.query, "Show 1080p Hevc");
        assert_eq!(out.items, items[..2].to_vec());
        assert_eq!((out.previous_count, out.delta()), (3, -1));
        assert_eq!(
            *calls.lock().unwrap(),
            vec![("nyaa".to_string(), "Show 1080p Hevc".to_string())]
        );
    }

    #[tokio::test]
    async fn nyaa_zero_results_keeps_the_inputs() {
        let items = tui_items();
        let (_, fetch) = fetcher(Ok(Vec::new()));
        let out = run(&nyaa(), "Show 1080p", &items, tokens_op(&["Hevc"]), &fetch).await;
        assert!(!out.applied);
        assert_eq!(out.reason.as_deref(), Some("zero_results"));
        assert_eq!(
            (out.query.as_str(), out.mode.as_str(), out.delta()),
            ("Show 1080p", "refetch", 0)
        );
        assert_eq!(out.items, items);
    }

    #[tokio::test]
    async fn nyaa_zero_results_for_each_op() {
        let items = tui_items();
        for op in [tokens_op(&["Hevc"]), term_op("Hevc"), exclude_op("Avc")] {
            let (_, fetch) = fetcher(Ok(Vec::new()));
            let out = run(&nyaa(), "Show 1080p", &items, op, &fetch).await;
            assert_eq!(
                (out.applied, out.reason.as_deref()),
                (false, Some("zero_results"))
            );
            assert_eq!(out.query, "Show 1080p");
            assert_eq!(out.items, items);
        }
    }

    #[tokio::test]
    async fn nyaa_refetch_error_propagates() {
        let items = tui_items();
        let (_, fetch) = fetcher(Err(CoreError::internal("down")));
        let err = apply(
            &nyaa(),
            "Show 1080p",
            &items,
            &term_op("Hevc"),
            &BestfitConfig::default(),
            &fetch,
            None,
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("down"), "{err}");
    }

    #[tokio::test]
    async fn nyaa_term_and_exclude_queries() {
        let mut items = tui_items();
        items.push(item("Show - 04 two words", "l4"));
        let (calls, fetch) = fetcher(Ok(items[..1].to_vec()));
        let out = run(&nyaa(), "Show", &items, term_op("  two words "), &fetch).await;
        assert_eq!(out.query, "Show \"two words\"");
        let out = run(&nyaa(), "Show", &items, exclude_op("-Hevc"), &fetch).await;
        assert_eq!(out.query, "Show -Hevc");
        let out = run(&nyaa(), "Show", &items, exclude_op("a b"), &fetch).await;
        assert_eq!(out.query, "Show -\"a b\"");
        let queries: Vec<String> = calls.lock().unwrap().iter().map(|c| c.1.clone()).collect();
        assert_eq!(
            queries,
            vec!["Show \"two words\"", "Show -Hevc", "Show -\"a b\""]
        );
    }

    #[tokio::test]
    async fn nyaa_enrich_only_applies_to_refetched_items() {
        let items = tui_items();
        let marked: Arc<Mutex<Vec<usize>>> = Arc::default();
        let log = marked.clone();
        let enrich = move |fetched: Vec<Release>| -> BoxFuture<'static, Vec<Release>> {
            log.lock().unwrap().push(fetched.len());
            Box::pin(async move {
                fetched
                    .into_iter()
                    .map(|it| Release {
                        seeders: Some(1),
                        ..it
                    })
                    .collect()
            })
        };
        let cfg = BestfitConfig::default();
        let (_, fetch) = fetcher(Ok(items[..2].to_vec()));
        let out = apply(
            &nyaa(),
            "Show",
            &items,
            &term_op("Hevc"),
            &cfg,
            &fetch,
            Some(&enrich),
        )
        .await
        .unwrap();
        assert_eq!(*marked.lock().unwrap(), vec![2]);
        assert_eq!(
            out.items.iter().map(|i| i.seeders).collect::<Vec<_>>(),
            vec![Some(1), Some(1)]
        );
        marked.lock().unwrap().clear();
        let (_, fetch) = fetcher(Ok(Vec::new()));
        apply(
            &nyaa(),
            "Show",
            &items,
            &term_op("Hevc"),
            &cfg,
            &fetch,
            Some(&enrich),
        )
        .await
        .unwrap();
        assert!(
            marked.lock().unwrap().is_empty(),
            "zero results: nothing to enrich"
        );
    }

    #[tokio::test]
    async fn nyaa_term_with_no_match_never_refetches() {
        let items = tui_items();
        let out = run(&nyaa(), "Show", &items, term_op("zzz"), &never()).await;
        assert_eq!(
            (out.applied, out.reason.as_deref()),
            (false, Some("no_match"))
        );
    }

    #[tokio::test]
    async fn nyaa_best_fit_refetches_with_the_best_query() {
        let items = vec![
            item("[Erai-raws] Show - 01 [1080p][HEVC]", "l1"),
            item("[Other] Show - 03 [720p]", "l3"),
        ];
        let want = best_fit_info("Show", &items, &BestfitConfig::default()).query;
        let (calls, fetch) = fetcher(Ok(items[..1].to_vec()));
        let out = run(&nyaa(), "Show", &items, RefineOp::BestFit, &fetch).await;
        assert!(out.applied);
        assert_eq!(out.query, want);
        assert_eq!(*calls.lock().unwrap(), vec![("nyaa".to_string(), want)]);
    }

    #[tokio::test]
    async fn best_fit_is_a_noop_when_the_query_is_already_pinned() {
        let items = vec![
            item("[Erai-raws] Show - 01 [1080p][HEVC]", "l1"),
            item("[Other] Show - 03 [720p]", "l3"),
        ];
        let pinned = best_fit_info("Show", &items, &BestfitConfig::default()).query;
        let out = run(&nyaa(), &pinned, &items, RefineOp::BestFit, &never()).await;
        assert_eq!((out.applied, out.reason.as_deref()), (false, Some("noop")));
    }

    #[tokio::test]
    async fn best_fit_on_empty_items_is_a_noop() {
        let out = run(&nyaa(), "Show", &[], RefineOp::BestFit, &never()).await;
        assert_eq!((out.applied, out.reason.as_deref()), (false, Some("noop")));
    }

    #[tokio::test]
    async fn empty_terms_are_refused_on_both_kinds() {
        let items = tui_items();
        for ep in [nyaa(), generic_rss()] {
            for op in [
                term_op(""),
                term_op("   "),
                exclude_op(""),
                exclude_op("-"),
                exclude_op("---"),
            ] {
                let out = run(&ep, "Show", &items, op, &never()).await;
                assert_eq!(
                    (out.applied, out.reason.as_deref()),
                    (false, Some("empty_term"))
                );
                assert_eq!(out.query, "Show");
                assert_eq!(out.items, items);
            }
        }
    }

    #[tokio::test]
    async fn exclude_strips_whitespace_before_the_dash() {
        let items = tui_items();
        let (_, fetch) = fetcher(Ok(items[..1].to_vec()));
        let out = run(&nyaa(), "Show", &items, exclude_op(" -Hevc"), &fetch).await;
        assert_eq!(out.query, "Show -Hevc");
        let out = run(&nyaa(), "Show", &items, exclude_op(" - "), &fetch).await;
        assert_eq!(out.reason.as_deref(), Some("empty_term"));
    }

    #[tokio::test]
    async fn empty_token_lists_are_invalid() {
        let items = tui_items();
        for tokens in [vec![], vec![String::new()]] {
            let err = apply(
                &generic_rss(),
                "Show",
                &items,
                &RefineOp::Tokens { tokens },
                &BestfitConfig::default(),
                &never(),
                None,
            )
            .await
            .unwrap_err();
            assert!(
                matches!(&err, CoreError::Invalid { field, .. } if field == "tokens"),
                "{err}"
            );
        }
    }

    // apply: client side

    #[tokio::test]
    async fn rss_tokens_filter_client_side() {
        let items = tui_items();
        let out = run(
            &generic_rss(),
            "Show 1080p",
            &items,
            tokens_op(&["Hevc"]),
            &never(),
        )
        .await;
        assert!(out.applied && out.mode == "client");
        assert_eq!(links(&out.items), vec!["l1", "l2"]);
        assert_eq!(out.query, "Show 1080p Hevc");
        assert_eq!(out.delta(), -1);
    }

    #[tokio::test]
    async fn rss_tokens_noop_cases() {
        let items = tui_items();
        let out = run(
            &generic_rss(),
            "Show 1080p",
            &items,
            tokens_op(&["Nope"]),
            &never(),
        )
        .await;
        assert_eq!((out.applied, out.reason.as_deref()), (false, Some("noop")));
        assert_eq!(out.items, items);
        assert_eq!(out.query, "Show 1080p");
        // A token every item carries changes nothing either.
        let out = run(
            &generic_rss(),
            "Show",
            &items,
            tokens_op(&["1080p"]),
            &never(),
        )
        .await;
        assert_eq!((out.applied, out.reason.as_deref()), (false, Some("noop")));
    }

    #[tokio::test]
    async fn rss_tokens_match_erai_metadata() {
        let mut items = vec![
            item("Show - 01 [1080p]", "l1"),
            item("Show - 02 [1080p]", "l2"),
            item("Show - 03 [1080p]", "l3"),
        ];
        items[0].linktype = "Torrent".into();
        items[1].linktype = "Torrent".into();
        items[2].linktype = "Magnet".into();
        let out = run(
            &erai_ep(),
            "Show 1080p",
            &items,
            tokens_op(&["Torrent"]),
            &never(),
        )
        .await;
        assert_eq!(links(&out.items), vec!["l1", "l2"]);
    }

    #[tokio::test]
    async fn rss_tokens_match_posters_exactly() {
        let items = vec![item("[A] Show - 01", "l1"), item("[B] Show - 02", "l2")];
        let out = run(
            &generic_rss(),
            "Show",
            &items,
            tokens_op(&["[A]"]),
            &never(),
        )
        .await;
        assert_eq!(links(&out.items), vec!["l1"]);
        assert_eq!(out.query, "[A] Show");
    }

    #[tokio::test]
    async fn rss_term_filters_and_reports_no_match() {
        let items = tui_items();
        let out = run(&generic_rss(), "Show", &items, term_op("avc"), &never()).await;
        assert!(out.applied);
        assert_eq!(links(&out.items), vec!["l3"]);
        assert_eq!(
            (out.query.as_str(), out.mode.as_str()),
            ("Show avc", "client")
        );
        let out = run(&generic_rss(), "Show", &items, term_op("zzz"), &never()).await;
        assert_eq!(
            (out.applied, out.reason.as_deref()),
            (false, Some("no_match"))
        );
        assert_eq!(out.items, items);
        assert_eq!(out.query, "Show");
    }

    #[tokio::test]
    async fn rss_term_matches_erai_metadata() {
        let mut items = vec![
            item("Show - 01 [1080p]", "l1"),
            item("Show - 02 [1080p]", "l2"),
        ];
        items[0].linktype = "Torrent".into();
        let out = run(&erai_ep(), "Show", &items, term_op("torrent"), &never()).await;
        assert_eq!(links(&out.items), vec!["l1"]);
    }

    #[tokio::test]
    async fn rss_exclude_filters_and_strips_the_dash() {
        let items = tui_items();
        let out = run(
            &generic_rss(),
            "Show",
            &items,
            exclude_op("-HEVC"),
            &never(),
        )
        .await;
        assert!(out.applied);
        assert_eq!(links(&out.items), vec!["l3"]);
        assert_eq!(out.query, "Show -HEVC");
    }

    #[tokio::test]
    async fn rss_exclude_everything_is_zero_results() {
        let items = tui_items();
        let out = run(&generic_rss(), "Show", &items, exclude_op("Show"), &never()).await;
        assert_eq!(
            (out.applied, out.reason.as_deref()),
            (false, Some("zero_results"))
        );
        assert_eq!(out.items, items);
    }

    #[tokio::test]
    async fn rss_best_fit_filters_to_the_profile_and_then_noops() {
        let items = vec![
            item("[Erai-raws] Show - 01 [1080p][HEVC]", "l1"),
            item("[Erai-raws] Show - 02 [1080p][HEVC]", "l2"),
            item("[Other] Show - 03 [720p]", "l3"),
        ];
        let out = run(&generic_rss(), "Show", &items, RefineOp::BestFit, &never()).await;
        assert!(out.applied && out.mode == "client");
        assert_eq!(links(&out.items), vec!["l1", "l2"]);
        assert_eq!(
            out.query,
            best_fit_info("Show", &items, &BestfitConfig::default()).query
        );
        // Nothing to narrow: every release already shares the profile.
        let out = run(
            &generic_rss(),
            "Show",
            &items[..2],
            RefineOp::BestFit,
            &never(),
        )
        .await;
        assert_eq!((out.applied, out.reason.as_deref()), (false, Some("noop")));
    }

    #[tokio::test]
    async fn rss_best_fit_without_a_poster_keeps_a_nonempty_subset() {
        let items = tui_items();
        let out = run(
            &erai_ep(),
            "Show 1080p",
            &items,
            RefineOp::BestFit,
            &never(),
        )
        .await;
        assert!(out.applied || out.reason.as_deref() == Some("noop"));
        assert!(!out.items.is_empty() && out.items.len() <= items.len());
    }

    #[tokio::test]
    async fn rss_exclude_matches_the_refine_loop() {
        let items = tui_items();
        let out = run(
            &erai_ep(),
            "Show 1080p",
            &items,
            exclude_op("Hevc"),
            &never(),
        )
        .await;
        assert_eq!(links(&out.items), vec!["l3"]);
    }

    #[test]
    fn client_side_is_anything_but_nyaa() {
        assert!(is_client_side(&generic_rss()));
        assert!(is_client_side(&erai_ep()));
        assert!(!is_client_side(&nyaa()));
    }

    // best fit info

    #[test]
    fn best_fit_info_describes_the_step() {
        let items = vec![
            item("[Erai-raws] Frieren - 02 [1080p][HEVC].mkv", "l1"),
            item("[SubsPlease] Frieren - 01 (720p) [AB].mkv", "l2"),
        ];
        let cfg = BestfitConfig::default();
        let best = best_fit_info("Frieren", &items, &cfg);
        assert!(best.available);
        assert!(items.contains(best.item.as_ref().unwrap()));
        assert!(!best.query.is_empty() && best.query != "Frieren");
        assert!(best.profile.contains(&"[Erai-raws]".to_string()));
        // When the query already is the best-fit query there is nothing to do.
        let again = best_fit_info(&best.query, &items, &cfg);
        assert!(!again.available);
        assert_eq!(again.query, "");
        assert!(again.item.is_some());
    }

    #[test]
    fn best_fit_info_of_nothing() {
        let info = best_fit_info("x", &[], &BestfitConfig::default());
        assert!(!info.available && info.item.is_none() && info.profile.is_empty());
    }

    #[test]
    fn metadata_hit_rules() {
        let mut it = item("Show", "l");
        assert!(!metadata_hit(&it, "1080p"), "no metadata, no hit");
        it.resolution = "1080p".into();
        it.infomkv = "NF WEB-DL AVC AAC".into();
        it.linktype = "Torrent".into();
        it.category = "Airing".into();
        assert!(metadata_hit(&it, "1080P"));
        assert!(metadata_hit(&it, "web-dl"));
        assert!(metadata_hit(&it, "web"), "infomkv is a substring match");
        assert!(
            !metadata_hit(&it, "1080"),
            "resolution is an equality match"
        );
        assert!(metadata_hit(&it, "torrent"));
        assert!(metadata_hit(&it, "airing"));
        assert!(!metadata_hit(&it, "magnet"));
    }
}
