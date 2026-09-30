//! Single instance, spec 4.5: a flock under $XDG_RUNTIME_DIR before anything else; a
//! second launch that loses it hands its activation token to the running window over
//! org.freedesktop.Application and exits. The core knows nothing of any of this.

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::time::Duration;

use cxx_qt::CxxQtThread;
use cxx_qt_lib::QString;
use rustix::fs::{FlockOperation, flock};
use zbus::zvariant::{OwnedValue, Value};

use crate::bridge::shell::qobject::Shell;

/// The element a run under --root adds to every well known name it owns (R35). A sandbox
/// exists so a dev run touches nothing outside itself, and the desktop's idea of which
/// process is AniBeam is outside itself: without this a sandbox takes the real app's names,
/// because zbus asks for a name with ReplaceExisting set.
const SANDBOX: &str = "Sandbox";

/// The application name: the app id, plus the sandbox element under --root.
fn app_name(sandboxed: bool) -> String {
    match sandboxed {
        true => format!("{}.{SANDBOX}", crate::APP_ID),
        false => crate::APP_ID.to_string(),
    }
}

/// The object path is the name with its dots as slashes, the convention every D-Bus
/// service follows, so a reader who has the name can guess the path.
fn app_path(sandboxed: bool) -> String {
    format!("/{}", app_name(sandboxed).replace('.', "/"))
}

/// What follows org.mpris.MediaPlayer2. in the media player's name. Lowercase, because
/// that half of the name is the binary's, and the sandbox adds its element the same way.
fn mpris_suffix(sandboxed: bool) -> String {
    match sandboxed {
        true => format!("anibeam.{}", SANDBOX.to_lowercase()),
        false => "anibeam".to_string(),
    }
}

/// A run under --root keeps its names to itself.
fn sandboxed() -> bool {
    crate::runtime::args().root.is_some()
}

pub fn bus_name() -> String {
    app_name(sandboxed())
}
pub fn object_path() -> String {
    app_path(sandboxed())
}
pub fn mpris_bus_suffix() -> String {
    mpris_suffix(sandboxed())
}

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

/// The token out of the a{sv} that arrived, empty when there is none. zvariant's
/// OwnedValue borrows out as &str rather than String, so the token is copied here and the
/// copy is what the Qt thread's closure takes.
fn token_of(platform_data: &HashMap<String, OwnedValue>) -> String {
    platform_data
        .get("activation-token")
        .and_then(|v| <&str>::try_from(v).ok())
        .unwrap_or_default()
        .to_string()
}

/// A bus that accepted the connection but never answers must not hold the launch open: two
/// seconds is far longer than a running window needs and far shorter than zbus's own
/// timeout, which would leave a launcher's spinner up for 25 seconds with nothing to show.
const HAND_OFF_TIMEOUT: Duration = Duration::from_secs(2);

/// The second launch's whole job: raise the running window, then exit 0.
pub async fn hand_off(action: Option<&str>) -> Result<(), String> {
    match tokio::time::timeout(HAND_OFF_TIMEOUT, reach(action)).await {
        Ok(result) => result,
        Err(_) => Err("did not answer within two seconds".to_string()),
    }
}

async fn reach(action: Option<&str>) -> Result<(), String> {
    let conn = zbus::Connection::session()
        .await
        .map_err(|e| format!("no session bus: {e}"))?;
    let token = std::env::var("XDG_ACTIVATION_TOKEN").ok();
    let data = platform_data(token.as_deref());
    let name = bus_name();
    let path = object_path();
    let result = match action {
        Some(action_name) => {
            conn.call_method(
                Some(name.as_str()),
                path.as_str(),
                Some("org.freedesktop.Application"),
                "ActivateAction",
                &(action_name, Vec::<Value<'_>>::new(), &data),
            )
            .await
        }
        None => {
            conn.call_method(
                Some(name.as_str()),
                path.as_str(),
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
        let token = token_of(platform_data);
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
        .at(object_path(), AppInterface { shell })
        .await?;
    conn.request_name(bus_name()).await?;
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

    /// The wire turns every Value into an OwnedValue on the way in, and the interface reads
    /// the token back out of that. This is both halves against each other.
    #[test]
    fn the_token_survives_the_trip_the_interface_takes_it_on() {
        let sent: HashMap<String, OwnedValue> = platform_data(Some("tok123"))
            .into_iter()
            .map(|(k, v)| (k, OwnedValue::try_from(v).unwrap()))
            .collect();
        assert_eq!(token_of(&sent), "tok123");
        assert_eq!(token_of(&HashMap::new()), "", "no token is the empty one");
    }

    #[test]
    fn a_sandbox_owns_its_own_names_and_nothing_else_moves() {
        assert_eq!(app_name(false), "com.marcusrosado.AniBeam");
        assert_eq!(app_path(false), "/com/marcusrosado/AniBeam");
        assert_eq!(mpris_suffix(false), "anibeam");
        assert_eq!(app_name(true), "com.marcusrosado.AniBeam.Sandbox");
        assert_eq!(app_path(true), "/com/marcusrosado/AniBeam/Sandbox");
        assert_eq!(mpris_suffix(true), "anibeam.sandbox");
    }
}
