//! Serves an episode the way a cast does, for poking at with curl, ffprobe and mpv.
//! cargo run --example cast_serve --features cast -- <file.mkv>

use std::sync::Arc;

use anibeam_core::cast::media::{Prepared, Receiver, Subs};
use anibeam_core::cast::server::{Registry, Served, serve};

#[tokio::main]
async fn main() {
    let path = std::path::PathBuf::from(std::env::args().nth(1).unwrap());
    let probe = anibeam_core::cast::mkv::probe(&anibeam_core::cast::mkv::open(&path).unwrap());
    let subs = probe.subs.first().map(|s| Subs::Track(s.number));
    let p = Arc::new(Prepared::new(&path, None, subs, Receiver::Cast).unwrap());
    let reg = Arc::new(Registry::default());
    reg.add(
        "t".into(),
        Served {
            path: path.clone(),
            mime: "video/x-matroska".into(),
            prepared: Some(p),
            relay: None,
        },
    );
    let at = serve(reg, 0).await.unwrap();
    println!("http://127.0.0.1:{}/c/t/master.m3u8", at.port());
    std::future::pending::<()>().await;
}
