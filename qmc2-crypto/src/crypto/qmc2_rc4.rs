extern crate static_assertions;

use super::qmc2_base::QMC2Crypto;

const FIRST_SEGMENT_SIZE: usize = 0x80;
const OTHER_SEGMENT_SIZE: usize = 0x1400;

/// Recommends 2.5M block, aligns to the segment size.
const RECOMMENDED_BLOCK_SIZE: usize = (1024 * 1024) * 5 / 2;
static_assertions::const_assert_eq!(RECOMMENDED_BLOCK_SIZE % OTHER_SEGMENT_SIZE, 0);

pub struct QMCStreamRC4Crypto {
    s: Vec<u8>,
    n: usize,
    hash: u32,
    rc4_key: Vec<u8>,
}

impl QMCStreamRC4Crypto {
    #[inline]
    pub(self) fn calc_segment_key(&self, id: usize, seed: u8) -> usize {
        let dividend = f64::from(self.hash);
        let divisor = ((id + 1) * usize::from(seed)) as f64;
        let key = dividend / divisor * 100.0;
        key as u64 as usize
    }

    #[inline]
    pub(self) fn encode_first_segment(&self, offset: usize, buf: &mut [u8]) {
        let mut offset = offset;
        for b in buf.iter_mut() {
            let key1 = self.rc4_key[offset % self.n];
            let key2 = self.calc_segment_key(offset, key1);
            *b ^= self.rc4_key[key2 % self.n];

            offset += 1;
        }
    }

    #[inline]
    pub(self) fn rc4_derive(n: usize, s: &mut Vec<u8>, j: &mut usize, k: &mut usize) -> u8 {
        *j = (*j + 1) % n;
        *k = (usize::from(s[*j]) + *k) % n;

        s.swap(*j, *k);
        s[(usize::from(s[*j]) + usize::from(s[*k])) % n]
    }

    #[inline]
    pub(self) fn encode_other_segment(&self, offset: usize, buf: &mut [u8]) {
        // segment_id: 0~511 (inclusive)
        let seg_id = offset / OTHER_SEGMENT_SIZE;
        let seg_id_small = seg_id & 0x1FF;

        let mut discard_count = self.calc_segment_key(seg_id, self.rc4_key[seg_id_small]) & 0x1FF;
        discard_count += offset % OTHER_SEGMENT_SIZE;

        let n = self.n;
        let mut s = self.s.clone();
        let mut j = 0usize;
        let mut k = 0usize;
        for _ in 0..discard_count {
            QMCStreamRC4Crypto::rc4_derive(n, &mut s, &mut j, &mut k);
        }

        for b in buf.iter_mut() {
            *b ^= QMCStreamRC4Crypto::rc4_derive(n, &mut s, &mut j, &mut k);
        }
    }

    #[inline]
    pub(self) fn update_hash_base(&mut self) {
        let mut hash: u32 = 1;

        for i in 0..self.n {
            let value = u32::from(self.rc4_key[i]);

            // Skip if the next byte is zero.
            if value == 0 {
                continue;
            }

            let new_hash = hash.wrapping_mul(value);
            if new_hash == 0 || new_hash <= hash {
                break;
            }

            hash = new_hash;
        }

        self.hash = hash;
    }
}

impl QMCStreamRC4Crypto {
    pub fn new(rc4_key: &[u8]) -> Self {
        let n = rc4_key.len();
        let mut s = vec![0u8; n];
        for (i, b) in s.iter_mut().enumerate() {
            *b = i as u8;
        }

        let mut j = 0usize;
        for i in 0..n {
            j = (usize::from(s[i]) + j + usize::from(rc4_key[i % n])) % n;
            s.swap(i, j);
        }

        let mut result = QMCStreamRC4Crypto {
            s,
            n,
            hash: 1,
            rc4_key: rc4_key.to_vec(),
        };
        result.update_hash_base();
        result
    }
}

impl QMC2Crypto for QMCStreamRC4Crypto {
    fn get_recommended_block_size(&self) -> usize {
        RECOMMENDED_BLOCK_SIZE
    }

    fn decrypt(&self, offset: usize, buf: &mut [u8]) {
        let mut offset = offset;
        let mut len = buf.len();
        let mut i = 0usize;

        // First segment have a different algorithm.
        if offset < FIRST_SEGMENT_SIZE {
            let len_processed = std::cmp::min(len, FIRST_SEGMENT_SIZE - offset);
            self.encode_first_segment(offset, &mut buf[i..i + len_processed]);
            i += len_processed;
            len -= len_processed;
            offset += len_processed;
        }

        // Align a segment
        let to_align = offset % OTHER_SEGMENT_SIZE;
        if to_align != 0 {
            let len_processed = std::cmp::min(len, OTHER_SEGMENT_SIZE - to_align);
            self.encode_other_segment(offset, &mut buf[i..i + len_processed]);
            i += len_processed;
            len -= len_processed;
            offset += len_processed;
        }

        // Process segments
        while len > OTHER_SEGMENT_SIZE {
            self.encode_other_segment(offset, &mut buf[i..i + OTHER_SEGMENT_SIZE]);
            i += OTHER_SEGMENT_SIZE;
            len -= OTHER_SEGMENT_SIZE;
            offset += OTHER_SEGMENT_SIZE;
        }

        // Left over
        if len > 0 {
            self.encode_other_segment(offset, &mut buf[i..i + len]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1_update_hash_base() {
        let crypto = QMCStreamRC4Crypto::new(&[1u8, 99]);
        assert_eq!(crypto.hash, 1);

        let crypto = QMCStreamRC4Crypto::new(&[
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // 8
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // 16
        ]);
        assert_eq!(crypto.hash, 0xfc05fc01);
    }

    #[test]
    fn test2_for_loop() {
        for i in (0..13).step_by(5) {
            println!("i = {}", i)
        }
    }
}
