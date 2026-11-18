//! The channel's Android end: `Control.java`'s receiver, given its native `run` and started.

use std::error::Error;

use jni::objects::{JClass, JObject, JString, JValue};
use jni::sys::jstring;
use jni::{JNIEnv, JavaVM, NativeMethod};

/// `Control.run`, which Java calls on the receiver's own thread with the command line.
extern "system" fn run(mut env: JNIEnv, _class: JClass, line: JString) -> jstring {
    let line: String = env.get_string(&line).map(Into::into).unwrap_or_default();
    // A panic must not cross into Java: it would take the whole app down.
    let out = std::panic::catch_unwind(|| super::serve(&line))
        .unwrap_or_else(|_| "error: the command panicked; logcat has it".into());
    env.new_string(out)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

/// Hands `Control.run` its body and registers the receiver. NativeActivity loads this
/// library itself, so the JVM would not find the method by its symbol name, and the class
/// comes through the app's own class loader, which a native thread would not see.
pub fn start() {
    let r = (|| -> Result<(), Box<dyn Error>> {
        let ctx = ndk_context::android_context();
        let vm = unsafe { JavaVM::from_raw(ctx.vm().cast())? };
        let mut env = vm.attach_current_thread()?;
        let context = unsafe { JObject::from_raw(ctx.context().cast()) };
        let out = env.with_local_frame(8, |env| -> Result<(), Box<dyn Error>> {
            let loader = env
                .call_method(&context, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])?
                .l()?;
            let name = env.new_string("com.marcusrosado.anibeam.Control")?;
            let class = JClass::from(
                env.call_method(
                    &loader,
                    "loadClass",
                    "(Ljava/lang/String;)Ljava/lang/Class;",
                    &[JValue::Object(&name)],
                )?
                .l()?,
            );
            env.register_native_methods(
                &class,
                &[NativeMethod {
                    name: "run".into(),
                    sig: "(Ljava/lang/String;)Ljava/lang/String;".into(),
                    fn_ptr: run as *mut std::ffi::c_void,
                }],
            )?;
            env.call_static_method(
                &class,
                "start",
                "(Landroid/content/Context;)V",
                &[JValue::Object(&context)],
            )?;
            Ok(())
        });
        if env.exception_check().unwrap_or(false) {
            let _ = env.exception_describe();
            let _ = env.exception_clear();
            return Err("Java threw".into());
        }
        out
    })();
    match r {
        Ok(()) => crate::local::log("control: listening for adb commands"),
        Err(e) => crate::local::log(&format!("control: not started: {e}")),
    }
}
