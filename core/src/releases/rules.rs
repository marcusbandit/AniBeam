//! Subscriptions without qBittorrent: what anirss registers with qBittorrent's RSS
//! auto-downloader (`qbt/actions.py` `subscribe_rule`), kept by the core, and the
//! auto-downloader itself, matching qBittorrent's rules: `mustContain` and
//! `mustNotContain` in its wildcard mode, each matching article taken once.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use serde::{Deserialize, Serialize};

use crate::contract::*;

/// One subscription: a feed, its filter, and where its episodes go.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    pub name: String,
    /// The feed as kept on disk: an installed endpoint's secrets stay placeholders, filled
    /// from the secret store each time the feed is read (`repos::filled_feed`).
    pub feed_url: String,
    /// The endpoint the feed came from, whose credentials fill it.
    #[serde(default)]
    pub endpoint: String,
    #[serde(default)]
    pub must_contain: String,
    /// `|`-joined, as qBittorrent's.
    #[serde(default)]
    pub must_not_contain: String,
    pub save_path: String,
    #[serde(default = "yes")]
    pub enabled: bool,
    /// The links already taken, so each episode downloads once.
    #[serde(default)]
    pub taken: BTreeSet<String>,
}

fn yes() -> bool {
    true
}

/// Held while a rule store reads, changes and writes its file, so two changes at once (a
/// subscribe, a poll marking an episode taken, the launch rewrite) cannot lose either.
static EDITING: Mutex<()> = Mutex::new(());

fn editing() -> MutexGuard<'static, ()> {
    EDITING.lock().unwrap_or_else(|e| e.into_inner())
}

/// The rules on disk, in one JSON file.
pub struct RuleStore {
    path: PathBuf,
}

impl RuleStore {
    pub fn new(path: PathBuf) -> RuleStore {
        RuleStore { path }
    }

    pub fn load(&self) -> Vec<Rule> {
        std::fs::read(&self.path)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, rules: &[Rule]) -> Result<(), CoreError> {
        let bytes =
            serde_json::to_vec_pretty(rules).map_err(|e| CoreError::internal(e.to_string()))?;
        crate::atomic_file::write(&self.path, bytes)
            .map_err(|e| CoreError::internal(e.source.to_string()))
    }

    /// Adds or replaces a rule. The same show from another endpoint gets ` @endpoint`
    /// after its name, as anirss does, so neither overwrites the other; the same feed again
    /// keeps the name and what it has taken. Two rules read the same feed when they read the
    /// same address once filled: `filled` is this one's, and `fill` fills a kept rule's. So a
    /// rule kept before #149, its secret in its address, is the same feed as this one and is
    /// replaced by it.
    pub fn subscribe(
        &self,
        name: &str,
        spec: &SubscriptionSpec,
        filled: &str,
        fill: impl Fn(&Rule) -> Option<String>,
        save_base: &Path,
    ) -> Result<(Rule, Subscribed), CoreError> {
        let _one = editing();
        let mut rules = self.load();
        let mut name = name.trim().to_string();
        let same = |r: &Rule| {
            (r.feed_url == spec.feed_url && r.endpoint == spec.endpoint_name)
                || fill(r).as_deref() == Some(filled)
        };
        if let Some(existing) = rules.iter().find(|r| r.name == name)
            && !same(existing)
            && !spec.endpoint_name.is_empty()
        {
            name = format!("{name} @{}", spec.endpoint_name);
        }
        let taken = rules
            .iter()
            .find(|r| r.name == name && same(r))
            .map(|r| r.taken.clone());
        let rule = Rule {
            name: name.clone(),
            feed_url: spec.feed_url.clone(),
            endpoint: spec.endpoint_name.clone(),
            must_contain: spec.must_contain.clone(),
            must_not_contain: spec.must_not_contain.clone(),
            save_path: save_base.join(&name).to_string_lossy().into_owned(),
            enabled: true,
            taken: taken.unwrap_or_default(),
        };
        rules.retain(|r| r.name != name);
        rules.push(rule.clone());
        self.save(&rules)?;
        let subscribed = Subscribed {
            name,
            feed_url: rule.feed_url.clone(),
            save_path: rule.save_path.clone(),
            must_contain: rule.must_contain.clone(),
            must_not_contain: rule.must_not_contain.clone(),
            tags: Vec::new(),
        };
        Ok((rule, subscribed))
    }

    /// Takes the named rules out; returns what went.
    pub fn remove(&self, names: &[String]) -> Result<Vec<Rule>, CoreError> {
        let _one = editing();
        let mut rules = self.load();
        let gone: Vec<Rule> = rules
            .iter()
            .filter(|r| names.contains(&r.name))
            .cloned()
            .collect();
        rules.retain(|r| !names.contains(&r.name));
        self.save(&rules)?;
        Ok(gone)
    }

    /// Notes links as taken by `rule`.
    pub fn mark_taken(&self, rule: &str, links: &[String]) -> Result<(), CoreError> {
        let _one = editing();
        let mut rules = self.load();
        if let Some(r) = rules.iter_mut().find(|r| r.name == rule) {
            r.taken.extend(links.iter().cloned());
        }
        self.save(&rules)
    }

    /// Reads the rules, lets `change` change them, and writes them back when it says it
    /// did, as one step no other change of this file can come between.
    pub fn edit<T>(
        &self,
        change: impl FnOnce(&mut Vec<Rule>) -> (T, bool),
    ) -> Result<T, CoreError> {
        let _one = editing();
        let mut rules = self.load();
        let (out, changed) = change(&mut rules);
        if changed {
            self.save(&rules)?;
        }
        Ok(out)
    }
}

/// qBittorrent's wildcard matching for one filter: `|` separates alternatives, and within
/// one, every space-separated word must appear, case-insensitively. `*` and `?` are
/// wildcards inside a word.
fn alternatives(filter: &str) -> Vec<Vec<String>> {
    filter
        .split('|')
        .map(|alt| {
            alt.split_whitespace()
                .map(|w| w.to_lowercase())
                .collect::<Vec<_>>()
        })
        .filter(|words| !words.is_empty())
        .collect()
}

fn word_in(title: &str, word: &str) -> bool {
    if !word.contains(['*', '?']) {
        return title.contains(word);
    }
    // A wildcard word: `*` any run, `?` one character, matched anywhere in the title.
    let pattern = format!(
        "(?i){}",
        word.chars()
            .map(|c| match c {
                '*' => ".*".to_string(),
                '?' => ".".to_string(),
                c => regex::escape(&c.to_string()),
            })
            .collect::<String>()
    );
    regex::Regex::new(&pattern).is_ok_and(|re| re.is_match(title))
}

/// Does qBittorrent's rule take this article? Empty `must_contain` takes everything.
pub fn matches(rule: &Rule, title: &str) -> bool {
    let title = title.to_lowercase();
    let any = |filter: &str| {
        alternatives(filter)
            .iter()
            .any(|words| words.iter().all(|w| word_in(&title, w)))
    };
    let must = rule.must_contain.trim().is_empty() || any(&rule.must_contain);
    let must_not = !rule.must_not_contain.trim().is_empty() && any(&rule.must_not_contain);
    must && !must_not
}

/// The articles of `items` the rule takes now: matching, and not taken before.
pub fn due<'a>(rule: &Rule, items: &'a [Release]) -> Vec<&'a Release> {
    if !rule.enabled {
        return Vec::new();
    }
    items
        .iter()
        .filter(|it| !rule.taken.contains(&it.link) && matches(rule, &it.title))
        .collect()
}

/// The rule as the downloads page lists it.
pub fn info(rule: &Rule, torrent_count: u64, file_count: u64) -> SubscriptionRule {
    SubscriptionRule {
        name: rule.name.clone(),
        feed_url: rule.feed_url.clone(),
        save_path: rule.save_path.clone(),
        rule_enabled: rule.enabled,
        torrent_count,
        file_count,
        query: query_of(&rule.feed_url),
        show: None,
        followable: true,
    }
}

/// The search a feed asks for, off its `q` or `s` parameter.
fn query_of(url: &str) -> String {
    let Some((_, qs)) = url.split_once('?') else {
        return String::new();
    };
    qs.split('&')
        .find_map(|pair| {
            let (k, v) = pair.split_once('=')?;
            matches!(k, "q" | "s").then(|| super::endpoints::unquote_plus(v))
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(must: &str, not: &str) -> Rule {
        Rule {
            name: "Frieren".into(),
            feed_url: "https://nyaa.si/?page=rss&q=Frieren+1080p".into(),
            endpoint: String::new(),
            must_contain: must.into(),
            must_not_contain: not.into(),
            save_path: "/x/Frieren".into(),
            enabled: true,
            taken: BTreeSet::new(),
        }
    }

    #[test]
    fn qbittorrent_wildcard_rules_take_what_they_should() {
        let r = rule("Frieren 1080p", "HEVC|480p");
        assert!(matches(&r, "[SubsPlease] Sousou no Frieren - 01 (1080p)"));
        assert!(!matches(
            &r,
            "[Erai-raws] Sousou no Frieren - 01 [1080p HEVC]"
        ));
        assert!(!matches(&r, "[SubsPlease] Sousou no Frieren - 01 (720p)"));
        assert!(matches(&rule("", ""), "anything"));
        assert!(matches(&rule("Fri*ren 108?p", ""), "Frieren 1080p"));
        assert!(matches(
            &rule("Frieren 720p|Frieren 1080p", ""),
            "Frieren 1080p"
        ));
    }

    #[test]
    fn a_taken_link_is_not_taken_again() {
        let mut r = rule("", "");
        let item = |link: &str| Release {
            title: "Frieren - 01".into(),
            link: link.into(),
            seeders: None,
            leechers: None,
            downloads: None,
            size: String::new(),
            category: String::new(),
            page: String::new(),
            source: String::new(),
            resolution: String::new(),
            infomkv: String::new(),
            linktype: String::new(),
        };
        let items = vec![item("a"), item("b")];
        r.taken.insert("a".into());
        assert_eq!(due(&r, &items).len(), 1);
        r.enabled = false;
        assert!(due(&r, &items).is_empty());
    }

    /// A rule's feed filled as it is, for rules that hold no placeholder.
    fn as_is(r: &Rule) -> Option<String> {
        Some(r.feed_url.clone())
    }

    #[test]
    fn the_same_show_from_another_endpoint_keeps_both_rules() {
        let dir = std::env::temp_dir().join(format!("anibeam-rules-{}", std::process::id()));
        let store = RuleStore::new(dir.join("rules.json"));
        let spec = |url: &str, ep: &str| SubscriptionSpec {
            feed_url: url.into(),
            must_contain: String::new(),
            must_not_contain: String::new(),
            endpoint_name: ep.into(),
        };
        store
            .subscribe(
                "Frieren",
                &spec("https://nyaa.si/?q=f", "nyaa"),
                "",
                as_is,
                &dir,
            )
            .unwrap();
        let (_, second) = store
            .subscribe(
                "Frieren",
                &spec("https://erai/?s=f", "erai"),
                "",
                as_is,
                &dir,
            )
            .unwrap();
        assert_eq!(second.name, "Frieren @erai");
        assert_eq!(store.load().len(), 2);
        assert_eq!(
            query_of("https://nyaa.si/?page=rss&q=Frieren+1080p&c=1_0"),
            "Frieren 1080p"
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_rule_kept_with_its_secret_is_replaced_by_the_same_feed_without_it() {
        let dir = tempfile::tempdir().unwrap();
        let store = RuleStore::new(dir.path().join("rules.json"));
        let spec = |url: &str| SubscriptionSpec {
            feed_url: url.into(),
            must_contain: String::new(),
            must_not_contain: String::new(),
            endpoint_name: "keyed".into(),
        };
        let filled = "https://feed.example/rss?q=Show&token=s3cret";
        let kept = "https://feed.example/rss?q=Show&token={token}";
        // Each endpoint's own token.
        let fill = |r: &Rule| {
            let token = if r.endpoint == "other" {
                "0ther"
            } else {
                "s3cret"
            };
            Some(r.feed_url.replace("{token}", token))
        };
        // As a rule was kept before #149: the token in the address, no endpoint.
        store
            .subscribe("Show", &spec(filled), filled, fill, dir.path())
            .unwrap();
        let mut rules = store.load();
        rules[0].endpoint = String::new();
        store.save(&rules).unwrap();
        store.mark_taken("Show", &["a".to_string()]).unwrap();

        let (_, again) = store
            .subscribe("Show", &spec(kept), filled, fill, dir.path())
            .unwrap();
        assert_eq!(again.name, "Show");
        let rules = store.load();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].feed_url, kept);
        assert_eq!(rules[0].endpoint, "keyed");
        assert!(rules[0].taken.contains("a"));

        // Another endpoint whose kept feed reads the same but fills another token is
        // another feed, and another rule.
        let other = SubscriptionSpec {
            endpoint_name: "other".into(),
            ..spec(kept)
        };
        let other_filled = "https://feed.example/rss?q=Show&token=0ther";
        let (_, second) = store
            .subscribe("Show", &other, other_filled, fill, dir.path())
            .unwrap();
        assert_eq!(second.name, "Show @other");
        assert_eq!(store.load().len(), 2);
    }
}
