//! SipHash-2-4, the keyed hash the room uses to make rejoin keys from its
//! secret. Written out here so the crate needs no hashing dependency and builds
//! for wasm32 as it is.

pub(crate) fn siphash24(key: [u8; 16], msg: &[u8]) -> u64 {
    let k0 = u64::from_le_bytes(key[..8].try_into().expect("8 bytes"));
    let k1 = u64::from_le_bytes(key[8..].try_into().expect("8 bytes"));
    let mut v = [
        0x736f_6d65_7073_6575 ^ k0,
        0x646f_7261_6e64_6f6d ^ k1,
        0x6c79_6765_6e65_7261 ^ k0,
        0x7465_6462_7974_6573 ^ k1,
    ];
    let (chunks, rest) = msg.as_chunks::<8>();
    for chunk in chunks {
        let m = u64::from_le_bytes(*chunk);
        v[3] ^= m;
        round(&mut v);
        round(&mut v);
        v[0] ^= m;
    }
    let mut last = (msg.len() as u64 & 0xff) << 56;
    for (i, b) in rest.iter().enumerate() {
        last |= u64::from(*b) << (8 * i);
    }
    v[3] ^= last;
    round(&mut v);
    round(&mut v);
    v[0] ^= last;
    v[2] ^= 0xff;
    for _ in 0..4 {
        round(&mut v);
    }
    v[0] ^ v[1] ^ v[2] ^ v[3]
}

fn round(v: &mut [u64; 4]) {
    v[0] = v[0].wrapping_add(v[1]);
    v[1] = v[1].rotate_left(13) ^ v[0];
    v[0] = v[0].rotate_left(32);
    v[2] = v[2].wrapping_add(v[3]);
    v[3] = v[3].rotate_left(16) ^ v[2];
    v[0] = v[0].wrapping_add(v[3]);
    v[3] = v[3].rotate_left(21) ^ v[0];
    v[2] = v[2].wrapping_add(v[1]);
    v[1] = v[1].rotate_left(17) ^ v[2];
    v[2] = v[2].rotate_left(32);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference_key() -> [u8; 16] {
        core::array::from_fn(|i| i as u8)
    }

    #[test]
    fn matches_the_reference_vectors() {
        // From the SipHash paper's reference implementation: key 00..0f,
        // messages 00..(n-1).
        assert_eq!(siphash24(reference_key(), &[]), 0x726f_db47_dd0e_0e31);
        let msg: Vec<u8> = (0..15).collect();
        assert_eq!(siphash24(reference_key(), &msg), 0xa129_ca61_49be_45e5);
    }

    #[test]
    #[allow(deprecated)]
    fn matches_the_standard_library() {
        use std::hash::{Hasher, SipHasher};
        let key = [7u8, 1, 9, 200, 3, 3, 3, 3, 250, 0, 1, 2, 3, 4, 5, 6];
        let k0 = u64::from_le_bytes(key[..8].try_into().unwrap());
        let k1 = u64::from_le_bytes(key[8..].try_into().unwrap());
        for len in 0..40 {
            let msg: Vec<u8> = (0..len).map(|i| (i * 37 + 11) as u8).collect();
            let mut std = SipHasher::new_with_keys(k0, k1);
            std.write(&msg);
            assert_eq!(siphash24(key, &msg), std.finish(), "length {len}");
        }
    }
}
