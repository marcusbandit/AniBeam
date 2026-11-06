//! Episode out on the Linux desktop (#212): org.freedesktop.Notifications on the session
//! bus, the spec every desktop's notification daemon answers, so nothing here knows which
//! compositor or shell is running. One notification per episode; a click on it comes back
//! as ActionInvoked with the id Notify handed out, and the window then opens the series, or
//! Get episodes when the library does not have it.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::StreamExt;
use zbus::Connection;
use zbus::zvariant::Value;

const NAME: &str = "org.freedesktop.Notifications";
const PATH: &str = "/org/freedesktop/Notifications";
/// The same patience the other bus calls in the shell have.
const BUS_TIMEOUT: Duration = Duration::from_secs(2);

/// What a click on a notification opens.
#[derive(Clone, Debug, PartialEq)]
pub struct Target {
    pub series: Option<u64>,
    pub anilist_id: u64,
    pub title: String,
}

/// The two lines a notification says: the news, and what a click does.
pub fn words(episode: u32, target: &Target) -> (String, String) {
    let summary = format!("Episode {episode} of {} is out", target.title);
    let body = match target.series {
        Some(_) => "Click to open it in AniBeam".to_string(),
        None => "Click to get the episodes in AniBeam".to_string(),
    };
    (summary, body)
}

/// Called with what to open and the xdg-activation token the daemon handed over for the
/// click (empty when it sent none), which lets the window come to the front on Wayland.
type OnClick = Arc<dyn Fn(Target, String) + Send + Sync>;

/// The connection and what each shown notification opens. One per process, made on the
/// first notification, so a run that never shows one never touches the bus for it.
pub struct Notifier {
    conn: Option<Connection>,
    /// The click listener runs once per connection.
    listening: bool,
    open: Arc<Mutex<HashMap<u32, Target>>>,
}

impl Notifier {
    pub fn new() -> Self {
        Notifier {
            conn: None,
            listening: false,
            open: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// On a bus of the caller's, for the tests: a private daemon, never the desktop's.
    #[cfg(test)]
    fn on(conn: Connection) -> Self {
        Notifier {
            conn: Some(conn),
            ..Notifier::new()
        }
    }

    /// The connection, with the listener for clicks and closes started beside it.
    async fn connection(&mut self, on_click: OnClick) -> zbus::Result<Connection> {
        let c = match &self.conn {
            Some(c) => c.clone(),
            None => Connection::session().await?,
        };
        self.conn = Some(c.clone());
        if self.listening {
            return Ok(c);
        }
        // One stream for all three signals, so they come in the order the bus delivered
        // them. Spec 1.2: a token just before the ActionInvoked it belongs to, and a click
        // is followed by the close it causes; three streams would lose that order.
        let rule = zbus::MatchRule::builder()
            .msg_type(zbus::message::Type::Signal)
            .interface(NAME)?
            .path(PATH)?
            .build();
        let mut signals = zbus::MessageStream::for_match_rule(rule, &c, None).await?;
        let open = self.open.clone();
        tokio::spawn(async move {
            let mut token: HashMap<u32, String> = HashMap::new();
            while let Some(msg) = signals.next().await {
                let Ok(msg) = msg else { continue };
                let header = msg.header();
                let Some(member) = header.member() else {
                    continue;
                };
                match member.as_str() {
                    "ActivationToken" => {
                        if let Ok((id, t)) = msg.body().deserialize::<(u32, String)>() {
                            token.insert(id, t);
                        }
                    }
                    "NotificationClosed" => {
                        if let Ok((id, _)) = msg.body().deserialize::<(u32, u32)>() {
                            open.lock().unwrap_or_else(|e| e.into_inner()).remove(&id);
                            token.remove(&id);
                        }
                    }
                    "ActionInvoked" => {
                        let Ok((id, action)) = msg.body().deserialize::<(u32, String)>() else {
                            continue;
                        };
                        if action != "default" {
                            continue;
                        }
                        let target = open.lock().unwrap_or_else(|e| e.into_inner()).remove(&id);
                        let t = token.remove(&id).unwrap_or_default();
                        if let Some(target) = target {
                            on_click(target, t);
                        }
                    }
                    _ => {}
                }
            }
        });
        self.listening = true;
        Ok(c)
    }

    /// Shows one notification. A desktop with no notification daemon, or no session bus,
    /// says so on stderr once per try and shows nothing.
    pub async fn show(&mut self, target: Target, episode: u32, on_click: OnClick) {
        let conn = match self.connection(on_click).await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("anibeam: notifications: {e}");
                self.conn = None;
                self.listening = false;
                return;
            }
        };
        let (summary, body) = words(episode, &target);
        let mut hints: HashMap<&str, Value> = HashMap::new();
        hints.insert("desktop-entry", Value::from(crate::APP_ID));
        let actions = vec![
            "default",
            if target.series.is_some() {
                "Open"
            } else {
                "Get episodes"
            },
        ];
        let payload = (
            "AniBeam",
            0u32,
            crate::APP_ID,
            summary.as_str(),
            body.as_str(),
            actions,
            hints,
            -1i32,
        );
        let call = conn.call_method(Some(NAME), PATH, Some(NAME), "Notify", &payload);
        match tokio::time::timeout(BUS_TIMEOUT, call).await {
            Ok(Ok(reply)) => {
                if let Ok(id) = reply.body().deserialize::<u32>() {
                    self.open
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .insert(id, target);
                }
            }
            Ok(Err(e)) => eprintln!("anibeam: notifications: {e}"),
            Err(_) => eprintln!("anibeam: notifications: the notification daemon did not answer"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(series: Option<u64>) -> Target {
        Target {
            series,
            anilist_id: 154587,
            title: "Frieren".to_string(),
        }
    }

    #[test]
    fn the_news_names_the_episode_and_the_show() {
        let (summary, _) = words(7, &target(Some(3)));
        assert_eq!(summary, "Episode 7 of Frieren is out");
    }

    /// A notification daemon of the test's own on a private bus: it keeps every Notify it
    /// gets and answers with ids from 7 up.
    /// One Notify as the daemon got it: app name, icon, summary, actions, body|desktop-entry.
    type Notified = (String, String, String, Vec<String>, String);
    type Seen = Arc<Mutex<Vec<Notified>>>;

    #[derive(Default)]
    struct Daemon {
        got: Seen,
    }

    #[zbus::interface(name = "org.freedesktop.Notifications")]
    impl Daemon {
        #[allow(clippy::too_many_arguments)]
        fn notify(
            &self,
            app_name: String,
            _replaces_id: u32,
            app_icon: String,
            summary: String,
            body: String,
            actions: Vec<String>,
            hints: HashMap<String, zbus::zvariant::OwnedValue>,
            _expire_timeout: i32,
        ) -> u32 {
            let entry = hints
                .get("desktop-entry")
                .and_then(|v| String::try_from(v.clone()).ok())
                .unwrap_or_default();
            let mut got = self.got.lock().unwrap();
            got.push((app_name, app_icon, summary, actions, body + "|" + &entry));
            6 + got.len() as u32
        }
    }

    /// Runs `body` against a dbus-daemon of its own, which goes when the test ends.
    fn on_private_bus<F, Fut>(body: F)
    where
        F: FnOnce(Connection, Connection, Seen) -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let dir = tempfile::tempdir().unwrap();
        let address = format!("unix:path={}", dir.path().join("bus").display());
        let mut daemon = std::process::Command::new("dbus-daemon")
            .args([
                "--session",
                "--nofork",
                "--nopidfile",
                &format!("--address={address}"),
            ])
            .stdout(std::process::Stdio::null())
            .spawn()
            .expect("dbus-daemon runs");
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let connect = || async {
                for _ in 0..100 {
                    if let Ok(c) = zbus::connection::Builder::address(address.as_str())
                        .unwrap()
                        .build()
                        .await
                    {
                        return c;
                    }
                    tokio::time::sleep(Duration::from_millis(20)).await;
                }
                panic!("the private bus never came up")
            };
            let server = connect().await;
            let fake = Daemon::default();
            let got = fake.got.clone();
            server.object_server().at(PATH, fake).await.unwrap();
            server.request_name(NAME).await.unwrap();
            let client = connect().await;
            body(server, client, got).await;
        });
        let _ = daemon.kill();
        let _ = daemon.wait();
    }

    #[test]
    fn a_notification_reaches_the_daemon_and_its_click_comes_back() {
        on_private_bus(|server, client, got| async move {
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
            let on_click: OnClick = Arc::new(move |t: Target, token: String| {
                let _ = tx.send((t, token));
            });
            let mut n = Notifier::on(client);
            n.show(target(Some(3)), 7, on_click.clone()).await;
            n.show(target(None), 8, on_click).await;
            let seen = got.lock().unwrap().clone();
            assert_eq!(seen.len(), 2);
            assert_eq!(seen[0].0, "AniBeam");
            assert_eq!(seen[0].1, crate::APP_ID);
            assert_eq!(seen[0].2, "Episode 7 of Frieren is out");
            assert_eq!(seen[0].3, vec!["default".to_string(), "Open".to_string()]);
            assert_eq!(
                seen[0].4,
                format!("Click to open it in AniBeam|{}", crate::APP_ID)
            );
            assert_eq!(
                seen[1].3,
                vec!["default".to_string(), "Get episodes".to_string()]
            );

            // The daemon hands over a token, says the second one was clicked, and closes
            // it, all at once as a real daemon does: the click still comes back
            server
                .emit_signal(None::<()>, PATH, NAME, "ActivationToken", &(8u32, "tok-1"))
                .await
                .unwrap();
            server
                .emit_signal(None::<()>, PATH, NAME, "ActionInvoked", &(8u32, "default"))
                .await
                .unwrap();
            server
                .emit_signal(None::<()>, PATH, NAME, "NotificationClosed", &(8u32, 2u32))
                .await
                .unwrap();
            let (clicked, token) = tokio::time::timeout(Duration::from_secs(5), rx.recv())
                .await
                .expect("the click comes back")
                .unwrap();
            assert_eq!(clicked, target(None));
            assert_eq!(token, "tok-1");

            // A closed notification is forgotten: a late click on it opens nothing
            server
                .emit_signal(None::<()>, PATH, NAME, "NotificationClosed", &(7u32, 2u32))
                .await
                .unwrap();
            server
                .emit_signal(None::<()>, PATH, NAME, "ActionInvoked", &(7u32, "default"))
                .await
                .unwrap();
            assert!(
                tokio::time::timeout(Duration::from_millis(500), rx.recv())
                    .await
                    .is_err(),
                "nothing opens"
            );
        });
    }

    #[test]
    fn the_second_line_says_what_a_click_does() {
        assert_eq!(words(7, &target(Some(3))).1, "Click to open it in AniBeam");
        assert_eq!(
            words(7, &target(None)).1,
            "Click to get the episodes in AniBeam"
        );
    }
}
