//! Where each Matroska cluster starts, read from the element headers alone, and a reader
//! that can be sent to one. The demuxer's own seek misreads the cues ffmpeg writes, and a
//! file with no cues has nothing for it to use; the clusters themselves are always there.

use std::fs::File;
use std::io::{self, BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::{Arc, Mutex};

const SEGMENT: u32 = 0x1853_8067;
const CLUSTER: u32 = 0x1F43_B675;
const TIMESTAMP: u32 = 0xE7;
const UNKNOWN: u64 = u64::MAX;

fn vint(r: &mut impl Read, keep_marker: bool) -> io::Result<(u64, usize)> {
    let mut first = [0u8];
    r.read_exact(&mut first)?;
    let len = first[0].leading_zeros() as usize + 1;
    if len > 8 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "bad EBML length",
        ));
    }
    // The bits after the length marker; none are left when the length is eight bytes.
    let mask = (0xffu16 >> len) as u8;
    let mut value = if keep_marker {
        first[0] as u64
    } else {
        (first[0] & mask) as u64
    };
    let mut all_ones = value == mask as u64;
    for _ in 1..len {
        let mut b = [0u8];
        r.read_exact(&mut b)?;
        all_ones &= b[0] == 0xff;
        value = (value << 8) | b[0] as u64;
    }
    if !keep_marker && all_ones {
        return Ok((UNKNOWN, len));
    }
    Ok((value, len))
}

/// Cluster start offsets and their timestamps in file units, ascending.
pub fn clusters(path: &Path) -> io::Result<Vec<(u64, u64)>> {
    let mut r = BufReader::with_capacity(64 * 1024, File::open(path)?);
    let end = r.get_ref().metadata()?.len();
    // EBML header, then the Segment.
    let (_, _) = vint(&mut r, true)?;
    let (size, _) = vint(&mut r, false)?;
    r.seek_relative(size as i64)?;
    let (id, _) = vint(&mut r, true)?;
    if id as u32 != SEGMENT {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "no Segment"));
    }
    let _ = vint(&mut r, false)?;
    let mut out = Vec::new();
    loop {
        let at = r.stream_position()?;
        if at >= end {
            break;
        }
        let Ok((id, _)) = vint(&mut r, true) else {
            break;
        };
        let Ok((size, _)) = vint(&mut r, false) else {
            break;
        };
        if id as u32 == CLUSTER {
            // The timestamp is the cluster's first child in every muxer seen; read children
            // until it turns up.
            let body = r.stream_position()?;
            let mut ts = None;
            for _ in 0..4 {
                let Ok((cid, _)) = vint(&mut r, true) else {
                    break;
                };
                let Ok((csize, _)) = vint(&mut r, false) else {
                    break;
                };
                if cid as u32 == TIMESTAMP && csize <= 8 {
                    let mut buf = [0u8; 8];
                    r.read_exact(&mut buf[8 - csize as usize..])?;
                    ts = Some(u64::from_be_bytes(buf));
                    break;
                }
                if csize == UNKNOWN {
                    break;
                }
                r.seek_relative(csize as i64)?;
            }
            if let Some(ts) = ts {
                out.push((at, ts));
            }
            if size == UNKNOWN {
                // A live-written cluster with no size: nothing to skip by; stop indexing.
                break;
            }
            r.seek(SeekFrom::Start(body + size))?;
        } else {
            if size == UNKNOWN {
                break;
            }
            r.seek_relative(size as i64)?;
        }
    }
    Ok(out)
}

/// The last cluster that starts at or before `ts`.
pub fn cluster_before(index: &[(u64, u64)], ts: u64) -> Option<u64> {
    let i = index.partition_point(|(_, t)| *t <= ts);
    index.get(i.saturating_sub(1)).map(|(at, _)| *at)
}

/// A file reader that can be sent somewhere else from outside, so the demuxer picks up at a
/// cluster of our choosing on its next read.
pub struct Jump {
    inner: BufReader<File>,
    pending: Arc<Mutex<Option<u64>>>,
}

impl Jump {
    pub fn open(path: &Path) -> io::Result<(Jump, Arc<Mutex<Option<u64>>>)> {
        let pending = Arc::new(Mutex::new(None));
        let inner = BufReader::with_capacity(1 << 20, File::open(path)?);
        Ok((
            Jump {
                inner,
                pending: pending.clone(),
            },
            pending,
        ))
    }

    fn apply(&mut self) -> io::Result<()> {
        if let Some(at) = self
            .pending
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take()
        {
            self.inner.seek(SeekFrom::Start(at))?;
        }
        Ok(())
    }
}

impl Read for Jump {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.apply()?;
        self.inner.read(buf)
    }
}

impl Seek for Jump {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.apply()?;
        self.inner.seek(pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cluster_before_a_time_is_the_last_that_starts_by_then() {
        let index = [(100, 0), (5000, 6000), (9000, 12000)];
        assert_eq!(cluster_before(&index, 0), Some(100));
        assert_eq!(cluster_before(&index, 6500), Some(5000));
        assert_eq!(cluster_before(&index, 99_999), Some(9000));
        assert_eq!(cluster_before(&[], 10), None);
    }

    #[test]
    fn element_lengths_read_with_and_without_the_marker() {
        let mut ids = &[0x1A, 0x45, 0xDF, 0xA3][..];
        assert_eq!(vint(&mut ids, true).unwrap(), (0x1A45_DFA3, 4));
        let mut size = &[0x81][..];
        assert_eq!(vint(&mut size, false).unwrap(), (1, 1));
        let mut unknown = &[0x01, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff][..];
        assert_eq!(vint(&mut unknown, false).unwrap().0, UNKNOWN);
    }
}
