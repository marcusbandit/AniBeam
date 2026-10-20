//! Which of a series' titles a shell shows. Carried from
//! src/renderer/contexts/TitleLanguageContext.tsx, plus Native: three
//! languages, three orders, the folder name as the title of last resort. The
//! native title is only ever first, for Native, and never a fallback.

use crate::contract::TitleLanguage;

/// An empty or blank string counts as absent, the way JavaScript's `||`
/// treated `""`.
fn present(s: Option<&str>) -> Option<&str> {
    s.filter(|v| !v.trim().is_empty())
}

/// Romaji: romaji, english, folder. English: english, romaji, folder.
/// Native: native, romaji, english, folder.
pub fn resolve(
    lang: TitleLanguage,
    romaji: Option<&str>,
    english: Option<&str>,
    native: Option<&str>,
    folder: &str,
) -> String {
    let (first, second) = match lang {
        TitleLanguage::Romaji => (romaji, english),
        TitleLanguage::English => (english, romaji),
        TitleLanguage::Native => match present(native) {
            Some(n) => return n.to_string(),
            None => (romaji, english),
        },
    };
    present(first)
        .or(present(second))
        .unwrap_or(folder)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_language_has_its_own_order() {
        let (r, e) = (
            Some("Sousou no Frieren"),
            Some("Frieren: Beyond Journey's End"),
        );
        assert_eq!(
            resolve(TitleLanguage::Romaji, r, e, None, "folder"),
            "Sousou no Frieren"
        );
        assert_eq!(
            resolve(TitleLanguage::English, r, e, None, "folder"),
            "Frieren: Beyond Journey's End"
        );
        assert_eq!(
            resolve(TitleLanguage::English, r, None, None, "folder"),
            "Sousou no Frieren"
        );
        assert_eq!(
            resolve(TitleLanguage::Romaji, None, e, None, "folder"),
            "Frieren: Beyond Journey's End"
        );
    }

    #[test]
    fn native_goes_first_and_falls_back_to_romaji() {
        let (r, e, n) = (
            Some("Kino no Tabi"),
            Some("Kino's Journey"),
            Some("キノの旅"),
        );
        assert_eq!(
            resolve(TitleLanguage::Native, r, e, n, "folder"),
            "キノの旅"
        );
        assert_eq!(
            resolve(TitleLanguage::Native, r, e, None, "folder"),
            "Kino no Tabi"
        );
        assert_eq!(
            resolve(TitleLanguage::Native, None, e, Some(" "), "folder"),
            "Kino's Journey"
        );
    }

    #[test]
    fn the_folder_name_is_the_title_of_last_resort() {
        assert_eq!(
            resolve(TitleLanguage::Romaji, None, None, None, "Some Folder"),
            "Some Folder"
        );
        assert_eq!(
            resolve(
                TitleLanguage::English,
                Some(""),
                Some("  "),
                None,
                "Some Folder"
            ),
            "Some Folder"
        );
    }
}
