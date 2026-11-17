//! A dbus-daemon of a test's own, for the tests that talk over a bus. It goes when the value
//! drops, so a failed assertion leaves no daemon behind.

use std::time::Duration;

use zbus::Connection;

pub struct PrivateBus {
    daemon: std::process::Child,
    address: String,
    _dir: tempfile::TempDir,
}

impl PrivateBus {
    pub fn start() -> PrivateBus {
        let dir = tempfile::tempdir().unwrap();
        let address = format!("unix:path={}", dir.path().join("bus").display());
        let daemon = std::process::Command::new("dbus-daemon")
            .args([
                "--session",
                "--nofork",
                "--nopidfile",
                &format!("--address={address}"),
            ])
            .stdout(std::process::Stdio::null())
            .spawn()
            .expect("dbus-daemon runs");
        PrivateBus {
            daemon,
            address,
            _dir: dir,
        }
    }

    /// A new connection to the bus, once the daemon answers.
    pub async fn connect(&self) -> Connection {
        for _ in 0..100 {
            if let Ok(c) = zbus::connection::Builder::address(self.address.as_str())
                .unwrap()
                .build()
                .await
            {
                return c;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        panic!("the private bus never came up")
    }
}

impl Drop for PrivateBus {
    fn drop(&mut self) {
        let _ = self.daemon.kill();
        let _ = self.daemon.wait();
    }
}
