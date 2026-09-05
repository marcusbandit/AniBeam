//! Where every node of a closure is drawn: one row per prequel/sequel
//! chain, the chains ordered around the root, and whatever is left over
//! hung off the chain it belongs to.
//!
//! Carried from Electron's `franchiseLayout.ts`, `laneAssignment.ts` and
//! `FranchiseGraphView.tsx`, with the ghost copies and the side-story
//! frames dropped and the rows anchored on the schema's root rather than
//! on the earliest release.
//!
//! Nothing here is a matter of taste at runtime: the same closure always
//! gives the same positions. Every map and set that could otherwise leak
//! its iteration order into the result is walked through a sorted vector,
//! or through the closure's own node and edge order, which the walk fixed.

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use super::closure::{Closure, Edge, Node};

/// One card, in layout units. Electron's `.franchise-node`: the poster at
/// 180 by 270 and about 150 of body under it. The shell scales; the
/// positions below are in the same units, so a card and its slot agree.
pub const NODE_W: f64 = 180.0;
pub const NODE_H: f64 = 420.0;

/// The step from one column to the next along a chain's row.
pub const SPINE_X_GAP: f64 = 320.0;

/// The step from one row to the next.
pub const V_GAP: f64 = 500.0;

/// The narrowest slot a node may be given. Electron measured subtrees with
/// it; the row grid here never goes below one column, so it is carried as
/// the shell's minimum rather than used in the arithmetic.
pub const H_GAP: f64 = 240.0;

/// A node's place on the grid: the column, then the row. Both count from
/// nought, and the positions a caller sees are these times the two gaps.
pub type Cell = (i64, i64);

/// The formats that count as the printed original rather than the screen
/// adaptation of one. AniList's own set; a media type of MANGA covers
/// anything new it invents.
const PRINT_FORMATS: [&str; 5] = ["MANGA", "NOVEL", "LIGHT_NOVEL", "ONE_SHOT", "VISUAL_NOVEL"];

/// A run of nodes joined by SEQUEL edges, in the order they are watched.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Chain {
    pub members: HashSet<u64>,
    pub ordered: Vec<u64>,
}

fn is_print(n: Option<&&Node>) -> bool {
    let Some(n) = n else { return false };
    n.format
        .as_deref()
        .is_some_and(|f| PRINT_FORMATS.contains(&f))
        || n.media_type.as_deref() == Some("MANGA")
}

/// How structural a relation is. AniList tags one link two ways often
/// enough that a pair of nodes can carry both a SPIN_OFF and a SIDE_STORY;
/// the lower number is the one the graph draws.
fn priority(relation: &str) -> u8 {
    match relation {
        "SEQUEL" | "PREQUEL" => 0,
        "ADAPTATION" | "SOURCE" => 1,
        "SIDE_STORY" | "PARENT" => 2,
        "SPIN_OFF" => 3,
        "SUMMARY" | "COMPILATION" | "CONTAINS" => 4,
        "ALTERNATIVE" => 5,
        "CHARACTER" => 6,
        "OTHER" => 7,
        _ => 8,
    }
}

/// Every link drawn once, in the direction the reader expects: forwards in
/// time, and from the printed original to what was made of it.
///
/// Four passes, Electron's `dedupeReciprocalEdges`: the direction is fixed,
/// exact duplicates go, a reciprocal drops when the twin it names is
/// already there the other way round, and several relations on one ordered
/// pair collapse to the most structural of them.
pub fn canonical_edges(edges: &[Edge], nodes: &HashMap<u64, &Node>) -> Vec<Edge> {
    let e = |from, to, relation: &str| Edge {
        from,
        to,
        relation: relation.to_string(),
    };
    let normalized: Vec<Edge> = edges
        .iter()
        .map(|x| match x.relation.as_str() {
            "PARENT" => e(x.to, x.from, "SIDE_STORY"),
            "PREQUEL" => e(x.to, x.from, "SEQUEL"),
            "ALTERNATIVE" => {
                if x.from <= x.to {
                    x.clone()
                } else {
                    e(x.to, x.from, "ALTERNATIVE")
                }
            }
            "ADAPTATION" => {
                let (fp, tp) = (is_print(nodes.get(&x.from)), is_print(nodes.get(&x.to)));
                if !fp && tp {
                    e(x.to, x.from, "ADAPTATION")
                } else {
                    x.clone()
                }
            }
            "SOURCE" => {
                let (fp, tp) = (is_print(nodes.get(&x.from)), is_print(nodes.get(&x.to)));
                if !fp && tp {
                    e(x.to, x.from, "ADAPTATION")
                } else {
                    e(x.from, x.to, "ADAPTATION")
                }
            }
            _ => x.clone(),
        })
        .collect();

    let mut seen = HashSet::new();
    let uniqued: Vec<Edge> = normalized
        .into_iter()
        .filter(|x| seen.insert((x.from, x.to, x.relation.clone())))
        .collect();

    let present: HashSet<(u64, u64, String)> = uniqued
        .iter()
        .map(|x| (x.from, x.to, x.relation.clone()))
        .collect();
    let drops = |r: &str| match r {
        "SOURCE" => Some("ADAPTATION"),
        "PARENT" => Some("SIDE_STORY"),
        "PREQUEL" => Some("SEQUEL"),
        _ => None,
    };
    let after: Vec<Edge> = uniqued
        .into_iter()
        .filter(|x| {
            !drops(&x.relation)
                .is_some_and(|keep| present.contains(&(x.to, x.from, keep.to_string())))
        })
        .collect();

    let mut best: Vec<Edge> = Vec::new();
    let mut index: HashMap<(u64, u64), usize> = HashMap::new();
    for x in after {
        match index.get(&(x.from, x.to)) {
            Some(&i) => {
                if priority(&x.relation) < priority(&best[i].relation) {
                    best[i] = x;
                }
            }
            None => {
                index.insert((x.from, x.to), best.len());
                best.push(x);
            }
        }
    }
    best
}

/// The year a node sorts by. No year sorts last, which is where an entry
/// that has not been dated yet belongs.
fn year_of(nodes: &HashMap<u64, &Node>, id: u64) -> u64 {
    nodes
        .get(&id)
        .and_then(|n| n.year)
        .map_or(u64::MAX, u64::from)
}

/// Every run of two or more nodes joined by SEQUEL edges, each in watch
/// order. A lone node is no chain: it is placed later, beside whatever it
/// hangs off.
///
/// `nodes` fixes the order the components are found in, so the same
/// closure always gives the same rows.
pub fn chains(nodes: &[Node], edges: &[Edge]) -> Vec<Chain> {
    let by_id: HashMap<u64, &Node> = nodes.iter().map(|n| (n.anilist_id, n)).collect();
    let mut adjacent: HashMap<u64, Vec<u64>> = HashMap::new();
    for x in edges.iter().filter(|x| x.relation == "SEQUEL") {
        adjacent.entry(x.from).or_default().push(x.to);
        adjacent.entry(x.to).or_default().push(x.from);
    }

    let mut seen: HashSet<u64> = HashSet::new();
    let mut out: Vec<Chain> = Vec::new();
    for n in nodes {
        if !seen.insert(n.anilist_id) {
            continue;
        }
        let mut members: HashSet<u64> = HashSet::from([n.anilist_id]);
        let mut queue: VecDeque<u64> = VecDeque::from([n.anilist_id]);
        while let Some(id) = queue.pop_front() {
            for other in adjacent.get(&id).map(Vec::as_slice).unwrap_or_default() {
                if members.insert(*other) {
                    seen.insert(*other);
                    queue.push_back(*other);
                }
            }
        }
        if members.len() >= 2 {
            let ordered = watch_order(&members, edges, &by_id);
            out.push(Chain { members, ordered });
        }
    }
    out
}

/// One chain in watch order: Kahn's algorithm over the SEQUEL edges inside
/// it, with the ready set taken year first and id second so a chain that
/// branches still reads chronologically. Data saying a show is its own
/// sequel has no topological order at all, and falls back to the years.
fn watch_order(members: &HashSet<u64>, edges: &[Edge], nodes: &HashMap<u64, &Node>) -> Vec<u64> {
    let internal: Vec<&Edge> = edges
        .iter()
        .filter(|x| x.relation == "SEQUEL" && members.contains(&x.from) && members.contains(&x.to))
        .collect();

    let mut in_degree: HashMap<u64, usize> = members.iter().map(|id| (*id, 0)).collect();
    let mut out: HashMap<u64, Vec<u64>> = HashMap::new();
    for x in &internal {
        *in_degree.entry(x.to).or_insert(0) += 1;
        out.entry(x.from).or_default().push(x.to);
    }

    let by_year = |a: &u64, b: &u64| (year_of(nodes, *a), *a).cmp(&(year_of(nodes, *b), *b));
    let mut sorted: Vec<u64> = members.iter().copied().collect();
    sorted.sort_by(by_year);

    let mut ready: Vec<u64> = sorted
        .iter()
        .copied()
        .filter(|id| in_degree.get(id) == Some(&0))
        .collect();
    let mut ordered: Vec<u64> = Vec::new();
    while !ready.is_empty() {
        let next = ready.remove(0);
        ordered.push(next);
        for successor in out.get(&next).map(Vec::as_slice).unwrap_or_default() {
            let left = in_degree.entry(*successor).or_insert(0);
            *left = left.saturating_sub(1);
            if *left == 0 {
                ready.push(*successor);
                ready.sort_by(by_year);
            }
        }
    }
    if ordered.len() == members.len() {
        ordered
    } else {
        sorted
    }
}

/// How far apart two nodes read: a different medium first, then a
/// different format, then the years between them. Lower is closer, and an
/// id with no node at all is as far away as anything gets.
fn affinity(nodes: &HashMap<u64, &Node>, from: u64, to: u64) -> u64 {
    let (Some(src), Some(tgt)) = (nodes.get(&from), nodes.get(&to)) else {
        return u64::MAX;
    };
    let mut score = 0u64;
    if src.media_type != tgt.media_type {
        score += 1_000_000;
    }
    if src.format != tgt.format {
        score += 1_000;
    }
    let (sy, ty) = (
        u64::from(src.year.unwrap_or(0)),
        u64::from(tgt.year.unwrap_or(0)),
    );
    score + sy.abs_diff(ty)
}

/// One chain's link to another, as the row walk sees it.
struct Link {
    chain: usize,
    alternative: bool,
    weight: u64,
}

/// The rows, top to bottom. The root's chain leads, whatever it was
/// adapted from sits above it, and the rest are found by walking outwards
/// across every edge that is not a sequel: an alternative is slipped in
/// directly under the chain it is an alternative to, everything else is
/// appended, and a chain joined to nothing at all comes last.
///
/// With no root chain, because the root is a lone node, the chain the
/// reader is looking at anchors the rows instead.
pub fn order_rows(
    chains: Vec<Chain>,
    edges: &[Edge],
    nodes: &HashMap<u64, &Node>,
    root: u64,
    current: u64,
) -> Vec<Chain> {
    let mut chain_of: HashMap<u64, usize> = HashMap::new();
    for (i, c) in chains.iter().enumerate() {
        for id in &c.ordered {
            chain_of.insert(*id, i);
        }
    }

    let Some(root_chain) = chain_of.get(&root).copied() else {
        let mut chains = chains;
        if let Some(anchor) = chain_of.get(&current).copied().filter(|i| *i > 0) {
            let c = chains.remove(anchor);
            chains.insert(0, c);
        }
        return chains;
    };

    let mut links: Vec<Vec<Link>> = (0..chains.len()).map(|_| Vec::new()).collect();
    let mut seen: HashSet<(usize, usize, String)> = HashSet::new();
    for x in edges {
        if x.relation == "SEQUEL" || x.relation == "PREQUEL" {
            continue;
        }
        let (Some(&from), Some(&to)) = (chain_of.get(&x.from), chain_of.get(&x.to)) else {
            continue;
        };
        if from == to
            || seen.contains(&(to, from, x.relation.clone()))
            || !seen.insert((from, to, x.relation.clone()))
        {
            continue;
        }
        let alternative = x.relation == "ALTERNATIVE";
        links[from].push(Link {
            chain: to,
            alternative,
            weight: affinity(nodes, x.from, x.to),
        });
        links[to].push(Link {
            chain: from,
            alternative,
            weight: affinity(nodes, x.to, x.from),
        });
    }

    // What the root was made from goes above it. The second pair of
    // relations cannot survive canonicalisation, so in practice only the
    // first ever matches; both are kept so the rule reads whole.
    let is_parent_edge = |x: &Edge| {
        (x.to == root && (x.relation == "ADAPTATION" || x.relation == "SIDE_STORY"))
            || (x.from == root && (x.relation == "PARENT" || x.relation == "SOURCE"))
    };
    let mut placed: Vec<usize> = Vec::new();
    for x in edges.iter().filter(|x| is_parent_edge(x)) {
        let other = if x.from == root { x.to } else { x.from };
        let Some(&chain) = chain_of.get(&other) else {
            continue;
        };
        if chain != root_chain && !placed.contains(&chain) {
            placed.push(chain);
        }
    }

    let mut queue: VecDeque<usize> = placed.iter().copied().collect();
    placed.push(root_chain);
    queue.push_back(root_chain);
    let mut taken: HashSet<usize> = placed.iter().copied().collect();
    while let Some(cur) = queue.pop_front() {
        // Alternatives first, and the farthest of them first: each one is
        // slipped in directly under `cur`, so the last one placed, which
        // is the closest, ends up nearest to it.
        let mut connections: Vec<&Link> = links[cur].iter().collect();
        connections.sort_by(|a, b| match (a.alternative, b.alternative) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (true, true) => b.weight.cmp(&a.weight),
            (false, false) => Ordering::Equal,
        });
        for link in connections {
            if !taken.insert(link.chain) {
                continue;
            }
            match placed.iter().position(|i| *i == cur) {
                Some(at) if link.alternative => placed.insert(at + 1, link.chain),
                _ => placed.push(link.chain),
            }
            queue.push_back(link.chain);
        }
    }
    for i in 0..chains.len() {
        if taken.insert(i) {
            placed.push(i);
        }
    }

    let mut slots: Vec<Option<Chain>> = chains.into_iter().map(Some).collect();
    placed
        .into_iter()
        .filter_map(|i| slots.get_mut(i).and_then(Option::take))
        .collect()
}

/// Everything the chains left over, put where it belongs.
///
/// A node in no chain that is a side story of a placed one takes that
/// chain's column grid, centred on the targets it names, one row above
/// when that row is free and one row below when it is not. The rows around
/// it shift by one to make the space, and satellites that share a target
/// spread sideways into free columns.
///
/// A node joined to nothing placed takes a last row of its own, one node
/// per column: it is in the graph because something reached it, and the
/// reader should still see it.
pub fn satellites(
    nodes: &[Node],
    edges: &[Edge],
    chains: &[Chain],
    positions: &mut HashMap<u64, Cell>,
) {
    struct Plan {
        id: u64,
        target: u64,
        row: i64,
        above: bool,
        column: i64,
    }

    let mut chain_of: HashMap<u64, usize> = HashMap::new();
    for (i, c) in chains.iter().enumerate() {
        for id in &c.ordered {
            chain_of.insert(*id, i);
        }
    }
    let occupied =
        |positions: &HashMap<u64, Cell>, row: i64| positions.values().any(|(_, r)| *r == row);

    let mut plans: Vec<Plan> = Vec::new();
    let mut loose: Vec<u64> = Vec::new();
    for n in nodes {
        let id = n.anilist_id;
        if positions.contains_key(&id) {
            continue;
        }
        let mut targets: Vec<u64> = Vec::new();
        for x in edges.iter().filter(|x| x.relation == "SIDE_STORY") {
            let far = if x.from == id {
                x.to
            } else if x.to == id {
                x.from
            } else {
                continue;
            };
            if positions.contains_key(&far) && !targets.contains(&far) {
                targets.push(far);
            }
        }
        if targets.is_empty() {
            loose.push(id);
            continue;
        }

        // The chain holding most of the targets is the one this is a side
        // story of; a tie goes to the earlier chain, which is the higher
        // row.
        let mut counts: BTreeMap<usize, usize> = BTreeMap::new();
        for t in &targets {
            if let Some(&c) = chain_of.get(t) {
                *counts.entry(c).or_insert(0) += 1;
            }
        }
        let primary = counts
            .iter()
            .max_by_key(|(chain, count)| (**count, std::cmp::Reverse(**chain)))
            .map(|(chain, _)| *chain);
        let mut mine: Vec<u64> = targets
            .iter()
            .copied()
            .filter(|t| chain_of.get(t).copied() == primary)
            .collect();
        mine.sort_by_key(|t| positions.get(t).map_or(i64::MAX, |(c, _)| *c));
        let Some(&target) = mine.first() else {
            continue;
        };
        let Some(&(_, row)) = positions.get(&target) else {
            continue;
        };

        let columns: Vec<i64> = mine
            .iter()
            .filter_map(|t| positions.get(t))
            .map(|(c, _)| *c)
            .collect();
        let (low, high) = (
            columns.iter().copied().min().unwrap_or(0),
            columns.iter().copied().max().unwrap_or(0),
        );
        // The midpoint of an odd span lands between two columns; it takes
        // the higher of the two rather than half a column.
        let column = low + (high - low + 1) / 2;
        plans.push(Plan {
            id,
            target,
            row,
            above: !occupied(positions, row - 1),
            column,
        });
    }

    // One band per row per side, then every row shifted by the bands above
    // it. A row's own band is above it, so a row shifts for its own.
    let mut above: BTreeMap<i64, i64> = BTreeMap::new();
    let mut below: BTreeMap<i64, i64> = BTreeMap::new();
    for p in &plans {
        let band = if p.above { &mut above } else { &mut below };
        band.insert(p.row, 1);
    }
    for (_, row) in positions.values_mut() {
        let shift: i64 = above
            .iter()
            .filter(|(band, _)| *row >= **band)
            .map(|(_, u)| *u)
            .sum::<i64>()
            + below
                .iter()
                .filter(|(band, _)| *row > **band)
                .map(|(_, u)| *u)
                .sum::<i64>();
        *row += shift;
    }

    let mut taken: HashSet<Cell> = positions.values().copied().collect();
    for p in &plans {
        let Some(&(_, row)) = positions.get(&p.target) else {
            continue;
        };
        let row = if p.above { row - 1 } else { row + 1 };
        let cell = free_column(&taken, p.column, row);
        taken.insert(cell);
        positions.insert(p.id, cell);
    }

    if !loose.is_empty() {
        let row = positions
            .values()
            .map(|(_, r)| *r)
            .max()
            .map_or(0, |r| r + 1);
        for (column, id) in loose.into_iter().enumerate() {
            positions.insert(id, (i64::try_from(column).unwrap_or(i64::MAX), row));
        }
    }
}

/// The wanted column on that row, or the nearest free one either side of
/// it. Nothing on a row is ever placed on top of anything else.
fn free_column(taken: &HashSet<Cell>, column: i64, row: i64) -> Cell {
    if !taken.contains(&(column, row)) {
        return (column, row);
    }
    for step in 1..=24 {
        for candidate in [column + step, column - step] {
            if !taken.contains(&(candidate, row)) {
                return (candidate, row);
            }
        }
    }
    (column, row)
}

/// The words a card reads under its title. Electron's `RELATION_LABEL`.
fn label_of(relation: &str) -> String {
    match relation {
        "SEQUEL" => "Sequel",
        "PREQUEL" => "Prequel",
        "PARENT" => "Parent story",
        "SIDE_STORY" => "Side story",
        "SUMMARY" => "Summary",
        "ALTERNATIVE" => "Alternative",
        "SPIN_OFF" => "Spin-off",
        "COMPILATION" => "Compilation",
        "SOURCE" => "Source",
        "ADAPTATION" => "Adaptation",
        "CHARACTER" => "Shared characters",
        "CONTAINS" => "Contains",
        "OTHER" => "Other",
        other => return other.replace('_', " ").to_lowercase(),
    }
    .to_string()
}

/// The same relation seen from the other end.
fn reversed(relation: &str) -> &str {
    match relation {
        "SOURCE" => "ADAPTATION",
        "ADAPTATION" => "SOURCE",
        "PARENT" => "SIDE_STORY",
        "SIDE_STORY" => "PARENT",
        "PREQUEL" => "SEQUEL",
        "SEQUEL" => "PREQUEL",
        other => other,
    }
}

/// AniList tags an adaptation both ways round often enough that the name
/// has to be checked against what it points at: anything pointing at print
/// is a source, anything pointing at a screen is an adaptation.
fn canonical_relation(relation: &str, target: Option<&&Node>) -> String {
    match (relation, is_print(target)) {
        ("ADAPTATION", true) => "SOURCE".to_string(),
        ("SOURCE", false) => "ADAPTATION".to_string(),
        _ => relation.to_string(),
    }
}

/// What one node is to another, in the reader's words. Two nodes in one
/// chain read by their place in it; otherwise the edge between them is
/// turned to face the reader and named. Nothing joining them at all is
/// None, and the walk's own discovery label is the caller's fallback.
pub fn relation_label(
    current: u64,
    node: u64,
    chains: &[Chain],
    edges: &[Edge],
    nodes: &HashMap<u64, &Node>,
) -> Option<String> {
    if node == current {
        return None;
    }
    if let Some(chain) = chains
        .iter()
        .find(|c| c.members.contains(&current) && c.members.contains(&node))
    {
        let mine = chain.ordered.iter().position(|id| *id == current);
        let theirs = chain.ordered.iter().position(|id| *id == node);
        if let (Some(mine), Some(theirs)) = (mine, theirs) {
            return Some(if theirs < mine {
                "Prequel".to_string()
            } else {
                "Sequel".to_string()
            });
        }
    }
    let target = nodes.get(&node);
    if let Some(x) = edges.iter().find(|x| x.from == current && x.to == node) {
        return Some(label_of(&canonical_relation(&x.relation, target)));
    }
    if let Some(x) = edges.iter().find(|x| x.from == node && x.to == current) {
        return Some(label_of(&canonical_relation(reversed(&x.relation), target)));
    }
    None
}

/// Everything one read of the graph needs: where each node goes, the edges
/// as they are drawn, and what each node is to the series being read.
pub(crate) struct Plan {
    pub positions: Vec<(u64, f64, f64)>,
    pub edges: Vec<Edge>,
    pub labels: HashMap<u64, String>,
}

/// Where every node of the closure goes: `(id, x, y)` per node, sorted by
/// id, each node placed exactly once.
pub fn layout(closure: &Closure, current: u64) -> Vec<(u64, f64, f64)> {
    plan(closure, current).positions
}

pub(crate) fn plan(closure: &Closure, current: u64) -> Plan {
    let by_id: HashMap<u64, &Node> = closure.nodes.iter().map(|n| (n.anilist_id, n)).collect();
    let edges = canonical_edges(&closure.edges, &by_id);
    let rows = order_rows(
        chains(&closure.nodes, &edges),
        &edges,
        &by_id,
        closure.root,
        current,
    );

    let mut cells: HashMap<u64, Cell> = HashMap::new();
    for (row, chain) in rows.iter().enumerate() {
        for (column, id) in chain.ordered.iter().enumerate() {
            cells.insert(
                *id,
                (
                    i64::try_from(column).unwrap_or(i64::MAX),
                    i64::try_from(row).unwrap_or(i64::MAX),
                ),
            );
        }
    }
    satellites(&closure.nodes, &edges, &rows, &mut cells);

    let mut positions: Vec<(u64, f64, f64)> = closure
        .nodes
        .iter()
        .filter_map(|n| {
            let (column, row) = cells.get(&n.anilist_id)?;
            Some((
                n.anilist_id,
                *column as f64 * SPINE_X_GAP,
                *row as f64 * V_GAP,
            ))
        })
        .collect();
    positions.sort_by_key(|(id, _, _)| *id);

    let labels = labels(closure, current, &rows, &edges, &by_id);
    Plan {
        positions,
        edges,
        labels,
    }
}

/// A label for every node the reader is not standing on. What the edges
/// say comes first; a node with no edge to the current series takes the
/// label of the edge that discovered it in a walk outwards from there, so
/// a card three hops away still says what it is doing in the graph.
fn labels(
    closure: &Closure,
    current: u64,
    rows: &[Chain],
    edges: &[Edge],
    nodes: &HashMap<u64, &Node>,
) -> HashMap<u64, String> {
    let mut out: HashMap<u64, String> = HashMap::new();
    for n in &closure.nodes {
        if let Some(label) = relation_label(current, n.anilist_id, rows, edges, nodes) {
            out.insert(n.anilist_id, label);
        }
    }

    let mut seen: HashSet<u64> = HashSet::from([current]);
    let mut queue: VecDeque<u64> = VecDeque::from([current]);
    while let Some(id) = queue.pop_front() {
        for x in edges {
            let (far, relation) = if x.from == id {
                (x.to, x.relation.as_str())
            } else if x.to == id {
                (x.from, reversed(&x.relation))
            } else {
                continue;
            };
            if !seen.insert(far) {
                continue;
            }
            out.entry(far)
                .or_insert_with(|| label_of(&canonical_relation(relation, nodes.get(&far))));
            queue.push_back(far);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::*;
    use crate::franchise::closure::{Closure, Edge, Node};

    fn node(id: u64, media_type: &str, format: &str, year: Option<u32>) -> Node {
        Node {
            anilist_id: id,
            media_type: Some(media_type.to_string()),
            format: Some(format.to_string()),
            year,
            ..Node::default()
        }
    }

    fn anime(id: u64, year: u32) -> Node {
        node(id, "ANIME", "TV", Some(year))
    }

    fn manga(id: u64, year: u32) -> Node {
        node(id, "MANGA", "MANGA", Some(year))
    }

    fn e(from: u64, to: u64, relation: &str) -> Edge {
        Edge {
            from,
            to,
            relation: relation.to_string(),
        }
    }

    fn by_id(nodes: &[Node]) -> HashMap<u64, &Node> {
        nodes.iter().map(|n| (n.anilist_id, n)).collect()
    }

    /// The edges a pass produced, as `from->to:relation`, smallest first.
    fn shown(edges: &[Edge]) -> Vec<String> {
        let mut v: Vec<String> = edges
            .iter()
            .map(|x| format!("{}->{}:{}", x.from, x.to, x.relation))
            .collect();
        v.sort();
        v
    }

    fn chain_ids(chains: &[Chain]) -> Vec<Vec<u64>> {
        chains.iter().map(|c| c.ordered.clone()).collect()
    }

    // -- 1. Canonicalisation ------------------------------------------------

    #[test]
    fn a_parent_edge_flips_into_a_side_story_from_the_parent() {
        let nodes = [anime(1, 2000), anime(2, 2001)];
        let out = canonical_edges(&[e(2, 1, "PARENT")], &by_id(&nodes));
        assert_eq!(shown(&out), vec!["1->2:SIDE_STORY".to_string()]);
    }

    #[test]
    fn a_prequel_edge_flips_into_a_sequel_running_forwards() {
        let nodes = [anime(1, 2000), anime(2, 2001)];
        let out = canonical_edges(&[e(2, 1, "PREQUEL")], &by_id(&nodes));
        assert_eq!(shown(&out), vec!["1->2:SEQUEL".to_string()]);
    }

    #[test]
    fn an_alternative_edge_is_ordered_smaller_id_first() {
        let nodes = [anime(3, 2000), anime(5, 2001)];
        let out = canonical_edges(&[e(5, 3, "ALTERNATIVE")], &by_id(&nodes));
        assert_eq!(shown(&out), vec!["3->5:ALTERNATIVE".to_string()]);
    }

    #[test]
    fn an_adaptation_and_a_source_both_end_up_running_from_print_to_screen() {
        let nodes = [anime(1, 2005), manga(5, 2000)];
        let map = by_id(&nodes);

        assert_eq!(
            shown(&canonical_edges(&[e(1, 5, "ADAPTATION")], &map)),
            vec!["5->1:ADAPTATION".to_string()]
        );
        assert_eq!(
            shown(&canonical_edges(&[e(1, 5, "SOURCE")], &map)),
            vec!["5->1:ADAPTATION".to_string()]
        );
        assert_eq!(
            shown(&canonical_edges(&[e(5, 1, "SOURCE")], &map)),
            vec!["5->1:ADAPTATION".to_string()],
            "a source claim that already runs print to screen only changes its name"
        );
        assert_eq!(
            shown(&canonical_edges(&[e(5, 1, "ADAPTATION")], &map)),
            vec!["5->1:ADAPTATION".to_string()],
            "the canonical direction is left alone"
        );
    }

    #[test]
    fn a_reciprocal_pair_collapses_to_one_edge() {
        let nodes = [anime(1, 2000), anime(2, 2001)];
        let out = canonical_edges(&[e(1, 2, "SEQUEL"), e(2, 1, "PREQUEL")], &by_id(&nodes));
        assert_eq!(shown(&out), vec!["1->2:SEQUEL".to_string()]);
    }

    #[test]
    fn several_edges_on_one_pair_collapse_to_the_most_structural() {
        let nodes = [anime(1, 2000), anime(3, 2001)];
        let out = canonical_edges(
            &[e(1, 3, "SPIN_OFF"), e(1, 3, "SIDE_STORY")],
            &by_id(&nodes),
        );
        assert_eq!(shown(&out), vec!["1->3:SIDE_STORY".to_string()]);
    }

    // -- 2. Chains ----------------------------------------------------------

    #[test]
    fn a_chain_orders_by_its_sequels_whatever_order_the_edges_arrive_in() {
        let nodes = [anime(1, 2003), anime(2, 2001), anime(3, 2002)];
        let forwards = chains(&nodes, &[e(1, 2, "SEQUEL"), e(2, 3, "SEQUEL")]);
        let backwards = chains(&nodes, &[e(2, 3, "SEQUEL"), e(1, 2, "SEQUEL")]);

        assert_eq!(chain_ids(&forwards), vec![vec![1, 2, 3]]);
        assert_eq!(
            chain_ids(&backwards),
            vec![vec![1, 2, 3]],
            "the years disagree with the sequels; the sequels win"
        );
        assert_eq!(forwards[0].members, HashSet::from([1, 2, 3]));
    }

    #[test]
    fn a_lone_node_is_no_chain_at_all() {
        let nodes = [anime(1, 2000), anime(2, 2001), anime(9, 2002)];
        let out = chains(&nodes, &[e(1, 2, "SEQUEL")]);
        assert_eq!(chain_ids(&out), vec![vec![1, 2]]);
    }

    #[test]
    fn a_cycle_falls_back_to_year_then_id() {
        let nodes = [anime(1, 2005), anime(2, 2001), anime(3, 2003)];
        let out = chains(
            &nodes,
            &[e(1, 2, "SEQUEL"), e(2, 3, "SEQUEL"), e(3, 1, "SEQUEL")],
        );
        assert_eq!(chain_ids(&out), vec![vec![2, 3, 1]]);
    }

    // -- 3. Rows ------------------------------------------------------------

    #[test]
    fn the_root_chain_leads_with_its_source_above_and_its_alternative_below() {
        let nodes = [
            anime(1, 2005),
            anime(2, 2006),
            manga(10, 2000),
            manga(11, 2001),
            anime(20, 2008),
            anime(21, 2009),
        ];
        let map = by_id(&nodes);
        let edges = canonical_edges(
            &[
                e(1, 2, "SEQUEL"),
                e(10, 11, "SEQUEL"),
                e(20, 21, "SEQUEL"),
                e(10, 1, "ADAPTATION"),
                e(1, 20, "ALTERNATIVE"),
            ],
            &map,
        );
        let rows = order_rows(chains(&nodes, &edges), &edges, &map, 1, 1);
        assert_eq!(
            chain_ids(&rows),
            vec![vec![10, 11], vec![1, 2], vec![20, 21]]
        );
    }

    #[test]
    fn with_no_root_chain_the_current_series_chain_is_the_anchor() {
        let nodes = [
            anime(1, 2000),
            anime(2, 2001),
            anime(3, 2002),
            anime(4, 2003),
        ];
        let map = by_id(&nodes);
        let edges = [e(1, 2, "SEQUEL"), e(3, 4, "SEQUEL")];
        let rows = order_rows(chains(&nodes, &edges), &edges, &map, 99, 3);
        assert_eq!(chain_ids(&rows), vec![vec![3, 4], vec![1, 2]]);
    }

    // -- 4. Labels ----------------------------------------------------------

    #[test]
    fn two_nodes_in_one_chain_read_prequel_or_sequel_by_their_order() {
        let nodes = [anime(1, 2000), anime(2, 2001), anime(3, 2002)];
        let map = by_id(&nodes);
        let edges = [e(1, 2, "SEQUEL"), e(2, 3, "SEQUEL")];
        let cs = chains(&nodes, &edges);

        assert_eq!(
            relation_label(2, 1, &cs, &edges, &map).as_deref(),
            Some("Prequel")
        );
        assert_eq!(
            relation_label(2, 3, &cs, &edges, &map).as_deref(),
            Some("Sequel")
        );
        assert_eq!(
            relation_label(2, 2, &cs, &edges, &map),
            None,
            "a node has no relation to itself"
        );
    }

    #[test]
    fn a_direct_edge_is_labelled_after_canonicalising_against_the_target() {
        let nodes = [anime(1, 2005), manga(5, 2000), anime(7, 2007)];
        let map = by_id(&nodes);

        let to_print = [e(1, 5, "ADAPTATION")];
        assert_eq!(
            relation_label(1, 5, &[], &to_print, &map).as_deref(),
            Some("Source")
        );

        let to_screen = [e(1, 7, "SOURCE")];
        assert_eq!(
            relation_label(1, 7, &[], &to_screen, &map).as_deref(),
            Some("Adaptation")
        );
    }

    #[test]
    fn a_reverse_edge_is_turned_round_first() {
        let nodes = [anime(1, 2005), anime(7, 2007)];
        let map = by_id(&nodes);
        assert_eq!(
            relation_label(1, 7, &[], &[e(7, 1, "PARENT")], &map).as_deref(),
            Some("Side story")
        );
        assert_eq!(
            relation_label(1, 7, &[], &[e(7, 1, "SIDE_STORY")], &map).as_deref(),
            Some("Parent story")
        );
    }

    #[test]
    fn an_unknown_relation_reads_as_its_own_words() {
        let nodes = [anime(1, 2005), anime(7, 2007)];
        let map = by_id(&nodes);
        assert_eq!(
            relation_label(1, 7, &[], &[e(1, 7, "WEIRD_NEW_TYPE")], &map).as_deref(),
            Some("weird new type")
        );
        assert_eq!(
            relation_label(1, 7, &[], &[e(1, 7, "CHARACTER")], &map).as_deref(),
            Some("Shared characters")
        );
        assert_eq!(
            relation_label(1, 7, &[], &[], &map),
            None,
            "nothing joins them at all"
        );
    }

    // -- 5. Positions -------------------------------------------------------

    /// A closure as the walk would have left it: whatever `nodes` and
    /// `edges` say, with every node a member and nothing owed.
    fn closure_of(root: u64, nodes: Vec<Node>, edges: Vec<Edge>, boundary: &[u64]) -> Closure {
        let boundary: HashSet<u64> = boundary.iter().copied().collect();
        let members: HashSet<u64> = nodes
            .iter()
            .map(|n| n.anilist_id)
            .filter(|id| !boundary.contains(id))
            .collect();
        Closure {
            root,
            nodes,
            edges,
            members,
            boundary,
            complete: true,
            owed: Vec::new(),
        }
    }

    #[test]
    fn positions_step_by_the_column_and_the_row_gap() {
        let closure = closure_of(
            1,
            vec![
                anime(1, 2000),
                anime(2, 2001),
                anime(3, 2002),
                anime(50, 1999),
            ],
            vec![e(1, 2, "SEQUEL"), e(2, 3, "SEQUEL"), e(1, 50, "CHARACTER")],
            &[50],
        );
        let mut placed = layout(&closure, 1);
        placed.sort_by_key(|(id, _, _)| *id);
        assert_eq!(
            placed,
            vec![
                (1, 0.0, 0.0),
                (2, 320.0, 0.0),
                (3, 640.0, 0.0),
                (50, 0.0, 500.0)
            ]
        );
    }

    #[test]
    fn every_node_in_the_closure_gets_exactly_one_position() {
        let closure = closure_of(
            1,
            vec![
                anime(1, 2000),
                anime(2, 2001),
                manga(10, 1998),
                anime(20, 2004),
                anime(21, 2005),
                anime(30, 2006),
                anime(40, 2007),
            ],
            vec![
                e(1, 2, "SEQUEL"),
                e(20, 21, "SEQUEL"),
                e(10, 1, "ADAPTATION"),
                e(1, 20, "ALTERNATIVE"),
                e(1, 30, "SIDE_STORY"),
            ],
            &[],
        );
        let placed = layout(&closure, 1);

        let mut ids: Vec<u64> = placed.iter().map(|(id, _, _)| *id).collect();
        ids.sort_unstable();
        assert_eq!(
            ids,
            vec![1, 2, 10, 20, 21, 30, 40],
            "every node once: 30 is a satellite, 40 hangs off nothing"
        );
        let cells: HashSet<(i64, i64)> = placed
            .iter()
            .map(|(_, x, y)| ((x / SPINE_X_GAP) as i64, (y / V_GAP) as i64))
            .collect();
        assert_eq!(cells.len(), placed.len(), "no two nodes share a cell");
    }

    #[test]
    fn a_satellite_takes_the_row_beside_its_chain_and_shares_no_column() {
        let mut positions: HashMap<u64, Cell> =
            HashMap::from([(1, (0, 0)), (2, (1, 0)), (3, (0, 1)), (4, (1, 1))]);
        let nodes = [
            anime(1, 2000),
            anime(2, 2001),
            anime(3, 2002),
            anime(4, 2003),
            anime(10, 2004),
            anime(11, 2005),
            anime(12, 2006),
        ];
        let edges = [
            e(1, 10, "SIDE_STORY"),
            e(1, 11, "SIDE_STORY"),
            e(3, 12, "SIDE_STORY"),
        ];
        let cs = chains(&nodes, &[e(1, 2, "SEQUEL"), e(3, 4, "SEQUEL")]);
        satellites(&nodes, &edges, &cs, &mut positions);

        assert_eq!(
            positions[&10].1, 0,
            "the row above the top chain is free, so the satellite takes it"
        );
        assert_eq!(positions[&1].1, 1, "the chain shifted down to make room");
        assert_eq!(positions[&11].1, 0);
        assert_ne!(
            positions[&10].0, positions[&11].0,
            "two satellites on one target spread across columns"
        );
        assert_eq!(
            positions[&12].1,
            positions[&3].1 + 1,
            "the row above the second chain is taken, so it hangs below"
        );
    }
}
