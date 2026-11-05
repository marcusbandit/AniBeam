//! HTML as text a person reads. The providers hand some of their text over as HTML:
//! AniList's descriptions and TVmaze's summaries. The core turns it into plain text in
//! paragraphs before any shell sees it, so no shell strips tags or decodes entities on its
//! own (#450).

/// HTML as plain text: tags gone, entities decoded, and one paragraph per
/// `<p>` or `<br>`, paragraphs apart by a blank line.
///
/// Only a `<` followed by a letter, a `/` or a `!` opens a tag, so "a < b"
/// and "1 <2" are text. Entities are decoded in the same single pass, so
/// an escaped one (`&amp;lt;`) is decoded once and never twice, and one
/// this cannot name is left as written.
pub fn plain_text(html: &str) -> String {
    let mut text = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(at) = rest.find(['<', '&']) {
        text.push_str(&rest[..at]);
        rest = &rest[at..];
        if rest.starts_with('<') {
            let opens_tag = rest[1..]
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic() || c == '/' || c == '!');
            match rest.find('>').filter(|_| opens_tag) {
                Some(close) => {
                    let name = rest[1..close]
                        .trim_start_matches('/')
                        .split(|c: char| c.is_whitespace() || c == '/')
                        .next()
                        .unwrap_or_default()
                        .to_ascii_lowercase();
                    if name == "p" || name == "br" {
                        text.push('\n');
                    }
                    rest = &rest[close + 1..];
                }
                None => {
                    text.push('<');
                    rest = &rest[1..];
                }
            }
        } else {
            match decode_entity(rest) {
                Some((c, len)) => {
                    text.push(c);
                    rest = &rest[len..];
                }
                None => {
                    text.push('&');
                    rest = &rest[1..];
                }
            }
        }
    }
    text.push_str(rest);
    text.lines()
        .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// The entity `s` starts with, as its character and the bytes it spans:
/// `&#NNN;`, `&#xHH;`, or one of the names TVmaze's summaries and AniList's
/// descriptions use.
fn decode_entity(s: &str) -> Option<(char, usize)> {
    // The longest name below is eight letters; a numeric one is at most
    // `#x10FFFF`.
    let end = s
        .get(1..)?
        .char_indices()
        .take(10)
        .find(|(_, c)| *c == ';')?
        .0
        + 1;
    let body = &s[1..end];
    let c = if let Some(num) = body.strip_prefix('#') {
        let value = match num.strip_prefix(['x', 'X']) {
            Some(hex) => u32::from_str_radix(hex, 16).ok()?,
            None => num.parse::<u32>().ok()?,
        };
        char::from_u32(value).filter(|c| *c != '\0')?
    } else {
        named_entity(body)?
    };
    Some((c, end + 1))
}

fn named_entity(name: &str) -> Option<char> {
    Some(match name {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "nbsp" => ' ',
        "hellip" => '\u{2026}',
        "mdash" => '\u{2014}',
        "ndash" => '\u{2013}',
        "lsquo" => '\u{2018}',
        "rsquo" => '\u{2019}',
        "sbquo" => '\u{201a}',
        "ldquo" => '\u{201c}',
        "rdquo" => '\u{201d}',
        "bdquo" => '\u{201e}',
        "laquo" => '\u{ab}',
        "raquo" => '\u{bb}',
        "bull" => '\u{2022}',
        "middot" => '\u{b7}',
        "deg" => '\u{b0}',
        "copy" => '\u{a9}',
        "reg" => '\u{ae}',
        "trade" => '\u{2122}',
        "times" => '\u{d7}',
        "eacute" => 'é',
        "Eacute" => 'É',
        "egrave" => 'è',
        "Egrave" => 'È',
        "ecirc" => 'ê',
        "euml" => 'ë',
        "aacute" => 'á',
        "Aacute" => 'Á',
        "agrave" => 'à',
        "acirc" => 'â',
        "auml" => 'ä',
        "Auml" => 'Ä',
        "aring" => 'å',
        "atilde" => 'ã',
        "iacute" => 'í',
        "igrave" => 'ì',
        "icirc" => 'î',
        "iuml" => 'ï',
        "oacute" => 'ó',
        "Oacute" => 'Ó',
        "ograve" => 'ò',
        "ocirc" => 'ô',
        "ouml" => 'ö',
        "Ouml" => 'Ö',
        "otilde" => 'õ',
        "oslash" => 'ø',
        "uacute" => 'ú',
        "ugrave" => 'ù',
        "ucirc" => 'û',
        "uuml" => 'ü',
        "Uuml" => 'Ü',
        "ntilde" => 'ñ',
        "Ntilde" => 'Ñ',
        "ccedil" => 'ç',
        "Ccedil" => 'Ç',
        "szlig" => 'ß',
        "aelig" => 'æ',
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_loses_its_tags_and_keeps_its_paragraphs() {
        assert_eq!(
            plain_text("<p>One <b>bold</b> &amp; plain.</p><p>Two<br/>Three</p>"),
            "One bold & plain.\n\nTwo\n\nThree"
        );
        assert_eq!(plain_text(""), "");
        assert_eq!(plain_text("a < b"), "a < b");
        assert_eq!(plain_text("Tom&#39;s &quot;show&quot;"), "Tom's \"show\"");
    }

    /// AniList's descriptions: `<br>` pairs between paragraphs, `<i>` around the source,
    /// and entities, numeric ones included (#450).
    #[test]
    fn an_anilist_description_reads_as_paragraphs() {
        assert_eq!(
            plain_text(
                "A mage &amp; her friends.<br><br>Lessons &mdash; quiet.<br>\n<br>\n(Source: the publisher&#039;s note)"
            ),
            "A mage & her friends.\n\nLessons \u{2014} quiet.\n\n(Source: the publisher's note)"
        );
    }
}
