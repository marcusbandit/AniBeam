//! Waking from a suspend, as logind says it on the system bus: `PrepareForSleep(true)` just
//! before the machine sleeps and `PrepareForSleep(false)` once it runs again. A suspend
//! stops the clock the core's timers sleep on, so on each wake the core is told, and the
//! episode out alarm is set again at once (#594). A machine with no logind or no system bus,
//! or a bus that goes away, says so on stderr and nothing else changes: the alarm's own cap
//! still covers it.

use futures_util::{Stream, StreamExt};
use zbus::{Connection, Message, MessageStream};

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
    match subscribe(&conn).await {
        Ok(signals) => {
            hear(signals, on_wake).await;
            eprintln!("anibeam: waking from a suspend goes unheard: the system bus went away");
        }
        Err(e) => eprintln!("anibeam: waking from a suspend goes unheard: {e}"),
    }
}

/// logind's `PrepareForSleep` on `conn`. The rule is in place once this answers.
async fn subscribe(conn: &Connection) -> zbus::Result<MessageStream> {
    let rule = zbus::MatchRule::builder()
        .msg_type(zbus::message::Type::Signal)
        .interface(MANAGER)?
        .path(PATH)?
        .member("PrepareForSleep")?
        .build();
    MessageStream::for_match_rule(rule, conn, None).await
}

/// Calls `on_wake` for each `PrepareForSleep(false)`, until the stream ends.
async fn hear(signals: impl Stream<Item = zbus::Result<Message>>, on_wake: impl Fn()) {
    let mut signals = std::pin::pin!(signals);
    while let Some(msg) = signals.next().await {
        let Ok(msg) = msg else { continue };
        if msg.body().deserialize::<bool>() == Ok(false) {
            on_wake();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dbus::testbus::PrivateBus;
    use std::time::Duration;

    fn prepare_for_sleep(sleeping: bool) -> zbus::Result<Message> {
        Message::signal(PATH, MANAGER, "PrepareForSleep")?.build(&sleeping)
    }

    #[test]
    fn only_the_wake_counts() {
        let woke = std::cell::Cell::new(0);
        let signals = futures_util::stream::iter([
            prepare_for_sleep(true),
            prepare_for_sleep(false),
            prepare_for_sleep(true),
            prepare_for_sleep(false),
        ]);
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(hear(signals, || woke.set(woke.get() + 1)));
        assert_eq!(woke.get(), 2, "two wakes, and going to sleep wakes nothing");
    }

    #[test]
    fn a_wake_on_the_bus_is_heard() {
        let bus = PrivateBus::start();
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let logind = bus.connect().await;
            let client = bus.connect().await;
            let signals = subscribe(&client).await.unwrap();
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
            tokio::spawn(hear(signals, move || {
                let _ = tx.send(());
            }));
            logind
                .emit_signal(None::<()>, PATH, MANAGER, "PrepareForSleep", &false)
                .await
                .unwrap();
            tokio::time::timeout(Duration::from_secs(5), rx.recv())
                .await
                .expect("the wake is heard")
                .unwrap();
        });
    }
}
