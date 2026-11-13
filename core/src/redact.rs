//! Keeping secrets out of what the core stores (#509). [`SECRET_PARAMS`] names the query
//! parameters that hold one, and [`scrub_secrets`] empties their values and drops the
//! login from every address in a text.
//!
//! It names nothing else in the crate, so the event bus can scrub what it stores without
//! reaching into the network layer (#856).

/// Query parameters that hold a secret, with the label a person reads for each. A source
/// added by hand has these lifted out of its address, a listed address has them masked, and
/// the tailnet snapshot leaves them out. Names match case-insensitively.
pub const SECRET_PARAMS: [(&str, &str); 15] = [
    ("apikey", "API key"),
    ("api_key", "API key"),
    ("jackett_apikey", "API key"),
    ("token", "Token"),
    ("passkey", "Passkey"),
    ("torrent_pass", "Passkey"),
    ("rsskey", "RSS key"),
    ("authkey", "Auth key"),
    ("auth", "Auth key"),
    ("key", "Key"),
    ("secret", "Secret"),
    ("sig", "Signature"),
    ("pass", "Password"),
    ("password", "Password"),
    ("uid", "User id"),
];

/// The secret query parameters [`scrub_secrets`] empties, as one alternation.
fn secret_names() -> String {
    SECRET_PARAMS
        .iter()
        .map(|(name, _)| regex::escape(name))
        .collect::<Vec<_>>()
        .join("|")
}

/// `user:password@` right after `scheme://`, up to the last `@` before the path, the way
/// the HTTP client and `repos::index::split_login` read it: `u:p@ss@host` is a login of
/// `u:p@ss`. A backslash ends the authority, as it does for them.
static LOGIN: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r#"(?i)([a-z][a-z0-9+.\-]*://)[^/?#\\\s"'<>]*@"#).unwrap()
});

/// A secret parameter's value, written plain: `?apikey=abc`, `&passkey=abc`, after an
/// `&amp;` in quoted XML, or in a fragment (`#token=abc`).
static SECRET: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(&format!(
        r#"(?i)([?&;#](?:{})=)[^&#\s"'<>]*"#,
        secret_names()
    ))
    .unwrap()
});

/// The same inside another address, percent-encoded: a magnet's tracker,
/// `&tr=https%3A%2F%2Ft.example%2Fannounce%3Fpasskey%3Dabc`. The value runs to
/// the next encoded `&`.
static SECRET_ENCODED: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(&format!(
        r#"(?i)((?:%3F|%26|%23)(?:{})%3D)(?:[^%&#\s"'<>]|%(?:[013-9a-f][0-9a-f]|2[0-57-9a-f]))*"#,
        secret_names()
    ))
    .unwrap()
});

/// Text as the stored events keep it (#509): every address in it loses its login
/// (`user:password@`) and the value of every secret query parameter (the names in
/// [`SECRET_PARAMS`], `apikey=` and `passkey=` among them), written plain or
/// percent-encoded inside another address.
/// The rest of the address stays, so a line still says which feed or tracker it was.
/// A key the address keeps in its path rather than its query is not told from any
/// other path, and stays.
pub fn scrub_secrets(text: &str) -> String {
    let text = LOGIN.replace_all(text, "$1");
    let text = SECRET.replace_all(&text, "$1...");
    let text = SECRET_ENCODED.replace_all(&text, "$1...");
    scrub_encoded_logins(&text)
}

/// [`scrub_secrets`]'s login rule for an address percent-encoded inside another:
/// `https%3A%2F%2Fuser%3Apass%40t.example` keeps `https%3A%2F%2Ft.example`, and so
/// does a login that ends in a raw `@`.
fn scrub_encoded_logins(text: &str) -> String {
    const START: &str = "%3a%2f%2f";
    let lower = text.to_ascii_lowercase();
    if !lower.contains(START) {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut from = 0;
    while let Some(found) = lower[from..].find(START) {
        let start = from + found + START.len();
        out.push_str(&text[from..start]);
        let rest = &lower[start..];
        // The encoded authority ends at a slash, a query or a fragment, encoded or not.
        let end = rest
            .find(|c: char| matches!(c, '/' | '?' | '#' | '&' | '"' | '\'') || c.is_whitespace())
            .unwrap_or(rest.len());
        let end = ["%2f", "%3f", "%23"]
            .iter()
            .filter_map(|stop| rest[..end].find(stop))
            .min()
            .unwrap_or(end);
        // The login ends at the last @, encoded or not.
        let at = [("%40", 3), ("@", 1)]
            .iter()
            .filter_map(|(sign, len)| rest[..end].rfind(sign).map(|at| at + len))
            .max();
        from = start + at.unwrap_or(0);
    }
    out.push_str(&text[from..]);
    out
}

#[cfg(test)]
mod tests {
    use super::scrub_secrets;

    #[test]
    fn an_address_keeps_everything_but_its_login_and_keys() {
        assert_eq!(
            scrub_secrets("see https://u:p@h.example:8080/a?apikey=k&q=x#f"),
            "see https://h.example:8080/a?apikey=...&q=x#f"
        );
        assert_eq!(
            scrub_secrets("http://t.example/announce?PassKey=abc"),
            "http://t.example/announce?PassKey=..."
        );
        assert_eq!(
            scrub_secrets("no address, a@b.example"),
            "no address, a@b.example"
        );
        // A password with an @ in it: the login runs to the last one, as the client reads it.
        assert_eq!(
            scrub_secrets("https://alice:p@ss@feed.example/rss"),
            "https://feed.example/rss"
        );
        // A key after an XML-escaped separator, or in a fragment.
        assert_eq!(
            scrub_secrets("<link>https://h/?q=x&amp;apikey=SEKRIT</link>"),
            "<link>https://h/?q=x&amp;apikey=...</link>"
        );
        assert_eq!(
            scrub_secrets("https://h/cb#token=SEKRIT"),
            "https://h/cb#token=..."
        );
        // A key-like word that is not a parameter stays.
        assert_eq!(scrub_secrets("https://h/monkey=1"), "https://h/monkey=1");
        assert_eq!(scrub_secrets("https://h/?monkey=1"), "https://h/?monkey=1");
    }

    #[test]
    fn an_encoded_address_inside_another_loses_its_login_and_keys() {
        assert_eq!(
            scrub_secrets(
                "magnet:?xt=1&tr=udp%3A%2F%2Fme%3Asecret%40t.example%3A80%2Fa%3Fpasskey%3Dzz%26b%3D2&dn=x"
            ),
            "magnet:?xt=1&tr=udp%3A%2F%2Ft.example%3A80%2Fa%3Fpasskey%3D...%26b%3D2&dn=x"
        );
        // An encoded address whose login ends in a raw @.
        assert_eq!(
            scrub_secrets("tr=https%3A%2F%2Fuser%3Apw@t.example%2Fannounce"),
            "tr=https%3A%2F%2Ft.example%2Fannounce"
        );
        // An encoded @ after the host is no login.
        assert_eq!(
            scrub_secrets("x=https%3A%2F%2Fh.example%2Fa%40b"),
            "x=https%3A%2F%2Fh.example%2Fa%40b"
        );
    }
}
