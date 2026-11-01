//! Streaming from the tailnet host end to end: a real episode file (made by
//! ffmpeg, so the test is skipped on a machine without it) is served with
//! ranges, probed, converted to HLS from its start and from a seek, and
//! played through the playback calls, which keep its resume point here.

mod common;
mod fixtures;

use std::time::{Duration, Instant};

use anibeam_core::*;
use anibeam_host_api as wire;

fn has(tool: &str) -> bool {
    std::process::Command::new(tool)
        .arg("-version")
        .output()
        .is_ok_and(|o| o.status.success())
}

/// Whether this machine's ffmpeg has the filter `name`.
fn has_filter(name: &str) -> bool {
    std::process::Command::new("ffmpeg")
        .args(["-hide_banner", "-filters"])
        .output()
        .is_ok_and(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .any(|l| l.split_whitespace().nth(1) == Some(name))
        })
}

fn block_on<T>(f: impl std::future::Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

/// 30 seconds of 10-bit H.264 with FLAC and a styled subtitle: the kind of
/// file a TV refuses.
fn make_episode(path: &std::path::Path) {
    let ass = path.with_extension("ass");
    std::fs::write(
        &ass,
        "[Script Info]\nScriptType: v4.00+\nPlayResX: 640\nPlayResY: 360\n\n[V4+ Styles]\n\
         Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n\
         Style: Default,Sans,40,&H0000FFFF,&H000000FF,&H00000000,&H00000000,1,0,0,0,100,100,0,0,1,2,0,2,10,10,20,1\n\n\
         [Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n\
         Dialogue: 0,0:00:00.00,0:00:30.00,Default,,0,0,0,,Styled line\n",
    )
    .unwrap();
    let ok = std::process::Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-y"])
        .args(["-f", "lavfi", "-i", "testsrc2=s=640x360:r=24:d=30"])
        .args(["-f", "lavfi", "-i", "sine=f=440:d=30"])
        .arg("-i")
        .arg(&ass)
        .args(["-map", "0", "-map", "1", "-map", "2"])
        .args([
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-pix_fmt",
            "yuv420p10le",
        ])
        .args(["-c:a", "flac", "-c:s", "ass"])
        .arg(path)
        .status()
        .unwrap()
        .success();
    assert!(ok, "ffmpeg made no episode");
}

#[test]
fn an_episode_streams_converts_and_keeps_its_place() {
    if !has("ffmpeg") || !has("ffprobe") {
        eprintln!("skipped: no ffmpeg here");
        return;
    }
    let (dir, core, _c) = common::open_core();
    let lib = dir.path().join("lib/Show");
    std::fs::create_dir_all(&lib).unwrap();
    let episode = lib.join("[Group] Show - 01 (360p).mkv");
    make_episode(&episode);
    let src = fixtures::insert_source(&core, dir.path().join("lib").to_str().unwrap());
    let series =
        fixtures::insert_series(&core, src, SeriesKind::Show, lib.to_str().unwrap(), "Show");
    let file = fixtures::insert_file(
        &core,
        series,
        episode.to_str().unwrap(),
        1.0,
        None,
        "episode",
        1,
    );

    core.set_host_bind(Some("127.0.0.1:0".to_string()));
    let mut preferences = match core.call(Call::GetPreferences).unwrap() {
        Reply::Preferences { preferences } => preferences,
        other => panic!("{other:?}"),
    };
    preferences.share_over_tailscale = true;
    core.call(Call::SetPreferences { preferences }).unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    let base = loop {
        if let Reply::Host { status } = core.call(Call::GetHost).unwrap()
            && let Some(url) = status.urls.first()
        {
            break url.trim_end_matches('/').to_string();
        }
        assert!(Instant::now() < deadline, "the host did not start");
        std::thread::sleep(Duration::from_millis(20));
    };
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .unwrap();

    block_on(async {
        // Hello answers at once; whether this machine converts shows once its
        // encoders have been tried, a moment after the host starts.
        let tried = Instant::now() + Duration::from_secs(30);
        let hello = loop {
            let hello: wire::Hello = http
                .get(format!("{base}{}", wire::HELLO))
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            if hello.features.iter().any(|f| f == wire::FEATURE_CONVERT) || Instant::now() > tried {
                break hello;
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        };
        assert!(hello.features.iter().any(|f| f == wire::FEATURE_STREAM));
        let snapshot: wire::Snapshot = http
            .get(format!("{base}{}", wire::SNAPSHOT))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(snapshot.series[0].episodes[0].file, Some(file));

        // The file seeks by byte.
        let len = std::fs::metadata(&episode).unwrap().len();
        let part = http
            .get(format!("{base}{}", wire::file_path(file)))
            .header("Range", "bytes=10-19")
            .send()
            .await
            .unwrap();
        assert_eq!(part.status(), 206);
        assert_eq!(
            part.headers()["content-range"].to_str().unwrap(),
            format!("bytes 10-19/{len}")
        );
        assert_eq!(part.bytes().await.unwrap().len(), 10);
        let none = http
            .get(format!("{base}{}", wire::file_path(9999)))
            .send()
            .await
            .unwrap();
        assert_eq!(none.status(), 404);

        let probe: wire::Probe = http
            .get(format!("{base}{}", wire::probe_path(file)))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert!((probe.duration - 30.0).abs() < 1.0, "{probe:?}");
        assert_eq!(probe.bit_depth, Some(10));
        assert_eq!(probe.audio[0].codec, "flac");
        assert_eq!(probe.subtitles[0].codec, "ass");
        // The subtitle track comes out on its own, for a client playing a conversion.
        let subs = http
            .get(format!("{base}{}", wire::subtitle_path(file, 0)))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        assert!(subs.contains("Styled line"), "{subs}");

        if hello.features.iter().any(|f| f == wire::FEATURE_CONVERT) {
            // Drawing subtitles in needs ffmpeg built with libass, which not
            // every build is.
            let burn = has_filter("subtitles").then_some(0);
            let query = burn.map_or(String::new(), |b| format!("?burn={b}"));
            let list_url = format!("{base}{}", wire::hls_path(file, 720, None, burn));
            let list = http
                .get(&list_url)
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap();
            assert!(list.contains(&format!("seg4.ts{query}")), "{list}");
            let dir_url = list_url.rsplit_once('/').unwrap().0.to_string();
            // From the start, then a seek past what the first ffmpeg wrote.
            for i in [0, 4] {
                let piece = http
                    .get(format!("{dir_url}/seg{i}.ts{query}"))
                    .send()
                    .await
                    .unwrap();
                assert_eq!(piece.status(), 200, "piece {i}");
                let bytes = piece.bytes().await.unwrap();
                let at = dir.path().join(format!("seg{i}.ts"));
                std::fs::write(&at, &bytes).unwrap();
                let out = std::process::Command::new("ffprobe")
                    .args(["-v", "error", "-select_streams", "v:0", "-show_entries"])
                    .args([
                        "stream=codec_name,pix_fmt,height:packet=pts_time",
                        "-of",
                        "csv=p=0",
                    ])
                    .arg(&at)
                    .output()
                    .unwrap();
                let text = String::from_utf8_lossy(&out.stdout);
                let stream = text
                    .lines()
                    .find(|l| l.starts_with("h264"))
                    .unwrap_or_default();
                assert!(
                    stream.contains("yuv420p") && stream.contains("360"),
                    "piece {i}: {stream}"
                );
                // The piece's own time is where it sits in the episode.
                let first: f64 = text
                    .lines()
                    .filter_map(|l| l.trim_end_matches(',').parse::<f64>().ok())
                    .fold(f64::MAX, f64::min);
                let want = f64::from(i) * 6.0;
                assert!((first - want).abs() < 2.5, "piece {i} starts at {first}");
            }
        }

        // The playback calls run here, and only they do.
        let call = |c: Call| {
            let http = http.clone();
            let url = format!("{base}{}", wire::CALL);
            async move { http.post(url).json(&c).send().await.unwrap() }
        };
        let opened = call(Call::OpenPlayback { file }).await;
        assert_eq!(opened.status(), 200);
        let session = match opened.json::<Reply>().await.unwrap() {
            Reply::Playback { session } => session.session,
            other => panic!("{other:?}"),
        };
        for position in [5.0, 12.0] {
            let r = call(Call::Tick {
                session,
                position,
                paused: false,
            })
            .await;
            assert_eq!(r.status(), 200);
        }
        let closed = call(Call::ClosePlayback {
            session,
            position: 12.0,
            reason: CloseReason::Stopped,
        })
        .await;
        assert_eq!(closed.status(), 200);
        let forbidden = call(Call::ForgetSeries { series }).await;
        assert_eq!(forbidden.status(), 403);
        // A call that is not sent as JSON is refused, so no web page can send one.
        let plain = http
            .post(format!("{base}{}", wire::CALL))
            .header("content-type", "text/plain")
            .body(
                serde_json::to_vec(&Call::Tick {
                    session,
                    position: 1.0,
                    paused: false,
                })
                .unwrap(),
            )
            .send()
            .await
            .unwrap();
        assert_eq!(plain.status(), 415);
        let wrong_track = http
            .get(format!(
                "{base}{}",
                wire::hls_path(file, 480, Some(7), None)
            ))
            .send()
            .await
            .unwrap();
        assert_eq!(wrong_track.status(), 400);
    });

    // The place is kept on this machine, as if it had played here.
    match core.call(Call::OpenPlayback { file }).unwrap() {
        Reply::Playback { session } => assert!(
            session.resume_from.is_some_and(|r| r > 4.0),
            "{:?}",
            session.resume_from
        ),
        other => panic!("{other:?}"),
    }
}
