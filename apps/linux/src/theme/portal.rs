//! org.freedesktop.portal.Settings over zbus: `ReadOne` for the three appearance keys and
//! the `SettingChanged` stream. Every failure is "unset", because on Hyprland with the gtk
//! backend a missing accent-color is the normal answer.

use futures_util::StreamExt;
use zbus::zvariant::{OwnedValue, Value};

use crate::theme::colour::Rgb;
use crate::theme::{Mode, Portal};

const NAMESPACE: &str = "org.freedesktop.appearance";

#[zbus::proxy(
    interface = "org.freedesktop.portal.Settings",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait Settings {
    fn read_one(&self, namespace: &str, key: &str) -> zbus::Result<OwnedValue>;

    #[zbus(signal)]
    fn setting_changed(&self, namespace: &str, key: &str, value: Value<'_>) -> zbus::Result<()>;
}

fn unwrap<'a>(v: &'a Value<'a>) -> &'a Value<'a> {
    match v {
        Value::Value(inner) => inner,
        other => other,
    }
}

pub fn parse_scheme(v: &Value) -> Option<Mode> {
    match unwrap(v) {
        Value::U32(1) => Some(Mode::Dark),
        Value::U32(2) => Some(Mode::Light),
        _ => None,
    }
}

pub fn parse_contrast(v: &Value) -> bool {
    matches!(unwrap(v), Value::U32(1))
}

pub fn parse_accent(v: &Value) -> Option<Rgb> {
    let Value::Structure(s) = unwrap(v) else {
        return None;
    };
    let fields = s.fields();
    if fields.len() != 3 {
        return None;
    }
    let mut out = [0.0; 3];
    for (i, f) in fields.iter().enumerate() {
        let Value::F64(x) = f else { return None };
        if !(0.0..=1.0).contains(x) {
            return None;
        }
        out[i] = *x;
    }
    Some(Rgb {
        r: out[0],
        g: out[1],
        b: out[2],
    })
}

pub async fn read(conn: &zbus::Connection) -> Portal {
    let Ok(proxy) = SettingsProxy::new(conn).await else {
        return Portal::default();
    };
    let get = |key: &'static str| {
        let proxy = proxy.clone();
        async move { proxy.read_one(NAMESPACE, key).await.ok().map(Value::from) }
    };
    let scheme = get("color-scheme").await.and_then(|v| parse_scheme(&v));
    let contrast = get("contrast").await.is_some_and(|v| parse_contrast(&v));
    let accent = get("accent-color").await.and_then(|v| parse_accent(&v));
    Portal {
        scheme,
        contrast,
        accent,
    }
}

/// Calls `on_change` for every SettingChanged in the appearance namespace, until the bus
/// goes away. The caller re-reads with `read`; the signal's value is not trusted alone.
pub async fn watch(conn: zbus::Connection, on_change: impl Fn() + Send + 'static) {
    let Ok(proxy) = SettingsProxy::new(&conn).await else {
        return;
    };
    let Ok(mut stream) = proxy.receive_setting_changed().await else {
        return;
    };
    while let Some(signal) = stream.next().await {
        if let Ok(args) = signal.args()
            && *args.namespace() == NAMESPACE
        {
            on_change();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zbus::zvariant::{Structure, Value};

    #[test]
    fn scheme_contrast_and_accent_parse_and_reject() {
        assert_eq!(parse_scheme(&Value::U32(1)), Some(Mode::Dark));
        assert_eq!(parse_scheme(&Value::U32(2)), Some(Mode::Light));
        assert_eq!(parse_scheme(&Value::U32(0)), None);
        assert_eq!(parse_scheme(&Value::U32(9)), None);
        assert_eq!(parse_scheme(&Value::Str("dark".into())), None);
        assert!(parse_contrast(&Value::U32(1)));
        assert!(!parse_contrast(&Value::U32(0)));
        let accent = Value::Structure(Structure::from((0.2078_f64, 0.5176_f64, 0.8941_f64)));
        assert_eq!(parse_accent(&accent).unwrap().to_hex(), "#3584e4");
        let out_of_range = Value::Structure(Structure::from((-1.0_f64, 0.5_f64, 0.5_f64)));
        assert_eq!(parse_accent(&out_of_range), None);
        // a value wrapped once more, the deprecated Read's shape, still parses
        let wrapped = Value::Value(Box::new(Value::U32(2)));
        assert_eq!(parse_scheme(&wrapped), Some(Mode::Light));
    }
}
