//! The command line. Nothing here opens a window: `--version` prints and leaves, `--shoot`
//! renders one page offscreen and leaves, `--root` sandboxes every path, `--action` is what
//! a second launch forwards as ActivateAction (Task 13), `--props` is the JSON object a
//! `--shoot`'d page opens with, for pages that need more than a bare page name (a series
//! id, a metadata query). `--drive <script.json>` is `--shoot`'s interactive sibling: it
//! runs a JSON array of input steps against the offscreen window and writes each `shot` step
//! into `--out` (default: `$ANIBEAM_CAPTURES`, else the checkout's `apps/linux/captures`).
//! `serve` runs the core and its tailnet host with no window, for other devices (`serve.rs`).
//! An `anibeam://room/<code>` link, as the desktop's scheme handler passes it, joins that
//! watch together room (#846); a second launch hands it to the running window. `ctl ...`
//! drives the window that is open (control.rs); everything after it is the command's.
//! With a window already open, `--page` moves that window to the page instead.

use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub struct Args {
    pub version: bool,
    pub shoot: Option<String>,
    pub page: Option<String>,
    pub props: Option<String>,
    pub width: u32,
    pub height: u32,
    pub root: Option<PathBuf>,
    pub action: Option<String>,
    pub drive: Option<String>,
    pub out: Option<PathBuf>,
    pub serve: bool,
    /// An `anibeam://` link to open, as given: a watch together room.
    pub link: Option<String>,
    /// `anibeam ctl ...`: the command and its arguments, `--root` taken out.
    pub ctl: Option<Vec<String>>,
}

impl Default for Args {
    fn default() -> Self {
        Args {
            version: false,
            shoot: None,
            page: None,
            props: None,
            width: 1280,
            height: 800,
            root: None,
            action: None,
            drive: None,
            out: None,
            serve: false,
            link: None,
            ctl: None,
        }
    }
}

pub fn parse(argv: &[String]) -> Result<Args, String> {
    let mut a = Args::default();
    let mut sized = false;
    let mut it = argv.iter().skip(1);
    while let Some(arg) = it.next() {
        let mut value = |name: &str| {
            it.next()
                .cloned()
                .ok_or_else(|| format!("{name} needs a value"))
        };
        match arg.as_str() {
            "--version" | "-V" => a.version = true,
            "--shoot" => a.shoot = Some(value("--shoot")?),
            "--page" => a.page = Some(value("--page")?),
            "--props" => a.props = Some(value("--props")?),
            "--width" => {
                sized = true;
                a.width = value("--width")?
                    .parse()
                    .map_err(|_| "--width needs a number".to_string())?
            }
            "--height" => {
                sized = true;
                a.height = value("--height")?
                    .parse()
                    .map_err(|_| "--height needs a number".to_string())?
            }
            "--root" => a.root = Some(PathBuf::from(value("--root")?)),
            "--action" => a.action = Some(value("--action")?),
            "--drive" => a.drive = Some(value("--drive")?),
            "--out" => a.out = Some(PathBuf::from(value("--out")?)),
            "serve" => a.serve = true,
            "ctl" => {
                let mut rest = Vec::new();
                while let Some(x) = it.next() {
                    // Past `--` every word is the command's own, --root included
                    if x == "--" {
                        rest.push(x.clone());
                        rest.extend(it.cloned());
                        break;
                    }
                    if x == "--root" {
                        let dir = it.next().ok_or("--root needs a value")?;
                        a.root = Some(PathBuf::from(dir));
                    } else {
                        rest.push(x.clone());
                    }
                }
                a.ctl = Some(rest);
                break;
            }
            // Qt's own: qmlprofiler and Qt Creator start the app with it, and
            // QGuiApplication reads it from argv after this parse
            other if other.starts_with("-qmljsdebugger") => {}
            // Discord opening the app to hand over a join (#966): the link carries nothing,
            // the join itself comes over Discord's socket
            link if is_discord_link(link) => {
                if a.link.is_some() {
                    return Err("one link at a time".into());
                }
                a.link = Some(link.to_string());
            }
            // The scheme handler's %u: one link, and only a kind this build opens
            link if link.starts_with(SCHEME) => {
                if room_code(link).is_none() {
                    return Err(format!("{link} is not a link Anibeam opens"));
                }
                if a.link.is_some() {
                    return Err("one link at a time".into());
                }
                a.link = Some(link.to_string());
            }
            other => return Err(format!("unknown argument {other}")),
        }
    }
    if a.shoot.is_some() && a.drive.is_some() {
        return Err(
            "--shoot and --drive are exclusive; a drive script shoots with its own steps".into(),
        );
    }
    if a.out.is_some() && a.drive.is_none() {
        return Err("--out only means something with --drive".into());
    }
    let windowed = a.shoot.is_some() || a.drive.is_some() || a.page.is_some() || sized;
    if a.ctl.is_some() && (windowed || a.serve || a.props.is_some() || a.action.is_some()) {
        return Err("ctl drives the open window; it takes --root and its own arguments".into());
    }
    if a.serve && (windowed || a.props.is_some() || a.action.is_some() || a.link.is_some()) {
        return Err("serve opens no window; it takes --root and nothing else".into());
    }
    Ok(a)
}

const SCHEME: &str = "anibeam://";

/// The room code in an `anibeam://room/<code>` link, as written: the core normalises it
/// (case, spaces and dashes) and says when it is not a code. A trailing slash, a query or a
/// fragment is dropped, so a link pasted from a chat still opens. None for any other link.
pub fn room_code(link: &str) -> Option<String> {
    let rest = link.strip_prefix(SCHEME)?;
    let rest = rest.strip_prefix("room/")?;
    let code = rest
        .split(['?', '#'])
        .next()
        .unwrap_or("")
        .trim_end_matches('/');
    let ok = !code.is_empty()
        && code.len() <= 16
        && code.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    ok.then(|| code.to_string())
}

/// A `discord-<application id>://` link, which a Discord client opens to launch the app
/// that a Join or an accepted invite is for.
pub fn is_discord_link(link: &str) -> bool {
    link.strip_prefix("discord-")
        .and_then(|rest| rest.split_once("://"))
        .is_some_and(|(id, _)| {
            !id.is_empty() && id.len() <= 24 && id.bytes().all(|b| b.is_ascii_digit())
        })
}

/// Where a drive script's `shot` steps land: `--out`, else `$ANIBEAM_CAPTURES`, else the
/// checkout's own captures directory (ignored by git), which is where `shoot.sh` writes too.
pub fn drive_out(out: Option<&std::path::Path>, env: Option<String>) -> PathBuf {
    match (out, env.filter(|e| !e.is_empty())) {
        (Some(o), _) => o.to_path_buf(),
        (None, Some(e)) => PathBuf::from(e),
        (None, None) => PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("captures"),
    }
}

/// Reads a drive script and checks its outer shape: a JSON array of objects. Each step's own
/// keys are the QML driver's business; this only catches the file that is not a script at
/// all before a window is ever built. Returns the text exactly as read.
pub fn load_drive(path: &std::path::Path) -> Result<String, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    check_drive(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(text)
}

fn check_drive(text: &str) -> Result<(), String> {
    let v: serde_json::Value = serde_json::from_str(text).map_err(|e| format!("bad JSON: {e}"))?;
    let steps = v
        .as_array()
        .ok_or("a drive script is a JSON array of steps")?;
    if let Some(i) = steps.iter().position(|s| !s.is_object()) {
        return Err(format!("step {} is not an object", i + 1));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(s: &str) -> Vec<String> {
        std::iter::once("anibeam".to_string())
            .chain(s.split_whitespace().map(String::from))
            .collect()
    }

    #[test]
    fn defaults_and_every_flag() {
        assert_eq!(parse(&argv("")).unwrap(), Args::default());
        let a = parse(&argv(
            "--shoot out.png --page library --props {\"id\":5} --width 1600 --height 1000 --root /tmp/x --action open",
        ))
        .unwrap();
        assert_eq!(a.shoot.as_deref(), Some("out.png"));
        assert_eq!(a.page.as_deref(), Some("library"));
        assert_eq!(a.props.as_deref(), Some("{\"id\":5}"));
        assert_eq!((a.width, a.height), (1600, 1000));
        assert_eq!(a.root.as_deref(), Some(std::path::Path::new("/tmp/x")));
        assert_eq!(a.action.as_deref(), Some("open"));
        assert!(parse(&argv("--version")).unwrap().version);
    }

    #[test]
    fn serve_takes_a_root_and_nothing_that_needs_a_window() {
        let a = parse(&argv("serve --root /tmp/x")).unwrap();
        assert!(a.serve);
        assert_eq!(a.root.as_deref(), Some(std::path::Path::new("/tmp/x")));
        assert!(!parse(&argv("")).unwrap().serve);
        for bad in [
            "serve --shoot a.png",
            "serve --page library",
            "serve --action open",
            "serve --width 800",
        ] {
            assert!(parse(&argv(bad)).is_err(), "{bad}");
        }
    }

    #[test]
    fn a_room_link_is_taken_and_any_other_link_refused() {
        let a = parse(&argv("anibeam://room/K7QM2X")).unwrap();
        assert_eq!(a.link.as_deref(), Some("anibeam://room/K7QM2X"));
        assert_eq!(
            room_code("anibeam://room/k7qm-2x/").as_deref(),
            Some("k7qm-2x")
        );
        assert_eq!(
            room_code("anibeam://room/K7QM2X?from=chat").as_deref(),
            Some("K7QM2X")
        );
        assert_eq!(room_code("anibeam://room/"), None);
        assert_eq!(room_code("anibeam://room/K7 QM"), None);
        assert_eq!(room_code("anibeam://series/12"), None);
        assert_eq!(room_code("https://example.com/room/K7QM2X"), None);
        assert!(parse(&argv("anibeam://series/12")).is_err());
        assert!(parse(&argv("anibeam://room/AAAAAA anibeam://room/BBBBBB")).is_err());
        assert!(parse(&argv("serve anibeam://room/K7QM2X")).is_err());
        // The scheme handler with no link passes nothing at all
        assert_eq!(parse(&argv("")).unwrap().link, None);
        // Discord launching the app for a join
        let a = parse(&argv("discord-1234567890://")).unwrap();
        assert_eq!(a.link.as_deref(), Some("discord-1234567890://"));
        assert!(parse(&argv("discord-abc://")).is_err());
    }

    #[test]
    fn ctl_keeps_its_arguments_and_takes_the_root_from_anywhere() {
        let a = parse(&argv("ctl go settings appearance --root /tmp/x --json")).unwrap();
        assert_eq!(
            a.ctl,
            Some(vec![
                "go".into(),
                "settings".into(),
                "appearance".into(),
                "--json".into()
            ])
        );
        assert_eq!(a.root.as_deref(), Some(std::path::Path::new("/tmp/x")));
        let a = parse(&argv("--root /tmp/y ctl where")).unwrap();
        assert_eq!(a.ctl, Some(vec!["where".to_string()]));
        assert_eq!(a.root.as_deref(), Some(std::path::Path::new("/tmp/y")));
        // its own words are never the shell's flags
        assert_eq!(
            parse(&argv("ctl click --page")).unwrap().ctl,
            Some(vec!["click".into(), "--page".into()])
        );
        assert_eq!(parse(&argv("ctl")).unwrap().ctl, Some(vec![]));
        let a = parse(&argv("ctl type -- see --root later")).unwrap();
        assert_eq!(a.root, None);
        assert_eq!(a.ctl.unwrap(), ["type", "--", "see", "--root", "later"]);
        assert!(parse(&argv("--page home ctl where")).is_err());
        assert!(parse(&argv("serve ctl where")).is_err());
    }

    #[test]
    fn a_missing_value_and_an_unknown_flag_are_errors() {
        assert_eq!(
            parse(&argv("--shoot")).unwrap_err(),
            "--shoot needs a value"
        );
        assert_eq!(
            parse(&argv("--bogus")).unwrap_err(),
            "unknown argument --bogus"
        );
        assert_eq!(
            parse(&argv("--width x")).unwrap_err(),
            "--width needs a number"
        );
    }
}
