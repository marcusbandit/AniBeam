//! An AniList description as a tile and a page say it: plain text, without
//! the source credit or the release note that trails most of them.

/// The description as plain text, cut where a source credit or a note
/// starts: those are about the release, not the story.
pub fn story(description: &str) -> String {
    let text = crate::html::plain_text(description);
    let cut = ["(Source:", "[Written by", "Note:", "(Note:"]
        .iter()
        .filter_map(|m| text.find(m))
        .min();
    text[..cut.unwrap_or(text.len())].trim().to_string()
}

/// The first sentence of the story, at most 140 bytes, cut at a word.
pub fn line(description: &str) -> String {
    let text = story(description)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let first = sentences(&text).first().copied().unwrap_or("");
    cut(first, 140)
}

/// Cut at a word boundary at most `max` bytes in, with an ellipsis when
/// anything was cut.
fn cut(text: &str, max: usize) -> String {
    if text.len() <= max {
        return text.to_owned();
    }
    let end = text
        .char_indices()
        .map(|(i, _)| i)
        .take_while(|i| *i <= max)
        .last()
        .unwrap_or(0);
    let head = &text[..end];
    let head = head.rfind(' ').map_or(head, |sp| &head[..sp]);
    format!("{}...", head.trim_end_matches([',', ';', ':']))
}

/// Sentences, without breaking on "Dr." or "Mr." or an initial like "J. R.".
fn sentences(text: &str) -> Vec<&str> {
    const SHORT: [&str; 8] = ["Dr", "Mr", "Mrs", "Ms", "St", "Jr", "vs", "Prof"];
    let mut out = Vec::new();
    let mut start = 0;
    let bytes = text.as_bytes();
    for (i, ch) in text.char_indices() {
        if !matches!(ch, '.' | '!' | '?') || bytes.get(i + 1).is_some_and(|b| *b != b' ') {
            continue;
        }
        let word = text[start..i].rsplit(' ').next().unwrap_or("");
        if ch == '.' && (SHORT.contains(&word) || word.chars().count() == 1) {
            continue;
        }
        out.push(text[start..=i].trim());
        start = i + 1;
    }
    if start < text.len() && !text[start..].trim().is_empty() {
        out.push(text[start..].trim());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_line_is_the_first_sentence_without_the_source_note() {
        let d = "Dr. Kim finds a door.<br><br>It leads nowhere. (Source: a site)";
        assert_eq!(line(d), "Dr. Kim finds a door.");
        assert_eq!(story(d), "Dr. Kim finds a door.\n\nIt leads nowhere.");
    }

    #[test]
    fn a_long_sentence_is_cut_at_a_word() {
        let d = "word ".repeat(60);
        let l = line(&d);
        assert!(l.len() <= 143 && l.ends_with("word..."), "{l}");
    }
}
