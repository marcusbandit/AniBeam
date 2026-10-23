//! What a Matroska file holds, read for casting: the tracks a receiver needs to know about,
//! one pass over the file for the keyframes and the subtitles, and the frames of one stretch
//! of time for a segment. Nothing is decoded; frames are copied as they are.

use std::io::Read;
use std::path::Path;
use std::sync::{Arc, Mutex};

use matroska_demuxer::{
    ContentCompAlgo, ContentEncodingValue, Frame, MatroskaFile, TrackEntry, TrackType,
};

use super::vtt::Cue;

#[derive(Clone, Debug, PartialEq)]
pub enum VideoCodec {
    /// `profile` is the avcC profile byte: 100 High, 110 High 10.
    H264 {
        profile: u8,
        level: u8,
        compat: u8,
    },
    /// `profile` is the general_profile_idc: 1 Main, 2 Main 10.
    Hevc {
        profile: u8,
    },
    Other(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum AudioCodec {
    Aac,
    Ac3,
    Eac3,
    Mp3,
    Opus,
    Flac,
    Other(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum SubCodec {
    Ass,
    Srt,
    /// Pictures (PGS, VobSub): nothing to turn into text.
    Other(String),
}

#[derive(Clone, Debug)]
pub struct VideoTrack {
    pub number: u64,
    pub codec: VideoCodec,
    pub width: u32,
    pub height: u32,
    /// avcC or hvcC, as the file stores it.
    pub config: Vec<u8>,
    /// The frame duration in ms, when the file says.
    pub frame_ms: Option<f64>,
    pub strip: Option<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub struct AudioTrack {
    pub number: u64,
    pub codec: AudioCodec,
    pub channels: u32,
    pub rate: u32,
    /// AudioSpecificConfig for AAC, OpusHead, the fLaC header and blocks.
    pub config: Vec<u8>,
    pub language: Option<String>,
    pub name: Option<String>,
    pub strip: Option<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub struct SubTrack {
    pub number: u64,
    pub codec: SubCodec,
    pub language: Option<String>,
    pub name: Option<String>,
    pub strip: Option<Vec<u8>>,
    pub zlib: bool,
}

#[derive(Clone, Debug)]
pub struct Probe {
    pub duration_ms: u64,
    /// Nanoseconds per timestamp unit; 1 ms in nearly every file.
    pub scale_ns: u64,
    pub video: Option<VideoTrack>,
    pub audio: Vec<AudioTrack>,
    pub subs: Vec<SubTrack>,
    /// Tracks whose frames are zlib compressed, which the copy has to undo.
    pub zlib: Vec<u64>,
}

impl Probe {
    pub fn audio_track(&self, number: Option<u64>) -> Option<&AudioTrack> {
        match number {
            Some(n) => self
                .audio
                .iter()
                .find(|a| a.number == n)
                .or(self.audio.first()),
            None => self.audio.first(),
        }
    }

    fn to_ms(&self, ts: u64) -> u64 {
        ((ts as u128 * self.scale_ns as u128) / 1_000_000) as u64
    }

    fn units(&self, ms: u64) -> u64 {
        ((ms as u128 * 1_000_000) / self.scale_ns.max(1) as u128) as u64
    }
}

/// The demuxer over a reader that can be sent to any cluster, with the cluster index.
pub struct Mkv {
    pub file: MatroskaFile<super::ebml::Jump>,
    jump: Arc<Mutex<Option<u64>>>,
    clusters: Vec<(u64, u64)>,
}

impl Mkv {
    /// Puts the reader at the last cluster starting at or before `ms`; frames before `ms`
    /// still come out first and are the caller's to skip.
    pub fn seek_ms(&mut self, probe: &Probe, ms: u64) -> Result<(), String> {
        let at = super::ebml::cluster_before(&self.clusters, probe.units(ms))
            .ok_or("the file has no clusters")?;
        *self.jump.lock().unwrap_or_else(|e| e.into_inner()) = Some(at);
        Ok(())
    }

    pub fn tracks(&self) -> &[TrackEntry] {
        self.file.tracks()
    }

    pub fn next_frame(&mut self, frame: &mut Frame) -> Result<bool, matroska_demuxer::DemuxError> {
        self.file.next_frame(frame)
    }
}

pub fn open(path: &Path) -> Result<Mkv, String> {
    let (reader, jump) =
        super::ebml::Jump::open(path).map_err(|e| format!("could not open the file: {e}"))?;
    let file = MatroskaFile::open(reader).map_err(|e| format!("not a Matroska file: {e}"))?;
    let clusters =
        super::ebml::clusters(path).map_err(|e| format!("could not index the file: {e}"))?;
    Ok(Mkv {
        file,
        jump,
        clusters,
    })
}

/// What each track needs undone before its frames are copied.
fn encodings(t: &TrackEntry) -> (Option<Vec<u8>>, bool) {
    let mut strip = None;
    let mut zlib = false;
    for e in t.content_encodings().unwrap_or_default() {
        if let ContentEncodingValue::Compression(c) = e.encoding() {
            match c.algo() {
                ContentCompAlgo::Stripping => strip = c.settings().map(<[u8]>::to_vec),
                ContentCompAlgo::Zlib => zlib = true,
                _ => {}
            }
        }
    }
    (strip, zlib)
}

pub fn probe(mkv: &Mkv) -> Probe {
    let info = mkv.file.info();
    let scale_ns = info.timestamp_scale().get();
    let duration_ms = info
        .duration()
        .map(|d| (d * scale_ns as f64 / 1e6) as u64)
        .unwrap_or(0);
    let mut probe = Probe {
        duration_ms,
        scale_ns,
        video: None,
        audio: Vec::new(),
        subs: Vec::new(),
        zlib: Vec::new(),
    };
    for t in mkv.tracks() {
        let number = t.track_number().get();
        let private = t.codec_private().map(<[u8]>::to_vec).unwrap_or_default();
        let (strip, zlib) = encodings(t);
        if zlib {
            probe.zlib.push(number);
        }
        match t.track_type() {
            TrackType::Video if probe.video.is_none() => {
                let codec = match t.codec_id() {
                    "V_MPEG4/ISO/AVC" if private.len() >= 4 => VideoCodec::H264 {
                        profile: private[1],
                        compat: private[2],
                        level: private[3],
                    },
                    "V_MPEGH/ISO/HEVC" if private.len() >= 2 => VideoCodec::Hevc {
                        profile: private[1] & 0x1f,
                    },
                    other => VideoCodec::Other(other.to_string()),
                };
                let v = t.video();
                probe.video = Some(VideoTrack {
                    number,
                    codec,
                    width: v.map(|v| v.pixel_width().get() as u32).unwrap_or(0),
                    height: v.map(|v| v.pixel_height().get() as u32).unwrap_or(0),
                    config: private,
                    frame_ms: t.default_duration().map(|d| d.get() as f64 / 1e6),
                    strip,
                });
            }
            TrackType::Audio => {
                let codec = match t.codec_id() {
                    id if id.starts_with("A_AAC") => AudioCodec::Aac,
                    "A_AC3" => AudioCodec::Ac3,
                    "A_EAC3" => AudioCodec::Eac3,
                    "A_MPEG/L3" => AudioCodec::Mp3,
                    "A_OPUS" => AudioCodec::Opus,
                    "A_FLAC" => AudioCodec::Flac,
                    other => AudioCodec::Other(other.to_string()),
                };
                let a = t.audio();
                let rate = a.map(|a| a.sampling_frequency() as u32).unwrap_or(48_000);
                let channels = a.map(|a| a.channels().get() as u32).unwrap_or(2);
                // Old files name the AAC profile in the codec id and carry no config.
                let config = if codec == AudioCodec::Aac && private.is_empty() {
                    aac_config(rate, channels)
                } else {
                    private
                };
                probe.audio.push(AudioTrack {
                    number,
                    codec,
                    channels,
                    rate,
                    config,
                    language: t.language().map(str::to_string),
                    name: t.name().map(str::to_string),
                    strip,
                });
            }
            TrackType::Subtitle => {
                let codec = match t.codec_id() {
                    "S_TEXT/ASS" | "S_TEXT/SSA" | "S_ASS" | "S_SSA" => SubCodec::Ass,
                    "S_TEXT/UTF8" | "S_TEXT/ASCII" => SubCodec::Srt,
                    other => SubCodec::Other(other.to_string()),
                };
                probe.subs.push(SubTrack {
                    number,
                    codec,
                    language: t.language().map(str::to_string),
                    name: t.name().map(str::to_string),
                    strip,
                    zlib,
                });
            }
            _ => {}
        }
    }
    probe
}

/// An AAC-LC AudioSpecificConfig for a rate and a channel count.
fn aac_config(rate: u32, channels: u32) -> Vec<u8> {
    const RATES: [u32; 13] = [
        96000, 88200, 64000, 48000, 44100, 32000, 24000, 22050, 16000, 12000, 11025, 8000, 7350,
    ];
    let idx = RATES.iter().position(|r| *r == rate).unwrap_or(3) as u8;
    let ch = channels.min(7) as u8;
    vec![(2 << 3) | (idx >> 1), ((idx & 1) << 7) | (ch << 3)]
}

/// A frame's payload with the track's compression undone.
pub fn payload(data: Vec<u8>, strip: Option<&[u8]>, zlib: bool) -> Vec<u8> {
    let data = if zlib {
        let mut out = Vec::with_capacity(data.len() * 2);
        match flate2::read::ZlibDecoder::new(&data[..]).read_to_end(&mut out) {
            Ok(_) => out,
            Err(_) => data,
        }
    } else {
        data
    };
    match strip {
        Some(prefix) => {
            let mut out = Vec::with_capacity(prefix.len() + data.len());
            out.extend_from_slice(prefix);
            out.extend_from_slice(&data);
            out
        }
        None => data,
    }
}

/// Whether a video frame starts a picture the decoder can begin at, read from the frame
/// itself: some muxers leave the keyframe flag off blocks inside groups.
pub fn is_keyframe(codec: &VideoCodec, config: &[u8], data: &[u8], flagged: Option<bool>) -> bool {
    if let Some(k) = flagged {
        return k;
    }
    let size = match codec {
        VideoCodec::H264 { .. } => config.get(4).map(|b| (b & 3) as usize + 1).unwrap_or(4),
        VideoCodec::Hevc { .. } => config.get(21).map(|b| (b & 3) as usize + 1).unwrap_or(4),
        VideoCodec::Other(_) => return false,
    };
    let mut i = 0;
    while i + size < data.len() {
        let len = data[i..i + size]
            .iter()
            .fold(0usize, |n, b| (n << 8) | *b as usize);
        let nal = data[i + size];
        match codec {
            VideoCodec::H264 { .. } if nal & 0x1f == 5 => return true,
            // IRAP pictures: BLA, IDR, CRA.
            VideoCodec::Hevc { .. } if (16..=21).contains(&((nal >> 1) & 0x3f)) => return true,
            _ => {}
        }
        i += size + len;
    }
    false
}

/// What one pass over the file found: where a segment can start, how far the decoder
/// reorders pictures, and every text subtitle.
#[derive(Clone, Debug, Default)]
pub struct Scan {
    /// Keyframe times in ms, ascending.
    pub keyframes: Vec<u64>,
    /// How far, in ms, decode order runs ahead of display order at most.
    pub reorder_ms: u64,
    /// Subtitle cues by track number.
    pub subs: Vec<(u64, Vec<Cue>)>,
    pub end_ms: u64,
}

/// Reads the whole file once. Subtitles in Matroska sit in among the frames, so finding
/// them all means reading it all; the keyframes come with that for free.
pub fn scan(mkv: &mut Mkv, probe: &Probe) -> Result<Scan, String> {
    let mut scan = Scan::default();
    let video = probe.video.clone();
    let mut subs: Vec<(u64, Vec<Cue>)> = probe
        .subs
        .iter()
        .filter(|s| !matches!(s.codec, SubCodec::Other(_)))
        .map(|s| (s.number, Vec::new()))
        .collect();
    let mut gop: Vec<u64> = Vec::new();
    let mut frame = Frame::default();
    loop {
        match mkv.next_frame(&mut frame) {
            Ok(true) => {}
            Ok(false) => break,
            Err(e) => {
                // A damaged tail still leaves everything before it usable.
                if scan.keyframes.is_empty() {
                    return Err(format!("could not read the file: {e}"));
                }
                break;
            }
        }
        let ms = probe.to_ms(frame.timestamp);
        scan.end_ms = scan.end_ms.max(ms);
        if let Some(v) = &video
            && frame.track == v.number
        {
            let data = payload(
                std::mem::take(&mut frame.data),
                v.strip.as_deref(),
                probe.zlib.contains(&v.number),
            );
            if is_keyframe(&v.codec, &v.config, &data, frame.is_keyframe) {
                scan.reorder_ms = scan.reorder_ms.max(reorder(&gop));
                gop.clear();
                if scan.keyframes.last().is_none_or(|k| ms > *k) {
                    scan.keyframes.push(ms);
                }
            }
            gop.push(ms);
            continue;
        }
        if let Some((_, cues)) = subs.iter_mut().find(|(n, _)| *n == frame.track) {
            let track = probe.subs.iter().find(|s| s.number == frame.track).unwrap();
            let data = payload(
                std::mem::take(&mut frame.data),
                track.strip.as_deref(),
                track.zlib,
            );
            let text = String::from_utf8_lossy(&data);
            let end = frame
                .duration
                .map(|d| ms + probe.to_ms(d))
                .unwrap_or(ms + 3000);
            let text = match track.codec {
                SubCodec::Ass => super::vtt::ass_text(&text),
                _ => Some(text.trim().to_string()),
            };
            if let Some(text) = text.filter(|t| !t.is_empty()) {
                cues.push(Cue {
                    start: ms,
                    end,
                    text,
                });
            }
        }
    }
    scan.reorder_ms = scan.reorder_ms.max(reorder(&gop));
    for (_, cues) in subs.iter_mut() {
        cues.sort_by_key(|c| c.start);
    }
    scan.subs = subs;
    if scan.keyframes.is_empty() {
        return Err("the video has no keyframes this can start from".into());
    }
    Ok(scan)
}

/// How far behind its sorted place the most delayed picture of a group sits: the shift
/// that keeps every decode time at or before its display time.
pub fn reorder(pts: &[u64]) -> u64 {
    let mut sorted = pts.to_vec();
    sorted.sort_unstable();
    sorted
        .iter()
        .zip(pts)
        .map(|(s, p)| s.saturating_sub(*p))
        .max()
        .unwrap_or(0)
}

/// One sample for the MP4 writer, times in ms.
#[derive(Clone, Debug, PartialEq)]
pub struct Sample {
    pub pts: u64,
    pub dts: u64,
    pub duration: u32,
    pub key: bool,
    pub data: Vec<u8>,
}

/// The frames of `[from, to)` ms for the video and one audio track. Video starts at the
/// keyframe at `from`. Decode times are the stretch's display times in sorted order, and
/// every display time moves forward by the file's reorder depth, so no decode time passes
/// its display time and none goes below zero.
pub fn segment(
    mkv: &mut Mkv,
    probe: &Probe,
    scan: &Scan,
    audio: Option<&AudioTrack>,
    from: u64,
    to: Option<u64>,
) -> Result<(Vec<Sample>, Vec<Sample>), String> {
    let video = probe.video.as_ref().ok_or("no video track")?;
    mkv.seek_ms(probe, from)?;
    let mut v: Vec<Sample> = Vec::new();
    let mut a: Vec<Sample> = Vec::new();
    let mut frame = Frame::default();
    let mut started = false;
    let end = to.unwrap_or(u64::MAX);
    // Audio runs a little past the last video frame's time in the file; read until both
    // are past the end, with a bound so a file with no more video still stops.
    let mut video_done = false;
    loop {
        match mkv.next_frame(&mut frame) {
            Ok(true) => {}
            Ok(false) => break,
            Err(e) => {
                if v.is_empty() {
                    return Err(format!("could not read the file: {e}"));
                }
                break;
            }
        }
        let ms = probe.to_ms(frame.timestamp);
        if frame.track == video.number {
            let data = payload(
                std::mem::take(&mut frame.data),
                video.strip.as_deref(),
                probe.zlib.contains(&video.number),
            );
            let key = is_keyframe(&video.codec, &video.config, &data, frame.is_keyframe);
            if !started {
                if !(key && ms >= from) {
                    continue;
                }
                started = true;
            } else if key && ms >= end {
                video_done = true;
            }
            if video_done {
                if a.last().is_some_and(|s| s.pts >= end) || audio.is_none() {
                    break;
                }
                continue;
            }
            v.push(Sample {
                pts: ms,
                dts: 0,
                duration: 0,
                key,
                data,
            });
        } else if let Some(at) = audio
            && frame.track == at.number
        {
            if ms < from {
                continue;
            }
            if ms >= end {
                if video_done {
                    break;
                }
                continue;
            }
            let data = payload(
                std::mem::take(&mut frame.data),
                at.strip.as_deref(),
                probe.zlib.contains(&at.number),
            );
            a.push(Sample {
                pts: ms,
                dts: ms,
                duration: 0,
                key: true,
                data,
            });
        }
        // Past the end with no audio frames left to take: a minute of slack is plenty.
        if video_done && ms > end.saturating_add(60_000) {
            break;
        }
    }
    let frame_ms = video.frame_ms.unwrap_or(1000.0 / 23.976).round().max(1.0) as u32;
    let mut sorted: Vec<u64> = v.iter().map(|s| s.pts).collect();
    sorted.sort_unstable();
    let shift = scan.reorder_ms;
    for (s, d) in v.iter_mut().zip(sorted) {
        s.dts = d;
        s.pts += shift;
    }
    for s in a.iter_mut() {
        s.pts += shift;
        s.dts += shift;
    }
    // The next stretch's first decode time is its keyframe's display time.
    durations(&mut v, to, frame_ms);
    let audio_ms = audio.map(frame_length_ms).unwrap_or(21);
    durations(&mut a, to.map(|t| t + shift), audio_ms);
    Ok((v, a))
}

/// Each sample lasts until the next one's decode time; the last until `end`, or one
/// typical frame when the stretch runs to the file's end.
fn durations(samples: &mut [Sample], end: Option<u64>, typical: u32) {
    for i in 0..samples.len() {
        let next = samples.get(i + 1).map(|n| n.dts).or(end);
        let d = next
            .map(|n| n.saturating_sub(samples[i].dts) as u32)
            .unwrap_or(typical);
        samples[i].duration = if d == 0 || d > typical * 8 {
            typical
        } else {
            d
        };
    }
}

/// A typical audio frame's length in ms, for the last frame of a stretch.
fn frame_length_ms(t: &AudioTrack) -> u32 {
    let samples = match t.codec {
        AudioCodec::Aac => 1024,
        AudioCodec::Ac3 | AudioCodec::Eac3 => 1536,
        AudioCodec::Mp3 => 1152,
        AudioCodec::Opus => 960,
        AudioCodec::Flac => 4096,
        AudioCodec::Other(_) => 1024,
    };
    ((samples * 1000) / t.rate.max(1)).max(1)
}

/// Peeks the first frame of an audio track, which AC-3 and E-AC-3 need for their config.
pub fn first_frame(mkv: &mut Mkv, probe: &Probe, track: u64) -> Option<Vec<u8>> {
    mkv.seek_ms(probe, 0).ok()?;
    let mut frame = Frame::default();
    for _ in 0..2000 {
        if !mkv.next_frame(&mut frame).ok()? {
            return None;
        }
        if frame.track == track {
            let t = probe.audio.iter().find(|a| a.number == track)?;
            return Some(payload(
                std::mem::take(&mut frame.data),
                t.strip.as_deref(),
                probe.zlib.contains(&track),
            ));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reorder_is_the_largest_lag_behind_display_order() {
        // I P B B in decode order with display times 0 120 40 80: decode times 40 ms back
        // of the sorted display times, -40 0 40 80, never pass a display time.
        assert_eq!(reorder(&[0, 120, 40, 80]), 40);
        assert_eq!(reorder(&[0, 40, 80]), 0);
        assert_eq!(reorder(&[]), 0);
    }

    #[test]
    fn keyframes_are_read_from_the_nal_units_when_the_block_does_not_say() {
        let codec = VideoCodec::H264 {
            profile: 100,
            compat: 0,
            level: 40,
        };
        let config = [1, 100, 0, 40, 0xff];
        let idr = [0, 0, 0, 2, 0x65, 0x88];
        let p = [0, 0, 0, 2, 0x41, 0x9a];
        assert!(is_keyframe(&codec, &config, &idr, None));
        assert!(!is_keyframe(&codec, &config, &p, None));
        assert!(!is_keyframe(&codec, &config, &idr, Some(false)));
    }

    #[test]
    fn aac_config_for_48k_stereo() {
        assert_eq!(aac_config(48_000, 2), vec![0x11, 0x90]);
    }

    #[test]
    fn stripped_headers_go_back_in_front() {
        assert_eq!(payload(vec![3, 4], Some(&[1, 2]), false), vec![1, 2, 3, 4]);
    }
}
