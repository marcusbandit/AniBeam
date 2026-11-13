//! Waking from a suspend, as logind says it on the system bus: `PrepareForSleep(true)` just
//! before the machine sleeps and `PrepareForSleep(false)` once it runs again. A suspend
//! stops the clock the core's timers sleep on, so on each wake the core is told, and the
//! episode out alarm is set again at once (#594). A machine with no logind or no system bus
//! says so on stderr and nothing else changes: the alarm's own cap still covers it.

use futures_util::StreamExt;
use zbus::Connection;

const PATH: &str = "/org/freedesktop/login1";
const MANAGER: &str = "org.freedesktop.login1.Manager";

/// Listens on the system bus for as long as the process runs, calling `on_wake` after each
/// suspend.
pub async fn start(on_wake: impl Fn() + Send + 'static) {
    let conn = match Connection::system().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("anibeam: waking from a suspend goes unheard: no system bus: {e}");
            return;
        }
    };
    if let Err(e) = listen(&conn, on_wake).await {
        eprintln!("anibeam: waking from a suspend goes unheard: {e}");
    }
}

/// Calls `on_wake` for each `PrepareForSleep(false)` on `conn`, until the bus goes.
async fn listen(conn: &Connection, on_wake: impl Fn()) -> zbus::Result<()> {
    let rule = zbus::MatchRule::builder()
        .msg_type(zbus::message::Type::Signal)
        .interface(MANAGER)?
        .path(PATH)?
        .member("PrepareForSleep")?
        .build();
    let mut signals = zbus::MessageStream::for_match_rule(rule, conn, None).await?;
    while let Some(msg) = signals.next().await {
        let Ok(msg) = msg else { continue };
        if msg.body().deserialize::<bool>() == Ok(false) {
            on_wake();
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// Runs `body` against a dbus-daemon of its own, which goes when the test ends.
    fn on_private_bus<F, Fut>(body: F)
    where
        F: FnOnce(Connection, Connection) -> Fut,
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
            let logind = connect().await;
            let client = connect().await;
            body(logind, client).await;
        });
        let _ = daemon.kill();
        let _ = daemon.wait();
    }

    #[test]
    fn a_wake_is_heard_and_going_to_sleep_is_not() {
        on_private_bus(|logind, client| async move {
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
            tokio::spawn(async move {
                let _ = listen(&client, move || {
                    let _ = tx.send(());
                })
                .await;
            });
            // The listener's task gets its moment to add its match rule.
            tokio::time::sleep(Duration::from_millis(200)).await;
            let say = |sleeping: bool| {
                let logind = logind.clone();
                async move {
                    logind
                        .emit_signal(None::<()>, PATH, MANAGER, "PrepareForSleep", &sleeping)
                        .await
                        .unwrap();
                }
            };
            say(true).await;
            assert!(
                tokio::time::timeout(Duration::from_millis(300), rx.recv())
                    .await
                    .is_err(),
                "going to sleep wakes nothing"
            );
            say(false).await;
            tokio::time::timeout(Duration::from_secs(5), rx.recv())
                .await
                .expect("the wake is heard")
                .unwrap();
        });
    }
}
