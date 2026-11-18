//! Refinement logic with no UI: token groups, query rebuilding, and one stateless refine
//! step. Ported from anirss's `refinement.py`.
//!
//! Nothing here prints, prompts or touches a terminal. `apply` takes the refetch as a callback,
//! so the caller decides how a feed is read.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::future::Future;

use super::endpoints::FIELD_RE;
use super::quirks::{field_core, matches_token, metadata_hit};
use super::titles::{RES_RE, poster_of, title_tokens};
use super::{BestfitConfig, Endpoint, bestfit};
use anibeam_contract::{BestFit, CoreError, RefineOp, Release, ReleaseGroup};

/// What `auto_resolution` decided: the query with its `<n>p` appended, the items at that
/// resolution, and the token.
#[derive(Clone, Debug, PartialEq)]
pub struct AutoResolution {
    pub query: String,
    pub items: Vec<Release>,
    pub token: String,
}

/// If `query` lacks a `<n>p` token and every item has one, append the highest and keep
/// only the items at that resolution. None when it changes nothing: no items, a query that
/// names a resolution already, or a title without one.
pub fn auto_resolution(query: &str, items: &[Release]) -> Option<AutoResolution> {
    if items.is_empty() || RES_RE.is_match(query) {
        return None;
    }
    let per_max: Vec<i64> = items
        .iter()
        .map(|item| {
            RES_RE
                .captures_iter(&item.title)
                .filter_map(|c| c[1].parse::<i64>().ok())
                .max()
        })
        .collect::<Option<_>>()?;
    let highest = per_max.iter().copied().max().unwrap_or(0);
    let token = format!("{highest}p");
    let kept = items
        .iter()
        .zip(&per_max)
        .filter(|(_, res)| **res == highest)
        .map(|(item, _)| item.clone())
        .collect();
    Some(AutoResolution {
        query: format!("{query} {token}"),
        items: kept,
        token,
    })
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

/// Whether `item`, read into `parsed`, carries `token`: a poster as its own poster, any other
/// token among its title's tokens or in the source's structured fields.
fn has_token(item: &Release, parsed: &Parsed, token: &str) -> bool {
    if token.starts_with('[') {
        parsed.poster.as_deref() == Some(token)
    } else {
        parsed.tokens.iter().any(|t| t == token) || metadata_hit(item, token)
    }
}

fn matches_all_tokens(item: &Release, tokens: &[String]) -> bool {
    let parsed = parse(item);
    tokens.iter().all(|t| has_token(item, &parsed, t))
}

/// A group's tokens in their order: the poster first, then the longest, then alphabetical.
fn token_order(token: &str) -> (bool, Reverse<usize>, &str) {
    (
        !token.starts_with('['),
        Reverse(token.chars().count()),
        token,
    )
}

/// The groups in their order: poster groups first, then the larger, then by label.
fn group_order(group: &ReleaseGroup) -> (bool, Reverse<u64>, &str) {
    (!group.has_poster, Reverse(group.member_count), &group.label)
}

/// Find tokens that partition `selected` into refinable subsets, grouped by membership.
pub fn compute_groups(selected: &[Release]) -> Vec<ReleaseGroup> {
    let total = selected.len();
    let parsed: Vec<Parsed> = selected.iter().map(parse).collect();
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for p in &parsed {
        let seen: BTreeSet<&str> = p
            .poster
            .iter()
            .chain(&p.tokens)
            .map(String::as_str)
            .collect();
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
        let members: Vec<usize> = (0..total)
            .filter(|&i| has_token(&selected[i], &parsed[i], token))
            .collect();
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
        .map(|(members, mut tokens)| {
            tokens.sort_by(|a, b| token_order(a).cmp(&token_order(b)));
            ReleaseGroup {
                label: tokens.join("+"),
                member_count: members.len() as u64,
                has_poster: tokens.iter().any(|t| t.starts_with('[')),
                members: members.iter().map(|&i| i as u64).collect(),
                tokens: tokens.iter().map(|t| t.to_string()).collect(),
            }
        })
        .collect();
    groups.sort_by(|a, b| group_order(a).cmp(&group_order(b)));
    groups
}

/// The query the best-fit step would refetch with, and the release it pins; None when
/// running it would be a no-op (nothing to rank, empty rebuild, or the current query
/// already matching the top release's profile).
fn best_fit_query_for<'a>(
    query: &str,
    selected: &'a [Release],
    cfg: &BestfitConfig,
) -> Option<(String, &'a Release)> {
    let best = bestfit::best_item(selected, cfg)?;
    let new_query = bestfit::best_fit_query(best);
    (!new_query.is_empty() && new_query.to_lowercase() != query.to_lowercase())
        .then_some((new_query, best))
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
    let insert_at = fields
        .iter()
        // Never slot in front of a poster; exclusions stay at the end.
        .position(|field| {
            !field.starts_with('[')
                && (field.starts_with('-')
                    || title_position(&field_core(field).to_lowercase(), selected) > token_pos)
        })
        .unwrap_or(fields.len());
    fields.insert(insert_at, token);
    fields.join(" ")
}

/// Extend `query` with `tokens`, each placed where it belongs.
///
/// A poster (`[...]`) leads the title, so it goes to the front unless the query already
/// starts with one. Every other token is positioned by where it occurs in the titles. The
/// caller refetches the source with the rebuilt query rather than filtering locally, so
/// episodes beyond the first RSS page can surface.
fn build_refined_query(query: &str, tokens: &[String], selected: &[Release]) -> String {
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

/// `query` with `term` after `sign` (`-` for an exclusion), quoted when it holds a space;
/// unchanged when the term is empty.
fn with_term(query: &str, sign: &str, term: &str) -> String {
    match term {
        "" => query.to_string(),
        _ if term.contains(' ') => format!("{query} {sign}\"{term}\""),
        _ => format!("{query} {sign}{term}"),
    }
}

/// An exclusion's term without its spaces and leading `-`.
fn excluded_term(term: &str) -> &str {
    term.trim().trim_start_matches('-').trim()
}

/// Append a `-tag` exclusion to `query`. The query comes back unchanged when the term is
/// empty after stripping.
fn add_exclude_to_query(query: &str, term: &str) -> String {
    with_term(query, "-", excluded_term(term))
}

/// Append a positive search term to `query`, quoting it if it contains spaces.
fn add_term_to_query(query: &str, term: &str) -> String {
    with_term(query, "", term.trim())
}

// One stateless refine step.

/// Generic RSS endpoints are word-AND over post text, so refinements filter the fetched
/// items locally; filtered-rss refetches server-side so a narrower query can surface older
/// episodes.
pub fn is_client_side(ep: &Endpoint) -> bool {
    ep.kind != super::kinds::FILTERED_RSS
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

/// Which releases a refine step keeps when it filters them here.
type Keep = Box<dyn Fn(&Release) -> bool + Send + Sync>;

/// Apply one refine action the way anirss's refine loop does.
///
/// `Tokens` narrows to a `compute_groups` group's tokens, `Term` is a custom substring,
/// `Exclude` drops a term, `BestFit` pins the top release's profile. filtered-rss endpoints
/// refetch with the rebuilt query through `refetch`, called at most once; rss endpoints
/// filter `items` client-side. A failed refetch is the callback's error.
pub async fn apply<F, Fut>(
    ep: &Endpoint,
    query: &str,
    items: &[Release],
    op: &RefineOp,
    cfg: &BestfitConfig,
    refetch: F,
) -> Result<Outcome, CoreError>
where
    F: FnOnce(String) -> Fut,
    Fut: Future<Output = Result<Vec<Release>, CoreError>>,
{
    let prev = items.len();
    let client = is_client_side(ep);
    let outcome = |query: String, items: Vec<Release>, reason: Option<&str>| Outcome {
        applied: reason.is_none(),
        query,
        items,
        mode: if client { "client" } else { "refetch" }.to_string(),
        previous_count: prev,
        reason: reason.map(String::from),
    };
    let unchanged = |reason| outcome(query.to_string(), items.to_vec(), Some(reason));

    // The rebuilt query, and which of `items` it keeps when they are filtered here.
    let (new_query, keep): (String, Keep) = match op {
        RefineOp::Tokens { tokens } => {
            let tokens: Vec<String> = tokens.iter().filter(|t| !t.is_empty()).cloned().collect();
            if tokens.is_empty() {
                return Err(CoreError::invalid(
                    "tokens",
                    anibeam_contract::i18n::tr("core-releases-refine-no-tokens", &[]),
                ));
            }
            let new_query = build_refined_query(query, &tokens, items);
            (
                new_query,
                Box::new(move |it| matches_all_tokens(it, &tokens)),
            )
        }
        RefineOp::Term { term } => {
            let term = term.trim();
            if term.is_empty() {
                return Ok(unchanged("empty_term"));
            }
            if !items.iter().any(|it| matches_token(it, term)) {
                return Ok(unchanged("no_match"));
            }
            let term_lc = term.to_lowercase();
            let new_query = add_term_to_query(query, term);
            (new_query, Box::new(move |it| matches_token(it, &term_lc)))
        }
        RefineOp::Exclude { term } => {
            let new_query = add_exclude_to_query(query, term);
            if new_query == query {
                return Ok(unchanged("empty_term"));
            }
            let term_lc = excluded_term(term).to_lowercase();
            let keep = move |it: &Release| !it.title.to_lowercase().contains(&term_lc);
            (new_query, Box::new(keep))
        }
        RefineOp::BestFit => {
            let Some((new_query, best)) = best_fit_query_for(query, items, cfg) else {
                return Ok(unchanged("noop"));
            };
            let parts = bestfit::profile_parts(best);
            let keep = move |it: &Release| bestfit::matches_profile(&it.title, &parts);
            (new_query, Box::new(keep))
        }
    };

    if !client {
        let found = refetch(new_query.clone()).await?;
        if found.is_empty() {
            return Ok(unchanged("zero_results"));
        }
        return Ok(outcome(new_query, found, None));
    }
    let kept: Vec<Release> = items.iter().filter(|it| keep(it)).cloned().collect();
    // Excluding everything finds nothing; a narrowing that keeps all or none changes nothing.
    let reason = match op {
        RefineOp::Exclude { .. } if kept.is_empty() => Some("zero_results"),
        RefineOp::Tokens { .. } | RefineOp::BestFit if kept.is_empty() || kept.len() == prev => {
            Some("noop")
        }
        _ => None,
    };
    Ok(match reason {
        Some(reason) => unchanged(reason),
        None => outcome(new_query, kept, None),
    })
}

/// What the best-fit step would do: whether it is available, the query it would refetch
/// with, the release it pins, and that release's profile.
pub fn best_fit_info(query: &str, items: &[Release], cfg: &BestfitConfig) -> BestFit {
    let best = bestfit::best_item(items, cfg);
    let best_query = best_fit_query_for(query, items, cfg).map(|(q, _)| q);
    BestFit {
        available: best_query.is_some(),
        query: best_query.unwrap_or_default(),
        item: best.cloned(),
        profile: best.map(bestfit::profile_parts).unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    fn item(title: &str, link: &str) -> Release {
        Release {
            link: link.to_string(),
            ..crate::titled(title)
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

    fn keyed_ep() -> Endpoint {
        Endpoint::new("keyed", "rss", "https://keyed.example/feed/?s={query}")
    }
    fn tracker() -> Endpoint {
        Endpoint::new(
            "tracker",
            crate::kinds::FILTERED_RSS,
            "https://tracker.example/",
        )
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

    type Calls = Arc<Mutex<Vec<String>>>;
    type Fetched = Result<Vec<Release>, CoreError>;

    /// A refetch answering `result` and recording every query it was asked.
    fn fetcher(result: Fetched) -> (Calls, impl Fn(String) -> std::future::Ready<Fetched>) {
        let calls: Calls = Arc::default();
        let log = calls.clone();
        (calls, move |q: String| {
            log.lock().unwrap().push(q);
            std::future::ready(match &result {
                Ok(items) => Ok(items.clone()),
                Err(e) => Err(CoreError::internal(e.to_string())),
            })
        })
    }

    /// A refetch that fails the test when called.
    fn never() -> impl Fn(String) -> std::future::Ready<Fetched> {
        |_| panic!("this step must not fetch")
    }

    async fn run(
        ep: &Endpoint,
        query: &str,
        items: &[Release],
        op: RefineOp,
        fetch: impl FnOnce(String) -> std::future::Ready<Fetched>,
    ) -> Outcome {
        apply(ep, query, items, &op, &BestfitConfig::default(), fetch)
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
        let mut items = frieren(2, "Lumen", "720p");
        items.extend(frieren(2, "Lumen", "1080p"));
        let out = auto_resolution("Frieren", &items).unwrap();
        assert_eq!(out.query, "Frieren 1080p");
        assert_eq!(out.token, "1080p");
        assert_eq!(out.items, items[2..].to_vec());
    }

    #[test]
    fn auto_resolution_skips_when_a_title_lacks_one() {
        let mut items = frieren(2, "Lumen", "1080p");
        items.push(item("[Group] Frieren - 03", "l3"));
        assert_eq!(auto_resolution("Frieren", &items), None);
    }

    #[test]
    fn auto_resolution_leaves_a_query_with_a_resolution_alone() {
        let mut items = frieren(1, "Lumen", "720p");
        items.extend(frieren(1, "Lumen", "1080p"));
        assert_eq!(auto_resolution("Frieren 720p", &items), None);
        assert_eq!(auto_resolution("Frieren", &[]), None);
    }

    #[test]
    fn auto_resolution_reads_the_highest_resolution_in_each_title() {
        let items = vec![
            item("Show [720p][1080p]", "a"),
            item("Show [1080p]", "b"),
            item("Show [720p]", "c"),
        ];
        let out = auto_resolution("Show", &items).unwrap();
        assert_eq!(out.items, items[..2].to_vec());
    }

    // groups

    #[test]
    fn groups_carry_member_indexes() {
        let items = vec![
            item("[Lumen] Frieren - 01 (1080p) [A1].mkv", "l1"),
            item("[Lumen] Frieren - 02 (1080p) [A2].mkv", "l2"),
            item("[Kumo-Subs] Frieren - 02 [1080p][HEVC].mkv", "l3"),
            item("[Kumo-Subs] Frieren - 03 [1080p][HEVC].mkv", "l4"),
            item("[Judas] Frieren - 04 [1080p].mkv", "l5"),
        ];
        let groups = compute_groups(&items);
        assert!(!groups.is_empty());
        let by_label: HashMap<&str, &ReleaseGroup> =
            groups.iter().map(|g| (g.label.as_str(), g)).collect();
        assert_eq!(by_label["[Lumen]"].members, vec![0, 1]);
        assert_eq!(by_label["[Kumo-Subs]+HEVC"].members, vec![2, 3]);
        assert_eq!(
            by_label["[Kumo-Subs]+HEVC"].tokens,
            vec!["[Kumo-Subs]", "HEVC"]
        );
        for g in &groups {
            let mut sorted = g.members.clone();
            sorted.sort_unstable();
            assert_eq!(g.members, sorted);
            assert_eq!(g.members.len() as u64, g.member_count);
            assert!(g.members.iter().all(|&i| (i as usize) < items.len()));
            assert!(1 < g.member_count && (g.member_count as usize) < items.len());
        }
        assert!(by_label["[Lumen]"].has_poster);
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
        let kept = auto_resolution("Show", &items).unwrap();
        assert_eq!(links(&kept.items), vec!["x1", "x2", "x3"]);
        let groups = compute_groups(&kept.items);
        let members: HashMap<&str, &Vec<u64>> = groups
            .iter()
            .map(|g| (g.label.as_str(), &g.members))
            .collect();
        assert_eq!(members["[A]"], &vec![0, 1]);
    }

    #[test]
    fn groups_use_extension_fields_for_membership() {
        let mut items = [
            item("Show - 01 [1080p]", "l1"),
            item("Show - 02 [1080p]", "l2"),
            item("Show - 03 [1080p]", "l3"),
        ];
        items[0].linktype = "Torrent".into();
        items[1].linktype = "Torrent".into();
        items[2].linktype = "Magnet".into();
        let members: Vec<usize> = (0..items.len())
            .filter(|&i| has_token(&items[i], &parse(&items[i]), "Torrent"))
            .collect();
        assert_eq!(members, vec![0, 1]);
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

    // apply: filtered-rss

    #[tokio::test]
    async fn tracker_tokens_refetches_with_the_rebuilt_query() {
        let items = tui_items();
        let (calls, fetch) = fetcher(Ok(items[..2].to_vec()));
        let out = run(
            &tracker(),
            "Show 1080p",
            &items,
            tokens_op(&["Hevc"]),
            &fetch,
        )
        .await;
        assert!(out.applied && out.mode == "refetch" && out.reason.is_none());
        assert_eq!(out.query, "Show 1080p Hevc");
        assert_eq!(out.items, items[..2].to_vec());
        assert_eq!((out.previous_count, out.items.len()), (3, 2));
        assert_eq!(*calls.lock().unwrap(), vec!["Show 1080p Hevc"]);
    }

    #[tokio::test]
    async fn tracker_zero_results_keeps_the_inputs() {
        let items = tui_items();
        let (_, fetch) = fetcher(Ok(Vec::new()));
        let out = run(
            &tracker(),
            "Show 1080p",
            &items,
            tokens_op(&["Hevc"]),
            &fetch,
        )
        .await;
        assert!(!out.applied);
        assert_eq!(out.reason.as_deref(), Some("zero_results"));
        assert_eq!(
            (out.query.as_str(), out.mode.as_str(), out.items.len()),
            ("Show 1080p", "refetch", 3)
        );
        assert_eq!(out.items, items);
    }

    #[tokio::test]
    async fn tracker_zero_results_for_each_op() {
        let items = tui_items();
        for op in [tokens_op(&["Hevc"]), term_op("Hevc"), exclude_op("Avc")] {
            let (_, fetch) = fetcher(Ok(Vec::new()));
            let out = run(&tracker(), "Show 1080p", &items, op, &fetch).await;
            assert_eq!(
                (out.applied, out.reason.as_deref()),
                (false, Some("zero_results"))
            );
            assert_eq!(out.query, "Show 1080p");
            assert_eq!(out.items, items);
        }
    }

    #[tokio::test]
    async fn tracker_refetch_error_propagates() {
        let items = tui_items();
        let (_, fetch) = fetcher(Err(CoreError::internal("down")));
        let err = apply(
            &tracker(),
            "Show 1080p",
            &items,
            &term_op("Hevc"),
            &BestfitConfig::default(),
            &fetch,
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("down"), "{err}");
    }

    #[tokio::test]
    async fn tracker_term_and_exclude_queries() {
        let mut items = tui_items();
        items.push(item("Show - 04 two words", "l4"));
        let (calls, fetch) = fetcher(Ok(items[..1].to_vec()));
        let out = run(&tracker(), "Show", &items, term_op("  two words "), &fetch).await;
        assert_eq!(out.query, "Show \"two words\"");
        let out = run(&tracker(), "Show", &items, exclude_op("-Hevc"), &fetch).await;
        assert_eq!(out.query, "Show -Hevc");
        let out = run(&tracker(), "Show", &items, exclude_op("a b"), &fetch).await;
        assert_eq!(out.query, "Show -\"a b\"");
        assert_eq!(
            *calls.lock().unwrap(),
            vec!["Show \"two words\"", "Show -Hevc", "Show -\"a b\""]
        );
    }

    #[tokio::test]
    async fn tracker_term_with_no_match_never_refetches() {
        let items = tui_items();
        let out = run(&tracker(), "Show", &items, term_op("zzz"), &never()).await;
        assert_eq!(
            (out.applied, out.reason.as_deref()),
            (false, Some("no_match"))
        );
    }

    #[tokio::test]
    async fn tracker_best_fit_refetches_with_the_best_query() {
        let items = vec![
            item("[Kumo-Subs] Show - 01 [1080p][HEVC]", "l1"),
            item("[Other] Show - 03 [720p]", "l3"),
        ];
        let want = best_fit_info("Show", &items, &BestfitConfig::default()).query;
        let (calls, fetch) = fetcher(Ok(items[..1].to_vec()));
        let out = run(&tracker(), "Show", &items, RefineOp::BestFit, &fetch).await;
        assert!(out.applied);
        assert_eq!(out.query, want);
        assert_eq!(*calls.lock().unwrap(), vec![want]);
    }

    #[tokio::test]
    async fn best_fit_is_a_noop_when_the_query_is_already_pinned() {
        let items = vec![
            item("[Kumo-Subs] Show - 01 [1080p][HEVC]", "l1"),
            item("[Other] Show - 03 [720p]", "l3"),
        ];
        let pinned = best_fit_info("Show", &items, &BestfitConfig::default()).query;
        let out = run(&tracker(), &pinned, &items, RefineOp::BestFit, &never()).await;
        assert_eq!((out.applied, out.reason.as_deref()), (false, Some("noop")));
    }

    #[tokio::test]
    async fn best_fit_on_empty_items_is_a_noop() {
        let out = run(&tracker(), "Show", &[], RefineOp::BestFit, &never()).await;
        assert_eq!((out.applied, out.reason.as_deref()), (false, Some("noop")));
    }

    #[tokio::test]
    async fn empty_terms_are_refused_on_both_kinds() {
        let items = tui_items();
        for ep in [tracker(), generic_rss()] {
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
        let out = run(&tracker(), "Show", &items, exclude_op(" -Hevc"), &fetch).await;
        assert_eq!(out.query, "Show -Hevc");
        let out = run(&tracker(), "Show", &items, exclude_op(" - "), &fetch).await;
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
        assert_eq!((out.previous_count, out.items.len()), (3, 2));
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
    async fn rss_tokens_match_extension_fields() {
        let mut items = vec![
            item("Show - 01 [1080p]", "l1"),
            item("Show - 02 [1080p]", "l2"),
            item("Show - 03 [1080p]", "l3"),
        ];
        items[0].linktype = "Torrent".into();
        items[1].linktype = "Torrent".into();
        items[2].linktype = "Magnet".into();
        let out = run(
            &keyed_ep(),
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
    async fn rss_term_matches_extension_fields() {
        let mut items = vec![
            item("Show - 01 [1080p]", "l1"),
            item("Show - 02 [1080p]", "l2"),
        ];
        items[0].linktype = "Torrent".into();
        let out = run(&keyed_ep(), "Show", &items, term_op("torrent"), &never()).await;
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
            item("[Kumo-Subs] Show - 01 [1080p][HEVC]", "l1"),
            item("[Kumo-Subs] Show - 02 [1080p][HEVC]", "l2"),
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

    #[test]
    fn client_side_is_anything_but_filtered_rss() {
        assert!(is_client_side(&generic_rss()));
        assert!(is_client_side(&keyed_ep()));
        assert!(!is_client_side(&tracker()));
    }

    // best fit info

    #[test]
    fn best_fit_info_describes_the_step() {
        let items = vec![
            item("[Kumo-Subs] Frieren - 02 [1080p][HEVC].mkv", "l1"),
            item("[Lumen] Frieren - 01 (720p) [AB].mkv", "l2"),
        ];
        let cfg = BestfitConfig::default();
        let best = best_fit_info("Frieren", &items, &cfg);
        assert!(best.available);
        assert!(items.contains(best.item.as_ref().unwrap()));
        assert!(!best.query.is_empty() && best.query != "Frieren");
        assert!(best.profile.contains(&"[Kumo-Subs]".to_string()));
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
