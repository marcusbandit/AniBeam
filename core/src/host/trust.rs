//! Who the host answers. Tailscale is a trusted network: every device on the tailnet is
//! trusted by default, and the person can switch any one of them off in Settings > Phone
//! app (#313). This machine itself is always trusted. Anything else gets nothing.
//!
//! The decision is a pure function of the asking address, the switched-off devices and
//! what `tailscale status --json` says the tailnet holds, so every case is tested without
//! a network. The host asks it for every request, before any route runs.
//!
//! Who asks follows the device too (#631): a device belongs to a Tailscale user, and the
//! host answers that person's own profile ([`asker`]). This machine and the devices of the
//! person it belongs to are the owner's.
//!
//! Trust follows the address a request comes from, as Tailscale delivers it. A device that
//! forwards other traffic onto the tailnet (a subnet router that rewrites the source, a
//! proxy on a trusted device) makes that traffic look like its own, so it is trusted as
//! that device is.

use std::net::IpAddr;

use super::addr;

/// A device on the tailnet, as `tailscale status --json` names it.
#[derive(Clone, Debug, PartialEq)]
pub struct Peer {
    /// Tailscale's stable node id, which outlives a changed address or name.
    pub id: String,
    /// The machine's own name ("pixel-10").
    pub name: String,
    /// The operating system Tailscale reports ("android", "linux").
    pub os: String,
    pub online: bool,
    pub ips: Vec<IpAddr>,
    /// Talking to this machine now with no direct address and a home relay:
    /// Tailscale carries it through DERP. An idle device has no direct
    /// address either, so it is not counted.
    pub relayed: bool,
    /// The Tailscale user the device belongs to, by Tailscale's user id.
    pub user: Option<String>,
    /// Tagged in the tailnet's ACLs: such a device belongs to the tailnet, not to a person.
    pub tagged: bool,
}

/// A person on the tailnet: the Tailscale user a device belongs to.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TailnetUser {
    /// Tailscale's stable user id.
    pub id: String,
    /// "someone@example.com", or a GitHub login.
    pub login: String,
    /// The name Tailscale shows for them.
    pub name: String,
}

/// This machine's tailnet addresses and every other device on it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Tailnet {
    pub me: Vec<IpAddr>,
    pub peers: Vec<Peer>,
    /// The Tailscale user this machine belongs to. None for a tagged machine, which
    /// belongs to no person, so the host cannot tell whose computer it is.
    pub me_user: Option<String>,
    /// Every user the status names, by id.
    pub users: Vec<TailnetUser>,
}

impl TailnetUser {
    /// What to call them: their Tailscale name, or their login when it gives none. None
    /// with neither, so no screen shows a bare user id (#633).
    pub fn shown(&self) -> Option<&str> {
        [self.name.trim(), self.login.trim()]
            .into_iter()
            .find(|s| !s.is_empty())
    }
}

impl Tailnet {
    pub fn peer(&self, ip: IpAddr) -> Option<&Peer> {
        self.peers.iter().find(|p| p.ips.contains(&ip))
    }

    pub fn user(&self, id: &str) -> Option<&TailnetUser> {
        self.users.iter().find(|u| u.id == id)
    }

    /// The person this machine belongs to, when the status names them.
    pub fn owner(&self) -> Option<&TailnetUser> {
        self.me_user.as_deref().and_then(|id| self.user(id))
    }
}

/// Whose request it is (#631).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Asker {
    /// This machine, or a device of the person it belongs to.
    Owner,
    /// A device of someone else on the tailnet.
    Person(TailnetUser),
    /// A tailnet device the host cannot tie to a person.
    Unknown,
}

/// Who is asking from `from`. `tailnet` is None when `tailscale status` has never been
/// read: the host then cannot tell anyone apart, so every device it trusts is the
/// owner's, as before there were profiles. The same goes for a status that names no
/// user for this machine.
pub fn asker(from: IpAddr, tailnet: Option<&Tailnet>) -> Asker {
    let from = canonical(from);
    if from.is_loopback() {
        return Asker::Owner;
    }
    let Some(t) = tailnet else {
        return Asker::Owner;
    };
    if t.me.contains(&from) {
        return Asker::Owner;
    }
    whose(t, t.peer(from))
}

/// Whose profile a device of the tailnet is answered with: the asker it makes, by the
/// rules of [`asker`]. Settings lists each device under that person (#633).
pub fn whose(t: &Tailnet, peer: Option<&Peer>) -> Asker {
    let Some(me) = t.me_user.as_deref() else {
        return Asker::Owner;
    };
    // A tagged device belongs to the tailnet rather than to anyone: the owner's.
    if peer.is_some_and(|p| p.tagged) {
        return Asker::Owner;
    }
    match peer.and_then(|p| p.user.as_deref()) {
        Some(user) if user == me => Asker::Owner,
        Some(user) => Asker::Person(t.user(user).cloned().unwrap_or(TailnetUser {
            id: user.to_string(),
            ..TailnetUser::default()
        })),
        None => Asker::Unknown,
    }
}

/// What a device the host cannot tie to a person is told.
pub const UNKNOWN_PERSON: &str =
    "That computer cannot tell whose device this is on the tailnet, so it does not answer.";

/// Whether a node carries ACL tags.
fn tagged(node: &serde_json::Value) -> bool {
    node["Tags"].as_array().is_some_and(|t| !t.is_empty())
}

/// A user id as the status writes it: a number, or a string in some builds.
fn user_id(v: &serde_json::Value) -> Option<String> {
    match v {
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::String(s) if !s.is_empty() => Some(s.clone()),
        _ => None,
    }
    .filter(|id| id != "0")
}

/// The tailnet out of `tailscale status --json`. None when it is not that document.
pub fn read_status(json: &[u8]) -> Option<Tailnet> {
    let v: serde_json::Value = serde_json::from_slice(json).ok()?;
    let ips = |node: &serde_json::Value| -> Vec<IpAddr> {
        node["TailscaleIPs"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|ip| ip.as_str()?.parse().ok())
                    .collect()
            })
            .unwrap_or_default()
    };
    let me = ips(v.get("Self")?);
    let mut peers: Vec<Peer> = v["Peer"]
        .as_object()
        .map(|all| {
            all.values()
                .filter_map(|p| {
                    let id = p["ID"].as_str().filter(|s| !s.is_empty())?.to_string();
                    let name = p["HostName"]
                        .as_str()
                        .filter(|s| !s.is_empty())
                        .map(str::to_string)
                        .or_else(|| {
                            p["DNSName"]
                                .as_str()
                                .and_then(|d| d.split('.').next())
                                .filter(|s| !s.is_empty())
                                .map(str::to_string)
                        })
                        .unwrap_or_else(|| id.clone());
                    Some(Peer {
                        id,
                        name,
                        os: p["OS"].as_str().unwrap_or_default().to_string(),
                        online: p["Online"].as_bool().unwrap_or(false),
                        ips: ips(p),
                        relayed: p["Active"].as_bool().unwrap_or(false)
                            && p["CurAddr"].as_str().unwrap_or_default().is_empty()
                            && !p["Relay"].as_str().unwrap_or_default().is_empty(),
                        user: user_id(&p["UserID"]),
                        tagged: tagged(p),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    peers.sort_by_key(|p| p.name.to_lowercase());
    let mut users: Vec<TailnetUser> = v["User"]
        .as_object()
        .map(|all| {
            all.iter()
                .filter_map(|(key, u)| {
                    let id = user_id(&u["ID"]).or_else(|| Some(key.clone()))?;
                    Some(TailnetUser {
                        id,
                        login: u["LoginName"].as_str().unwrap_or_default().to_string(),
                        name: u["DisplayName"].as_str().unwrap_or_default().to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    users.sort_by(|a, b| a.id.cmp(&b.id));
    // A tagged machine reports Tailscale's shared tagged-devices user, which is no person.
    let me_user = Some(&v["Self"])
        .filter(|s| !tagged(s))
        .and_then(|s| user_id(&s["UserID"]));
    Some(Tailnet {
        me,
        peers,
        me_user,
        users,
    })
}

/// The tailnet as far as this machine alone can tell, for before `tailscale status` has
/// ever been read: its own tailnet addresses, from its interfaces, and no other device.
pub fn local_tailnet(all: &[IpAddr]) -> Tailnet {
    Tailnet {
        me: addr::pick(all),
        ..Tailnet::default()
    }
}

/// What the host does with one request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// This machine, or a trusted device on the tailnet.
    Serve,
    /// A tailnet device the person switched off, by its name.
    SwitchedOff(String),
    /// A tailnet address the host cannot place while some device is switched off: it
    /// might be that device, so it is refused.
    Unknown,
    /// Not on the tailnet at all.
    Outside,
}

/// Whether to answer a request from `from`. `off` holds the ids of the switched-off
/// devices; `tailnet` is None when `tailscale status` could not be read.
pub fn decide(from: IpAddr, off: &[String], tailnet: Option<&Tailnet>) -> Verdict {
    let from = canonical(from);
    if from.is_loopback() {
        return Verdict::Serve;
    }
    if !addr::is_tailscale(&from) {
        return Verdict::Outside;
    }
    if let Some(t) = tailnet {
        if t.me.contains(&from) {
            return Verdict::Serve;
        }
        if let Some(p) = t.peer(from) {
            return if off.contains(&p.id) {
                Verdict::SwitchedOff(p.name.clone())
            } else {
                Verdict::Serve
            };
        }
    }
    if off.is_empty() {
        // Every device on the tailnet is trusted, so one the status has not named yet is
        // too: it reached a tailnet address.
        Verdict::Serve
    } else {
        Verdict::Unknown
    }
}

/// Whether deciding on `from` needs the tailnet read at all: only a tailnet
/// address while some device is off does. This machine, anything off the
/// tailnet, and the whole tailnet while nothing is off are decided by the
/// address alone, so they never wait on `tailscale`.
pub fn needs_tailnet(from: IpAddr, off: &[String]) -> bool {
    let from = canonical(from);
    !off.is_empty() && !from.is_loopback() && addr::is_tailscale(&from)
}

/// An IPv4 address that arrived mapped into IPv6 (`::ffff:100.64.0.1`), as itself.
pub(crate) fn canonical(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(v6) => v6.to_ipv4_mapped().map_or(ip, IpAddr::V4),
        v4 => v4,
    }
}

/// What a refused device is told.
pub fn refusal(v: &Verdict) -> Option<&'static str> {
    match v {
        Verdict::Serve => None,
        Verdict::SwitchedOff(_) => Some(
            "This device is switched off on that computer. Turn it back on there, under Settings, Phone app.",
        ),
        Verdict::Unknown => Some(
            "That computer cannot tell which device this is on the tailnet, so it does not answer.",
        ),
        Verdict::Outside => Some("This computer answers only devices on your tailnet."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    /// #631: a status with people in it. The phone is the owner's, the tablet a
    /// friend's, and the laptop names no user.
    const PEOPLE: &[u8] = br#"{
        "Self": {"ID": "nSelf", "UserID": 100, "TailscaleIPs": ["100.65.105.9"]},
        "Peer": {
            "nodekey:aa": {"ID": "nPhone", "HostName": "pixel", "UserID": 100, "TailscaleIPs": ["100.70.1.2"]},
            "nodekey:bb": {"ID": "nTablet", "HostName": "tab", "UserID": 200, "TailscaleIPs": ["100.70.1.3"]},
            "nodekey:cc": {"ID": "nLaptop", "HostName": "lap", "TailscaleIPs": ["100.70.1.4"]}
        },
        "User": {
            "100": {"ID": 100, "LoginName": "owner@example.com", "DisplayName": "Owner"},
            "200": {"ID": 200, "LoginName": "friend@example.com", "DisplayName": "Friend"}
        }
    }"#;

    #[test]
    fn each_device_is_asked_for_by_the_person_it_belongs_to() {
        let t = read_status(PEOPLE).unwrap();
        assert_eq!(t.owner().map(|u| u.name.as_str()), Some("Owner"));
        assert_eq!(asker(ip("127.0.0.1"), Some(&t)), Asker::Owner);
        assert_eq!(
            asker(ip("100.65.105.9"), Some(&t)),
            Asker::Owner,
            "this machine"
        );
        assert_eq!(
            asker(ip("100.70.1.2"), Some(&t)),
            Asker::Owner,
            "the owner's phone"
        );
        assert_eq!(
            asker(ip("100.70.1.3"), Some(&t)),
            Asker::Person(TailnetUser {
                id: "200".into(),
                login: "friend@example.com".into(),
                name: "Friend".into(),
            })
        );
        assert_eq!(
            asker(ip("100.70.1.4"), Some(&t)),
            Asker::Unknown,
            "no user named"
        );
        assert_eq!(
            asker(ip("100.99.9.9"), Some(&t)),
            Asker::Unknown,
            "not in the status"
        );
    }

    /// Review of #661: a tagged machine reports Tailscale's tagged-devices user as its
    /// own, which is nobody, so the host cannot tell whose computer it is and every
    /// device is the owner's, as before profiles. A tagged peer is the owner's too.
    #[test]
    fn a_tagged_machine_cannot_tell_people_apart() {
        let status = br#"{
            "Self": {"ID": "nSelf", "UserID": 9, "Tags": ["tag:server"], "TailscaleIPs": ["100.65.105.9"]},
            "Peer": {
                "nodekey:aa": {"ID": "nPhone", "HostName": "pixel", "UserID": 100, "TailscaleIPs": ["100.70.1.2"]},
                "nodekey:bb": {"ID": "nTv", "HostName": "tv", "UserID": 9, "Tags": ["tag:tv"], "TailscaleIPs": ["100.70.1.5"]}
            }
        }"#;
        let t = read_status(status).unwrap();
        assert_eq!(t.me_user, None);
        assert_eq!(asker(ip("100.70.1.2"), Some(&t)), Asker::Owner);
        let t = read_status(PEOPLE).unwrap();
        let mut t = t;
        t.peers.push(Peer {
            id: "nTv".into(),
            name: "tv".into(),
            os: String::new(),
            online: true,
            ips: vec![ip("100.70.1.5")],
            relayed: false,
            user: Some("9".into()),
            tagged: true,
        });
        assert_eq!(asker(ip("100.70.1.5"), Some(&t)), Asker::Owner);
    }

    #[test]
    fn with_no_people_to_tell_apart_every_device_is_the_owners() {
        // Never read, or a status that names no user for this machine: as before profiles.
        assert_eq!(asker(ip("100.70.1.3"), None), Asker::Owner);
        let t = read_status(STATUS).unwrap();
        assert_eq!(t.me_user, None);
        assert_eq!(asker(ip("100.70.1.2"), Some(&t)), Asker::Owner);
    }

    const STATUS: &[u8] = br#"{
        "Self": {"ID": "nSelf", "HostName": "banditbox", "TailscaleIPs": ["100.65.105.9", "fd7a:115c:a1e0::9"]},
        "Peer": {
            "nodekey:aa": {"ID": "nPhone", "HostName": "pixel", "OS": "android", "Online": true,
                           "TailscaleIPs": ["100.70.1.2", "fd7a:115c:a1e0::2"]},
            "nodekey:bb": {"ID": "nLaptop", "HostName": "", "DNSName": "kangaeru.tail24c6a9.ts.net.",
                           "OS": "linux", "Online": false, "TailscaleIPs": ["100.80.3.4"]},
            "nodekey:cc": {"HostName": "no-id", "TailscaleIPs": ["100.90.0.1"]}
        }
    }"#;

    #[test]
    fn the_status_names_every_device_with_an_id() {
        let t = read_status(STATUS).unwrap();
        assert_eq!(t.me, vec![ip("100.65.105.9"), ip("fd7a:115c:a1e0::9")]);
        let names: Vec<_> = t.peers.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["kangaeru", "pixel"]);
        let phone = t.peer(ip("fd7a:115c:a1e0::2")).unwrap();
        assert_eq!(
            (phone.id.as_str(), phone.os.as_str(), phone.online),
            ("nPhone", "android", true)
        );
        assert_eq!(read_status(b"not json"), None);
        assert_eq!(read_status(b"{}"), None);
    }

    #[test]
    fn an_active_device_with_no_direct_address_is_relayed() {
        let t = read_status(br#"{"Self": {"TailscaleIPs": []}, "Peer": {
            "a": {"ID": "nDirect", "HostName": "a", "Active": true, "CurAddr": "192.168.1.20:41641", "Relay": "fra"},
            "b": {"ID": "nRelay", "HostName": "b", "Active": true, "CurAddr": "", "Relay": "fra"},
            "c": {"ID": "nUnknown", "HostName": "c", "Active": true},
            "d": {"ID": "nIdle", "HostName": "d", "Active": false, "CurAddr": "", "Relay": "fra"}
        }}"#)
        .unwrap();
        let relayed: Vec<_> = t.peers.iter().map(|p| (p.id.as_str(), p.relayed)).collect();
        assert_eq!(
            relayed,
            [
                ("nDirect", false),
                ("nRelay", true),
                ("nUnknown", false),
                ("nIdle", false)
            ]
        );
    }

    #[test]
    fn this_machines_own_addresses_are_known_without_a_status_read() {
        let mine = local_tailnet(&[ip("192.168.1.5"), ip("100.65.105.9"), ip("::1")]);
        assert_eq!(mine.me, vec![ip("100.65.105.9")]);
        assert!(mine.peers.is_empty());
        let off = vec!["nPhone".to_string()];
        assert_eq!(
            decide(ip("100.65.105.9"), &off, Some(&mine)),
            Verdict::Serve
        );
        // Anyone else is still unplaceable while a device is off.
        assert_eq!(
            decide(ip("100.70.1.2"), &off, Some(&mine)),
            Verdict::Unknown
        );
    }

    #[test]
    fn this_machine_is_always_served() {
        let t = read_status(STATUS).unwrap();
        let off = vec!["nPhone".to_string()];
        assert_eq!(decide(ip("127.0.0.1"), &off, None), Verdict::Serve);
        assert_eq!(decide(ip("::1"), &off, Some(&t)), Verdict::Serve);
        assert_eq!(decide(ip("100.65.105.9"), &off, Some(&t)), Verdict::Serve);
    }

    #[test]
    fn every_tailnet_device_is_trusted_by_default() {
        let t = read_status(STATUS).unwrap();
        assert_eq!(decide(ip("100.70.1.2"), &[], Some(&t)), Verdict::Serve);
        assert_eq!(decide(ip("100.80.3.4"), &[], Some(&t)), Verdict::Serve);
        // One the status has not caught up with yet, and no status at all.
        assert_eq!(decide(ip("100.99.9.9"), &[], Some(&t)), Verdict::Serve);
        assert_eq!(decide(ip("100.70.1.2"), &[], None), Verdict::Serve);
    }

    #[test]
    fn a_switched_off_device_is_refused_on_every_address_it_has() {
        let t = read_status(STATUS).unwrap();
        let off = vec!["nPhone".to_string()];
        let refused = Verdict::SwitchedOff("pixel".into());
        assert_eq!(decide(ip("100.70.1.2"), &off, Some(&t)), refused);
        assert_eq!(decide(ip("fd7a:115c:a1e0::2"), &off, Some(&t)), refused);
        assert_eq!(decide(ip("::ffff:100.70.1.2"), &off, Some(&t)), refused);
        // The others stay trusted.
        assert_eq!(decide(ip("100.80.3.4"), &off, Some(&t)), Verdict::Serve);
    }

    #[test]
    fn an_address_that_cannot_be_placed_is_refused_once_any_device_is_off() {
        let t = read_status(STATUS).unwrap();
        let off = vec!["nPhone".to_string()];
        assert_eq!(decide(ip("100.99.9.9"), &off, Some(&t)), Verdict::Unknown);
        assert_eq!(decide(ip("100.70.1.2"), &off, None), Verdict::Unknown);
        // A device without an id cannot be switched off, so it cannot be told apart.
        assert_eq!(decide(ip("100.90.0.1"), &off, Some(&t)), Verdict::Unknown);
    }

    #[test]
    fn nothing_outside_the_tailnet_is_served() {
        let t = read_status(STATUS).unwrap();
        for from in [
            "192.168.1.20",
            "10.0.2.2",
            "8.8.8.8",
            "fe80::1",
            "::ffff:192.168.1.20",
        ] {
            assert_eq!(decide(ip(from), &[], Some(&t)), Verdict::Outside, "{from}");
            assert_eq!(decide(ip(from), &[], None), Verdict::Outside, "{from}");
        }
    }

    #[test]
    fn only_a_tailnet_address_with_a_device_off_waits_on_the_status() {
        let off = vec!["nPhone".to_string()];
        assert!(needs_tailnet(ip("100.70.1.2"), &off));
        assert!(needs_tailnet(ip("::ffff:100.70.1.2"), &off));
        assert!(!needs_tailnet(ip("100.70.1.2"), &[]));
        assert!(!needs_tailnet(ip("127.0.0.1"), &off));
        assert!(!needs_tailnet(ip("192.168.1.20"), &off));
    }
}
