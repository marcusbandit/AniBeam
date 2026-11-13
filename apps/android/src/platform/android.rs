//! Android: the entry point NativeActivity calls, and the few things only Java can do,
//! reached over JNI: opening a link in the browser, the download service and its
//! notifications, the audio focus, turning the screen for the player, and reading the
//! phone's language and light or dark setting.

use super::FocusAsk;

/// The whole entry point. The stock NativeActivity loads this library and calls it; Java
/// only holds the download service's notification and turns the screen for the player.
#[unsafe(no_mangle)]
fn android_main(android: slint::android::AndroidApp) {
    use slint::android::android_activity::{MainEvent, PollEvent};

    let data_dir = android
        .internal_data_path()
        .unwrap_or_else(|| "/data/data/com.marcusrosado.anibeam/files".into());
    // The activity itself, for the calls only an Activity answers; ndk_context holds the
    // Application. The reference lives as long as `android`, which outlives `run`.
    set_activity(android.activity_as_ptr());
    let (commands, inbox) = std::sync::mpsc::channel();

    // Coming back to the app is a good moment to look at the host again.
    let on_resume = commands.clone();
    slint::android::init_with_event_listener(android, move |event| {
        if let PollEvent::Main(MainEvent::Resume { .. }) = event {
            let _ = on_resume.send(crate::worker::Command::Sync);
        }
        // The phone's light or dark setting or its language changed: Appearance's System mode
        // and a language left to follow the phone follow it.
        if let PollEvent::Main(MainEvent::ConfigChanged { .. }) = event {
            let _ = slint::invoke_from_event_loop(crate::system_changed);
        }
    })
    .expect("the Slint Android backend starts");

    if let Err(e) = crate::run(data_dir, commands, inbox) {
        eprintln!("AniBeam stopped: {e}");
    }
}

/// Writes `line` to logcat, under AniBeamCore.
pub fn log(line: &str) {
    use std::ffi::CString;
    unsafe extern "C" {
        fn __android_log_write(
            prio: i32,
            tag: *const std::ffi::c_char,
            text: *const std::ffi::c_char,
        ) -> i32;
    }
    let tag = CString::new("AniBeamCore").unwrap();
    let text = CString::new(line.replace('\0', " ")).unwrap();
    // Priority 4 is INFO.
    unsafe { __android_log_write(4, tag.as_ptr(), text.as_ptr()) };
}

/// Opens `url` in the user's browser: AniList and MAL sign in there, then send the browser
/// back to the core's local callback.
pub fn open_url(url: &str) {
    if let Err(e) = view(url) {
        crate::local::log(&format!("could not open the browser: {e}"));
    }
}

fn view(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    use jni::JavaVM;
    use jni::objects::{JObject, JValue};
    let ctx = ndk_context::android_context();
    let vm = unsafe { JavaVM::from_raw(ctx.vm().cast())? };
    let mut env = vm.attach_current_thread()?;
    let app = unsafe { JObject::from_raw(ctx.context().cast()) };
    let text = env.new_string(url)?;
    let uri = env
        .call_static_method(
            "android/net/Uri",
            "parse",
            "(Ljava/lang/String;)Landroid/net/Uri;",
            &[JValue::Object(&text)],
        )?
        .l()?;
    let action = env.new_string("android.intent.action.VIEW")?;
    let intent = env.new_object(
        "android/content/Intent",
        "(Ljava/lang/String;Landroid/net/Uri;)V",
        &[JValue::Object(&action), JValue::Object(&uri)],
    )?;
    // FLAG_ACTIVITY_NEW_TASK: the call may come from a thread with no activity of its own, and the context is the Application.
    env.call_method(
        &intent,
        "addFlags",
        "(I)Landroid/content/Intent;",
        &[JValue::Int(0x1000_0000)],
    )?;
    env.call_method(
        &app,
        "startActivity",
        "(Landroid/content/Intent;)V",
        &[JValue::Object(&intent)],
    )?;
    Ok(())
}

/// Opens Android's file picker through `PickActivity`, which copies between the picked
/// place and `path` and leaves one word in `result` when it is done.
pub fn pick_file(
    kind: crate::files::Pick,
    path: &std::path::Path,
    name: &str,
    result: &std::path::Path,
) -> Result<(), String> {
    use jni::JavaVM;
    use jni::objects::{JObject, JValue};
    let go = || -> Result<(), Box<dyn std::error::Error>> {
        let ctx = ndk_context::android_context();
        let vm = unsafe { JavaVM::from_raw(ctx.vm().cast())? };
        let mut env = vm.attach_current_thread()?;
        let app = unsafe { JObject::from_raw(ctx.context().cast()) };
        let started = env.with_local_frame(16, |env| -> Result<(), Box<dyn std::error::Error>> {
            let intent = env.new_object("android/content/Intent", "()V", &[])?;
            let pkg = env.new_string("com.marcusrosado.anibeam")?;
            let class = env.new_string("com.marcusrosado.anibeam.PickActivity")?;
            env.call_method(
                &intent,
                "setClassName",
                "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;",
                &[JValue::Object(&pkg), JValue::Object(&class)],
            )?;
            for (key, value) in [
                ("path", path.to_string_lossy().into_owned()),
                ("name", name.to_string()),
                ("result", result.to_string_lossy().into_owned()),
            ] {
                let key = env.new_string(key)?;
                let value = env.new_string(value)?;
                env.call_method(
                    &intent,
                    "putExtra",
                    "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;",
                    &[JValue::Object(&key), JValue::Object(&value)],
                )?;
            }
            let key = env.new_string("mode")?;
            env.call_method(
                &intent,
                "putExtra",
                "(Ljava/lang/String;I)Landroid/content/Intent;",
                &[
                    JValue::Object(&key),
                    JValue::Int(i32::from(kind == crate::files::Pick::Open)),
                ],
            )?;
            // FLAG_ACTIVITY_NEW_TASK: the context is the Application.
            env.call_method(
                &intent,
                "addFlags",
                "(I)Landroid/content/Intent;",
                &[JValue::Int(0x1000_0000)],
            )?;
            env.call_method(
                &app,
                "startActivity",
                "(Landroid/content/Intent;)V",
                &[JValue::Object(&intent)],
            )?;
            Ok(())
        });
        // A Java exception left pending would fail the next call on this thread, so it is
        // cleared whether or not the frame got to its end.
        if env.exception_check().unwrap_or(false) {
            let _ = env.exception_clear();
            return Err("Java threw".into());
        }
        started
    };
    go().map_err(|e| tr!("files-did-not-open", error = e.to_string()))
}

/// The phone's language as Android reports it: `da-DK`, `pt-BR`, `zh-Hans-CN`.
pub fn system_locale() -> String {
    use jni::JavaVM;
    use jni::objects::{JObject, JString};
    let read = || -> Result<String, Box<dyn std::error::Error>> {
        let ctx = ndk_context::android_context();
        let vm = unsafe { JavaVM::from_raw(ctx.vm().cast())? };
        let mut env = vm.attach_current_thread()?;
        let app = unsafe { JObject::from_raw(ctx.context().cast()) };
        env.with_local_frame(8, |env| -> Result<String, Box<dyn std::error::Error>> {
            let res = env
                .call_method(
                    &app,
                    "getResources",
                    "()Landroid/content/res/Resources;",
                    &[],
                )?
                .l()?;
            let cfg = env
                .call_method(
                    &res,
                    "getConfiguration",
                    "()Landroid/content/res/Configuration;",
                    &[],
                )?
                .l()?;
            let list = env
                .call_method(&cfg, "getLocales", "()Landroid/os/LocaleList;", &[])?
                .l()?;
            let first = env
                .call_method(&list, "get", "(I)Ljava/util/Locale;", &[0.into()])?
                .l()?;
            let tag = env
                .call_method(&first, "toLanguageTag", "()Ljava/lang/String;", &[])?
                .l()?;
            Ok(env.get_string(&JString::from(tag))?.into())
        })
    };
    read().unwrap_or_default()
}

/// Whether the phone is set to dark.
pub fn system_dark() -> bool {
    use jni::JavaVM;
    use jni::objects::JObject;
    let read = || -> Result<bool, Box<dyn std::error::Error>> {
        let ctx = ndk_context::android_context();
        let vm = unsafe { JavaVM::from_raw(ctx.vm().cast())? };
        let mut env = vm.attach_current_thread()?;
        let app = unsafe { JObject::from_raw(ctx.context().cast()) };
        env.with_local_frame(8, |env| -> Result<bool, Box<dyn std::error::Error>> {
            let res = env
                .call_method(
                    &app,
                    "getResources",
                    "()Landroid/content/res/Resources;",
                    &[],
                )?
                .l()?;
            let cfg = env
                .call_method(
                    &res,
                    "getConfiguration",
                    "()Landroid/content/res/Configuration;",
                    &[],
                )?
                .l()?;
            let ui = env.get_field(&cfg, "uiMode", "I")?.i()?;
            // UI_MODE_NIGHT_MASK is 0x30 and UI_MODE_NIGHT_YES 0x20.
            Ok(ui & 0x30 == 0x20)
        })
    };
    read().unwrap_or(true)
}

/// Readies the process for libmpv, once: libavcodec is handed the Java VM, which MediaCodec
/// and AudioTrack reach Java through.
pub fn prepare_mpv() {
    unsafe extern "C" {
        /// libavcodec's.
        fn av_jni_set_java_vm(vm: *mut std::ffi::c_void, log_ctx: *mut std::ffi::c_void) -> i32;
    }
    static VM: std::sync::Once = std::sync::Once::new();
    VM.call_once(|| unsafe {
        av_jni_set_java_vm(ndk_context::android_context().vm(), std::ptr::null_mut());
    });
}

/// Whether this is the Android emulator rather than a phone.
pub fn emulator() -> bool {
    use std::ffi::{CStr, c_char};
    unsafe extern "C" {
        fn __system_property_get(name: *const c_char, value: *mut c_char) -> i32;
    }
    let mut value = [0 as c_char; 92];
    let n = unsafe { __system_property_get(c"ro.hardware".as_ptr(), value.as_mut_ptr()) };
    let hw = unsafe { CStr::from_ptr(value.as_ptr()) }.to_string_lossy();
    n > 0 && (hw == "ranchu" || hw == "goldfish")
}

fn with_env<R>(
    f: impl FnOnce(&mut jni::JNIEnv, &jni::objects::JObject) -> Result<R, Box<dyn std::error::Error>>,
) -> Result<R, Box<dyn std::error::Error>> {
    use jni::JavaVM;
    use jni::objects::JObject;
    let ctx = ndk_context::android_context();
    let vm = unsafe { JavaVM::from_raw(ctx.vm().cast())? };
    let mut env = vm.attach_current_thread()?;
    let app = unsafe { JObject::from_raw(ctx.context().cast()) };
    let out = f(&mut env, &app);
    // A Java exception left pending would fail the next call on this thread, and means the
    // call did not do what it was asked.
    if env.exception_check().unwrap_or(false) {
        let _ = env.exception_describe();
        let _ = env.exception_clear();
        return Err("Java threw".into());
    }
    out
}

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

/// The words the download service shows through Android, which Java cannot read from the
/// catalog: the two channel names and the title it falls back to. They go to the service
/// translated; its English is only for a call that carries none.
fn service_words() -> [(&'static str, String); 3] {
    [
        ("channel_downloads", tr!("notify-channel-downloads")),
        ("channel_ready", tr!("notify-channel-ready")),
        ("fallback_title", tr!("notify-fallback-title")),
    ]
}

/// Hands `TorrentService.words` the service's words in the app's language, so the channels
/// in the phone's notification settings are named in it too.
fn set_service_words(
    env: &mut jni::JNIEnv,
    app: &jni::objects::JObject,
    class: &jni::objects::JClass,
) -> Result<(), Box<dyn std::error::Error>> {
    use jni::objects::JValue;
    let words = service_words();
    let downloads = env.new_string(&words[0].1)?;
    let ready = env.new_string(&words[1].1)?;
    let fallback = env.new_string(&words[2].1)?;
    env.call_static_method(
        class,
        "words",
        "(Landroid/content/Context;Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V",
        &[
            JValue::Object(app),
            JValue::Object(&downloads),
            JValue::Object(&ready),
            JValue::Object(&fallback),
        ],
    )?;
    Ok(())
}

/// The language changed: the service's channel names and fallback title are sent again.
pub fn relocalize() {
    let r = with_env(|env, app| {
        env.with_local_frame(32, |env| -> Result<(), Box<dyn std::error::Error>> {
            let class = app_class(env, app, "TorrentService")?;
            set_service_words(env, app, &class)
        })
    });
    if let Err(e) = r {
        crate::local::log(&format!("the notification channels were not renamed: {e}"));
    }
}

/// Starts the download service with `shown` as its notification. False when Android
/// refused, as it does for an app in the background on Android 12 and later; the caller
/// tries again on its next look. A running service's notification changes through
/// `update_service` instead.
pub fn start_service(shown: &crate::notify::Ongoing) -> bool {
    // The UI thread stays attached, so the call's local references go with a frame.
    let r = with_env(|env, app| {
        env.with_local_frame(32, |env| -> Result<(), Box<dyn std::error::Error>> {
            use jni::objects::JValue;
            let intent = service_intent(env)?;
            let words = service_words();
            let mut extras: Vec<(&str, &String)> =
                vec![("title", &shown.title), ("text", &shown.text)];
            extras.extend(words.iter().map(|(key, value)| (*key, value)));
            for (key, value) in extras {
                let key = env.new_string(key)?;
                let value = env.new_string(value)?;
                env.call_method(
                    &intent,
                    "putExtra",
                    "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;",
                    &[JValue::Object(&key), JValue::Object(&value)],
                )?;
            }
            let key = env.new_string("progress")?;
            env.call_method(
                &intent,
                "putExtra",
                "(Ljava/lang/String;I)Landroid/content/Intent;",
                &[JValue::Object(&key), JValue::Int(shown.progress)],
            )?;
            env.call_method(
                app,
                "startForegroundService",
                "(Landroid/content/Intent;)Landroid/content/ComponentName;",
                &[JValue::Object(&intent)],
            )?;
            Ok(())
        })
    });
    if let Err(e) = r {
        crate::local::log(&format!("the download service did not start: {e}"));
        return false;
    }
    true
}

/// Changes the running download service's notification to `shown`, in place
/// (`TorrentService.update`). False when the call failed, so the next look tries again.
pub fn update_service(shown: &crate::notify::Ongoing) -> bool {
    let r = with_env(|env, app| {
        env.with_local_frame(8, |env| -> Result<(), Box<dyn std::error::Error>> {
            use jni::objects::JValue;
            let class = app_class(env, app, "TorrentService")?;
            let title = env.new_string(&shown.title)?;
            let text = env.new_string(&shown.text)?;
            env.call_static_method(
                &class,
                "update",
                "(Landroid/content/Context;Ljava/lang/String;Ljava/lang/String;I)V",
                &[
                    JValue::Object(app),
                    JValue::Object(&title),
                    JValue::Object(&text),
                    JValue::Int(shown.progress),
                ],
            )?;
            Ok(())
        })
    });
    if let Err(e) = r {
        crate::local::log(&format!("the download notification did not change: {e}"));
        return false;
    }
    true
}

/// Posts "<show> episode 7 is ready" for a download that finished, under `id`, on the
/// channel for finished episodes (`TorrentService.ready`).
pub fn episode_ready(id: i32, title: &str, text: &str) {
    let r = with_env(|env, app| {
        use jni::objects::JValue;
        env.with_local_frame(32, |env| -> Result<(), Box<dyn std::error::Error>> {
            // The Application's class loader sees the app's own classes, as the
            // activity's does.
            let class = app_class(env, app, "TorrentService")?;
            // The channel it posts on is made in the app's language, in a process
            // that has not started the service yet too.
            // A rename that fails must not cost the notification itself.
            if let Err(e) = set_service_words(env, app, &class) {
                let _ = env.exception_clear();
                crate::local::log(&format!("notification words not set: {e}"));
            }
            let title = env.new_string(title)?;
            let text = env.new_string(text)?;
            env.call_static_method(
                &class,
                "ready",
                "(Landroid/content/Context;ILjava/lang/String;Ljava/lang/String;)V",
                &[
                    JValue::Object(app),
                    JValue::Int(id),
                    JValue::Object(&title),
                    JValue::Object(&text),
                ],
            )?;
            Ok(())
        })
    });
    if let Err(e) = r {
        crate::local::log(&format!("could not say an episode is ready: {e}"));
    }
}

/// Stops the download service once nothing is downloading.
pub fn stop_service() {
    let r = with_env(|env, app| {
        use jni::objects::JValue;
        let intent = service_intent(env)?;
        env.call_method(
            app,
            "stopService",
            "(Landroid/content/Intent;)Z",
            &[JValue::Object(&intent)],
        )?;
        Ok(())
    });
    if let Err(e) = r {
        crate::local::log(&format!("the download service did not stop: {e}"));
    }
}

/// Whether the network in use costs nothing per byte (Wi-Fi, Ethernet): `None` when the
/// phone has no network, or Android would not say.
pub fn unmetered() -> Option<bool> {
    with_env(|env, app| {
        use jni::objects::JValue;
        let name = env.new_string("connectivity")?;
        let cm = env
            .call_method(
                app,
                "getSystemService",
                "(Ljava/lang/String;)Ljava/lang/Object;",
                &[JValue::Object(&name)],
            )?
            .l()?;
        let network = env
            .call_method(&cm, "getActiveNetwork", "()Landroid/net/Network;", &[])?
            .l()?;
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
        let free = env
            .call_method(&caps, "hasCapability", "(I)Z", &[JValue::Int(11)])?
            .z()?;
        Ok(Some(free))
    })
    .unwrap_or(None)
}

/// Asks for the notification permission Android 13 and later want before a service may
/// show its notification: whether the ask went out, or was not needed. Only an Activity has
/// `requestPermissions`, and `with_env`'s context is the Application, so the ask goes
/// through `TorrentService.askPermission`.
pub fn ask_notifications() -> bool {
    matches!(
        java_call(
            "TorrentService",
            "askPermission",
            "(Landroid/app/Activity;)Z",
            None
        ),
        Some(jni::objects::JValueOwned::Bool(1))
    )
}

/// Opens the app's notification settings, where notifications go back on once Android has
/// stopped asking.
pub fn notification_settings() {
    java_call(
        "TorrentService",
        "openSettings",
        "(Landroid/app/Activity;)Z",
        None,
    );
}

/// Whether the app may show notifications: Android 13 and later ask the person first.
pub fn notifications_allowed() -> bool {
    // Read every second on a thread that stays attached, so its local references go
    // with a frame of their own rather than piling up.
    let r = with_env(|env, app| {
        env.with_local_frame(4, |env| -> Result<bool, Box<dyn std::error::Error>> {
            use jni::objects::JValue;
            let sdk = env
                .get_static_field("android/os/Build$VERSION", "SDK_INT", "I")?
                .i()?;
            if sdk < 33 {
                return Ok(true);
            }
            let permission = env.new_string("android.permission.POST_NOTIFICATIONS")?;
            let granted = env
                .call_method(
                    app,
                    "checkSelfPermission",
                    "(Ljava/lang/String;)I",
                    &[JValue::Object(&permission)],
                )?
                .i()?;
            Ok(granted == 0)
        })
    });
    r.unwrap_or(true)
}

/// Asks for the player's audio focus. Every change after a grant reaches
/// `crate::audio_focus` on the UI loop.
pub fn take_audio_focus() -> FocusAsk {
    // Without its native body the listener would throw on the first change, so no
    // request goes out until `Focus.changed` has one.
    if !FOCUS_NATIVES.load(std::sync::atomic::Ordering::Acquire) {
        if !register_focus() {
            return FocusAsk::Unavailable;
        }
        FOCUS_NATIVES.store(true, std::sync::atomic::Ordering::Release);
    }
    match java_call("Focus", "take", "(Landroid/app/Activity;)I", None) {
        Some(jni::objects::JValueOwned::Int(1)) => FocusAsk::Granted,
        Some(jni::objects::JValueOwned::Int(_)) => FocusAsk::Refused,
        _ => FocusAsk::Unavailable,
    }
}

/// Gives the player's audio focus back, so the sound it paused can play on.
pub fn give_audio_focus() {
    java_call("Focus", "give", "(Landroid/app/Activity;)V", None);
}

static FOCUS_NATIVES: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// `Focus.changed`, which Java calls on its UI thread.
extern "system" fn focus_changed(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    change: jni::sys::jint,
) {
    let _ = slint::invoke_from_event_loop(move || crate::audio_focus(change));
}

/// Hands `Focus.changed` its native body. NativeActivity loads this library itself, so the
/// JVM would not find the method by its symbol name.
fn register_focus() -> bool {
    let raw = ACTIVITY.load(std::sync::atomic::Ordering::Acquire);
    if raw == 0 {
        return false;
    }
    let r = with_env(|env, _app| {
        use jni::objects::JObject;
        env.with_local_frame(8, |env| -> Result<(), Box<dyn std::error::Error>> {
            let activity = &unsafe { JObject::from_raw(raw as jni::sys::jobject) };
            let class = app_class(env, activity, "Focus")?;
            env.register_native_methods(
                &class,
                &[jni::NativeMethod {
                    name: "changed".into(),
                    sig: "(I)V".into(),
                    fn_ptr: focus_changed as *mut std::ffi::c_void,
                }],
            )?;
            Ok(())
        })
    });
    match r {
        Ok(()) => true,
        Err(e) => {
            crate::local::log(&format!("audio focus: {e}"));
            false
        }
    }
}

/// Puts `text` on the clipboard, so a connection check can be pasted into a message.
pub fn copy_text(label: &str, text: &str) -> bool {
    let r = with_env(|env, app| {
        use jni::objects::JValue;
        let name = env.new_string("clipboard")?;
        let manager = env
            .call_method(
                app,
                "getSystemService",
                "(Ljava/lang/String;)Ljava/lang/Object;",
                &[JValue::Object(&name)],
            )?
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
        env.call_method(
            &manager,
            "setPrimaryClip",
            "(Landroid/content/ClipData;)V",
            &[JValue::Object(&clip)],
        )?;
        Ok(())
    });
    if let Err(e) = r {
        crate::local::log(&format!("could not copy: {e}"));
        return false;
    }
    true
}

/// The Activity's global reference, set by `android_main`.
static ACTIVITY: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn set_activity(activity: *mut std::ffi::c_void) {
    ACTIVITY.store(activity as usize, std::sync::atomic::Ordering::Release);
}

/// Calls one of `Screen.java`'s static methods with the activity and one more argument, and
/// hands back what it returns.
fn screen_call(
    method: &str,
    sig: &str,
    arg: Option<jni::objects::JValue>,
) -> Option<jni::objects::JValueOwned<'static>> {
    java_call("Screen", method, sig, arg)
}

/// One of the app's own Java classes, loaded through the activity's class loader: a lookup
/// from a native thread would only see the system's classes.
fn app_class<'l>(
    env: &mut jni::JNIEnv<'l>,
    activity: &jni::objects::JObject,
    class: &str,
) -> Result<jni::objects::JClass<'l>, Box<dyn std::error::Error>> {
    use jni::objects::{JClass, JValue};
    let loader = env
        .call_method(activity, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])?
        .l()?;
    let name = env.new_string(format!("com.marcusrosado.anibeam.{class}"))?;
    let class = env
        .call_method(
            &loader,
            "loadClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            &[JValue::Object(&name)],
        )?
        .l()?;
    Ok(JClass::from(class))
}

/// Calls a static method of one of the app's own Java classes with the activity and one more
/// argument, and hands back what it returns.
fn java_call(
    class: &str,
    method: &str,
    sig: &str,
    arg: Option<jni::objects::JValue>,
) -> Option<jni::objects::JValueOwned<'static>> {
    let raw = ACTIVITY.load(std::sync::atomic::Ordering::Acquire);
    if raw == 0 {
        return None;
    }
    // `with_env`'s context is the Application, and these classes need the Activity.
    let r = with_env(|env, _app| {
        use jni::objects::{JObject, JValue, JValueOwned};
        // The thread stays attached for the life of the app, so the local references made
        // here are freed with a frame of their own rather than piling up per call.
        env.with_local_frame(
            8,
            |env| -> Result<Option<JValueOwned<'static>>, Box<dyn std::error::Error>> {
                let activity = &unsafe { JObject::from_raw(raw as jni::sys::jobject) };
                let class = app_class(env, activity, class)?;
                let mut args = vec![JValue::Object(activity)];
                args.extend(arg);
                let out = env.call_static_method(&class, method, sig, &args)?;
                // Only plain values come back, which hold no reference into the frame.
                Ok(match out {
                    JValueOwned::Float(f) => Some(JValueOwned::Float(f)),
                    JValueOwned::Int(i) => Some(JValueOwned::Int(i)),
                    JValueOwned::Bool(b) => Some(JValueOwned::Bool(b)),
                    JValueOwned::Long(l) => Some(JValueOwned::Long(l)),
                    _ => None,
                })
            },
        )
    });
    match r {
        Ok(v) => v,
        Err(e) => {
            crate::local::log(&format!("{class}.{method}: {e}"));
            None
        }
    }
}

fn screen(method: &str, on: bool) {
    screen_call(
        method,
        "(Landroid/app/Activity;Z)V",
        Some(jni::objects::JValue::Bool(on.into())),
    );
}

/// How the screen is folded: the hinge angle in degrees, None on a phone without a hinge,
/// and whether the display is turned a quarter, which lays the crease across the screen.
pub fn fold() -> (Option<f32>, bool) {
    if let Some(jni::objects::JValueOwned::Float(f)) =
        screen_call("fold", "(Landroid/app/Activity;)F", None)
    {
        let rotation = (f / 1000.0).floor();
        let angle = f - rotation * 1000.0;
        return ((angle < 900.0).then_some(angle), rotation as i32 % 2 == 1);
    }
    (None, false)
}

/// Where the crease runs, as the phone reports it through `Crease.java`. On a phone without
/// the window extensions it never says.
pub fn crease() -> crate::crease::Reading {
    if let Some(jni::objects::JValueOwned::Long(packed)) =
        java_call("Crease", "at", "(Landroid/app/Activity;)J", None)
    {
        return crate::crease::decode(packed);
    }
    crate::crease::Reading::Silent
}

/// The window's brightness, 0 to 1.
pub fn brightness() -> f32 {
    match screen_call("brightness", "(Landroid/app/Activity;)F", None) {
        Some(jni::objects::JValueOwned::Float(f)) => f,
        _ => 0.5,
    }
}

/// Sets this window's brightness, 0 to 1; below 0 hands it back to the phone's setting.
pub fn set_brightness(level: f32) {
    screen_call(
        "setBrightness",
        "(Landroid/app/Activity;F)V",
        Some(jni::objects::JValue::Float(level)),
    );
}

/// The media volume, 0 to 1.
pub fn volume() -> f32 {
    match screen_call("volume", "(Landroid/app/Activity;)F", None) {
        Some(jni::objects::JValueOwned::Float(f)) => f,
        _ => 0.5,
    }
}

/// Sets the media volume, 0 to 1.
pub fn set_volume(level: f32) {
    screen_call(
        "setVolume",
        "(Landroid/app/Activity;F)V",
        Some(jni::objects::JValue::Float(level)),
    );
}

/// The player's screen: landscape with the system bars hidden, or back as it was.
pub fn player_screen(on: bool) {
    screen("player", on);
}

/// Holds Wi-Fi multicast while devices are being looked for.
pub fn multicast(on: bool) {
    screen("multicast", on);
}

/// Keeps the display on while a file plays, and lets it sleep when paused or closed.
pub fn player_awake(on: bool) {
    screen("awake", on);
}

/// The launcher icon showing now: 0 Default, 1 Dark, 2 Line art.
pub fn app_icon() -> i32 {
    if let Some(jni::objects::JValueOwned::Int(i)) =
        java_call("Icons", "current", "(Landroid/app/Activity;)I", None)
    {
        return i;
    }
    0
}

/// Shows launcher icon `which` and hides the others.
pub fn set_app_icon(which: i32) {
    java_call(
        "Icons",
        "pick",
        "(Landroid/app/Activity;I)V",
        Some(jni::objects::JValue::Int(which)),
    );
}

/// The line art icon's ground and ink as the launcher draws them, ARGB: the phone's Material
/// You colours on Android 12 and later.
pub fn line_colours() -> Option<(u32, u32)> {
    if let Some(jni::objects::JValueOwned::Long(l)) =
        java_call("Icons", "lineColours", "(Landroid/app/Activity;)J", None)
    {
        return Some(((l >> 32) as u32, l as u32));
    }
    None
}

/// Paints the window and the system bars for a page whose ground is `argb`: the bars' icons
/// turn dark over a light ground, so they stay readable.
pub fn bars(argb: u32) {
    java_call(
        "Bars",
        "ground",
        "(Landroid/app/Activity;I)V",
        Some(jni::objects::JValue::Int(argb as i32)),
    );
}
