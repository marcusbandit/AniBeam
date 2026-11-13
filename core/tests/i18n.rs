//! The language `tr` speaks is one setting for the whole process, so the test that changes
//! it runs alone in this binary, where no other test can read it mid-switch.

use anibeam_core::i18n::{Arg, Lang, lang, set_lang, tr};

#[test]
fn set_lang_switches_what_tr_speaks() {
    let args = [("count", Arg::from(2))];
    assert_eq!(lang(), Lang::En);
    set_lang(Lang::Da);
    assert_eq!(lang(), Lang::Da);
    assert_eq!(tr("count-shows", &args), "2 serier");
    set_lang(Lang::En);
    assert_eq!(tr("count-shows", &args), "2 shows");
}
