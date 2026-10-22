//! A whole sync against scripts/mock-host.py, the dev stand-in for the desktop's server.
//! Skips when python3 is not on PATH.

use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use anibeam_companion::address::HostAddress;
use anibeam_companion::client::HostError;
use anibeam_companion::store::Store;
use anibeam_companion::sync::Syncer;

struct Mock(Child);

impl Drop for Mock {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

fn start_mock(port: u16) -> Option<Mock> {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../scripts/mock-host.py");
    let child = Command::new("python3")
        .arg(script)
        .args(["--port", &port.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let mock = Mock(child);
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return Some(mock);
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    None
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("anibeam-mock-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn a_sync_stores_the_snapshot_and_the_posters() {
    let port = free_port();
    let Some(_mock) = start_mock(port) else {
        eprintln!("skipped: python3 or the mock host would not start");
        return;
    };
    let address = HostAddress::parse(&format!("127.0.0.1:{port}")).unwrap();
    let syncer = Syncer::new(Store::open(scratch("sync")).unwrap());

    let hello = syncer.test(&address).unwrap();
    assert_eq!(hello.host, "banditbox");

    let cache = syncer.fetch_snapshot(&address, 1_000).unwrap();
    assert_eq!(cache.host, "banditbox");
    assert_eq!(syncer.store().cache().unwrap(), cache);

    let mut fetched = Vec::new();
    syncer.fetch_posters(&address, &cache, |id| fetched.push(id));
    fetched.sort();
    assert_eq!(fetched, vec![3, 12], "Perfect Blue has no poster");
    assert!(syncer.store().poster(12).is_some_and(|b| b.len() > 100));

    // A second sync with the same tags fetches nothing.
    let mut again = Vec::new();
    syncer.fetch_posters(&address, &cache, |id| again.push(id));
    assert!(again.is_empty());
}

#[test]
fn a_host_that_is_not_there_is_unreachable() {
    let port = free_port();
    let address = HostAddress::parse(&format!("127.0.0.1:{port}")).unwrap();
    let syncer = Syncer::new(Store::open(scratch("down")).unwrap());
    match syncer.fetch_snapshot(&address, 1_000) {
        Err(HostError::Unreachable(why)) => assert!(why.contains("nothing listens"), "{why}"),
        other => panic!("expected unreachable, got {other:?}"),
    }
}
