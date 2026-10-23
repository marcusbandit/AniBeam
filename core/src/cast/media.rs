//! One episode made ready for a receiver: what it holds, whether the device can play it, and
//! the pieces a server hands out. An MP4 goes as it is. A Matroska file is read once for its
//! keyframes and subtitles, then cut on keyframes into fragmented MP4 segments as they are
//! asked for, so a receiver can seek anywhere without anything being re-encoded.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use super::fmp4;
use super::mkv::{self, AudioCodec, AudioTrack, Mkv, Probe, Scan, SubCodec, VideoCodec};
use super::vtt::{self, Cue};

/// What kind of receiver the episode is going to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Receiver {
    Cast,
    Dlna,
}

/// Segments aim for this length; each still starts on a keyframe.
const SEGMENT_MS: u64 = 6000;

#[derive(Clone, Debug, PartialEq)]
pub struct Segment {
    pub start: u64,
    pub end: u64,
}

/// Cuts at keyframes, each segment at least `SEGMENT_MS` long but for the last.
pub fn plan(keyframes: &[u64], end_ms: u64) -> Vec<Segment> {
    let mut out: Vec<Segment> = Vec::new();
    let mut start = match keyframes.first() {
        Some(k) => *k,
        None => return out,
    };
    for &k in &keyframes[1..] {
        if k - start >= SEGMENT_MS {
            out.push(Segment { start, end: k });
            start = k;
        }
    }
    out.push(Segment {
        start,
        end: end_ms.max(start + 1),
    });
    out
}

/// Why a file cannot go to a receiver, in words for the page. HEVC is let through: which
/// Chromecasts decode it cannot be told from what they announce, so the device decides, and
/// a refusal names HEVC as the likely reason.
pub fn verdict(
    probe: &Probe,
    audio: Option<&AudioTrack>,
    _receiver: Receiver,
) -> Result<(), String> {
    let video = probe.video.as_ref().ok_or("This file has no video.")?;
    match &video.codec {
        VideoCodec::H264 { profile, .. } if *profile >= 110 => {
            return Err("This episode is 10-bit H.264 (Hi10P), which no TV or Chromecast decodes. Casting it needs a transcode, which is not built yet.".into());
        }
        VideoCodec::Other(id) => {
            return Err(format!("This episode's video ({id}) cannot be cast."));
        }
        _ => {}
    }
    if let Some(a) = audio
        && let AudioCodec::Other(id) = &a.codec
    {
        return Err(format!("This episode's audio ({id}) cannot be cast."));
    }
    Ok(())
}

/// A subtitle choice for the receiver: a track in the file, or a file beside it.
#[derive(Clone, Debug, PartialEq)]
pub enum Subs {
    Track(u64),
    File(PathBuf),
}

/// An episode being served: its probe and scan, the init segment, the segment plan, and
/// one open reader that segments are cut with.
pub struct Prepared {
    pub path: PathBuf,
    pub probe: Probe,
    pub scan: Scan,
    pub audio: Option<AudioTrack>,
    pub init: Vec<u8>,
    pub segments: Vec<Segment>,
    pub cues: Option<Vec<Cue>>,
    reader: Mutex<Mkv>,
}

impl Prepared {
    /// Reads the file through once and makes everything a receiver asks for first.
    pub fn new(
        path: &Path,
        audio_track: Option<u64>,
        subs: Option<Subs>,
        receiver: Receiver,
    ) -> Result<Prepared, String> {
        let mut reader = mkv::open(path)?;
        let probe = mkv::probe(&reader);
        let audio = probe.audio_track(audio_track).cloned();
        verdict(&probe, audio.as_ref(), receiver)?;
        let video = probe.video.clone().ok_or("This file has no video.")?;
        let first = match &audio {
            Some(a) if matches!(a.codec, AudioCodec::Ac3 | AudioCodec::Eac3) => {
                mkv::first_frame(&mut reader, &probe, a.number)
            }
            _ => None,
        };
        let init = fmp4::init(&video, audio.as_ref(), first.as_deref())?;
        reader.seek_ms(&probe, 0)?;
        let scan = mkv::scan(&mut reader, &probe)?;
        let end = probe.duration_ms.max(scan.end_ms);
        let segments = plan(&scan.keyframes, end);
        let cues = match subs {
            Some(Subs::Track(n)) => scan
                .subs
                .iter()
                .find(|(t, _)| *t == n)
                .map(|(_, c)| c.clone()),
            Some(Subs::File(p)) => std::fs::read(&p)
                .ok()
                .map(|b| vtt::parse_file(&String::from_utf8_lossy(&b))),
            None => None,
        };
        Ok(Prepared {
            path: path.to_path_buf(),
            probe,
            scan,
            audio,
            init,
            segments,
            cues,
            reader: Mutex::new(reader),
        })
    }

    pub fn duration_ms(&self) -> u64 {
        self.segments.last().map(|s| s.end).unwrap_or(0)
    }

    /// The decode-order shift every timestamp carries; positions read back subtract it.
    pub fn offset_ms(&self) -> u64 {
        self.scan.reorder_ms
    }

    /// Segment `i` as a movie fragment.
    pub fn segment(&self, i: usize) -> Result<Vec<u8>, String> {
        let s = self.segments.get(i).ok_or("no such segment")?;
        let last = i + 1 == self.segments.len();
        let mut reader = self.reader.lock().unwrap_or_else(|e| e.into_inner());
        let (v, a) = mkv::segment(
            &mut reader,
            &self.probe,
            &self.scan,
            self.audio.as_ref(),
            s.start,
            (!last).then_some(s.end),
        )?;
        Ok(fmp4::fragment(i as u32 + 1, &v, &a))
    }

    /// Where a progressive stream asked for at `secs` really starts: its segment's keyframe.
    pub fn stream_start(&self, secs: f64) -> f64 {
        self.segments
            .get(self.segment_at((secs * 1000.0) as u64))
            .map(|s| s.start as f64 / 1000.0)
            .unwrap_or(0.0)
    }

    /// The index of the segment holding `ms`.
    pub fn segment_at(&self, ms: u64) -> usize {
        self.segments
            .iter()
            .position(|s| ms < s.end)
            .unwrap_or(self.segments.len().saturating_sub(1))
    }

    /// The HLS master playlist: the media playlist, and the subtitles when there are any.
    pub fn master(&self) -> String {
        let video = self
            .probe
            .video
            .as_ref()
            .map(fmp4::video_codec_string)
            .unwrap_or_default();
        let codecs = match &self.audio {
            Some(a) => format!("{video},{}", fmp4::audio_codec_string(a)),
            None => video,
        };
        let v = self.probe.video.as_ref();
        let (w, h) = v.map(|v| (v.width, v.height)).unwrap_or((0, 0));
        let mut out = String::from("#EXTM3U\n#EXT-X-VERSION:7\n#EXT-X-INDEPENDENT-SEGMENTS\n");
        let subs = if self.cues.is_some() {
            out.push_str("#EXT-X-MEDIA:TYPE=SUBTITLES,GROUP-ID=\"subs\",NAME=\"Subtitles\",DEFAULT=YES,AUTOSELECT=YES,FORCED=NO,LANGUAGE=\"und\",URI=\"subs.m3u8\"\n");
            ",SUBTITLES=\"subs\""
        } else {
            ""
        };
        out.push_str(&format!(
            "#EXT-X-STREAM-INF:BANDWIDTH={},RESOLUTION={w}x{h},CODECS=\"{codecs}\"{subs}\nmedia.m3u8\n",
            self.bandwidth()
        ));
        out
    }

    fn bandwidth(&self) -> u64 {
        let bytes = std::fs::metadata(&self.path).map(|m| m.len()).unwrap_or(0);
        let secs = (self.duration_ms() / 1000).max(1);
        (bytes * 8 / secs).max(500_000)
    }

    pub fn media_playlist(&self) -> String {
        let target = self
            .segments
            .iter()
            .map(|s| (s.end - s.start).div_ceil(1000))
            .max()
            .unwrap_or(6);
        let mut out = format!(
            "#EXTM3U\n#EXT-X-VERSION:7\n#EXT-X-TARGETDURATION:{target}\n#EXT-X-MEDIA-SEQUENCE:0\n#EXT-X-PLAYLIST-TYPE:VOD\n#EXT-X-INDEPENDENT-SEGMENTS\n#EXT-X-MAP:URI=\"init.mp4\"\n"
        );
        for (i, s) in self.segments.iter().enumerate() {
            out.push_str(&format!(
                "#EXTINF:{:.3},\nseg/{i}.m4s\n",
                (s.end - s.start) as f64 / 1000.0
            ));
        }
        out.push_str("#EXT-X-ENDLIST\n");
        out
    }

    pub fn subs_playlist(&self) -> String {
        let secs = self.duration_ms().div_ceil(1000);
        format!(
            "#EXTM3U\n#EXT-X-VERSION:3\n#EXT-X-TARGETDURATION:{secs}\n#EXT-X-MEDIA-SEQUENCE:0\n#EXT-X-PLAYLIST-TYPE:VOD\n#EXTINF:{secs}.000,\nsubs.vtt\n#EXT-X-ENDLIST\n"
        )
    }

    /// The subtitles as WebVTT, shifted with the media.
    pub fn vtt(&self) -> String {
        let shift = self.scan.reorder_ms;
        let cues: Vec<Cue> = self
            .cues
            .iter()
            .flatten()
            .map(|c| Cue {
                start: c.start + shift,
                end: c.end + shift,
                text: c.text.clone(),
            })
            .collect();
        vtt::render(&cues)
    }

    /// The subtitles as SRT, which the DLNA TVs that show subtitles at all read.
    pub fn srt(&self) -> String {
        vtt::render_srt(self.cues.as_deref().unwrap_or_default())
    }

    /// Which text tracks the file has, for the page's picker.
    pub fn text_tracks(probe: &Probe) -> Vec<(u64, String)> {
        probe
            .subs
            .iter()
            .filter(|s| !matches!(s.codec, SubCodec::Other(_)))
            .map(|s| {
                (
                    s.number,
                    s.name
                        .clone()
                        .or(s.language.clone())
                        .unwrap_or_else(|| format!("Track {}", s.number)),
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segments_start_on_keyframes_and_run_six_seconds_or_more() {
        let k = [0, 2000, 4000, 6000, 8000, 13000, 20000];
        assert_eq!(
            plan(&k, 24000),
            vec![
                Segment {
                    start: 0,
                    end: 6000
                },
                Segment {
                    start: 6000,
                    end: 13000
                },
                Segment {
                    start: 13000,
                    end: 20000
                },
                Segment {
                    start: 20000,
                    end: 24000
                },
            ]
        );
        assert!(plan(&[], 1000).is_empty());
    }
}
