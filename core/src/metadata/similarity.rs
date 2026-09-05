//! Tokenised Dice similarity, carried from Electron's
//! `src/main/utils/titleSimilarity.ts` rule for rule.
//!
//! The tokenisation is deliberately minimal: lower case, every character
//! that is not a letter or a digit becomes a space, split on whitespace.
//! Nothing strips particles, stopwords, brackets or release tags. A folder
//! name goes in verbatim, because the user keeps folder names clean and
//! that is the contract; the cleaning that does happen is the file
//! classifier's, not this.
//!
//! Dice over tokens rather than a character metric, because character
//! metrics are exactly what "Otaku ni Yasashii Gal wa Inai" against
//! "Wotaku ni Koi wa Muzukashii" fools: they share the substring "otaku"
//! and a good deal of their letters, while their token sets overlap only
//! on the particles. Dice rather than Jaccard, because Jaccard punishes a
//! size mismatch too hard for a short title: "Frieren" against "Sousou no
//! Frieren" is 0.50 under Dice and 0.33 under Jaccard, and the first of
//! those is the honest reading.

use std::collections::HashSet;

pub fn tokenize(s: &str) -> Vec<String> {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

/// `2 * |A n B| / (|A| + |B|)` over the two token sets. Duplicated tokens
/// count once: the sets are what is compared, so a repeated word cannot
/// buy a candidate a better score.
pub fn dice(a: &[String], b: &[String]) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let sa: HashSet<&String> = a.iter().collect();
    let sb: HashSet<&String> = b.iter().collect();
    let inter = sa.iter().filter(|t| sb.contains(*t)).count();
    let denom = sa.len() + sb.len();
    if denom == 0 { 0.0 } else { 2.0 * inter as f64 / denom as f64 }
}

/// The best Dice score of the query against every candidate title. 0 means
/// no overlap at all, and an empty query scores 0 against anything rather
/// than matching everything.
pub fn best_title_score(query: &str, candidates: &[Option<&str>]) -> f64 {
    let q = tokenize(query);
    if q.is_empty() {
        return 0.0;
    }
    candidates.iter().flatten().map(|t| dice(&q, &tokenize(t))).fold(0.0, f64::max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_and_dice() {
        assert_eq!(tokenize("Sousou no Frieren (2023)"), vec!["sousou", "no", "frieren", "2023"]);
        let s = best_title_score("Frieren", &[Some("Sousou no Frieren"), Some("Frieren: Beyond Journey's End")]);
        assert!((s - 0.5).abs() < 1e-9, "{s}");
        let s = best_title_score("Otaku ni Yasashii Gal wa Inai", &[Some("Wotaku ni Koi wa Muzukashii")]);
        assert!(s < 0.4, "{s}");
        assert_eq!(best_title_score("", &[Some("x")]), 0.0);
        assert_eq!(best_title_score("x", &[None]), 0.0);
        assert!((best_title_score("Show Name", &[Some("show name")]) - 1.0).abs() < 1e-9);
    }
}
