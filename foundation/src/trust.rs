//! Who a request belongs to: the owner of this computer, someone else on the
//! tailnet, or a device nobody can be tied to. The host decides which, and
//! the profiles answer each person as their own.

/// The profile of the person the computer belongs to.
pub const OWNER: u64 = 1;

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

impl TailnetUser {
    /// What to call them: their Tailscale name, or their login when it gives none. None
    /// with neither, so no screen shows a bare user id (#633).
    pub fn shown(&self) -> Option<&str> {
        [self.name.trim(), self.login.trim()]
            .into_iter()
            .find(|s| !s.is_empty())
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
