//! The channel's Android end: `Control.java`'s receiver, given its native `run` and started,
//! through the platform's own JNI helpers.

use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::jstring;

/// `Control.run`, which Java calls on the receiver's own thread with the command line. The
/// answer's first character is the broadcast's result code, 0 when every command ran and 1
/// when one failed, so Java never reads it out of the text.
extern "system" fn run(mut env: JNIEnv, _class: JClass, line: JString) -> jstring {
    let line: String = env.get_string(&line).map(Into::into).unwrap_or_default();
    // A panic must not cross into Java: it would take the whole app down.
    let (ok, said) = std::panic::catch_unwind(|| super::serve(&line))
        .unwrap_or_else(|_| (false, "error: the command panicked; logcat has it".into()));
    let out = format!("{}{said}", if ok { '0' } else { '1' });
    env.new_string(out)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

/// Hands `Control.run` its body, then registers the receiver.
pub fn start() {
    let natives = [(
        "run",
        "(Ljava/lang/String;)Ljava/lang/String;",
        run as *mut std::ffi::c_void,
    )];
    if let Err(e) = crate::platform::register_natives("Control", &natives) {
        crate::local::log(&format!("control: not started: {e}"));
        return;
    }
    // A failure here is logged by the call itself, as `Control.start: ...`.
    crate::platform::java_call("Control", "start", "(Landroid/content/Context;)V", None);
    crate::local::log("control: listening for adb commands");
}
