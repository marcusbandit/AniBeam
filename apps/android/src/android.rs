//! The few things only Java can do, reached over JNI: opening a link in the browser, and
//! asking how much room the phone has left.

/// Opens `url` in the user's browser: AniList and MAL sign in there, then send the browser
/// back to the core's local callback.
pub fn open_url(url: &str) {
    #[cfg(target_os = "android")]
    if let Err(e) = view(url) {
        crate::local::log(&format!("could not open the browser: {e}"));
    }
    #[cfg(not(target_os = "android"))]
    crate::local::log(&format!("open {url}"));
}

#[cfg(target_os = "android")]
fn view(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    use jni::JavaVM;
    use jni::objects::{JObject, JValue};
    let ctx = ndk_context::android_context();
    let vm = unsafe { JavaVM::from_raw(ctx.vm().cast())? };
    let mut env = vm.attach_current_thread()?;
    let activity = unsafe { JObject::from_raw(ctx.context().cast()) };
    let text = env.new_string(url)?;
    let uri = env
        .call_static_method("android/net/Uri", "parse", "(Ljava/lang/String;)Landroid/net/Uri;", &[JValue::Object(&text)])?
        .l()?;
    let action = env.new_string("android.intent.action.VIEW")?;
    let intent = env.new_object(
        "android/content/Intent",
        "(Ljava/lang/String;Landroid/net/Uri;)V",
        &[JValue::Object(&action), JValue::Object(&uri)],
    )?;
    // FLAG_ACTIVITY_NEW_TASK: the call may come from a thread with no activity of its own.
    env.call_method(&intent, "addFlags", "(I)Landroid/content/Intent;", &[JValue::Int(0x1000_0000)])?;
    env.call_method(&activity, "startActivity", "(Landroid/content/Intent;)V", &[JValue::Object(&intent)])?;
    Ok(())
}

/// Bytes free and in total on the filesystem that holds `dir`.
pub fn space(dir: &std::path::Path) -> Option<(u64, u64)> {
    let s = rustix::fs::statvfs(dir).ok()?;
    Some((s.f_bavail * s.f_frsize, s.f_blocks * s.f_frsize))
}

/// Bytes the files under `dir` take.
pub fn used(dir: &std::path::Path) -> u64 {
    let mut total = 0;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            match e.metadata() {
                Ok(m) if m.is_dir() => stack.push(e.path()),
                Ok(m) => total += m.len(),
                Err(_) => {}
            }
        }
    }
    total
}
