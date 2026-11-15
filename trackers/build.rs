include!("env_local.rs");

/// The tracker apps a build bundles, for `option_env!` in `accounts.rs`. Electron's Vite
/// read them out of the workspace's `.env.local`; Cargo has no such convention, so this
/// does the same read itself. The Discord application id is the playback's (playback/build.rs).
fn main() {
    bundle(&[
        "ANIBEAM_ANILIST_CLIENT_ID",
        "ANIBEAM_MAL_CLIENT_ID",
        "ANIBEAM_MAL_CLIENT_SECRET",
    ]);
}
