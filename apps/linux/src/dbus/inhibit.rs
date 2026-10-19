//! The idle inhibit on the session bus. The portal first, because it is the one door that
//! also works from a sandbox; org.freedesktop.ScreenSaver second, which is what hypridle
//! and the other idle daemons answer directly. Either hold is tied to this connection, so
//! it also goes if the process dies without releasing.

use std::collections::HashMap;

use zbus::zvariant::{OwnedObjectPath, Value};
use zbus::{Connection, Result};

const PORTAL: &str = "org.freedesktop.portal.Desktop";
/// Inhibit's flags are a bitmask: 1 logout, 2 user switch, 4 suspend, 8 idle.
const IDLE: u32 = 8;
const REASON: &str = "Playing an episode";

pub enum Hold {
    /// The Request object the portal returned; closing it releases the inhibit.
    Portal(OwnedObjectPath),
    /// The cookie UnInhibit takes back.
    ScreenSaver(u32),
}

pub struct Backend {
    conn: Option<Connection>,
}

impl Backend {
    pub fn new() -> Self {
        Backend { conn: None }
    }

    async fn connection(&mut self) -> Result<Connection> {
        if let Some(c) = &self.conn {
            return Ok(c.clone());
        }
        let c = Connection::session().await?;
        self.conn = Some(c.clone());
        Ok(c)
    }

    pub async fn acquire(&mut self) -> Option<Hold> {
        let conn = match self.connection().await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("anibeam: idle inhibit: no session bus: {e}");
                return None;
            }
        };
        let portal_error = match portal_inhibit(&conn).await {
            Ok(handle) => return Some(Hold::Portal(handle)),
            Err(e) => e,
        };
        match screensaver_inhibit(&conn).await {
            Ok(cookie) => Some(Hold::ScreenSaver(cookie)),
            Err(e) => {
                eprintln!("anibeam: idle inhibit: portal: {portal_error}; ScreenSaver: {e}");
                None
            }
        }
    }

    pub async fn release(&mut self, hold: Hold) {
        let Some(conn) = self.conn.clone() else {
            return;
        };
        let result = match hold {
            Hold::Portal(handle) => conn
                .call_method(
                    Some(PORTAL),
                    handle.as_str(),
                    Some("org.freedesktop.portal.Request"),
                    "Close",
                    &(),
                )
                .await
                .map(|_| ()),
            Hold::ScreenSaver(cookie) => conn
                .call_method(
                    Some("org.freedesktop.ScreenSaver"),
                    "/org/freedesktop/ScreenSaver",
                    Some("org.freedesktop.ScreenSaver"),
                    "UnInhibit",
                    &(cookie,),
                )
                .await
                .map(|_| ()),
        };
        if let Err(e) = result {
            eprintln!("anibeam: idle inhibit: release: {e}");
        }
    }
}

async fn portal_inhibit(conn: &Connection) -> Result<OwnedObjectPath> {
    let mut options: HashMap<&str, Value<'_>> = HashMap::new();
    options.insert("reason", Value::from(REASON));
    let reply = conn
        .call_method(
            Some(PORTAL),
            "/org/freedesktop/portal/desktop",
            Some("org.freedesktop.portal.Inhibit"),
            "Inhibit",
            &("", IDLE, options),
        )
        .await?;
    reply.body().deserialize()
}

async fn screensaver_inhibit(conn: &Connection) -> Result<u32> {
    let reply = conn
        .call_method(
            Some("org.freedesktop.ScreenSaver"),
            "/org/freedesktop/ScreenSaver",
            Some("org.freedesktop.ScreenSaver"),
            "Inhibit",
            &(crate::APP_ID, REASON),
        )
        .await?;
    reply.body().deserialize()
}
