//! Fragmented MP4, the container every Cast receiver and most TVs take: an init segment
//! that describes the tracks, then one movie fragment per segment holding the frames as the
//! Matroska file stored them. H.264 and HEVC already sit length-prefixed in Matroska, the
//! same way MP4 wants them, so nothing in a frame changes. Times are in ms throughout.

use super::mkv::{AudioCodec, AudioTrack, Sample, VideoCodec, VideoTrack};

pub const VIDEO_ID: u32 = 1;
pub const AUDIO_ID: u32 = 2;
const TIMESCALE: u32 = 1000;

fn mp4box(name: &[u8; 4], body: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(body.len() + 8);
    out.extend_from_slice(&((body.len() + 8) as u32).to_be_bytes());
    out.extend_from_slice(name);
    out.extend_from_slice(body);
    out
}

fn full(name: &[u8; 4], version: u8, flags: u32, body: &[u8]) -> Vec<u8> {
    let mut b = Vec::with_capacity(body.len() + 4);
    b.push(version);
    b.extend_from_slice(&flags.to_be_bytes()[1..]);
    b.extend_from_slice(body);
    mp4box(name, &b)
}

fn cat(parts: &[Vec<u8>]) -> Vec<u8> {
    parts.concat()
}

const MATRIX: [u32; 9] = [0x0001_0000, 0, 0, 0, 0x0001_0000, 0, 0, 0, 0x4000_0000];

fn be32(v: u32) -> [u8; 4] {
    v.to_be_bytes()
}

fn mvhd() -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(&be32(0)); // creation
    b.extend_from_slice(&be32(0)); // modification
    b.extend_from_slice(&be32(TIMESCALE));
    b.extend_from_slice(&be32(0)); // duration: fragments carry it
    b.extend_from_slice(&be32(0x0001_0000)); // rate
    b.extend_from_slice(&[0x01, 0x00]); // volume
    b.extend_from_slice(&[0; 10]);
    for m in MATRIX {
        b.extend_from_slice(&be32(m));
    }
    b.extend_from_slice(&[0; 24]);
    b.extend_from_slice(&be32(AUDIO_ID + 1)); // next track id
    full(b"mvhd", 0, 0, &b)
}

fn tkhd(id: u32, audio: bool, width: u32, height: u32) -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(&be32(0));
    b.extend_from_slice(&be32(0));
    b.extend_from_slice(&be32(id));
    b.extend_from_slice(&be32(0));
    b.extend_from_slice(&be32(0)); // duration
    b.extend_from_slice(&[0; 8]);
    b.extend_from_slice(&[0, 0]); // layer
    b.extend_from_slice(&[0, 0]); // alternate group
    b.extend_from_slice(if audio { &[0x01, 0x00] } else { &[0, 0] });
    b.extend_from_slice(&[0, 0]);
    for m in MATRIX {
        b.extend_from_slice(&be32(m));
    }
    b.extend_from_slice(&be32(width << 16));
    b.extend_from_slice(&be32(height << 16));
    // Enabled and in the movie.
    full(b"tkhd", 0, 3, &b)
}

fn mdhd() -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(&be32(0));
    b.extend_from_slice(&be32(0));
    b.extend_from_slice(&be32(TIMESCALE));
    b.extend_from_slice(&be32(0));
    b.extend_from_slice(&[0x55, 0xc4]); // "und"
    b.extend_from_slice(&[0, 0]);
    full(b"mdhd", 0, 0, &b)
}

fn hdlr(audio: bool) -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(&be32(0));
    b.extend_from_slice(if audio { b"soun" } else { b"vide" });
    b.extend_from_slice(&[0; 12]);
    b.extend_from_slice(if audio { b"Sound\0" } else { b"Video\0" });
    full(b"hdlr", 0, 0, &b)
}

fn stbl(entry: Vec<u8>) -> Vec<u8> {
    let mut stsd = be32(1).to_vec();
    stsd.extend_from_slice(&entry);
    mp4box(
        b"stbl",
        &cat(&[
            full(b"stsd", 0, 0, &stsd),
            full(b"stts", 0, 0, &be32(0)),
            full(b"stsc", 0, 0, &be32(0)),
            full(b"stsz", 0, 0, &[0; 8]),
            full(b"stco", 0, 0, &be32(0)),
        ]),
    )
}

fn minf(audio: bool, entry: Vec<u8>) -> Vec<u8> {
    let head = if audio {
        full(b"smhd", 0, 0, &[0; 4])
    } else {
        full(b"vmhd", 0, 1, &[0; 8])
    };
    let dref = full(
        b"dref",
        0,
        0,
        &cat(&[be32(1).to_vec(), full(b"url ", 0, 1, &[])]),
    );
    mp4box(b"minf", &cat(&[head, mp4box(b"dinf", &dref), stbl(entry)]))
}

fn trak(id: u32, audio: bool, width: u32, height: u32, entry: Vec<u8>) -> Vec<u8> {
    let mdia = mp4box(b"mdia", &cat(&[mdhd(), hdlr(audio), minf(audio, entry)]));
    mp4box(b"trak", &cat(&[tkhd(id, audio, width, height), mdia]))
}

fn video_entry(v: &VideoTrack) -> Result<Vec<u8>, String> {
    let (name, config_name): (&[u8; 4], &[u8; 4]) = match v.codec {
        VideoCodec::H264 { .. } => (b"avc1", b"avcC"),
        VideoCodec::Hevc { .. } => (b"hvc1", b"hvcC"),
        VideoCodec::Other(ref id) => return Err(format!("{id} video does not go in MP4 here")),
    };
    let mut b = Vec::new();
    b.extend_from_slice(&[0; 6]);
    b.extend_from_slice(&[0, 1]); // data reference index
    b.extend_from_slice(&[0; 16]);
    b.extend_from_slice(&(v.width as u16).to_be_bytes());
    b.extend_from_slice(&(v.height as u16).to_be_bytes());
    b.extend_from_slice(&be32(0x0048_0000));
    b.extend_from_slice(&be32(0x0048_0000));
    b.extend_from_slice(&be32(0));
    b.extend_from_slice(&[0, 1]); // frame count
    b.extend_from_slice(&[0; 32]); // compressor name
    b.extend_from_slice(&[0, 0x18]); // depth
    b.extend_from_slice(&[0xff, 0xff]);
    b.extend_from_slice(&mp4box(config_name, &v.config));
    Ok(mp4box(name, &b))
}

fn descriptor(tag: u8, body: &[u8]) -> Vec<u8> {
    let mut out = vec![tag];
    let len = body.len();
    // Four-byte length form, which every reader takes.
    out.extend_from_slice(&[
        0x80 | ((len >> 21) & 0x7f) as u8,
        0x80 | ((len >> 14) & 0x7f) as u8,
        0x80 | ((len >> 7) & 0x7f) as u8,
        (len & 0x7f) as u8,
    ]);
    out.extend_from_slice(body);
    out
}

fn esds(object: u8, config: &[u8]) -> Vec<u8> {
    let mut dcd = vec![object, 0x15, 0, 0, 0];
    dcd.extend_from_slice(&be32(0)); // max bitrate
    dcd.extend_from_slice(&be32(0)); // average bitrate
    if !config.is_empty() {
        dcd.extend_from_slice(&descriptor(0x05, config));
    }
    let mut es = vec![0, AUDIO_ID as u8, 0];
    es.extend_from_slice(&descriptor(0x04, &dcd));
    es.extend_from_slice(&descriptor(0x06, &[0x02]));
    full(b"esds", 0, 0, &descriptor(0x03, &es))
}

/// The AC-3 config box from the first sync frame's header.
fn dac3(frame: &[u8]) -> Option<Vec<u8>> {
    if frame.len() < 7 || frame[0] != 0x0b || frame[1] != 0x77 {
        return None;
    }
    let fscod = frame[4] >> 6;
    let frmsizecod = frame[4] & 0x3f;
    let bsid = frame[5] >> 3;
    let bsmod = frame[5] & 7;
    let acmod = frame[6] >> 5;
    // lfeon sits after optional fields that depend on acmod.
    let mut bit = 3u32;
    if acmod & 1 != 0 && acmod != 1 {
        bit += 2;
    }
    if acmod & 4 != 0 {
        bit += 2;
    }
    if acmod == 2 {
        bit += 2;
    }
    let read_bit = |pos: u32| -> u8 {
        let byte = 6 + (pos / 8) as usize;
        (frame.get(byte).copied().unwrap_or(0) >> (7 - pos % 8)) & 1
    };
    let lfeon = read_bit(bit);
    let bit_rate_code = frmsizecod >> 1;
    let v: u32 = ((fscod as u32) << 22)
        | ((bsid as u32) << 17)
        | ((bsmod as u32) << 14)
        | ((acmod as u32) << 11)
        | ((lfeon as u32) << 10)
        | ((bit_rate_code as u32) << 5);
    Some(mp4box(b"dac3", &v.to_be_bytes()[1..]))
}

/// The E-AC-3 config box from the first frame's header, one independent substream.
fn dec3(frame: &[u8], rate_kbps: u32) -> Option<Vec<u8>> {
    if frame.len() < 6 || frame[0] != 0x0b || frame[1] != 0x77 {
        return None;
    }
    let fscod = frame[4] >> 6;
    let acmod = (frame[4] >> 1) & 7;
    let lfeon = frame[4] & 1;
    let bsid = frame[5] >> 3;
    let mut b = Vec::new();
    b.extend_from_slice(&(((rate_kbps.min(8191)) << 3) as u16).to_be_bytes()); // num_ind_sub = 0
    b.push((fscod << 6) | (bsid << 1));
    b.push(acmod << 1 | lfeon); // asvc 0, bsmod 0
    b.push(0); // num_dep_sub 0, reserved
    Some(mp4box(b"dec3", &b))
}

/// Opus in MP4 wants dOps, which is OpusHead without its magic, in big-endian.
fn dops(head: &[u8]) -> Option<Vec<u8>> {
    if head.len() < 19 || &head[..8] != b"OpusHead" {
        return None;
    }
    let mut b = vec![0, head[9]];
    b.extend_from_slice(&u16::from_le_bytes([head[10], head[11]]).to_be_bytes());
    b.extend_from_slice(
        &u32::from_le_bytes([head[12], head[13], head[14], head[15]]).to_be_bytes(),
    );
    b.extend_from_slice(&i16::from_le_bytes([head[16], head[17]]).to_be_bytes());
    b.push(head[18]);
    if head[18] != 0 {
        b.extend_from_slice(head.get(19..)?);
    }
    Some(mp4box(b"dOps", &b))
}

fn audio_entry(a: &AudioTrack, first: Option<&[u8]>) -> Result<Vec<u8>, String> {
    let (name, config): (&[u8; 4], Vec<u8>) = match a.codec {
        AudioCodec::Aac => (b"mp4a", esds(0x40, &a.config)),
        AudioCodec::Mp3 => (b"mp4a", esds(0x6b, &[])),
        AudioCodec::Ac3 => (
            b"ac-3",
            first
                .and_then(dac3)
                .ok_or("the AC-3 track has no readable header")?,
        ),
        AudioCodec::Eac3 => (
            b"ec-3",
            first
                .and_then(|f| dec3(f, 640))
                .ok_or("the E-AC-3 track has no readable header")?,
        ),
        AudioCodec::Opus => (
            b"Opus",
            dops(&a.config).ok_or("the Opus track has no OpusHead")?,
        ),
        AudioCodec::Flac => {
            let blocks = a
                .config
                .strip_prefix(b"fLaC")
                .ok_or("the FLAC track has no header")?;
            (b"fLaC", full(b"dfLa", 0, 0, blocks))
        }
        AudioCodec::Other(ref id) => return Err(format!("{id} audio does not go in MP4 here")),
    };
    let mut b = Vec::new();
    b.extend_from_slice(&[0; 6]);
    b.extend_from_slice(&[0, 1]);
    b.extend_from_slice(&[0; 8]);
    b.extend_from_slice(&(a.channels as u16).to_be_bytes());
    b.extend_from_slice(&[0, 16]); // sample size
    b.extend_from_slice(&[0; 4]);
    // The 16.16 rate field holds 65535 at most; Opus always reads 48000 here.
    let rate = if a.codec == AudioCodec::Opus {
        48_000
    } else {
        a.rate
    };
    b.extend_from_slice(&be32(rate.min(65535) << 16));
    b.extend_from_slice(&config);
    Ok(mp4box(name, &b))
}

/// ftyp and moov: what the tracks are, before any frame.
pub fn init(
    video: &VideoTrack,
    audio: Option<&AudioTrack>,
    audio_first: Option<&[u8]>,
) -> Result<Vec<u8>, String> {
    let ftyp = mp4box(
        b"ftyp",
        &cat(&[
            b"isom".to_vec(),
            be32(0x200).to_vec(),
            b"isomiso6iso2avc1mp41".to_vec(),
        ]),
    );
    let mut traks = vec![trak(
        VIDEO_ID,
        false,
        video.width,
        video.height,
        video_entry(video)?,
    )];
    let mut trexs = vec![trex(VIDEO_ID)];
    if let Some(a) = audio {
        traks.push(trak(AUDIO_ID, true, 0, 0, audio_entry(a, audio_first)?));
        trexs.push(trex(AUDIO_ID));
    }
    let mvex = mp4box(b"mvex", &trexs.concat());
    let moov = mp4box(b"moov", &cat(&[mvhd(), traks.concat(), mvex]));
    Ok(cat(&[ftyp, moov]))
}

fn trex(id: u32) -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(&be32(id));
    b.extend_from_slice(&be32(1)); // sample description index
    b.extend_from_slice(&be32(0));
    b.extend_from_slice(&be32(0));
    b.extend_from_slice(&be32(0));
    full(b"trex", 0, 0, &b)
}

const KEY_FLAGS: u32 = 0x0200_0000;
const DELTA_FLAGS: u32 = 0x0101_0000;

fn traf(id: u32, samples: &[Sample], video: bool, data_offset: u32) -> Vec<u8> {
    // default-base-is-moof: data offsets count from the moof's first byte.
    let tfhd = full(b"tfhd", 0, 0x02_0000, &be32(id));
    let base = samples.first().map(|s| s.dts).unwrap_or(0);
    let tfdt = full(b"tfdt", 1, 0, &base.to_be_bytes());
    let mut flags = 0x1 | 0x100 | 0x200 | 0x400;
    if video {
        flags |= 0x800;
    }
    let mut b = Vec::new();
    b.extend_from_slice(&be32(samples.len() as u32));
    b.extend_from_slice(&be32(data_offset));
    for s in samples {
        b.extend_from_slice(&be32(s.duration));
        b.extend_from_slice(&be32(s.data.len() as u32));
        b.extend_from_slice(&be32(if s.key { KEY_FLAGS } else { DELTA_FLAGS }));
        if video {
            b.extend_from_slice(&((s.pts as i64 - s.dts as i64) as i32).to_be_bytes());
        }
    }
    let trun = full(b"trun", 1, flags, &b);
    mp4box(b"traf", &cat(&[tfhd, tfdt, trun]))
}

/// One movie fragment: moof and mdat for a segment's video and audio frames.
pub fn fragment(sequence: u32, video: &[Sample], audio: &[Sample]) -> Vec<u8> {
    let mfhd = full(b"mfhd", 0, 0, &be32(sequence));
    let video_bytes: usize = video.iter().map(|s| s.data.len()).sum();
    // The moof's size does not depend on the offsets inside it, so build once with zeros
    // to measure, then again with the real offsets.
    let build = |vo: u32, ao: u32| {
        let mut parts = vec![mfhd.clone(), traf(VIDEO_ID, video, true, vo)];
        if !audio.is_empty() {
            parts.push(traf(AUDIO_ID, audio, false, ao));
        }
        mp4box(b"moof", &parts.concat())
    };
    let size = build(0, 0).len() as u32;
    let moof = build(size + 8, size + 8 + video_bytes as u32);
    let mut mdat_body =
        Vec::with_capacity(video_bytes + audio.iter().map(|s| s.data.len()).sum::<usize>());
    for s in video.iter().chain(audio) {
        mdat_body.extend_from_slice(&s.data);
    }
    cat(&[moof, mp4box(b"mdat", &mdat_body)])
}

/// The RFC 6381 codec string a playlist names the video with.
pub fn video_codec_string(v: &VideoTrack) -> String {
    match &v.codec {
        VideoCodec::H264 {
            profile,
            compat,
            level,
        } => format!("avc1.{profile:02X}{compat:02X}{level:02X}"),
        VideoCodec::Hevc { .. } => {
            let c = &v.config;
            if c.len() < 13 {
                return "hvc1.1.6.L93.B0".into();
            }
            let space = ["", "A", "B", "C"][(c[1] >> 6) as usize];
            let tier = if c[1] & 0x20 != 0 { 'H' } else { 'L' };
            let profile = c[1] & 0x1f;
            let compat = u32::from_be_bytes([c[2], c[3], c[4], c[5]]).reverse_bits();
            let level = c[12];
            let mut constraints: Vec<u8> = c[6..12].to_vec();
            while constraints.last() == Some(&0) && constraints.len() > 1 {
                constraints.pop();
            }
            let cons: Vec<String> = constraints.iter().map(|b| format!("{b:X}")).collect();
            format!(
                "hvc1.{space}{profile}.{compat:X}.{tier}{level}.{}",
                cons.join(".")
            )
        }
        VideoCodec::Other(id) => id.clone(),
    }
}

pub fn audio_codec_string(a: &AudioTrack) -> String {
    match a.codec {
        AudioCodec::Aac => {
            let object = a.config.first().map(|b| b >> 3).unwrap_or(2);
            format!("mp4a.40.{object}")
        }
        AudioCodec::Mp3 => "mp4a.6B".into(),
        AudioCodec::Ac3 => "ac-3".into(),
        AudioCodec::Eac3 => "ec-3".into(),
        AudioCodec::Opus => "opus".into(),
        AudioCodec::Flac => "fLaC".into(),
        AudioCodec::Other(ref id) => id.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn video() -> VideoTrack {
        VideoTrack {
            number: 1,
            codec: VideoCodec::H264 {
                profile: 100,
                compat: 0,
                level: 40,
            },
            width: 1280,
            height: 720,
            config: vec![1, 100, 0, 40, 0xff, 0xe0, 0, 0],
            frame_ms: Some(41.7),
            strip: None,
        }
    }

    #[test]
    fn the_codec_strings_follow_rfc_6381() {
        assert_eq!(video_codec_string(&video()), "avc1.640028");
        let a = AudioTrack {
            number: 2,
            codec: AudioCodec::Aac,
            channels: 2,
            rate: 48000,
            config: vec![0x11, 0x90],
            language: None,
            name: None,
            strip: None,
        };
        assert_eq!(audio_codec_string(&a), "mp4a.40.2");
    }

    #[test]
    fn a_fragment_points_its_runs_into_its_own_mdat() {
        let v = vec![
            Sample {
                pts: 80,
                dts: 0,
                duration: 40,
                key: true,
                data: vec![1; 10],
            },
            Sample {
                pts: 40,
                dts: 40,
                duration: 40,
                key: false,
                data: vec![2; 5],
            },
        ];
        let a = vec![Sample {
            pts: 0,
            dts: 0,
            duration: 21,
            key: true,
            data: vec![3; 4],
        }];
        let f = fragment(7, &v, &a);
        let moof_len = u32::from_be_bytes(f[0..4].try_into().unwrap()) as usize;
        assert_eq!(&f[4..8], b"moof");
        assert_eq!(&f[moof_len + 4..moof_len + 8], b"mdat");
        // The video run's data offset is the first byte after the mdat header.
        let trun = f.windows(4).position(|w| w == b"trun").unwrap();
        let offset = u32::from_be_bytes(f[trun + 12..trun + 16].try_into().unwrap()) as usize;
        assert_eq!(offset, moof_len + 8);
        assert_eq!(f[offset], 1);
        assert_eq!(f.len(), moof_len + 8 + 19);
    }

    #[test]
    fn the_init_segment_starts_with_ftyp_and_holds_both_tracks() {
        let i = init(&video(), None, None).unwrap();
        assert_eq!(&i[4..8], b"ftyp");
        assert!(i.windows(4).any(|w| w == b"avcC"));
        assert!(i.windows(4).any(|w| w == b"trex"));
    }
}
