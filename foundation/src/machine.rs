//! What this computer calls itself and the addresses its interfaces hold. The
//! host serves on them and the downloads ask them whether a torrent client
//! runs here.

use std::net::IpAddr;

/// The machine's name, for `Hello::host`.
pub fn hostname() -> String {
    #[cfg(unix)]
    return rustix::system::uname()
        .nodename()
        .to_string_lossy()
        .into_owned();
    #[cfg(not(unix))]
    return std::env::var("COMPUTERNAME").unwrap_or_else(|_| "Windows".to_string());
}

/// Every address on this machine's interfaces. A machine whose interfaces
/// cannot be listed has none to offer.
pub fn local_addresses() -> Vec<IpAddr> {
    if_addrs::get_if_addrs()
        .map(|all| all.into_iter().map(|i| i.ip()).collect())
        .unwrap_or_default()
}
