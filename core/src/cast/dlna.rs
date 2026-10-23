//! UPnP AV, the protocol smart TVs take as DLNA renderers: the device description names the
//! AVTransport control URL, and five SOAP actions are a remote: SetAVTransportURI, Play,
//! Pause, Seek and Stop, with GetTransportInfo and GetPositionInfo read back each second.

use std::time::Duration;

const AVT: &str = "urn:schemas-upnp-org:service:AVTransport:1";

/// A renderer as its description names it.
#[derive(Clone, Debug, PartialEq)]
pub struct Renderer {
    pub name: String,
    pub model: Option<String>,
    pub control: String,
}

fn child_text<'a>(node: roxmltree::Node<'a, 'a>, name: &str) -> Option<&'a str> {
    node.descendants()
        .find(|n| n.has_tag_name(name))
        .and_then(|n| n.text())
        .map(str::trim)
}

/// The friendly name and the AVTransport control URL from a device description.
pub fn parse_description(xml: &str, location: &str) -> Option<Renderer> {
    let doc = roxmltree::Document::parse(xml).ok()?;
    let root = doc.root_element();
    let device = root.descendants().find(|n| n.has_tag_name("device"))?;
    let name = child_text(device, "friendlyName")?.to_string();
    let model = child_text(device, "modelName").map(str::to_string);
    let service = root
        .descendants()
        .filter(|n| n.has_tag_name("service"))
        .find(|s| {
            child_text(*s, "serviceType")
                .is_some_and(|t| t.starts_with("urn:schemas-upnp-org:service:AVTransport:"))
        })?;
    let control = child_text(service, "controlURL")?;
    let base = child_text(root, "URLBase").unwrap_or(location);
    Some(Renderer {
        name,
        model,
        control: join(base, control),
    })
}

/// A control URL made absolute against the description's location.
fn join(base: &str, path: &str) -> String {
    if path.starts_with("http://") || path.starts_with("https://") {
        return path.to_string();
    }
    let origin = match base
        .find("://")
        .and_then(|i| base[i + 3..].find('/').map(|j| i + 3 + j))
    {
        Some(end) => &base[..end],
        None => base.trim_end_matches('/'),
    };
    if path.starts_with('/') {
        format!("{origin}{path}")
    } else {
        format!("{origin}/{path}")
    }
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// The DIDL-Lite item a TV reads the title, the type and the subtitles from.
pub fn didl(url: &str, mime: &str, title: &str, subtitles: Option<&str>) -> String {
    let caption = subtitles
        .map(|s| format!("<sec:CaptionInfoEx sec:type=\"srt\">{}</sec:CaptionInfoEx><res protocolInfo=\"http-get:*:text/srt:*\">{}</res>", escape(s), escape(s)))
        .unwrap_or_default();
    format!(
        "<DIDL-Lite xmlns=\"urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/\" xmlns:dc=\"http://purl.org/dc/elements/1.1/\" \
         xmlns:upnp=\"urn:schemas-upnp-org:metadata-1-0/upnp/\" xmlns:sec=\"http://www.sec.co.kr/\">\
         <item id=\"0\" parentID=\"-1\" restricted=\"1\"><dc:title>{}</dc:title><upnp:class>object.item.videoItem</upnp:class>\
         <res protocolInfo=\"http-get:*:{mime}:DLNA.ORG_OP=01;DLNA.ORG_FLAGS=01700000000000000000000000000000\">{}</res>{caption}</item></DIDL-Lite>",
        escape(title),
        escape(url)
    )
}

fn envelope(action: &str, args: &[(&str, String)]) -> String {
    let body: String = args
        .iter()
        .map(|(k, v)| format!("<{k}>{}</{k}>", escape(v)))
        .collect();
    format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?><s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\" \
         s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\"><s:Body><u:{action} xmlns:u=\"{AVT}\">{body}</u:{action}></s:Body></s:Envelope>"
    )
}

/// h:mm:ss for Seek, the form every renderer reads.
pub fn hms(secs: f64) -> String {
    let t = secs.max(0.0) as u64;
    format!("{}:{:02}:{:02}", t / 3600, t / 60 % 60, t % 60)
}

pub fn parse_hms(s: &str) -> Option<f64> {
    let mut parts = s.trim().split(':');
    let h: f64 = parts.next()?.parse().ok()?;
    let m: f64 = parts.next()?.parse().ok()?;
    let sec: f64 = parts.next()?.parse().ok()?;
    Some(h * 3600.0 + m * 60.0 + sec)
}

pub struct Client {
    http: reqwest::Client,
    control: String,
}

impl Client {
    pub fn new(control: String) -> Client {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(8))
            .build()
            .unwrap_or_default();
        Client { http, control }
    }

    pub async fn action(&self, action: &str, args: &[(&str, String)]) -> Result<String, String> {
        let r = self
            .http
            .post(&self.control)
            .header("Content-Type", "text/xml; charset=\"utf-8\"")
            .header("SOAPAction", format!("\"{AVT}#{action}\""))
            .body(envelope(action, args))
            .send()
            .await
            .map_err(|e| format!("the TV did not answer: {e}"))?;
        let ok = r.status().is_success();
        let text = r.text().await.unwrap_or_default();
        if !ok {
            let code = roxmltree::Document::parse(&text).ok().and_then(|d| {
                d.descendants()
                    .find(|n| n.has_tag_name("errorDescription") || n.has_tag_name("errorCode"))
                    .and_then(|n| n.text().map(str::to_string))
            });
            return Err(format!(
                "the TV refused {action}{}",
                code.map(|c| format!(" ({c})")).unwrap_or_default()
            ));
        }
        Ok(text)
    }

    pub async fn set_uri(&self, url: &str, metadata: &str) -> Result<(), String> {
        let args = [
            ("InstanceID", "0".to_string()),
            ("CurrentURI", url.to_string()),
            ("CurrentURIMetadata", metadata.to_string()),
        ];
        self.action("SetAVTransportURI", &args).await.map(|_| ())
    }

    pub async fn play(&self) -> Result<(), String> {
        self.action("Play", &[("InstanceID", "0".into()), ("Speed", "1".into())])
            .await
            .map(|_| ())
    }

    pub async fn pause(&self) -> Result<(), String> {
        self.action("Pause", &[("InstanceID", "0".into())])
            .await
            .map(|_| ())
    }

    pub async fn stop(&self) -> Result<(), String> {
        self.action("Stop", &[("InstanceID", "0".into())])
            .await
            .map(|_| ())
    }

    pub async fn seek(&self, to: f64) -> Result<(), String> {
        let args = [
            ("InstanceID", "0".to_string()),
            ("Unit", "REL_TIME".to_string()),
            ("Target", hms(to)),
        ];
        self.action("Seek", &args).await.map(|_| ())
    }

    /// The transport state (PLAYING, PAUSED_PLAYBACK, STOPPED, TRANSITIONING, NO_MEDIA_PRESENT)
    /// and the position and duration in seconds.
    pub async fn status(&self) -> Result<(String, Option<f64>, Option<f64>), String> {
        let info = self
            .action("GetTransportInfo", &[("InstanceID", "0".into())])
            .await?;
        let state = field(&info, "CurrentTransportState").unwrap_or_default();
        let pos = self
            .action("GetPositionInfo", &[("InstanceID", "0".into())])
            .await?;
        let at = field(&pos, "RelTime").and_then(|t| parse_hms(&t));
        let duration = field(&pos, "TrackDuration")
            .and_then(|t| parse_hms(&t))
            .filter(|d| *d > 0.0);
        Ok((state, at, duration))
    }
}

fn field(xml: &str, name: &str) -> Option<String> {
    let doc = roxmltree::Document::parse(xml).ok()?;
    doc.descendants()
        .find(|n| n.has_tag_name(name))
        .and_then(|n| n.text())
        .map(|s| s.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_description_gives_the_name_and_the_absolute_control_url() {
        let xml = r#"<?xml version="1.0"?><root xmlns="urn:schemas-upnp-org:device-1-0"><device>
          <friendlyName>[TV] Living Room</friendlyName><modelName>UE55</modelName><serviceList>
          <service><serviceType>urn:schemas-upnp-org:service:RenderingControl:1</serviceType><controlURL>/rc</controlURL></service>
          <service><serviceType>urn:schemas-upnp-org:service:AVTransport:1</serviceType><controlURL>upnp/control/AVTransport1</controlURL></service>
          </serviceList></device></root>"#;
        let r = parse_description(xml, "http://192.168.1.40:9197/dmr").unwrap();
        assert_eq!(r.name, "[TV] Living Room");
        assert_eq!(
            r.control,
            "http://192.168.1.40:9197/upnp/control/AVTransport1"
        );
    }

    #[test]
    fn times_go_both_ways() {
        assert_eq!(hms(3725.9), "1:02:05");
        assert_eq!(parse_hms("0:23:40.500"), Some(1420.5));
        assert_eq!(parse_hms("NOT_IMPLEMENTED"), None);
    }

    #[test]
    fn the_soap_body_escapes_its_arguments() {
        let e = envelope(
            "SetAVTransportURI",
            &[("CurrentURI", "http://x/a?b=1&c=2".into())],
        );
        assert!(e.contains("<CurrentURI>http://x/a?b=1&amp;c=2</CurrentURI>"));
        assert!(e.contains(
            "u:SetAVTransportURI xmlns:u=\"urn:schemas-upnp-org:service:AVTransport:1\""
        ));
    }
}
