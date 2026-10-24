//! Endpoint repos on the phone (#143): the repo address pasted in, its fingerprint shown
//! before it is trusted, and the endpoints it offers installed, switched and removed. Every
//! call that may fetch runs on its own thread and lands back on the UI thread.

use std::cell::RefCell;

use anibeam_core::{Call, CoreError, EndpointCatalog, EndpointCredential, EndpointRepoPreview, Reply};
use slint::{ComponentHandle, ModelRc, VecModel, Weak};

use crate::{AppWindow, AuthRow, EndpointRow, RepoData, RepoRow};

/// What a tap on a row of the endpoint list stands for.
#[derive(Clone)]
enum Row {
    /// `fill` when it still needs a value of its own.
    Installed { install: String, update: bool, fill: bool },
    Offered { repo: String, id: String },
    /// The values an installed endpoint is missing, asked for.
    Fill { install: String },
}

/// An install or update waiting for the person's values.
struct Asking {
    row: Row,
    keys: Vec<String>,
    values: Vec<String>,
}

struct Repos {
    window: Weak<AppWindow>,
    catalog: Option<EndpointCatalog>,
    preview: Option<EndpointRepoPreview>,
    rows: Vec<Row>,
    /// The endpoint list, kept and updated row by row: a row that did not change keeps its
    /// element, so an open form's field keeps what was typed when the list is read again.
    lines: std::rc::Rc<VecModel<EndpointRow>>,
    asking: Option<Asking>,
    /// The saved catalog was read and the launch refresh sent.
    opened: bool,
}

thread_local! {
    static REPOS: RefCell<Option<Repos>> = const { RefCell::new(None) };
}

fn with(f: impl FnOnce(&mut Repos)) {
    REPOS.with(|cell| {
        if let Some(r) = cell.borrow_mut().as_mut() {
            f(r);
        }
    });
}

/// Runs `call` on a thread and hands the answer to `done` on the UI thread.
fn spawn(call: Call, done: impl FnOnce(&mut Repos, Result<Reply, CoreError>) + Send + 'static) {
    let Some(core) = crate::local::core() else { return };
    let _ = std::thread::Builder::new().name("repos".into()).spawn(move || {
        let answer = core.call(call);
        let _ = slint::invoke_from_event_loop(move || with(|r| done(r, answer)));
    });
}

pub fn install(window: &AppWindow) {
    REPOS.with(|cell| {
        *cell.borrow_mut() =
            Some(Repos {
                window: window.as_weak(),
                catalog: None,
                preview: None,
                rows: Vec::new(),
                lines: std::rc::Rc::new(VecModel::default()),
                asking: None,
                opened: false,
            })
    });
    let d = window.global::<RepoData>();
    REPOS.with(|cell| {
        if let Some(r) = cell.borrow().as_ref() {
            d.set_endpoints(ModelRc::from(r.lines.clone()));
        }
    });
    d.on_add(|url| with(|r| r.add(url.trim().to_string())));
    d.on_trust(|| with(Repos::trust));
    d.on_cancel(|| {
        with(|r| {
            r.preview = None;
            r.data(|d| {
                d.set_previewing(false);
                d.set_error("".into());
            });
        })
    });
    d.on_remove_repo(|i| with(|r| r.remove_repo(i)));
    d.on_refresh(|| with(Repos::refresh));
    d.on_act(|i| with(|r| r.act(i)));
    d.on_remove_endpoint(|i| with(|r| r.remove_endpoint(i)));
    d.on_toggle(|i| with(|r| r.toggle(i)));
    d.on_open(|i| with(|r| r.open(i)));
    d.on_set_value(|i, text| {
        with(|r| {
            if let Some(a) = r.asking.as_mut()
                && let Some(v) = usize::try_from(i).ok().and_then(|i| a.values.get_mut(i))
            {
                *v = text.to_string();
            }
        })
    });
    d.on_confirm(|| with(Repos::confirm));
    d.on_dismiss(|| with(Repos::dismiss));
    d.on_open_link(|url| crate::android::open_url(&url));
    d.on_add_custom(|name, url| with(|r| r.add_custom(name.to_string(), url.to_string())));
}

/// The core is open: show what is saved, then fetch every repo again, as the core does at
/// launch and nowhere else. Asked on every poll; acts once.
pub fn opened() {
    with(|r| {
        if r.opened || crate::local::core().is_none() {
            return;
        }
        r.opened = true;
        r.load();
        r.refresh();
    });
}

/// Whether two rows stand for the same endpoint, whatever state each row is in.
fn same(a: &Row, b: &Row) -> bool {
    let install = |r: &Row| match r {
        Row::Installed { install, .. } | Row::Fill { install } => Some(install.clone()),
        Row::Offered { .. } => None,
    };
    match (a, b) {
        (Row::Offered { repo, id }, Row::Offered { repo: r, id: i }) => repo == r && id == i,
        _ => install(a).is_some() && install(a) == install(b),
    }
}

/// Puts a list's rows in place, touching only the rows that changed.
fn update_rows(model: &VecModel<EndpointRow>, rows: Vec<EndpointRow>) {
    use slint::Model;
    if model.row_count() != rows.len() {
        model.set_vec(rows);
        return;
    }
    for (i, row) in rows.into_iter().enumerate() {
        if model.row_data(i).as_ref() != Some(&row) {
            model.set_row_data(i, row);
        }
    }
}

impl Repos {
    fn data(&self, f: impl FnOnce(RepoData)) {
        if let Some(w) = self.window.upgrade() {
            f(w.global::<RepoData>());
        }
    }

    fn load(&mut self) {
        let Some(core) = crate::local::core() else { return };
        if let Ok(Reply::EndpointCatalog { catalog }) = core.call(Call::ListRepoEndpoints) {
            self.show(catalog);
        }
    }

    fn busy(&self, busy: bool) {
        self.data(|d| d.set_busy(busy));
    }

    /// Takes a catalog answer, or puts its error up.
    fn answer(&mut self, answer: Result<Reply, CoreError>) -> bool {
        self.busy(false);
        match answer {
            Ok(Reply::EndpointCatalog { catalog }) => {
                self.show(catalog);
                self.data(|d| d.set_error("".into()));
                // The search page reads its sources from the core again.
                crate::local::touch();
                crate::app::endpoints_changed();
                true
            }
            Ok(_) => false,
            Err(e) => {
                let text = e.to_string();
                self.data(|d| d.set_error(text.into()));
                false
            }
        }
    }

    fn show(&mut self, catalog: EndpointCatalog) {
        let repos: Vec<RepoRow> = catalog
            .repos
            .iter()
            .map(|r| RepoRow {
                name: r.name.clone().into(),
                url: r.url.clone().into(),
                fingerprint: r.fingerprint.clone().into(),
                nsfw: r.nsfw,
                error: r.error.clone().unwrap_or_default().into(),
            })
            .collect();
        let mut rows = Vec::new();
        let mut lines = Vec::new();
        for i in &catalog.installed {
            // The note under the row, and whether it is something to fix (yellow) or a hint.
            let (note, warn) = if !i.missing.is_empty() {
                let labels: Vec<&str> = i
                    .auth
                    .iter()
                    .filter(|a| i.missing.contains(&a.key))
                    .map(|a| a.label.as_str())
                    .collect();
                (format!("Needs its {} before it searches", labels.join(", ")), true)
            } else if let Some(note) = &i.desktop_note {
                (note.clone(), true)
            } else if !i.offered {
                ("No longer offered by its repo".into(), true)
            } else if let Some(host) = &i.update_host {
                (format!("The update sends requests to {host}"), true)
            } else if !i.auth.is_empty() {
                let labels: Vec<&str> = i.auth.iter().map(|a| a.label.as_str()).collect();
                (format!("Tap to change its {}", labels.join(", ")), false)
            } else {
                (String::new(), false)
            };
            lines.push(EndpointRow {
                name: i.name.clone().into(),
                line: if i.custom {
                    format!("{}  {}  added by you", i.host, i.kind)
                } else {
                    format!("{}  {}  v{}", i.host, i.kind, i.version)
                }
                .into(),
                installed: true,
                enabled: i.enabled,
                nsfw: i.nsfw,
                note: note.into(),
                update: match i.update {
                    Some(v) => format!("Update to v{v}"),
                    None if !i.missing.is_empty() => "Add its value".into(),
                    None => String::new(),
                }
                .into(),
                editable: !i.auth.is_empty(),
                warn,
            });
            rows.push(Row::Installed {
                install: i.install_id.clone(),
                update: i.update.is_some(),
                fill: !i.missing.is_empty(),
            });
        }
        for o in catalog.offered.iter().filter(|o| o.installed.is_none()) {
            let needs: Vec<&str> = o.auth.iter().map(|a| a.label.as_str()).collect();
            lines.push(EndpointRow {
                name: o.name.clone().into(),
                line: format!("{}  {}  {}", o.host, o.kind, o.lang).into(),
                installed: false,
                enabled: false,
                nsfw: o.nsfw,
                note: if needs.is_empty() { String::new() } else { format!("Asks for your {}", needs.join(", ")) }.into(),
                update: "".into(),
                editable: false,
                warn: !o.auth.is_empty(),
            });
            rows.push(Row::Offered { repo: o.repo_url.clone(), id: o.id.clone() });
        }
        // An open form follows its endpoint to wherever the list now has it, and closes when
        // the endpoint is gone. A form that moved gets a new, empty field, so what was typed
        // is dropped too, and what is sent always matches what is on screen.
        if let Some(a) = &self.asking {
            let target = match &a.row {
                Row::Fill { install } | Row::Installed { install, .. } => rows.iter().position(
                    |r| matches!(r, Row::Installed { install: i, .. } if i == install),
                ),
                Row::Offered { repo, id } => rows
                    .iter()
                    .position(|r| matches!(r, Row::Offered { repo: rr, id: ii } if rr == repo && ii == id)),
            };
            let before = self.rows.iter().position(|r| same(r, &a.row));
            match target {
                None => self.dismiss(),
                Some(at) => {
                    if Some(at) != before
                        && let Some(a) = self.asking.as_mut()
                    {
                        a.values.iter_mut().for_each(String::clear);
                    }
                    self.data(|d| d.set_asking_index(at as i32));
                }
            }
        }
        self.rows = rows;
        update_rows(&self.lines, lines);
        self.data(|d| d.set_repos(ModelRc::new(VecModel::from(repos))));
        self.catalog = Some(catalog);
    }

    fn add(&mut self, url: String) {
        if url.is_empty() {
            return;
        }
        self.busy(true);
        self.data(|d| d.set_error("".into()));
        spawn(Call::AddEndpointRepo { url }, |r, answer| {
            r.busy(false);
            match answer {
                Ok(Reply::EndpointRepoPreview { preview }) => {
                    let count = preview.endpoints.len();
                    let line = format!(
                        "{} {}, signed{}",
                        count,
                        if count == 1 { "endpoint" } else { "endpoints" },
                        if preview.trusted { ", already trusted" } else { "" }
                    );
                    r.data(|d| {
                        d.set_preview_name(preview.name.clone().into());
                        d.set_preview_url(preview.url.clone().into());
                        d.set_preview_fingerprint(preview.fingerprint.clone().into());
                        d.set_preview_line(line.into());
                        d.set_preview_nsfw(preview.nsfw);
                        d.set_previewing(true);
                    });
                    r.preview = Some(preview);
                }
                Ok(_) => {}
                Err(e) => {
                    let text = e.to_string();
                    r.data(|d| d.set_error(text.into()));
                }
            }
        });
    }

    fn add_custom(&mut self, name: String, url: String) {
        self.data(|d| d.set_custom_error("".into()));
        spawn(Call::AddCustomEndpoint { name, url, credentials: Vec::new() }, |r, answer| {
            match answer {
                Ok(Reply::EndpointCatalog { catalog }) => {
                    r.show(catalog);
                    crate::local::touch();
                    crate::app::endpoints_changed();
                    r.data(|d| {
                        d.set_custom_name("".into());
                        d.set_custom_url("".into());
                    });
                }
                Ok(_) => {}
                Err(e) => {
                    let text = e.to_string();
                    r.data(|d| d.set_custom_error(text.into()));
                }
            }
        });
    }

    fn trust(&mut self) {
        let Some(p) = self.preview.clone() else { return };
        self.busy(true);
        spawn(Call::TrustEndpointRepo { url: p.url, key: p.key }, |r, answer| {
            if r.answer(answer) {
                r.preview = None;
                r.data(|d| {
                    d.set_previewing(false);
                    d.set_url("".into());
                });
            }
        });
    }

    fn remove_repo(&mut self, i: i32) {
        let Some(url) = self
            .catalog
            .as_ref()
            .and_then(|c| usize::try_from(i).ok().and_then(|i| c.repos.get(i)))
            .map(|r| r.url.clone())
        else {
            return;
        };
        spawn(Call::RemoveEndpointRepo { url }, |r, answer| {
            r.answer(answer);
        });
    }

    fn refresh(&mut self) {
        spawn(Call::RefreshEndpointRepos, |r, answer| {
            // A refresh that cannot reach anything is said on each repo's row; it is not an
            // error of the page's.
            if let Ok(Reply::EndpointCatalog { catalog }) = answer {
                r.show(catalog);
                crate::local::touch();
                crate::app::endpoints_changed();
            }
        });
    }

    fn row(&self, i: i32) -> Option<Row> {
        usize::try_from(i).ok().and_then(|i| self.rows.get(i)).cloned()
    }

    /// Install, or take an update. Either asks for values first when the endpoint needs them.
    fn act(&mut self, i: i32) {
        let Some(row) = self.row(i) else { return };
        let Some(catalog) = &self.catalog else { return };
        let (fields, asks) = match &row {
            Row::Offered { repo, id } => {
                let Some(o) = catalog.offered.iter().find(|o| &o.repo_url == repo && &o.id == id) else { return };
                (o.auth.clone(), !o.auth.is_empty())
            }
            Row::Fill { .. } => return,
            Row::Installed { install, update: false, fill: true } => {
                let Some(e) = catalog.installed.iter().find(|e| &e.install_id == install) else { return };
                let fields = e.auth.iter().filter(|a| e.missing.contains(&a.key)).cloned().collect();
                let fill = Row::Fill { install: install.clone() };
                self.ask(
                    i,
                    fill,
                    fields,
                    "Kept on this phone only, never in the library or an export.".into(),
                );
                return;
            }
            Row::Installed { install, update, .. } => {
                if !update {
                    return;
                }
                let Some(e) = catalog.installed.iter().find(|e| &e.install_id == install) else { return };
                // The update's own fields are the offered definition's.
                let fields = catalog
                    .offered
                    .iter()
                    .find(|o| o.repo_url == e.repo_url && o.id == e.id)
                    .map(|o| o.auth.clone())
                    .unwrap_or_default();
                (fields, e.update_asks)
            }
        };
        if !asks {
            self.run(row, Vec::new());
            return;
        }
        let note = match (&row, catalog.installed.iter().find(|e| matches!(&row, Row::Installed { install, .. } if install == &e.install_id))) {
            (Row::Installed { .. }, Some(e)) => match &e.update_host {
                Some(host) => format!("The new version sends this to {host}, so it asks again."),
                None => "The new version asks for these again.".into(),
            },
            _ => "Kept on this phone only, never in the library or an export.".into(),
        };
        self.ask(i, row, fields, note);
    }

    fn ask(&mut self, index: i32, row: Row, fields: Vec<anibeam_core::EndpointField>, note: String) {
        let rows: Vec<AuthRow> = fields
            .iter()
            .map(|f| AuthRow {
                label: f.label.clone().into(),
                help: f.help.clone().into(),
                help_url: f.help_url.clone().unwrap_or_default().into(),
                secret: f.secret,
            })
            .collect();
        let action = match &row {
            Row::Offered { .. } => "Install",
            Row::Installed { .. } => "Update",
            Row::Fill { .. } => "Save",
        };
        self.asking = Some(Asking {
            row,
            keys: fields.iter().map(|f| f.key.clone()).collect(),
            values: vec![String::new(); fields.len()],
        });
        self.data(|d| {
            d.set_asking_action(action.into());
            d.set_asking_index(index);
            d.set_asking_note(note.into());
            d.set_fields(ModelRc::new(VecModel::from(rows)));
            d.set_asking(true);
            d.set_asking_error("".into());
        });
    }

    /// Opens an installed endpoint's values: every field, to add one that is missing or
    /// change one that is stored. A field left empty keeps its value.
    fn open(&mut self, i: i32) {
        let Some(Row::Installed { install, update, fill }) = self.row(i) else { return };
        // A row still missing a value asks for just that, as its button does.
        if fill && !update {
            self.act(i);
            return;
        }
        let Some(e) = self.catalog.as_ref().and_then(|c| c.installed.iter().find(|e| e.install_id == install)).cloned()
        else {
            return;
        };
        if e.auth.is_empty() {
            return;
        }
        self.ask(
            i,
            Row::Fill { install },
            e.auth.clone(),
            "Kept on this phone only. A field left empty keeps what is stored.".into(),
        );
    }

    fn confirm(&mut self) {
        let Some(a) = &self.asking else { return };
        if matches!(a.row, Row::Fill { .. }) && a.values.iter().all(|v| v.trim().is_empty()) {
            self.data(|d| d.set_asking_error("Type a value first.".into()));
            return;
        }
        let credentials = a
            .keys
            .iter()
            .zip(&a.values)
            .map(|(k, v)| EndpointCredential { key: k.clone(), value: v.clone() })
            .collect();
        self.run(a.row.clone(), credentials);
    }

    fn dismiss(&mut self) {
        self.asking = None;
        self.data(|d| {
            d.set_asking(false);
            d.set_fields(ModelRc::default());
        });
    }

    fn run(&mut self, row: Row, credentials: Vec<EndpointCredential>) {
        let call = match row {
            Row::Offered { repo, id } => Call::InstallEndpoint { repo, id, credentials },
            Row::Installed { install, .. } => Call::UpdateEndpoint { install, credentials },
            Row::Fill { install } => Call::SetEndpointCredentials { install, credentials },
        };
        self.busy(true);
        spawn(call, |r, answer| match answer {
            Err(e) => {
                r.busy(false);
                let text = e.to_string();
                r.data(|d| d.set_asking_error(text.into()));
            }
            ok => {
                if r.answer(ok) {
                    r.dismiss();
                }
            }
        });
    }

    fn remove_endpoint(&mut self, i: i32) {
        let Some(Row::Installed { install, .. }) = self.row(i) else { return };
        spawn(Call::RemoveEndpoint { install }, |r, answer| {
            r.answer(answer);
        });
    }

    fn toggle(&mut self, i: i32) {
        let Some(Row::Installed { install, .. }) = self.row(i) else { return };
        let enabled = !self
            .catalog
            .as_ref()
            .and_then(|c| c.installed.iter().find(|e| e.install_id == install))
            .is_some_and(|e| e.enabled);
        spawn(Call::SetEndpointEnabled { install, enabled }, |r, answer| {
            r.answer(answer);
        });
    }
}
