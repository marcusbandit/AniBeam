//! What developer options record (#287): every Door call with its cost and the size of its
//! answer, and every event with the size of what crossed to QML. Nothing is kept or measured
//! while developer options are off, so the only cost then is one relaxed atomic load per call.
//!
//! Times are milliseconds on the dev clock (`dev_now_ms`, shared with the decode records and
//! with QML), so a page load can ask for everything that happened since it began.

use std::collections::VecDeque;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use serde_json::{Value, json};

static ON: AtomicBool = AtomicBool::new(false);

/// How many calls and events are kept: a few minutes of a busy session.
const KEEP: usize = 3000;

#[derive(Clone, Serialize)]
pub struct CallStat {
    pub at: f64,
    pub name: String,
    /// The core's own work.
    pub core: f64,
    /// The reply walked to a serde value.
    pub json: f64,
    /// That value walked to a QJsonObject for QML.
    pub qjson: f64,
    pub total: f64,
    /// The answer as JSON text, the size of what QML is handed.
    pub bytes: usize,
    pub hit: bool,
    /// Ran on a thread of its own (callAsync) rather than on the Qt thread.
    #[serde(rename = "async")]
    pub off_thread: bool,
}

#[derive(Clone, Serialize)]
pub struct EventStat {
    pub at: f64,
    pub seq: u64,
    pub level: Value,
    pub stage: Value,
    pub kind: Value,
    pub message: String,
    pub plain: Option<String>,
    pub job: Value,
    pub bytes: usize,
}

static CALLS: Mutex<VecDeque<CallStat>> = Mutex::new(VecDeque::new());
static EVENTS: Mutex<VecDeque<EventStat>> = Mutex::new(VecDeque::new());

pub fn on() -> bool {
    ON.load(Ordering::Relaxed)
}

pub fn set_on(on: bool) {
    ON.store(on, Ordering::Relaxed);
}

pub fn now() -> f64 {
    crate::bridge::helpers::ffi::dev_now_ms()
}

fn push<T>(ring: &Mutex<VecDeque<T>>, v: T) {
    let mut r = ring.lock().unwrap_or_else(|e| e.into_inner());
    r.push_back(v);
    while r.len() > KEEP {
        r.pop_front();
    }
}

pub fn call(stat: CallStat) {
    push(&CALLS, stat);
}

/// Called on the core's thread with the envelope the Qt thread is about to get.
pub fn event(envelope: &Value) {
    let bytes = serde_json::to_vec(envelope).map(|v| v.len()).unwrap_or(0);
    push(
        &EVENTS,
        EventStat {
            at: now(),
            seq: envelope["seq"].as_u64().unwrap_or(0),
            level: envelope["level"].clone(),
            stage: envelope["stage"].clone(),
            kind: envelope["kind"].clone(),
            message: envelope["message"].as_str().unwrap_or("").to_string(),
            plain: envelope["plain"].as_str().map(str::to_string),
            job: envelope["job"]["kind"].clone(),
            bytes,
        },
    );
}

/// The size of a reply as the JSON text it amounts to; measured only when someone is looking.
pub fn size_of(v: &Value) -> usize {
    serde_json::to_vec(v).map(|v| v.len()).unwrap_or(0)
}

/// Every call that started at or after `since`, oldest first.
pub fn calls_since(since: f64) -> Value {
    let r = CALLS.lock().unwrap_or_else(|e| e.into_inner());
    json!(r.iter().filter(|c| c.at >= since).collect::<Vec<_>>())
}

/// The newest `limit` events, oldest first.
pub fn events(limit: usize) -> Value {
    let r = EVENTS.lock().unwrap_or_else(|e| e.into_inner());
    let skip = r.len().saturating_sub(limit);
    json!(r.iter().skip(skip).collect::<Vec<_>>())
}

/// The process's memory as the kernel counts it, in kilobytes. Linux reads /proc; elsewhere
/// the object is empty and the tab says so.
pub fn memory() -> Value {
    #[cfg(target_os = "linux")]
    {
        let mut o = serde_json::Map::new();
        if let Ok(s) = std::fs::read_to_string("/proc/self/status") {
            for line in s.lines() {
                let Some((k, v)) = line.split_once(':') else {
                    continue;
                };
                if matches!(
                    k,
                    "VmRSS" | "VmHWM" | "RssAnon" | "RssFile" | "RssShmem" | "VmSwap" | "Threads"
                ) {
                    let n: u64 = v
                        .split_whitespace()
                        .next()
                        .and_then(|n| n.parse().ok())
                        .unwrap_or(0);
                    o.insert(k.to_string(), json!(n));
                }
            }
        }
        Value::Object(o)
    }
    #[cfg(not(target_os = "linux"))]
    {
        json!({})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reply_is_measured_as_its_json_text() {
        assert_eq!(size_of(&json!({"a": [1, 2]})), r#"{"a":[1,2]}"#.len());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn memory_reads_the_resident_set() {
        assert!(memory()["VmRSS"].as_u64().unwrap_or(0) > 0);
    }
}
