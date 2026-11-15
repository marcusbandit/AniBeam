include!("../trackers/env_local.rs");

/// The Discord application id a build bundles, for `option_env!` in `presence/mod.rs`,
/// read the way trackers/build.rs reads the tracker apps.
fn main() {
    bundle(&["ANIBEAM_DISCORD_CLIENT_ID"]);
}
