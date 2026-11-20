//! The channel's Android end: `Control.java`'s receiver, given its native `run` and started,
//! through the platform's own JNI helpers.

use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{jlong, jstring};
use std::time::Duration;

/// `Control.run`, which Java calls on the receiver's own thread with the command line. The
/// answer's first character is the broadcast's result code, 0 when every command ran and 1
/// when one failed, so Java never reads it out of the text.
extern "system" fn run(mut env: JNIEnv, _class: JClass, line: JString, late_ms: jlong) -> jstring {
    let line: String = env.get_string(&line).map(Into::into).unwrap_or_default();
    let late = Duration::from_millis(u64::try_from(late_ms).unwrap_or(0));
    // A panic must not cross into Java: it would take the whole app down. Its message goes to
    // logcat here, since a panic's own print goes to a stderr Android throws away.
    let (ok, said) = std::panic::catch_unwind(|| super::serve(&line, late)).unwrap_or_else(|p| {
        let why = p
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| p.downcast_ref::<String>().cloned())
            .unwrap_or_default();
        crate::local::log(&format!("control: {line:?} panicked: {why}"));
        (
            false,
            "error: the command panicked; logcat has it under AnibeamCore".into(),
        )
    });
    let out = format!("{}{said}", if ok { '0' } else { '1' });
    env.new_string(out)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

/// Android's API level, from `ro.build.version.sdk`; 0 when it cannot be read.
pub fn sdk() -> i32 {
    use std::ffi::{CStr, c_char};
    unsafe extern "C" {
        fn __system_property_get(name: *const c_char, value: *mut c_char) -> i32;
    }
    let mut value = [0 as c_char; 92];
    let n = unsafe { __system_property_get(c"ro.build.version.sdk".as_ptr(), value.as_mut_ptr()) };
    if n <= 0 {
        return 0;
    }
    unsafe { CStr::from_ptr(value.as_ptr()) }
        .to_str()
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

/// Hands `Control.run` its body, then registers the receiver.
pub fn start() {
    let natives = [(
        "run",
        "(Ljava/lang/String;J)Ljava/lang/String;",
        run as *mut std::ffi::c_void,
    )];
    if let Err(e) = crate::platform::register_natives("Control", &natives) {
        crate::local::log(&format!("control: not started: {e}"));
        return;
    }
    match crate::platform::java_call("Control", "start", "(Landroid/content/Context;)Z", None) {
        Some(jni::objects::JValueOwned::Bool(1)) => {
            crate::local::log("control: listening for adb commands")
        }
        // A throw is logged by the call itself, as `Control.start: ...`.
        _ => crate::local::log("control: not started: the receiver did not register"),
    }
}
