//! Which of a series' titles a shell shows. Carried from
//! src/renderer/contexts/TitleLanguageContext.tsx: two languages, two
//! orders, the folder name as the title of last resort. The native title is
//! carried on every card but is never a fallback.

use crate::contract::TitleLanguage;

/// An empty or blank string counts as absent, the way JavaScript's `||`
/// treated `""`.
fn present(s: Option<&str>) -> Option<&str> {
    s.filter(|v| !v.trim().is_empty())
}

/// Romaji: romaji, english, folder. English: english, romaji, folder.
pub fn resolve(lang: TitleLanguage, romaji: Option<&str>, english: Option<&str>, folder: &str) -> String {
    let (first, second) = match lang {
        TitleLanguage::Romaji => (romaji, english),
        TitleLanguage::English => (english, romaji),
    };
    present(first).or(present(second)).unwrap_or(folder).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_language_has_its_own_order() {
        let (r, e) = (Some("Sousou no Frieren"), Some("Frieren: Beyond Journey's End"));
        assert_eq!(resolve(TitleLanguage::Romaji, r, e, "folder"), "Sousou no Frieren");
        assert_eq!(resolve(TitleLanguage::English, r, e, "folder"), "Frieren: Beyond Journey's End");
        assert_eq!(resolve(TitleLanguage::English, r, None, "folder"), "Sousou no Frieren");
        assert_eq!(resolve(TitleLanguage::Romaji, None, e, "folder"), "Frieren: Beyond Journey's End");
    }

    #[test]
    fn the_folder_name_is_the_title_of_last_resort() {
        assert_eq!(resolve(TitleLanguage::Romaji, None, None, "Some Folder"), "Some Folder");
        assert_eq!(resolve(TitleLanguage::English, Some(""), Some("  "), "Some Folder"), "Some Folder");
    }
}
