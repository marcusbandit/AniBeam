//! Episodes converted as they play, for a client that cannot take the file as
//! it is: too big for its link, or a codec its decoder or a TV it casts to
//! cannot read. ffmpeg writes 8-bit H.264 and stereo AAC as HLS, in pieces of
//! [`PIECE`] seconds, into a directory under the cache.
//!
//! The playlist lists every piece of the whole episode up front, so a player
//! seeks it like a file. One ffmpeg runs per conversion and writes pieces in
//! order from wherever it was started. A request for a piece it is about to
//! write waits for it; a request far from it (a seek) starts ffmpeg again at
//! that piece. Running too far ahead of what is asked stops ffmpeg until the
//! player catches up, and so does a player that stops asking, so a paused
//! episode costs nothing. A conversion nobody has asked for in [`IDLE`] is
//! ended and its directory removed.
//!
//! The encoder is the first that works of NVENC, VAAPI and libx264, tried
//! once per launch on a tiny test picture.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anibeam_host_api as wire;
use tokio::process::Child;
use tokio::sync::OnceCell;

use crate::process::command;
use crate::sync::recover;

/// Seconds per piece. Every piece starts on a keyframe forced at its start.
pub const PIECE: f64 = 6.0;
/// A request this many pieces past what ffmpeg has written still waits for it
/// instead of starting over there.
const NEAR: u32 = 4;
/// ffmpeg stops when it is this many pieces past the last one asked for, and
/// goes on once the player is within `RESUME_AT` of it again.
const AHEAD: u32 = 40;
const RESUME_AT: u32 = 20;
/// How long a piece request waits for ffmpeg before giving up.
const WAIT: Duration = Duration::from_secs(40);
/// A conversion nobody asked for in this long is ended and removed.
const IDLE: Duration = Duration::from_secs(10 * 60);
/// At most this many conversions run at once; a new one ends the oldest.
const MAX_JOBS: usize = 2;

/// Which conversion: one file at one height, with one audio track and maybe
/// one subtitle track drawn in.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Key {
    pub file: u64,
    pub height: u32,
    pub audio: Option<usize>,
    pub burn: Option<usize>,
}

impl Key {
    /// The directory name: every part of the key, so two conversions never
    /// share one.
    fn dir_name(&self) -> String {
        let opt = |v: Option<usize>| v.map_or("d".to_string(), |v| v.to_string());
        format!(
            "{}-{}-a{}-s{}",
            self.file,
            self.height,
            opt(self.audio),
            opt(self.burn)
        )
    }
}

/// The nearest height a conversion comes in at or below `asked`.
pub fn height_for(asked: u32) -> u32 {
    wire::CONVERT_HEIGHTS
        .iter()
        .copied()
        .find(|h| *h <= asked)
        .unwrap_or(*wire::CONVERT_HEIGHTS.last().unwrap_or(&480))
}

/// The most bits a second a height may spend, so a converted episode fits a
/// phone's mobile data.
fn max_rate(height: u32) -> &'static str {
    match height {
        h if h >= 1080 => "8M",
        h if h >= 720 => "4M",
        _ => "1500k",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Encoder {
    Nvenc,
    Vaapi,
    X264,
}

const VAAPI_DEVICE: &str = "/dev/dri/renderD128";

impl Encoder {
    fn args(self, height: u32, out: &mut Vec<String>) {
        let rate = max_rate(height).to_string();
        let push = |out: &mut Vec<String>, a: &[&str]| out.extend(a.iter().map(|s| s.to_string()));
        match self {
            Encoder::Nvenc => {
                push(
                    out,
                    &[
                        "-c:v",
                        "h264_nvenc",
                        "-preset",
                        "p4",
                        "-rc",
                        "vbr",
                        "-cq",
                        "23",
                        "-b:v",
                        "0",
                    ],
                );
                push(
                    out,
                    &["-maxrate", &rate, "-bufsize", &rate, "-profile:v", "high"],
                );
            }
            Encoder::Vaapi => {
                push(
                    out,
                    &[
                        "-c:v",
                        "h264_vaapi",
                        "-b:v",
                        &rate,
                        "-maxrate",
                        &rate,
                        "-profile:v",
                        "high",
                    ],
                );
            }
            Encoder::X264 => {
                push(
                    out,
                    &["-c:v", "libx264", "-preset", "veryfast", "-crf", "22"],
                );
                push(
                    out,
                    &["-maxrate", &rate, "-bufsize", &rate, "-profile:v", "high"],
                );
            }
        }
    }

    /// The end of the picture's filter chain: VAAPI wants the frames on the GPU.
    fn upload(self) -> &'static str {
        match self {
            Encoder::Vaapi => ",format=nv12,hwupload",
            _ => ",format=yuv420p",
        }
    }
}

static FOUND: OnceCell<Option<Encoder>> = OnceCell::const_new();

/// The encoder this machine has, or None when ffmpeg is missing or none of
/// them works.
async fn encoder() -> Option<Encoder> {
    *FOUND
        .get_or_init(|| async {
            for enc in [Encoder::Nvenc, Encoder::Vaapi, Encoder::X264] {
                if works(enc).await {
                    tracing::info!("conversions use {enc:?}");
                    return Some(enc);
                }
            }
            None
        })
        .await
}

async fn works(enc: Encoder) -> bool {
    let mut args: Vec<String> = ["-hide_banner", "-loglevel", "error", "-nostdin"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    if enc == Encoder::Vaapi {
        args.extend(["-vaapi_device".into(), VAAPI_DEVICE.into()]);
    }
    args.extend(
        ["-f", "lavfi", "-i", "color=black:s=256x144:d=0.2"]
            .iter()
            .map(|s| s.to_string()),
    );
    args.extend(["-vf".into(), format!("null{}", enc.upload())]);
    enc.args(144, &mut args);
    args.extend(["-f".into(), "null".into(), "-".into()]);
    let run = command("ffmpeg")
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .status();
    matches!(
        tokio::time::timeout(Duration::from_secs(10), run).await,
        Ok(Ok(s)) if s.success()
    )
}

/// Whether this machine can convert at all.
pub async fn available() -> bool {
    encoder().await.is_some()
}

/// Whether this machine can convert, without waiting: None until the encoders
/// have been tried, which this starts in the background.
pub fn known() -> Option<bool> {
    if let Some(found) = FOUND.get() {
        return Some(found.is_some());
    }
    if let Ok(rt) = tokio::runtime::Handle::try_current() {
        rt.spawn(async {
            encoder().await;
        });
    }
    None
}

/// What ffprobe reads of a file, or None without ffprobe.
pub async fn probe(path: &Path) -> Option<wire::Probe> {
    let out = command("ffprobe")
        .args([
            "-v",
            "error",
            "-show_format",
            "-show_streams",
            "-of",
            "json",
        ])
        .arg(path)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .output();
    let out = tokio::time::timeout(Duration::from_secs(20), out)
        .await
        .ok()?
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
    Some(read_probe(&v))
}

fn read_probe(v: &serde_json::Value) -> wire::Probe {
    let num = |v: &serde_json::Value| -> Option<f64> {
        v.as_str()
            .and_then(|s| s.parse().ok())
            .or_else(|| v.as_f64())
    };
    let format = &v["format"];
    let mut p = wire::Probe {
        duration: num(&format["duration"]).unwrap_or(0.0),
        size: num(&format["size"]).unwrap_or(0.0) as u64,
        bitrate: num(&format["bit_rate"]).unwrap_or(0.0) as u64,
        ..Default::default()
    };
    let streams = v["streams"].as_array().cloned().unwrap_or_default();
    for s in &streams {
        let codec = s["codec_name"].as_str().unwrap_or_default().to_string();
        let track = || wire::Track {
            codec: codec.clone(),
            language: s["tags"]["language"].as_str().map(str::to_string),
            title: s["tags"]["title"].as_str().map(str::to_string),
            default: s["disposition"]["default"].as_i64() == Some(1),
        };
        match s["codec_type"].as_str() {
            // The first real picture: a cover image attached as a stream is not it.
            Some("video")
                if p.video_codec.is_none()
                    && s["disposition"]["attached_pic"].as_i64() != Some(1) =>
            {
                p.video_codec = Some(codec.clone());
                p.video_profile = s["profile"].as_str().map(|s| s.to_lowercase());
                p.width = s["width"].as_u64().map(|v| v as u32);
                p.height = s["height"].as_u64().map(|v| v as u32);
                p.bit_depth = s["bits_per_raw_sample"]
                    .as_str()
                    .and_then(|b| b.parse().ok())
                    .or_else(|| {
                        let fmt = s["pix_fmt"].as_str()?;
                        Some(if fmt.contains("10") {
                            10
                        } else if fmt.contains("12") {
                            12
                        } else {
                            8
                        })
                    });
            }
            Some("audio") => p.audio.push(track()),
            Some("subtitle") => p.subtitles.push(track()),
            _ => {}
        }
    }
    p
}

/// The playlist of a whole episode `duration` seconds long, piece by piece.
/// `query` goes on every piece's address, so it asks for the same conversion.
pub fn playlist(duration: f64, query: &str) -> String {
    let count = pieces(duration);
    let mut out = String::from("#EXTM3U\n#EXT-X-VERSION:3\n#EXT-X-PLAYLIST-TYPE:VOD\n");
    out.push_str(&format!(
        "#EXT-X-TARGETDURATION:{}\n#EXT-X-MEDIA-SEQUENCE:0\n#EXT-X-INDEPENDENT-SEGMENTS\n",
        PIECE.ceil() as u32
    ));
    for i in 0..count {
        let len = (duration - f64::from(i) * PIECE).min(PIECE);
        out.push_str(&format!("#EXTINF:{len:.3},\nseg{i}.ts{query}\n"));
    }
    out.push_str("#EXT-X-ENDLIST\n");
    out
}

fn pieces(duration: f64) -> u32 {
    (duration / PIECE).ceil().max(1.0) as u32
}

/// One conversion and the ffmpeg writing it.
struct Job {
    dir: PathBuf,
    /// When a player last asked for a piece.
    used: Instant,
    /// The newest piece a player asked for.
    asked: u32,
    run: Option<Run>,
    /// Counts the runs, so a pacer knows when its run was replaced.
    runs: u64,
}

struct Run {
    child: Child,
    /// The piece this ffmpeg started at.
    from: u32,
    stopped: bool,
    /// How it ended, once it has.
    ended: Option<bool>,
}

impl Run {
    fn signal(&mut self, stop: bool) {
        if self.stopped == stop || self.ended.is_some() {
            return;
        }
        #[cfg(unix)]
        if let Some(pid) = self
            .child
            .id()
            .and_then(|p| rustix::process::Pid::from_raw(p as i32))
        {
            let sig = if stop {
                rustix::process::Signal::STOP
            } else {
                rustix::process::Signal::CONT
            };
            let _ = rustix::process::kill_process(pid, sig);
        }
        self.stopped = stop;
    }

    /// Reaps an ffmpeg that has ended, so none is left a zombie, and says
    /// whether it is still running.
    fn alive(&mut self) -> bool {
        if self.ended.is_none()
            && let Ok(Some(status)) = self.child.try_wait()
        {
            self.ended = Some(status.success());
        }
        self.ended.is_none()
    }

    fn kill(mut self) {
        self.signal(false);
        let _ = self.child.start_kill();
    }
}

/// What a conversion needs to start ffmpeg: the source and its probe.
pub struct Source {
    pub path: PathBuf,
    pub probe: wire::Probe,
}

pub struct Converter {
    root: PathBuf,
    jobs: Mutex<HashMap<Key, Arc<tokio::sync::Mutex<Job>>>>,
    /// Names each job's directory, so a job that is replaced while a request
    /// still holds it never shares, or removes, its successor's.
    made: std::sync::atomic::AtomicU64,
}

impl Converter {
    pub fn new(cache_dir: &Path) -> Converter {
        let root = cache_dir.join("convert");
        // Whatever an earlier launch left is no use to this one.
        let _ = std::fs::remove_dir_all(&root);
        Converter {
            root,
            jobs: Mutex::new(HashMap::new()),
            made: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// The bytes of one piece, converting as needed. None when it cannot be
    /// had: no encoder, ffmpeg failed, or the piece is past the end.
    pub async fn piece(&self, key: &Key, source: &Source, i: u32) -> Option<Vec<u8>> {
        if i >= pieces(source.probe.duration) {
            return None;
        }
        let enc = encoder().await?;
        let job = self.job(key);
        let deadline = Instant::now() + WAIT;
        loop {
            let mut j = job.lock().await;
            j.used = Instant::now();
            j.asked = i;
            let file = j.dir.join(format!("seg{i}.ts"));
            if let Ok(bytes) = tokio::fs::read(&file).await {
                let written = written(&j.dir, j.run.as_ref().map_or(i, |r| r.from)).await;
                pace(&mut j, written);
                return Some(bytes);
            }
            let from = j.run.as_ref().map(|r| r.from);
            let written = written(&j.dir, from.unwrap_or(i)).await;
            let alive = j.run.as_mut().is_some_and(Run::alive);
            let covers = from.is_some_and(|f| f <= i);
            if covers && !alive {
                // This ffmpeg ended without writing the piece: it failed, or
                // the file is shorter than its probe said. Starting it again
                // would only fail again.
                tracing::warn!("conversion of file {} ended before piece {i}", key.file);
                return None;
            }
            if covers && i <= written + NEAR {
                if let Some(r) = j.run.as_mut() {
                    r.signal(false);
                }
            } else {
                if let Some(old) = j.run.take() {
                    old.kill();
                }
                let child = spawn(enc, key, source, &j.dir, i)?;
                j.run = Some(Run {
                    child,
                    from: i,
                    stopped: false,
                    ended: None,
                });
                j.runs += 1;
                tokio::spawn(pacer(Arc::downgrade(&job), j.runs));
            }
            drop(j);
            if Instant::now() > deadline {
                tracing::warn!("conversion of file {} gave no piece {i} in time", key.file);
                return None;
            }
            tokio::time::sleep(Duration::from_millis(150)).await;
        }
    }

    /// The job for `key`, made when there is none. Drops idle jobs, and the
    /// oldest when there are too many, on the way.
    fn job(&self, key: &Key) -> Arc<tokio::sync::Mutex<Job>> {
        let mut jobs = recover(&self.jobs);
        jobs.retain(|_, job| match job.try_lock() {
            Ok(j) => j.used.elapsed() < IDLE,
            Err(_) => true,
        });
        if let Some(job) = jobs.get(key) {
            return job.clone();
        }
        while jobs.len() >= MAX_JOBS {
            let oldest = jobs
                .iter()
                .filter_map(|(k, j)| j.try_lock().ok().map(|j| (k.clone(), j.used)))
                .min_by_key(|(_, used)| *used)
                .map(|(k, _)| k);
            match oldest {
                Some(k) => {
                    jobs.remove(&k);
                }
                None => break,
            }
        }
        let n = self.made.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = self.root.join(format!("{}-{n}", key.dir_name()));
        let _ = std::fs::create_dir_all(&dir);
        let job = Arc::new(tokio::sync::Mutex::new(Job {
            dir,
            used: Instant::now(),
            asked: 0,
            run: None,
            runs: 0,
        }));
        jobs.insert(key.clone(), job.clone());
        job
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        if let Some(r) = self.run.take() {
            r.kill();
        }
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// A player that stopped asking for this long has paused or gone; its ffmpeg
/// stops until it asks again.
const QUIET: Duration = Duration::from_secs(20);
/// How often a pacer looks.
const PACE_EVERY: Duration = Duration::from_secs(1);

/// Watches one run for as long as it is the job's run: stops it far ahead of
/// the player or once the player goes quiet, lets it go on when the player
/// comes near, reaps it when it ends, and ends it once the job is idle. It
/// runs on a clock, since a paused player asks for nothing.
async fn pacer(job: std::sync::Weak<tokio::sync::Mutex<Job>>, run: u64) {
    loop {
        tokio::time::sleep(PACE_EVERY).await;
        let Some(job) = job.upgrade() else { return };
        let mut j = job.lock().await;
        if j.runs != run {
            return;
        }
        let Some(r) = j.run.as_mut() else { return };
        if !r.alive() {
            return;
        }
        let from = r.from;
        if j.used.elapsed() >= IDLE {
            if let Some(r) = j.run.take() {
                r.kill();
            }
            return;
        }
        let written = written(&j.dir, from).await;
        pace(&mut j, written);
    }
}

/// Stops ffmpeg far ahead of the player or once it goes quiet, and lets it
/// go on when the player comes near. `written` is the last piece on disk.
fn pace(j: &mut Job, written: u32) {
    let (asked, quiet) = (j.asked, j.used.elapsed() >= QUIET);
    let Some(r) = j.run.as_mut() else { return };
    let ahead = written.saturating_sub(asked);
    if quiet || ahead >= AHEAD {
        r.signal(true);
    } else if ahead <= RESUME_AT {
        r.signal(false);
    }
}

/// The last piece on disk, counting up from `from`.
async fn written(dir: &Path, from: u32) -> u32 {
    let mut n = from;
    while tokio::fs::try_exists(dir.join(format!("seg{}.ts", n + 1)))
        .await
        .unwrap_or(false)
    {
        n += 1;
    }
    n
}

/// One text subtitle track of the file as ASS, read out whole by ffmpeg.
pub async fn subtitle(path: &Path, index: usize) -> Option<Vec<u8>> {
    let out = command("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-nostdin", "-i"])
        .arg(path)
        .args([
            "-map",
            &format!("0:s:{index}"),
            "-c:s",
            "ass",
            "-f",
            "ass",
            "-",
        ])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .output();
    let out = tokio::time::timeout(Duration::from_secs(60), out)
        .await
        .ok()?
        .ok()?;
    (out.status.success() && !out.stdout.is_empty()).then_some(out.stdout)
}

/// Bitmap subtitles are laid over the picture; text ones are drawn by libass.
pub fn is_bitmap(codec: &str) -> bool {
    matches!(
        codec,
        "hdmv_pgs_subtitle" | "dvd_subtitle" | "dvb_subtitle" | "xsub"
    )
}

fn spawn(enc: Encoder, key: &Key, source: &Source, dir: &Path, from: u32) -> Option<Child> {
    let args = ffmpeg_args(enc, key, source, dir, from);
    // libass reads the file again by name for the subtitle track, so the
    // file is linked into the directory under a name that needs no escaping.
    if key.burn.is_some() {
        let link = dir.join(input_name(&source.path));
        let _ = std::fs::remove_file(&link);
        #[cfg(unix)]
        let _ = std::os::unix::fs::symlink(&source.path, &link);
        #[cfg(not(unix))]
        let _ = std::fs::hard_link(&source.path, &link);
    }
    match command("ffmpeg")
        .args(&args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
    {
        Ok(child) => Some(child),
        Err(e) => {
            tracing::warn!("ffmpeg did not start: {e}");
            None
        }
    }
}

/// `in.mkv`, keeping the extension so ffmpeg reads the container right.
fn input_name(path: &Path) -> String {
    match path.extension() {
        Some(ext) => format!("in.{}", ext.to_string_lossy()),
        None => "in".to_string(),
    }
}

fn ffmpeg_args(enc: Encoder, key: &Key, source: &Source, dir: &Path, from: u32) -> Vec<String> {
    let start = f64::from(from) * PIECE;
    let height = height_for(key.height).min(source.probe.height.unwrap_or(u32::MAX).max(144));
    let mut a: Vec<String> = Vec::new();
    let mut push = |v: &[&str]| a.extend(v.iter().map(|s| s.to_string()));
    push(&["-hide_banner", "-loglevel", "error", "-nostdin", "-y"]);
    if enc == Encoder::Vaapi {
        push(&["-vaapi_device", VAAPI_DEVICE]);
    }
    // Each run counts from zero at its first piece and is moved to that
    // piece's place in the episode on the way out, so a piece made by a later
    // ffmpeg lines up with one made by an earlier, whatever time the file's
    // own clock starts at.
    let input = if key.burn.is_some() {
        input_name(&source.path)
    } else {
        source.path.to_string_lossy().into_owned()
    };
    push(&["-ss", &format!("{start:.3}"), "-i", &input]);
    let scale = format!("scale=-2:{height}:flags=bicubic{}", enc.upload());
    let bitmap = key
        .burn
        .and_then(|b| source.probe.subtitles.get(b))
        .is_some_and(|t| is_bitmap(&t.codec));
    match key.burn {
        Some(b) if bitmap => {
            push(&[
                "-filter_complex",
                &format!("[0:v:0][0:s:{b}]overlay,{scale}[v]"),
                "-map",
                "[v]",
            ]);
        }
        Some(b) => {
            push(&[
                "-map",
                "0:v:0",
                "-vf",
                // libass reads the subtitles on the file's clock, so the
                // picture is moved onto it for the drawing and back after.
                &format!(
                    "setpts=PTS+{start:.3}/TB,subtitles={}:si={b},setpts=PTS-{start:.3}/TB,{scale}",
                    input_name(&source.path)
                ),
            ]);
        }
        None => push(&["-map", "0:v:0", "-vf", &scale]),
    }
    let audio = key.audio.unwrap_or_else(|| {
        source
            .probe
            .audio
            .iter()
            .position(|t| t.default)
            .unwrap_or(0)
    });
    if !source.probe.audio.is_empty() {
        push(&[
            "-map",
            &format!("0:a:{audio}"),
            "-c:a",
            "aac",
            "-ac",
            "2",
            "-b:a",
            "160k",
        ]);
    }
    push(&["-sn", "-dn"]);
    let mut enc_args = Vec::new();
    enc.args(height, &mut enc_args);
    a.extend(enc_args);
    // A keyframe at every piece's start: this run starts at one.
    let gop = format!("expr:gte(t,n_forced*{PIECE})");
    let mut push = |v: &[&str]| a.extend(v.iter().map(|s| s.to_string()));
    push(&["-force_key_frames", &gop, "-sc_threshold", "0"]);
    push(&[
        "-f",
        "hls",
        "-hls_time",
        &format!("{PIECE}"),
        "-hls_list_size",
        "0",
        "-hls_playlist_type",
        "vod",
        "-hls_flags",
        "temp_file+independent_segments",
        "-start_number",
        &from.to_string(),
        "-output_ts_offset",
        &format!("{start:.3}"),
        "-hls_segment_filename",
        &dir.join("seg%d.ts").to_string_lossy(),
        &dir.join("ffmpeg.m3u8").to_string_lossy(),
    ]);
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heights_round_down_to_one_a_conversion_comes_in() {
        assert_eq!(height_for(2160), 1080);
        assert_eq!(height_for(1080), 1080);
        assert_eq!(height_for(900), 720);
        assert_eq!(height_for(100), 480);
    }

    #[test]
    fn the_playlist_covers_the_whole_episode() {
        let list = playlist(14.5, "?audio=1");
        assert!(list.contains("#EXT-X-PLAYLIST-TYPE:VOD"));
        assert!(list.contains("#EXTINF:6.000,\nseg0.ts?audio=1\n"));
        assert!(list.contains("#EXTINF:2.500,\nseg2.ts?audio=1\n"));
        assert!(!list.contains("seg3.ts"));
        assert!(list.ends_with("#EXT-X-ENDLIST\n"));
    }

    #[test]
    fn a_probe_reads_the_picture_and_every_track() {
        let v = serde_json::json!({
            "format": {"duration": "1420.5", "size": "734003200", "bit_rate": "4133000"},
            "streams": [
                {"codec_type": "video", "codec_name": "h264", "profile": "High 10",
                 "width": 1920, "height": 1080, "pix_fmt": "yuv420p10le"},
                {"codec_type": "audio", "codec_name": "flac", "tags": {"language": "jpn"},
                 "disposition": {"default": 1}},
                {"codec_type": "subtitle", "codec_name": "ass", "tags": {"language": "eng", "title": "Full"}},
                {"codec_type": "video", "codec_name": "mjpeg", "disposition": {"attached_pic": 1}}
            ]
        });
        let p = read_probe(&v);
        assert_eq!(p.duration, 1420.5);
        assert_eq!(p.video_codec.as_deref(), Some("h264"));
        assert_eq!(p.video_profile.as_deref(), Some("high 10"));
        assert_eq!(p.bit_depth, Some(10));
        assert_eq!(p.height, Some(1080));
        assert!(p.audio[0].default);
        assert_eq!(p.subtitles[0].title.as_deref(), Some("Full"));
    }

    #[test]
    fn a_burned_subtitle_reads_the_linked_file_and_the_pieces_line_up() {
        let source = Source {
            path: PathBuf::from("/lib/[Group] Show - 01 (1080p).mkv"),
            probe: wire::Probe {
                duration: 1400.0,
                height: Some(1080),
                audio: vec![
                    wire::Track::default(),
                    wire::Track {
                        default: true,
                        ..Default::default()
                    },
                ],
                subtitles: vec![wire::Track {
                    codec: "ass".into(),
                    ..Default::default()
                }],
                ..Default::default()
            },
        };
        let key = Key {
            file: 1,
            height: 720,
            audio: None,
            burn: Some(0),
        };
        let a = ffmpeg_args(Encoder::X264, &key, &source, Path::new("/c/x"), 10);
        let joined = a.join(" ");
        assert!(joined.contains("-ss 60.000 -i in.mkv"));
        assert!(joined.contains(
            "setpts=PTS+60.000/TB,subtitles=in.mkv:si=0,setpts=PTS-60.000/TB,scale=-2:720"
        ));
        assert!(joined.contains("-map 0:a:1"));
        assert!(joined.contains("expr:gte(t,n_forced*6)"));
        assert!(joined.contains("-output_ts_offset 60.000"));
        assert!(joined.contains("-start_number 10"));
    }
}
