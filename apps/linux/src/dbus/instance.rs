//! Single instance, spec 4.5: a flock under $XDG_RUNTIME_DIR before anything else; a
//! second launch that loses it hands its activation token to the running window over
//! org.freedesktop.Application and exits. The core knows nothing of any of this.

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::path::Path;

use cxx_qt::CxxQtThread;
use cxx_qt_lib::QString;
use rustix::fs::{FlockOperation, flock};
use zbus::zvariant::{OwnedValue, Value};

use crate::bridge::shell::qobject::Shell;

pub const BUS_NAME: &str = "com.marcusrosado.AniBeam";
pub const OBJECT_PATH: &str = "/com/marcusrosado/AniBeam";

/// The open file is the lock: closing it releases the flock, so this is held in `main` for
/// the life of the process and never read from.
pub struct Lock(#[allow(dead_code)] File);

pub fn try_lock(path: &Path) -> std::io::Result<Option<Lock>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)?;
    match flock(&file, FlockOperation::NonBlockingLockExclusive) {
        Ok(()) => Ok(Some(Lock(file))),
        Err(rustix::io::Errno::WOULDBLOCK) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// The a{sv} the Application interface carries. Empty without a token, which is what a
/// launch from a terminal looks like.
pub fn platform_data(token: Option<&str>) -> HashMap<String, Value<'static>> {
    let mut m = HashMap::new();
    if let Some(t) = token.filter(|t| !t.is_empty()) {
        m.insert("activation-token".to_string(), Value::from(t.to_string()));
    }
    m
}

/// The second launch's whole job: raise the running window, then exit 0.
pub async fn hand_off(action: Option<&str>) -> Result<(), String> {
    let conn = zbus::Connection::session()
        .await
        .map_err(|e| format!("no session bus: {e}"))?;
    let token = std::env::var("XDG_ACTIVATION_TOKEN").ok();
    let data = platform_data(token.as_deref());
    let result = match action {
        Some(name) => {
            conn.call_method(
                Some(BUS_NAME),
                OBJECT_PATH,
                Some("org.freedesktop.Application"),
                "ActivateAction",
                &(name, Vec::<Value<'_>>::new(), &data),
            )
            .await
        }
        None => {
            conn.call_method(
                Some(BUS_NAME),
                OBJECT_PATH,
                Some("org.freedesktop.Application"),
                "Activate",
                &(&data,),
            )
            .await
        }
    };
    result
        .map(|_| ())
        .map_err(|e| format!("could not reach the running AniBeam: {e}"))
}

pub struct AppInterface {
    shell: CxxQtThread<Shell>,
}

impl AppInterface {
    fn raise(&self, platform_data: &HashMap<String, OwnedValue>) {
        // zvariant's OwnedValue borrows out as &str rather than String, so the token is
        // copied here and moved into the closure the Qt thread runs.
        let token = platform_data
            .get("activation-token")
            .and_then(|v| <&str>::try_from(v).ok())
            .unwrap_or_default()
            .to_string();
        self.shell
            .queue(move |shell| shell.activate_requested(QString::from(&token)))
            .ok();
    }
}

#[zbus::interface(name = "org.freedesktop.Application")]
impl AppInterface {
    fn activate(&self, platform_data: HashMap<String, OwnedValue>) {
        self.raise(&platform_data);
    }
    /// The app opens nothing from the launcher: Open behaves as Activate.
    fn open(&self, _uris: Vec<String>, platform_data: HashMap<String, OwnedValue>) {
        self.raise(&platform_data);
    }
    /// The action table is empty today; an unknown action raises the window.
    fn activate_action(
        &self,
        action_name: String,
        _parameter: Vec<OwnedValue>,
        platform_data: HashMap<String, OwnedValue>,
    ) {
        eprintln!("anibeam: no action named {action_name}");
        self.raise(&platform_data);
    }
}

pub async fn serve(conn: &zbus::Connection, shell: CxxQtThread<Shell>) -> zbus::Result<()> {
    conn.object_server()
        .at(OBJECT_PATH, AppInterface { shell })
        .await?;
    conn.request_name(BUS_NAME).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_second_lock_on_one_file_is_refused_until_the_first_drops() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("anibeam.lock");
        let first = try_lock(&path).unwrap();
        assert!(first.is_some());
        assert!(try_lock(&path).unwrap().is_none(), "held");
        drop(first);
        assert!(try_lock(&path).unwrap().is_some(), "free again");
    }

    #[test]
    fn platform_data_carries_the_token_when_set() {
        let d = platform_data(Some("tok123"));
        assert_eq!(
            d.get("activation-token")
                .and_then(|v| v.downcast_ref::<String>().ok()),
            Some("tok123".to_string())
        );
        assert!(platform_data(None).is_empty());
    }
}
