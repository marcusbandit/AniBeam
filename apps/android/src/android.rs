//! The few things only Java can do, reached over JNI: opening a link in the browser,
//! asking how much room the phone has left, and turning the screen for the player.

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

#[cfg(target_os = "android")]
fn with_env<R>(
    f: impl FnOnce(&mut jni::JNIEnv, &jni::objects::JObject) -> Result<R, Box<dyn std::error::Error>>,
) -> Result<R, Box<dyn std::error::Error>> {
    use jni::JavaVM;
    use jni::objects::JObject;
    let ctx = ndk_context::android_context();
    let vm = unsafe { JavaVM::from_raw(ctx.vm().cast())? };
    let mut env = vm.attach_current_thread()?;
    let activity = unsafe { JObject::from_raw(ctx.context().cast()) };
    let out = f(&mut env, &activity);
    // A Java exception left pending would fail the next call on this thread, and means the
    // call did not do what it was asked.
    if env.exception_check().unwrap_or(false) {
        let _ = env.exception_describe();
        let _ = env.exception_clear();
        return Err("Java threw".into());
    }
    out
}

#[cfg(target_os = "android")]
fn service_intent<'a>(
    env: &mut jni::JNIEnv<'a>,
) -> Result<jni::objects::JObject<'a>, Box<dyn std::error::Error>> {
    use jni::objects::JValue;
    let intent = env.new_object("android/content/Intent", "()V", &[])?;
    let pkg = env.new_string("com.marcusrosado.anibeam")?;
    let class = env.new_string("com.marcusrosado.anibeam.TorrentService")?;
    // setClassName: a class lookup from a native thread would miss the app's own classes.
    env.call_method(
        &intent,
        "setClassName",
        "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;",
        &[JValue::Object(&pkg), JValue::Object(&class)],
    )?;
    Ok(intent)
}

/// Starts the download service, or updates its notification to `text` when it runs.
/// False when Android refused, as it does for an app in the background on Android 12 and
/// later; the caller tries again on its next look.
pub fn start_service(text: &str) -> bool {
    #[cfg(target_os = "android")]
    {
        let r = with_env(|env, activity| {
            use jni::objects::JValue;
            let intent = service_intent(env)?;
            let key = env.new_string("text")?;
            let value = env.new_string(text)?;
            env.call_method(
                &intent,
                "putExtra",
                "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;",
                &[JValue::Object(&key), JValue::Object(&value)],
            )?;
            env.call_method(
                activity,
                "startForegroundService",
                "(Landroid/content/Intent;)Landroid/content/ComponentName;",
                &[JValue::Object(&intent)],
            )?;
            Ok(())
        });
        if let Err(e) = r {
            crate::local::log(&format!("the download service did not start: {e}"));
            return false;
        }
        true
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = text;
        true
    }
}

/// Stops the download service once nothing is downloading.
pub fn stop_service() {
    #[cfg(target_os = "android")]
    {
        let r = with_env(|env, activity| {
            use jni::objects::JValue;
            let intent = service_intent(env)?;
            env.call_method(activity, "stopService", "(Landroid/content/Intent;)Z", &[JValue::Object(&intent)])?;
            Ok(())
        });
        if let Err(e) = r {
            crate::local::log(&format!("the download service did not stop: {e}"));
        }
    }
}

/// Whether the network in use costs nothing per byte (Wi-Fi, Ethernet): `None` when the
/// phone has no network, or Android would not say.
pub fn unmetered() -> Option<bool> {
    #[cfg(target_os = "android")]
    {
        with_env(|env, activity| {
            use jni::objects::JValue;
            let name = env.new_string("connectivity")?;
            let cm = env
                .call_method(activity, "getSystemService", "(Ljava/lang/String;)Ljava/lang/Object;", &[JValue::Object(&name)])?
                .l()?;
            let network = env.call_method(&cm, "getActiveNetwork", "()Landroid/net/Network;", &[])?.l()?;
            if network.is_null() {
                return Ok(None);
            }
            let caps = env
                .call_method(
                    &cm,
                    "getNetworkCapabilities",
                    "(Landroid/net/Network;)Landroid/net/NetworkCapabilities;",
                    &[JValue::Object(&network)],
                )?
                .l()?;
            if caps.is_null() {
                return Ok(None);
            }
            // NET_CAPABILITY_NOT_METERED is 11.
            let free = env.call_method(&caps, "hasCapability", "(I)Z", &[JValue::Int(11)])?.z()?;
            Ok(Some(free))
        })
        .unwrap_or(None)
    }
    #[cfg(not(target_os = "android"))]
    Some(true)
}

/// Asks once for the notification permission Android 13 and later want before a service
/// may show its notification.
pub fn ask_notifications() {
    #[cfg(target_os = "android")]
    {
        let r = with_env(|env, activity| {
            use jni::objects::JValue;
            let sdk = env.get_static_field("android/os/Build$VERSION", "SDK_INT", "I")?.i()?;
            if sdk < 33 {
                return Ok(());
            }
            let permission = env.new_string("android.permission.POST_NOTIFICATIONS")?;
            let granted =
                env.call_method(activity, "checkSelfPermission", "(Ljava/lang/String;)I", &[JValue::Object(&permission)])?.i()?;
            if granted == 0 {
                return Ok(());
            }
            let list = env.new_object_array(1, "java/lang/String", &permission)?;
            env.call_method(activity, "requestPermissions", "([Ljava/lang/String;I)V", &[JValue::Object(&list), JValue::Int(1)])?;
            Ok(())
        });
        if let Err(e) = r {
            crate::local::log(&format!("could not ask for notifications: {e}"));
        }
    }
}

/// Puts `text` on the clipboard, so a connection check can be pasted into a message.
pub fn copy_text(label: &str, text: &str) -> bool {
    #[cfg(target_os = "android")]
    {
        let r = with_env(|env, activity| {
            use jni::objects::JValue;
            let name = env.new_string("clipboard")?;
            let manager = env
                .call_method(activity, "getSystemService", "(Ljava/lang/String;)Ljava/lang/Object;", &[JValue::Object(&name)])?
                .l()?;
            let label = env.new_string(label)?;
            let text = env.new_string(text)?;
            let clip = env
                .call_static_method(
                    "android/content/ClipData",
                    "newPlainText",
                    "(Ljava/lang/CharSequence;Ljava/lang/CharSequence;)Landroid/content/ClipData;",
                    &[JValue::Object(&label), JValue::Object(&text)],
                )?
                .l()?;
            env.call_method(&manager, "setPrimaryClip", "(Landroid/content/ClipData;)V", &[JValue::Object(&clip)])?;
            Ok(())
        });
        if let Err(e) = r {
            crate::local::log(&format!("could not copy: {e}"));
            return false;
        }
        true
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = (label, text);
        false
    }
}

/// The Activity's global reference, set by `android_main`.
#[cfg(target_os = "android")]
static ACTIVITY: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[cfg(target_os = "android")]
pub fn set_activity(activity: *mut std::ffi::c_void) {
    ACTIVITY.store(activity as usize, std::sync::atomic::Ordering::Release);
}

/// Calls one of `Screen.java`'s static methods with the activity and a flag. The class is
/// the app's own, so it is loaded through the activity's class loader: a lookup from a
/// native thread would only see the system's classes.
#[cfg(target_os = "android")]
fn screen(method: &str, on: bool) {
    let raw = ACTIVITY.load(std::sync::atomic::Ordering::Acquire);
    if raw == 0 {
        return;
    }
    // `with_env`'s context is the Application, and Screen needs the Activity.
    let r = with_env(|env, _app| {
        use jni::objects::{JClass, JObject, JValue};
        // The thread stays attached for the life of the app, so the local references made
        // here are freed with a frame of their own rather than piling up per call.
        env.with_local_frame(8, |env| -> Result<(), Box<dyn std::error::Error>> {
            let activity = &unsafe { JObject::from_raw(raw as jni::sys::jobject) };
            let loader = env.call_method(activity, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])?.l()?;
            let name = env.new_string("com.marcusrosado.anibeam.Screen")?;
            let class = env
                .call_method(&loader, "loadClass", "(Ljava/lang/String;)Ljava/lang/Class;", &[JValue::Object(&name)])?
                .l()?;
            let class = JClass::from(class);
            env.call_static_method(
                &class,
                method,
                "(Landroid/app/Activity;Z)V",
                &[JValue::Object(activity), JValue::Bool(on.into())],
            )?;
            Ok(())
        })
    });
    if let Err(e) = r {
        crate::local::log(&format!("Screen.{method}: {e}"));
    }
}

/// The player's screen: landscape with the system bars hidden, or back as it was.
pub fn player_screen(on: bool) {
    #[cfg(target_os = "android")]
    screen("player", on);
    #[cfg(not(target_os = "android"))]
    let _ = on;
}

/// Holds Wi-Fi multicast while devices are being looked for.
pub fn multicast(on: bool) {
    #[cfg(target_os = "android")]
    screen("multicast", on);
    #[cfg(not(target_os = "android"))]
    let _ = on;
}

/// Keeps the display on while a file plays, and lets it sleep when paused or closed.
pub fn player_awake(on: bool) {
    #[cfg(target_os = "android")]
    screen("awake", on);
    #[cfg(not(target_os = "android"))]
    let _ = on;
}
