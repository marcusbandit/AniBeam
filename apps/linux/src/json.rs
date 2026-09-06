//! The shapes that cross the bridge as JSON. The core's enums are externally tagged
//! (`{"ListSeries": {...}}`, `"ListSources"`); QML sees one flat shape per direction.

use anibeam_core::{Call, Core, CoreError, Event, Reply};
use cxx_qt_lib::{QJsonArray, QJsonObject, QJsonValue, QString};
use serde_json::{Map, Value, json};

pub fn call_from(name: &str, args: Value) -> Result<Call, String> {
    let empty = args.is_null() || args.as_object().is_some_and(Map::is_empty);
    let wrapped = if empty {
        Value::String(name.to_string())
    } else {
        json!({ name: args })
    };
    serde_json::from_value(wrapped).map_err(|e| e.to_string())
}

/// `{secs_since_epoch, nanos_since_epoch}`, serde's SystemTime, becomes seconds.
pub fn flatten_times(v: &mut Value) {
    match v {
        Value::Object(o) => {
            if o.len() == 2
                && let (Some(s), Some(n)) = (
                    o.get("secs_since_epoch").and_then(Value::as_f64),
                    o.get("nanos_since_epoch").and_then(Value::as_f64),
                )
            {
                *v = Value::from(s + n / 1e9);
                return;
            }
            for (_, child) in o.iter_mut() {
                flatten_times(child);
            }
        }
        Value::Array(a) => a.iter_mut().for_each(flatten_times),
        _ => {}
    }
}

/// An externally tagged enum value as (variant name, fields object).
fn split_tag(v: Value) -> (String, Value) {
    match v {
        Value::String(kind) => (kind, json!({})),
        Value::Object(o) if o.len() == 1 => {
            let (kind, inner) = o.into_iter().next().expect("one entry");
            (kind, if inner.is_null() { json!({}) } else { inner })
        }
        other => ("Unknown".to_string(), other),
    }
}

pub fn reply_json(r: Result<Reply, CoreError>) -> Value {
    match r {
        Ok(reply) => {
            let mut v = serde_json::to_value(&reply).unwrap_or(Value::Null);
            flatten_times(&mut v);
            let (kind, inner) = split_tag(v);
            json!({ "kind": kind, "reply": inner })
        }
        Err(e) => json!({ "error": error_json(&e) }),
    }
}

pub fn error_json(e: &CoreError) -> Value {
    let (kind, fields) = split_tag(serde_json::to_value(e).unwrap_or(Value::Null));
    let mut o = match fields {
        Value::Object(o) => o,
        _ => Map::new(),
    };
    o.insert("kind".into(), Value::String(kind));
    o.insert("message".into(), Value::String(e.to_string()));
    Value::Object(o)
}

pub fn event_json(e: &Event) -> Value {
    let mut body = serde_json::to_value(&e.body).unwrap_or(Value::Null);
    flatten_times(&mut body);
    let (kind, fields) = split_tag(body);
    let mut at = serde_json::to_value(e.at).unwrap_or(Value::Null);
    flatten_times(&mut at);
    json!({
        "seq": e.seq,
        "at": at,
        "level": serde_json::to_value(e.level).unwrap_or(Value::Null),
        "stage": serde_json::to_value(e.stage).unwrap_or(Value::Null),
        "message": e.message,
        "job": e.job.as_ref().map(|j| json!({
            "id": j.id,
            "kind": serde_json::to_value(j.kind).unwrap_or(Value::Null),
            "phase": serde_json::to_value(j.phase).unwrap_or(Value::Null),
        })),
        "kind": kind,
        "body": fields,
    })
}

pub fn dispatch(core: &Core, call: Call) -> Value {
    reply_json(core.call(call))
}

pub fn to_qjson(v: &Value) -> QJsonValue {
    match v {
        Value::Null => QJsonValue::default(),
        Value::Bool(b) => QJsonValue::from(*b),
        Value::Number(n) => match n.as_i64() {
            Some(i) => QJsonValue::from(i),
            None => QJsonValue::from(n.as_f64().unwrap_or(0.0)),
        },
        Value::String(s) => QJsonValue::from(&QString::from(s)),
        Value::Array(_) => QJsonValue::from(&to_qjson_array(v)),
        Value::Object(_) => QJsonValue::from(&to_qjson_object(v)),
    }
}

pub fn to_qjson_array(v: &Value) -> QJsonArray {
    let mut arr = QJsonArray::default();
    if let Value::Array(items) = v {
        for x in items {
            arr.append(&to_qjson(x));
        }
    }
    arr
}

/// A non-object is wrapped as `{ "value": v }`.
pub fn to_qjson_object(v: &Value) -> QJsonObject {
    let mut o = QJsonObject::default();
    match v {
        Value::Object(m) => {
            for (k, x) in m {
                o.insert(&QString::from(k), &to_qjson(x));
            }
        }
        other => o.insert(&QString::from("value"), &to_qjson(other)),
    }
    o
}

pub fn from_qjson(v: &QJsonValue) -> Value {
    if v.is_bool() {
        Value::Bool(v.to_bool())
    } else if v.is_double() {
        let d = v.to_double();
        if d.fract() == 0.0 && d.abs() < 9.0e15 {
            Value::from(d as i64)
        } else {
            Value::from(d)
        }
    } else if v.is_string() {
        Value::String(v.to_string().to_string())
    } else if v.is_array() {
        Value::Array(v.to_array().iter().map(|x| from_qjson(&x)).collect())
    } else if v.is_object() {
        from_qjson_object(&v.to_object())
    } else {
        Value::Null
    }
}

pub fn from_qjson_object(o: &QJsonObject) -> Value {
    let mut m = Map::new();
    for key in o.keys().iter() {
        m.insert(key.to_string(), from_qjson(&o.value(key)));
    }
    Value::Object(m)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anibeam_core::*;
    use serde_json::json;
    use std::sync::Arc;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[test]
    fn calls_come_from_a_name_and_a_json_object() {
        assert_eq!(
            call_from("ListSources", Value::Null).unwrap(),
            Call::ListSources
        );
        assert_eq!(
            call_from("ListSources", json!({})).unwrap(),
            Call::ListSources
        );
        let c = call_from(
            "ListSeries",
            json!({"tab": "Movies", "query": "gup", "sort": "LastViewed", "direction": "Desc", "reveal_hidden": true}),
        )
        .unwrap();
        assert_eq!(
            c,
            Call::ListSeries {
                tab: Tab::Movies,
                query: "gup".into(),
                sort: Sort::LastViewed,
                direction: Direction::Desc,
                reveal_hidden: true
            }
        );
        assert!(
            call_from("ListSeries", json!({})).is_err(),
            "missing fields are an error"
        );
        assert!(call_from("Nope", Value::Null).is_err());
        let c = call_from(
            "ApplyMatch",
            json!({"series": 3, "target": {"Anilist": {"id": 21, "season": null}}}),
        )
        .unwrap();
        assert_eq!(
            c,
            Call::ApplyMatch {
                series: 3,
                target: MatchTarget::Anilist {
                    id: 21,
                    season: None
                }
            }
        );
    }

    #[test]
    fn replies_and_errors_take_one_shape() {
        let ok = reply_json(Ok(Reply::Started { job: 7 }));
        assert_eq!(ok, json!({"kind": "Started", "reply": {"job": 7}}));
        assert_eq!(
            reply_json(Ok(Reply::Ok)),
            json!({"kind": "Ok", "reply": {}})
        );
        let err = reply_json(Err(CoreError::NotFound {
            what: Entity::Series,
            id: 9,
        }));
        assert_eq!(err["error"]["kind"], "NotFound");
        assert_eq!(err["error"]["what"], "Series");
        assert_eq!(err["error"]["id"], 9);
        assert_eq!(err["error"]["message"], "Series 9 not found");
        let refused = reply_json(Err(CoreError::Refused {
            reason: Refusal::Hidden,
        }));
        assert_eq!(refused["error"]["reason"], "Hidden");
    }

    #[test]
    fn times_flatten_to_seconds_everywhere() {
        let at = UNIX_EPOCH + Duration::from_secs(1_700_000_000) + Duration::from_millis(250);
        let mut v = serde_json::to_value(vec![Some(at), None::<SystemTime>]).unwrap();
        flatten_times(&mut v);
        assert_eq!(v, json!([1700000000.25, null]));
        let mut nested = json!({"a": {"secs_since_epoch": 5, "nanos_since_epoch": 0, "extra": 1}});
        flatten_times(&mut nested);
        assert_eq!(
            nested["a"]["secs_since_epoch"], 5,
            "an object with more keys is not a time"
        );
    }

    #[test]
    fn an_event_carries_its_kind_flat() {
        let e = Event {
            seq: 4,
            at: UNIX_EPOCH + Duration::from_secs(10),
            level: Level::Info,
            stage: Stage::Library,
            message: "scan finished: 1 added".into(),
            job: Some(JobRef {
                id: 2,
                kind: JobKind::Scan,
                phase: JobPhase::Finished,
            }),
            body: EventBody::ScanFinished {
                source: None,
                added: 1,
                changed: 0,
                removed: 0,
            },
        };
        let v = event_json(&e);
        assert_eq!(v["kind"], "ScanFinished");
        assert_eq!(v["body"]["added"], 1);
        assert_eq!(v["job"]["kind"], "Scan");
        assert_eq!(v["job"]["phase"], "Finished");
        assert_eq!(v["at"], 10.0);
        assert_eq!(v["level"], "Info");
        let unit = Event {
            body: EventBody::Ready,
            job: None,
            ..e
        };
        let v = event_json(&unit);
        assert_eq!(v["kind"], "Ready");
        assert_eq!(v["body"], json!({}));
        assert_eq!(v["job"], Value::Null);
    }

    #[test]
    fn qjson_round_trips_and_keeps_whole_numbers_whole() {
        let v = json!({
            "id": 42,
            "series": 1_700_000_000u64,
            "score": 7.5,
            "hidden": false,
            "title": "Frieren",
            "poster": null,
            "watched": { "done": 3, "total": null },
            "cards": [1, 2.5, "x", { "a": 1 }, [true], null],
        });
        let back = from_qjson_object(&to_qjson_object(&v));
        assert_eq!(back, v, "an object survives the walk out and back");
        // Qt has one number type, so the way back has to decide; an id must not come back
        // as 42.0, which QML would render "42" but Rust would refuse as a u64.
        assert!(back["id"].is_i64(), "a whole number comes back an integer");
        assert!(back["series"].is_i64());
        assert!(back["watched"]["done"].is_i64(), "nested, too");
        assert!(back["cards"][0].is_i64(), "and inside an array");
        assert!(
            back["score"].is_f64() && !back["score"].is_i64(),
            "a real number stays a real number"
        );

        // An array round-trips as itself, and a value that is not an object is wrapped
        // rather than lost, because a QJsonObject is the only shape the bridge carries.
        assert_eq!(from_qjson(&to_qjson(&json!([1, "two"]))), json!([1, "two"]));
        assert_eq!(
            from_qjson_object(&to_qjson_object(&json!(5))),
            json!({"value": 5})
        );
        assert_eq!(
            from_qjson_object(&to_qjson_object(&Value::Null)),
            json!({"value": null})
        );
    }

    #[test]
    fn dispatch_answers_from_a_real_core() {
        let dir = tempfile::tempdir().unwrap();
        let paths = CorePaths::under(dir.path());
        let secrets = anibeam_core::trackers::Secrets::file_only(paths.secrets_path());
        let core: Arc<Core> = Core::open_with_secrets(paths, secrets).unwrap();
        let about = dispatch(&core, call_from("About", Value::Null).unwrap());
        assert_eq!(about["kind"], "About");
        assert_eq!(about["reply"]["about"]["version"], anibeam_core::VERSION);
        let sources = dispatch(&core, Call::ListSources);
        assert_eq!(
            sources,
            json!({"kind": "Sources", "reply": {"sources": []}})
        );
        let missing = dispatch(&core, Call::GetSeries { series: 1 });
        assert_eq!(missing["error"]["kind"], "NotFound");
        core.shutdown();
    }
}
