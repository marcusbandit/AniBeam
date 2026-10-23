//! Writes an episode as the HLS a Chromecast would get, to look at with ffprobe and mpv.
//! cargo run --example cast_hls --features cast -- <file.mkv> <out dir>

use anibeam_core::cast::media::{Prepared, Receiver, Subs};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (input, out) = (
        std::path::Path::new(&args[1]),
        std::path::Path::new(&args[2]),
    );
    let t = std::time::Instant::now();
    let probe = anibeam_core::cast::mkv::probe(&anibeam_core::cast::mkv::open(input).unwrap());
    let subs = probe.subs.first().map(|s| Subs::Track(s.number));
    let p = Prepared::new(input, None, subs, Receiver::Cast).unwrap();
    eprintln!(
        "prepared in {:?}: {} segments, reorder {} ms",
        t.elapsed(),
        p.segments.len(),
        p.scan.reorder_ms
    );
    std::fs::create_dir_all(out.join("seg")).unwrap();
    std::fs::write(out.join("master.m3u8"), p.master()).unwrap();
    std::fs::write(out.join("media.m3u8"), p.media_playlist()).unwrap();
    std::fs::write(out.join("subs.m3u8"), p.subs_playlist()).unwrap();
    std::fs::write(out.join("subs.vtt"), p.vtt()).unwrap();
    std::fs::write(out.join("init.mp4"), &p.init).unwrap();
    for i in 0..p.segments.len() {
        std::fs::write(out.join(format!("seg/{i}.m4s")), p.segment(i).unwrap()).unwrap();
    }
    eprintln!("written in {:?}", t.elapsed());
}
