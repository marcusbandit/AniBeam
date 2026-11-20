//! The control channel's command line: what an agent types after `scripts/ctl.sh`, read into
//! a `Command`. Pure text in and out, so it is tested on the host.
//!
//! A line holds one command or several split by `;`. Words split on blanks; `"..."` and
//! `'...'` keep blanks inside a word, and a backslash outside single quotes takes the next
//! character as it is.

/// A tab of the bar, by the page number `Nav.page` uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Home,
    Calendar,
    Library,
    Downloads,
    More,
}

impl Tab {
    pub const ALL: [Tab; 5] = [
        Tab::Home,
        Tab::Calendar,
        Tab::Library,
        Tab::Downloads,
        Tab::More,
    ];

    /// `Nav.page`'s number for the tab.
    pub fn page(self) -> i32 {
        match self {
            Tab::Home => 0,
            Tab::Library => 1,
            Tab::Downloads => 2,
            Tab::More => 3,
            Tab::Calendar => 4,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Tab::Home => "home",
            Tab::Calendar => "calendar",
            Tab::Library => "library",
            Tab::Downloads => "downloads",
            Tab::More => "more",
        }
    }

    pub fn of_page(page: i32) -> Option<Tab> {
        Tab::ALL.into_iter().find(|t| t.page() == page)
    }

    fn parse(word: &str) -> Option<Tab> {
        Some(match word.to_lowercase().as_str() {
            "home" => Tab::Home,
            "calendar" | "cal" => Tab::Calendar,
            "library" | "lib" => Tab::Library,
            "downloads" | "download" | "dl" => Tab::Downloads,
            "more" | "settings" => Tab::More,
            _ => return None,
        })
    }
}

/// What a touch lands on: a control by its label, the `nth` match counting from 1, or a
/// point in the window's logical pixels, as `dump` prints them.
#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    /// `nth` counts from 1 and is None when not given: then several matches are a question
    /// back rather than a guess.
    Label {
        label: String,
        nth: Option<usize>,
    },
    Point(f32, f32),
    /// One word shaped like a point, `4,5`: a label that shows by that name, else the point.
    /// Stats writes hours with a decimal comma in most of the app's languages.
    Spot {
        word: String,
        x: f32,
        y: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Back,
    Enter,
    Tab,
    Escape,
    Backspace,
    Up,
    Down,
    Left,
    Right,
}

impl Key {
    pub const NAMES: [&'static str; 9] = [
        "back",
        "enter",
        "tab",
        "escape",
        "backspace",
        "up",
        "down",
        "left",
        "right",
    ];

    fn parse(word: &str) -> Option<Key> {
        Some(match word.to_lowercase().as_str() {
            "back" => Key::Back,
            "enter" | "return" => Key::Enter,
            "tab" => Key::Tab,
            "escape" | "esc" => Key::Escape,
            "backspace" => Key::Backspace,
            "up" => Key::Up,
            "down" => Key::Down,
            "left" => Key::Left,
            "right" => Key::Right,
            _ => return None,
        })
    }
}

/// A series to open: its id on the phone, or words of its title.
#[derive(Debug, Clone, PartialEq)]
pub enum Series {
    Id(i32),
    Title(String),
}

/// A text field: its number among the visible fields, counting from 1, or the placeholder
/// or label it shows.
#[derive(Debug, Clone, PartialEq)]
pub enum Field {
    Index(usize),
    Named(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Onboarding {
    Status,
    Next,
    Back,
    Step(i32),
    Finish,
    Start,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Help,
    Go(Tab),
    Back,
    Key(Vec<Key>),
    Open(Series),
    Press(Target),
    Tap(Target),
    Hold(Target),
    Type(String),
    Fill(Field, String),
    Search(String),
    Scroll {
        dir: Dir,
        amount: f32,
        at: Option<Target>,
    },
    Swipe {
        from: (f32, f32),
        to: (f32, f32),
        ms: u64,
    },
    Onboarding(Onboarding),
    Where,
    /// What shows, only the lines holding the filter when there is one.
    Tree(String),
    Find(String),
    Wait {
        label: String,
        secs: f32,
    },
    State,
    Perf(f32),
}

/// How far a `scroll` goes without a distance: most of a phone screen.
pub const SCROLL: f32 = 600.0;
/// How long `wait` looks for its label without a time, and the most it may be given: the
/// broadcast that carries the command has to answer within Android's limit.
pub const WAIT: f32 = 10.0;
pub const WAIT_MAX: f32 = 30.0;
/// How long `perf` samples without a time, and the most.
pub const PERF: f32 = 5.0;
pub const PERF_MAX: f32 = 30.0;

/// One line per command, what `help` prints. The words are the desktop's `anibeam ctl`
/// ones (#1205) where the two apps do the same thing: go, where, tree, click, type, key, shot.
pub const HELP: &str = "\
go <home|calendar|library|downloads|more>   switch tab
go series <title|id>                        open a series on the phone; open too
go downloads <words>                        search Downloads; search too
where                                       the page and what is open over it
tree [filter]                               what shows: each control, its label, state, centre
click <words|x,y> [--nth n]                 a control's own action, or a tap on it; press too
tap <words|x,y> [--nth n]                   a tap at its centre, or at a point
hold <words|x,y> [--nth n]                  a long press
type <text>                                 keys into the focused field
fill <n|placeholder> <text>                 set a field's text
key <name> ...                              back enter tab escape backspace up down left right
back                                        Android's Back
scroll <up|down|left|right> [px] [at <words|x,y>]
swipe <x1> <y1> <x2> <y2> [ms]
shot <file.png>                             the screen, from the device; ctl.sh runs it, on its own
onboarding [next|back|step <n>|finish|start]
find <text>                                 every element whose label holds it, offscreen too
wait <label> [<n>s]                         until a control with the label shows
state                                       threads, memory, frames, core jobs
perf [<n>s]                                 sample the UI loop and frames for a while";

/// Reads a whole line into its commands, in order.
pub fn parse_line(line: &str) -> Result<Vec<Command>, String> {
    let mut out = Vec::new();
    for words in split(line)? {
        out.push(parse(&words)?);
    }
    if out.is_empty() {
        return Err("no command; try help".into());
    }
    Ok(out)
}

/// Splits a line into commands at `;`, and each into its words.
fn split(line: &str) -> Result<Vec<Vec<String>>, String> {
    let mut commands = Vec::new();
    let mut words: Vec<String> = Vec::new();
    let mut word = String::new();
    // A word was started, so `""` counts as an empty word.
    let mut started = false;
    let mut chars = line.chars();
    fn end_word(word: &mut String, started: &mut bool, words: &mut Vec<String>) {
        if *started {
            words.push(std::mem::take(word));
            *started = false;
        }
    }
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                let next = chars.next().ok_or("a backslash ends the line")?;
                word.push(next);
                started = true;
            }
            '\'' => {
                started = true;
                loop {
                    match chars.next() {
                        Some('\'') => break,
                        Some(c) => word.push(c),
                        None => return Err("a ' is not closed".into()),
                    }
                }
            }
            '"' => {
                started = true;
                loop {
                    match chars.next() {
                        Some('"') => break,
                        Some('\\') => word.push(chars.next().ok_or("a \" is not closed")?),
                        Some(c) => word.push(c),
                        None => return Err("a \" is not closed".into()),
                    }
                }
            }
            ';' => {
                end_word(&mut word, &mut started, &mut words);
                if !words.is_empty() {
                    commands.push(std::mem::take(&mut words));
                }
            }
            c if c.is_whitespace() => end_word(&mut word, &mut started, &mut words),
            c => {
                word.push(c);
                started = true;
            }
        }
    }
    end_word(&mut word, &mut started, &mut words);
    if !words.is_empty() {
        commands.push(words);
    }
    Ok(commands)
}

fn number(word: &str) -> Option<f32> {
    word.parse::<f32>().ok().filter(|n| n.is_finite())
}

/// `10s` or `2.5s`.
fn seconds(word: &str) -> Option<f32> {
    word.strip_suffix('s')
        .and_then(number)
        .filter(|n| *n >= 0.0)
}

/// The words after the command joined back into one, as a label or a text.
fn rest(words: &[String]) -> String {
    words.join(" ")
}

/// `x,y`, or `x y`, for a point.
fn point(words: &[String]) -> Option<Target> {
    match words {
        [one] => {
            let (x, y) = one.split_once(',')?;
            Some(Target::Spot {
                word: one.clone(),
                x: number(x.trim())?,
                y: number(y.trim())?,
            })
        }
        [x, y] => Some(Target::Point(number(x)?, number(y)?)),
        _ => None,
    }
}

fn nth(word: &str) -> Result<usize, String> {
    word.parse()
        .ok()
        .filter(|n| *n > 0)
        .ok_or_else(|| format!("{word} is not a match number like 2"))
}

/// A point, or a label with an optional `--nth n` at its end. A label's own last word may
/// be `#2`, so that is never read as a match number.
fn target(words: &[String]) -> Result<Target, String> {
    if let Some(point) = point(words) {
        return Ok(point);
    }
    let (words, nth) = match words {
        [head @ .., flag, n] if !head.is_empty() && flag == "--nth" => (head, Some(nth(n)?)),
        _ => (words, None),
    };
    let label = rest(words);
    if label.trim().is_empty() {
        return Err("which control? give its label".into());
    }
    Ok(Target::Label { label, nth })
}

/// A lone number is the phone's id for the series, anything else words of its title.
fn series(words: &[String]) -> Command {
    match words {
        [id] if id.parse::<i32>().is_ok_and(|id| id >= 0) => {
            Command::Open(Series::Id(id.parse().unwrap()))
        }
        _ => Command::Open(Series::Title(rest(words))),
    }
}

/// The words after the command as one, refused when they hold nothing but blanks.
fn named(head: &str, words: &[String], what: &str) -> Result<String, String> {
    let all = rest(words);
    if all.trim().is_empty() {
        return Err(format!("{head} needs {what}"));
    }
    Ok(all)
}

fn parse(words: &[String]) -> Result<Command, String> {
    let (head, args) = words.split_first().ok_or("no command")?;
    let need = |what: &str| -> Result<(), String> {
        if args.is_empty() {
            Err(format!("{head} needs {what}"))
        } else {
            Ok(())
        }
    };
    let none = || -> Result<(), String> {
        if args.is_empty() {
            Ok(())
        } else {
            Err(format!("{head} takes no arguments"))
        }
    };
    Ok(match head.to_lowercase().as_str() {
        "help" => Command::Help,
        "go" => {
            need("a page: home, calendar, library, downloads, more or series")?;
            let words = &args[1..];
            match (args[0].to_lowercase().as_str(), Tab::parse(&args[0])) {
                ("series" | "show", _) if words.is_empty() => {
                    return Err("go series needs a title or an id".into());
                }
                ("series" | "show", _) => {
                    named("go series", words, "a title or an id")?;
                    series(words)
                }
                (_, Some(Tab::Downloads)) if !words.is_empty() => Command::Search(rest(words)),
                (_, Some(tab)) if words.is_empty() => Command::Go(tab),
                (_, Some(tab)) => return Err(format!("go {} takes no words", tab.name())),
                (other, None) => {
                    return Err(format!(
                        "{other} is not a page: home, calendar, library, downloads, more or series"
                    ));
                }
            }
        }
        // The tabs on their own, `home` for `go home`.
        word if Tab::parse(word).is_some() => {
            let tab = Tab::parse(word).unwrap();
            if !args.is_empty() {
                return Err(format!(
                    "{} takes no words; go {} does the same",
                    tab.name(),
                    tab.name()
                ));
            }
            Command::Go(tab)
        }
        // ctl.sh takes a screenshot itself, from the device, so the app never sees it alone.
        "shot" => return Err("shot runs in ctl.sh on its own line: ctl.sh shot <file.png>".into()),
        "back" => {
            none()?;
            Command::Back
        }
        "key" => {
            need("a key name")?;
            let keys = args
                .iter()
                .map(|k| {
                    Key::parse(k)
                        .ok_or_else(|| format!("{k} is not a key: {}", Key::NAMES.join(", ")))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Command::Key(keys)
        }
        "open" => {
            named(head, args, "a series id or title")?;
            series(args)
        }
        "click" | "press" => Command::Press(target(args)?),
        "tap" => Command::Tap(target(args)?),
        "hold" => Command::Hold(target(args)?),
        "type" => {
            need("the text to type")?;
            Command::Type(rest(args))
        }
        "fill" => {
            if args.len() < 2 {
                return Err("fill needs a field (its number or placeholder) and the text".into());
            }
            let field = match args[0].parse::<usize>() {
                Ok(0) => return Err("fill counts fields from 1; tree lists them".into()),
                Ok(n) => Field::Index(n),
                _ if args[0].trim().is_empty() => {
                    return Err("fill needs a field: its number or placeholder".into());
                }
                _ => Field::Named(args[0].clone()),
            };
            Command::Fill(field, rest(&args[1..]))
        }
        "search" => Command::Search(named(head, args, "the words to search for")?),
        "scroll" => {
            need("a direction: up, down, left or right")?;
            let dir = match args[0].to_lowercase().as_str() {
                "up" => Dir::Up,
                "down" => Dir::Down,
                "left" => Dir::Left,
                "right" => Dir::Right,
                other => return Err(format!("{other} is not up, down, left or right")),
            };
            let mut more = &args[1..];
            let mut amount = SCROLL;
            if let Some(n) = more.first().and_then(|w| number(w)) {
                if n <= 0.0 {
                    return Err("scroll by a distance above 0".into());
                }
                amount = n;
                more = &more[1..];
            }
            let at = match more.split_first() {
                None => None,
                Some((at, label)) if at == "at" => Some(target(label)?),
                Some((other, _)) => {
                    return Err(format!("scroll {other}: expected a distance or at <label>"));
                }
            };
            Command::Scroll { dir, amount, at }
        }
        "swipe" => {
            let n: Vec<f32> = args.iter().filter_map(|w| number(w)).collect();
            if !(args.len() == 4 || args.len() == 5) || n.len() != args.len() {
                return Err("swipe needs x1 y1 x2 y2 and an optional time in ms".into());
            }
            let ms = n.get(4).map(|m| *m as u64).unwrap_or(300).clamp(50, 5000);
            Command::Swipe {
                from: (n[0], n[1]),
                to: (n[2], n[3]),
                ms,
            }
        }
        "onboarding" => Command::Onboarding(match args.first().map(|w| w.to_lowercase()) {
            None => Onboarding::Status,
            Some(w) => match (w.as_str(), args.len()) {
                ("next", 1) => Onboarding::Next,
                ("back", 1) => Onboarding::Back,
                ("finish" | "skip" | "done", 1) => Onboarding::Finish,
                ("start" | "restart", 1) => Onboarding::Start,
                ("step", 2) => Onboarding::Step(
                    args[1]
                        .parse()
                        .ok()
                        .filter(|n| *n >= 0)
                        .ok_or("step takes a number from 0")?,
                ),
                _ => return Err("onboarding takes next, back, step <n>, finish or start".into()),
            },
        }),
        "tree" | "dump" => Command::Tree(rest(args)),
        "where" => {
            none()?;
            Command::Where
        }
        "find" => Command::Find(named(head, args, "the words to look for")?),
        "wait" => {
            need("a label")?;
            let (label, secs) = match args.split_last() {
                Some((last, words)) if !words.is_empty() && seconds(last).is_some() => {
                    (named(head, words, "a label")?, seconds(last).unwrap())
                }
                // `wait Play 5` most likely means five seconds, `wait Season 2` a label:
                // neither is guessed.
                Some((last, words)) if !words.is_empty() && number(last).is_some() => {
                    return Err(format!(
                        "wait: give the time with an s ({last}s), or quote a label that ends in a number"
                    ));
                }
                _ => (named(head, args, "a label")?, WAIT),
            };
            Command::Wait {
                label,
                secs: secs.min(WAIT_MAX),
            }
        }
        "state" => {
            none()?;
            Command::State
        }
        "perf" => {
            let secs = match args {
                [] => PERF,
                [w] => seconds(w)
                    .or_else(|| number(w))
                    .ok_or("perf takes a time like 5s")?,
                _ => return Err("perf takes one time, like 5s".into()),
            };
            Command::Perf(secs.clamp(0.5, PERF_MAX))
        }
        other => return Err(format!("{other} is not a command; try help")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one(line: &str) -> Command {
        let mut all = parse_line(line).unwrap();
        assert_eq!(all.len(), 1, "{line}");
        all.remove(0)
    }

    /// A label target; `1` stands for no match number given.
    fn label(label: &str, nth: usize) -> Target {
        Target::Label {
            label: label.into(),
            nth: (nth > 1).then_some(nth),
        }
    }

    #[test]
    fn blank_names_tabs_with_words_and_shot_are_refused() {
        for line in [
            "wait ''",
            "wait '  ' 5s",
            "fill '' x",
            "find ' '",
            "search ''",
            "open ''",
            "go series ' '",
        ] {
            assert!(parse_line(line).is_err(), "{line}");
        }
        assert_eq!(
            parse_line("library frieren").unwrap_err(),
            "library takes no words; go library does the same"
        );
        assert!(
            parse_line("shot a.png")
                .unwrap_err()
                .contains("ctl.sh shot")
        );
        // Fields count from 1; a 0 is never a placeholder to search for.
        assert_eq!(
            parse_line("fill 0 x").unwrap_err(),
            "fill counts fields from 1; tree lists them"
        );
        assert_eq!(
            one("click Cancel --nth 1"),
            Command::Press(Target::Label {
                label: "Cancel".into(),
                nth: Some(1)
            })
        );
    }

    #[test]
    fn words_split_on_blanks_and_keep_quoted_blanks() {
        assert_eq!(
            split(r#"press "Get episodes"  'a b' c\ d "" "#).unwrap(),
            vec![vec!["press", "Get episodes", "a b", "c d", ""]]
        );
        assert_eq!(
            split(r#"type "say \"hi\"""#).unwrap(),
            vec![vec!["type", r#"say "hi""#]]
        );
        // Inside single quotes a backslash is a plain character, as in sh.
        assert_eq!(
            split(r"press 'it\'s").unwrap(),
            vec![vec!["press", r"it\s"]]
        );
        assert_eq!(split("press 'open").unwrap_err(), "a ' is not closed");
        assert!(split("type \"open").is_err());
        assert!(split("type trailing\\").is_err());
    }

    #[test]
    fn a_semicolon_splits_commands_outside_quotes_only() {
        assert_eq!(
            parse_line("go downloads; tree ;; type \"a;b\"").unwrap(),
            vec![
                Command::Go(Tab::Downloads),
                Command::Tree(String::new()),
                Command::Type("a;b".into())
            ]
        );
        assert!(parse_line("  ;  ").is_err());
    }

    #[test]
    fn tabs_go_by_name_alias_or_alone() {
        assert_eq!(one("go home"), Command::Go(Tab::Home));
        assert_eq!(one("go DL"), Command::Go(Tab::Downloads));
        assert_eq!(one("go cal"), Command::Go(Tab::Calendar));
        assert_eq!(one("library"), Command::Go(Tab::Library));
        assert_eq!(one("settings"), Command::Go(Tab::More));
        assert!(parse_line("go nowhere").is_err());
        assert!(parse_line("go").is_err());
        assert!(parse_line("go home now").is_err());
        // The desktop's `go <page> [words]`: a series by title or id, a search in Downloads.
        assert_eq!(
            one("go series made in abyss"),
            Command::Open(Series::Title("made in abyss".into()))
        );
        assert_eq!(one("go show 12"), Command::Open(Series::Id(12)));
        assert_eq!(
            one("go downloads one piece"),
            Command::Search("one piece".into())
        );
        assert!(parse_line("go series").is_err());
        for tab in Tab::ALL {
            assert_eq!(Tab::of_page(tab.page()), Some(tab));
            assert_eq!(one(&format!("go {}", tab.name())), Command::Go(tab));
        }
    }

    #[test]
    fn press_takes_the_whole_label_and_an_optional_match_number() {
        assert_eq!(
            one("press Get episodes"),
            Command::Press(label("Get episodes", 1))
        );
        // A label may end in "#2": it stays part of the label.
        assert_eq!(
            one("press Episode #2"),
            Command::Press(label("Episode #2", 1))
        );
        assert_eq!(
            one("click Get episodes --nth 3"),
            Command::Press(label("Get episodes", 3))
        );
        assert_eq!(
            one("click 10,20"),
            Command::Press(Target::Spot {
                word: "10,20".into(),
                x: 10.0,
                y: 20.0
            })
        );
        assert_eq!(
            one("click 10 20"),
            Command::Press(Target::Point(10.0, 20.0))
        );
        assert_eq!(one("press #1"), Command::Press(label("#1", 1)));
        assert_eq!(one("press --nth"), Command::Press(label("--nth", 1)));
        assert_eq!(one("press Play #0"), Command::Press(label("Play #0", 1)));
        assert!(parse_line("click Play --nth x").is_err());
        assert!(parse_line("press").is_err());
        assert!(parse_line("press \"  \"").is_err());
    }

    #[test]
    fn tap_and_hold_take_a_point_or_a_label() {
        assert_eq!(
            one("tap 120 640.5"),
            Command::Tap(Target::Point(120.0, 640.5))
        );
        assert_eq!(one("tap Season 2"), Command::Tap(label("Season 2", 1)));
        assert_eq!(
            one("hold Frieren --nth 3"),
            Command::Hold(label("Frieren", 3))
        );
        assert_eq!(one("hold 5 6"), Command::Hold(Target::Point(5.0, 6.0)));
        assert_eq!(one("tap 5, 6"), Command::Tap(label("5, 6", 1)));
        assert_eq!(
            one("tap \"4,5\""),
            Command::Tap(Target::Spot {
                word: "4,5".into(),
                x: 4.0,
                y: 5.0
            })
        );
    }

    #[test]
    fn open_reads_a_lone_number_as_an_id() {
        assert_eq!(one("open 42"), Command::Open(Series::Id(42)));
        assert_eq!(
            one("open 86 Eighty Six"),
            Command::Open(Series::Title("86 Eighty Six".into()))
        );
        assert_eq!(
            one("open \"Kaguya-sama\""),
            Command::Open(Series::Title("Kaguya-sama".into()))
        );
        assert_eq!(one("open -1"), Command::Open(Series::Title("-1".into())));
    }

    #[test]
    fn text_commands_keep_their_words() {
        assert_eq!(one("type hello there"), Command::Type("hello there".into()));
        assert_eq!(
            one("search 'one piece'"),
            Command::Search("one piece".into())
        );
        assert_eq!(
            one("fill 2 http://198.51.100.7"),
            Command::Fill(Field::Index(2), "http://198.51.100.7".into())
        );
        assert_eq!(
            one("fill \"Search this phone\" frieren"),
            Command::Fill(Field::Named("Search this phone".into()), "frieren".into())
        );
        assert!(parse_line("fill 1").is_err());
        assert!(parse_line("type").is_err());
    }

    #[test]
    fn scroll_reads_a_direction_a_distance_and_a_place() {
        assert_eq!(
            one("scroll down"),
            Command::Scroll {
                dir: Dir::Down,
                amount: SCROLL,
                at: None
            }
        );
        assert_eq!(
            one("scroll up 200"),
            Command::Scroll {
                dir: Dir::Up,
                amount: 200.0,
                at: None
            }
        );
        assert_eq!(
            one("scroll right at Continue watching"),
            Command::Scroll {
                dir: Dir::Right,
                amount: SCROLL,
                at: Some(label("Continue watching", 1))
            }
        );
        assert_eq!(
            one("scroll left 90 at 10 400"),
            Command::Scroll {
                dir: Dir::Left,
                amount: 90.0,
                at: Some(Target::Point(10.0, 400.0))
            }
        );
        assert!(parse_line("scroll sideways").is_err());
        assert!(parse_line("scroll down -5").is_err());
        assert!(parse_line("scroll down 5 near X").is_err());
    }

    #[test]
    fn swipe_needs_four_numbers_and_clamps_its_time() {
        assert_eq!(
            one("swipe 1 2 3 4"),
            Command::Swipe {
                from: (1.0, 2.0),
                to: (3.0, 4.0),
                ms: 300
            }
        );
        assert_eq!(
            one("swipe 1 2 3 4 99999"),
            Command::Swipe {
                from: (1.0, 2.0),
                to: (3.0, 4.0),
                ms: 5000
            }
        );
        assert!(parse_line("swipe 1 2 3").is_err());
        assert!(parse_line("swipe 1 2 3 x").is_err());
    }

    #[test]
    fn onboarding_steps() {
        assert_eq!(one("onboarding"), Command::Onboarding(Onboarding::Status));
        assert_eq!(
            one("onboarding next"),
            Command::Onboarding(Onboarding::Next)
        );
        assert_eq!(
            one("onboarding skip"),
            Command::Onboarding(Onboarding::Finish)
        );
        assert_eq!(
            one("onboarding step 3"),
            Command::Onboarding(Onboarding::Step(3))
        );
        assert_eq!(
            one("onboarding restart"),
            Command::Onboarding(Onboarding::Start)
        );
        assert!(parse_line("onboarding step").is_err());
        assert!(parse_line("onboarding step -1").is_err());
        assert!(parse_line("onboarding sideways").is_err());
    }

    #[test]
    fn wait_and_perf_read_seconds_and_cap_them() {
        assert_eq!(
            one("wait Get episodes"),
            Command::Wait {
                label: "Get episodes".into(),
                secs: WAIT
            }
        );
        assert_eq!(
            one("wait \"Get episodes\" 4s"),
            Command::Wait {
                label: "Get episodes".into(),
                secs: 4.0
            }
        );
        assert_eq!(
            one("wait Play 600s"),
            Command::Wait {
                label: "Play".into(),
                secs: WAIT_MAX
            }
        );
        // A lone time is the label, there being nothing else to wait for.
        assert_eq!(
            one("wait 4s"),
            Command::Wait {
                label: "4s".into(),
                secs: WAIT
            }
        );
        // A bare number after the label is neither guessed as seconds nor folded in.
        assert!(
            parse_line("wait Play 5")
                .unwrap_err()
                .contains("give the time with an s (5s)")
        );
        assert_eq!(
            one("wait \"Season 2\""),
            Command::Wait {
                label: "Season 2".into(),
                secs: WAIT
            }
        );
        assert_eq!(one("perf"), Command::Perf(PERF));
        assert_eq!(one("perf 2s"), Command::Perf(2.0));
        assert_eq!(one("perf 3"), Command::Perf(3.0));
        assert_eq!(one("perf 900s"), Command::Perf(PERF_MAX));
        assert!(parse_line("perf soon").is_err());
    }

    #[test]
    fn plain_commands_take_no_arguments() {
        assert_eq!(one("STATE"), Command::State);
        assert_eq!(one("back"), Command::Back);
        assert_eq!(one("key enter"), Command::Key(vec![Key::Enter]));
        assert_eq!(
            one("key Down down Return"),
            Command::Key(vec![Key::Down, Key::Down, Key::Enter])
        );
        assert_eq!(one("help"), Command::Help);
        assert!(parse_line("where now").is_err());
        assert!(parse_line("back twice").is_err());
        assert!(parse_line("key hyper").is_err());
        assert!(parse_line("dance").is_err());
    }

    #[test]
    fn help_names_every_command_and_each_parses() {
        let words = [
            "go home",
            "where",
            "back",
            "key back",
            "click x",
            "tap x",
            "hold x",
            "type x",
            "fill 1 x",
            "scroll down",
            "swipe 1 2 3 4",
            "onboarding",
            "tree",
            "find x",
            "wait x",
            "state",
            "perf",
        ];
        for line in words {
            let word = line.split(' ').next().unwrap();
            assert!(
                HELP.lines().any(|l| l.starts_with(word)),
                "help misses {word}"
            );
            assert!(parse_line(line).is_ok(), "{line}");
        }
        // The older words stay, as aliases.
        assert_eq!(one("press Play"), Command::Press(label("Play", 1)));
        assert_eq!(one("open 1"), Command::Open(Series::Id(1)));
        assert_eq!(one("search x"), Command::Search("x".into()));
        assert_eq!(one("dump"), Command::Tree(String::new()));
        assert_eq!(one("tree Get"), Command::Tree("Get".into()));
        assert_eq!(one("where"), Command::Where);
    }
}
