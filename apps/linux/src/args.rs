//! The command line. Nothing here opens a window: `--version` prints and leaves, `--shoot`
//! renders one page offscreen and leaves, `--root` sandboxes every path, `--action` is what
//! a second launch forwards as ActivateAction (Task 13), `--props` is the JSON object a
//! `--shoot`'d page opens with, for pages that need more than a bare page name (a series
//! id, a metadata query).

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
        }
    }
}

pub fn parse(argv: &[String]) -> Result<Args, String> {
    let mut a = Args::default();
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
                a.width = value("--width")?
                    .parse()
                    .map_err(|_| "--width needs a number".to_string())?
            }
            "--height" => {
                a.height = value("--height")?
                    .parse()
                    .map_err(|_| "--height needs a number".to_string())?
            }
            "--root" => a.root = Some(PathBuf::from(value("--root")?)),
            "--action" => a.action = Some(value("--action")?),
            other => return Err(format!("unknown argument {other}")),
        }
    }
    Ok(a)
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
