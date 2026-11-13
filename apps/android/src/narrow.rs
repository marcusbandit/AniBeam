//! The first run on a narrow screen (#874): the outer display of a folding phone gives a page
//! about 320 px. Laid out headless by Slint's testing backend with the app's own fonts, the
//! sites step must keep every element inside the page, so nothing scrolls sideways and no
//! button is cut off at the edge.

use i_slint_backend_testing::ElementHandle;
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};

use crate::tr::{Arg, Lang, tr_in};
use crate::{AppData, AppWindow, AuthRow, EndpointRow, RepoData, RepoRow, Tr};

const WIDTH: f32 = 320.0;

fn english(id: &str, args: &[(&str, &str)]) -> String {
    let args: Vec<(&str, Arg)> = args.iter().map(|&(k, v)| (k, Arg::from(v))).collect();
    tr_in(Lang::En, id, &args)
}

fn model<T: Clone + 'static>(rows: Vec<T>) -> ModelRc<T> {
    ModelRc::new(VecModel::from(rows))
}

/// The sites step with every button row it can show at once: the checks above the repos, a
/// repo, another waiting to be trusted, an installed site being edited, one with an update,
/// and one offered asking for a token.
fn sites_step() -> AppWindow {
    i_slint_backend_testing::init_no_event_loop();
    let page = AppWindow::new().unwrap();
    page.window()
        .set_size(slint::LogicalSize::new(WIDTH, 2400.0));
    page.global::<Tr>().on_lookup(|id, _rev, args| {
        let flat: Vec<SharedString> = args.iter().collect();
        let pairs: Vec<(&str, &str)> = flat
            .as_chunks::<2>()
            .0
            .iter()
            .map(|[k, v]| (k.as_str(), v.as_str()))
            .collect();
        english(&id, &pairs).into()
    });
    let app = page.global::<AppData>();
    app.set_onboarding(true);
    app.set_step(6);

    let repos = page.global::<RepoData>();
    repos.set_repos(model(vec![RepoRow {
        name: "Community endpoints".into(),
        url: "https://repo.example/index.json".into(),
        fingerprint: "SHA256:4f9a1c27b3e8d05f6a2c9e1b7d4f0a3c8e6b2d9f1a7c5e3b".into(),
        nsfw: true,
        error: "".into(),
    }]));
    repos.set_on_count(2);
    repos.set_previewing(true);
    repos.set_preview_name("Another endpoint repo".into());
    repos.set_preview_url("https://another-repo.example/endpoints/index.json".into());
    repos.set_preview_fingerprint("SHA256:0b6e2f9d4a8c1e7b3f5d9a2c6e0b4f8d".into());
    repos.set_preview_line("Twelve sites, two of them 18+".into());
    repos.set_preview_nsfw(true);

    let site = |name: &str, installed: bool| EndpointRow {
        name: name.into(),
        line: "tracker.example  rss  en".into(),
        installed,
        enabled: installed,
        nsfw: false,
        note: "".into(),
        update: "".into(),
        editable: installed,
        warn: false,
        health: "Answered in 240 ms".into(),
        health_ok: true,
    };
    let updated = EndpointRow {
        update: english("repos-update-to", &[("version", "2")]).into(),
        ..site("Second site", true)
    };
    repos.set_endpoints(model(vec![
        site("First site", true),
        updated,
        site("Third site", false),
    ]));
    repos.set_editing_index(0);
    repos.set_edit_name("First site".into());
    repos.set_edit_url("https://tracker.example/rss?passkey=0123456789abcdef".into());
    repos.set_edit_keys("API key".into());
    repos.set_asking(true);
    repos.set_asking_index(2);
    repos.set_asking_note("Third site needs a token to search.".into());
    repos.set_fields(model(vec![AuthRow {
        label: "RSS token".into(),
        help: "Under your account's settings.".into(),
        help_url: "https://tracker.example/account/settings".into(),
        secret: true,
    }]));
    // The window's size reaches Fold through a change handler, which an event loop would run.
    slint::platform::update_timers_and_animations();
    page
}

fn rect(e: &ElementHandle) -> (f32, f32, f32, f32) {
    let p = e.absolute_position();
    let s = e.size();
    (p.x, p.y, p.x + s.width, p.y + s.height)
}

#[test]
fn the_sites_step_fits_a_narrow_screen() {
    let page = sites_step();
    let onboarding: Vec<_> =
        ElementHandle::find_by_element_type_name(&page, "Onboarding").collect();
    assert_eq!(onboarding.len(), 1, "the first run is not on screen");
    // Laying an element out can make the branches under it, so measure until nothing new
    // turns up.
    let mut all = onboarding[0].query_descendants().find_all();
    loop {
        all.iter().for_each(|e| {
            rect(e);
        });
        let again = onboarding[0].query_descendants().find_all();
        if again.len() == all.len() {
            break;
        }
        all = again;
    }
    let page_rect = rect(&onboarding[0]);
    let outside: Vec<String> = all
        .iter()
        .filter(|e| {
            let (left, _, right, _) = rect(e);
            left < page_rect.0 - 0.5 || right > page_rect.2 + 0.5
        })
        .map(|e| {
            format!(
                "{} {} {:?} at {:?}",
                e.type_name().unwrap_or_default(),
                e.id().unwrap_or_default(),
                e.accessible_label().unwrap_or_default(),
                rect(e),
            )
        })
        .collect();
    assert!(
        outside.is_empty(),
        "outside the page at {page_rect:?}:\n{}",
        outside.join("\n")
    );

    // Every button row is there, each label inside the page and clear of every other one:
    // a row that wraps onto a second line pushes what follows it down.
    let labels = [
        english("repos-check-repos", &[]),
        english("repos-check-sites", &[]),
        english("repos-trust", &[]),
        english("common-save", &[]),
        english("repos-change-its", &[("keys", "API key")]),
        english("repos-update-to", &[("version", "2")]),
        english("common-remove", &[]),
        english("repos-install", &[]),
        english(
            "repos-open-link",
            &[("url", "https://tracker.example/account/settings")],
        ),
    ];
    for label in &labels {
        let found: Vec<_> = all
            .iter()
            .filter(|e| e.accessible_label().as_deref() == Some(label.as_str()))
            .collect();
        assert!(!found.is_empty(), "no {label:?} on the page");
    }
    let texts: Vec<_> = all
        .iter()
        .filter(|e| e.accessible_label().is_some_and(|l| !l.is_empty()))
        .filter(|e| e.size().width > 0.0 && e.size().height > 0.0)
        .collect();
    for (i, a) in texts.iter().enumerate() {
        for b in &texts[i + 1..] {
            let (al, at, ar, ab) = rect(a);
            let (bl, bt, br, bb) = rect(b);
            let apart = ar <= bl + 0.5 || br <= al + 0.5 || ab <= bt + 0.5 || bb <= at + 0.5;
            assert!(
                apart,
                "{:?} at {:?} overlaps {:?} at {:?}",
                a.accessible_label(),
                rect(a),
                b.accessible_label(),
                rect(b),
            );
        }
    }
}
