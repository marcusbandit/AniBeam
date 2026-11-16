/// `phone_os`: the target is a phone's, Android or iOS. The library's trash reads it
/// (`trash.rs`): a phone deletes its own files outright. net/build.rs makes the same switch
/// for the net crate, so a new phone platform is one line here and one there.
fn main() {
    // It reads nothing but the target, which Cargo already tracks.
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rustc-check-cfg=cfg(phone_os)");
    let phone = std::env::var("CARGO_CFG_TARGET_OS").is_ok_and(|os| os == "android" || os == "ios");
    if phone {
        println!("cargo::rustc-cfg=phone_os");
    }
}
