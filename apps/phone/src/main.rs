//! The iPhone's executable. `ios/cargo.sh`, Xcode's build step, builds it into the app
//! bundle; Android loads the library instead, through `android_main`. On any other target
//! it does nothing: the desktop has its own app.

fn main() {
    #[cfg(target_os = "ios")]
    anibeam::ios_main();
}
