use crate::crypto::utils_stream::StreamHelper;
use std::str::from_utf8;

use super::errors::DetectionError;

#[derive(std::fmt::Debug, Eq, PartialEq)]
pub struct Detection<'a> {
    pub eof_position: i64,
    pub ekey_position: i64,
    pub ekey_len: usize,
    pub song_id: &'a str,
}

// 'QTag' in LittleEndian
const MAGIC_QMC2_QTAG: u32 = 0x67615451;

fn find_comma(buf: &[u8], start: usize, end: usize) -> Option<usize> {
    for (i, &byte) in buf[start..end].iter().enumerate() {
        if byte == b',' {
            return Some(i + start);
        }
    }

    None
}

pub const RECOMMENDED_DETECTION_SIZE: usize = 0x40;

fn detect_v1(buf: &[u8]) -> Result<Detection, DetectionError> {
    // key size is always unsigned.
    let key_size = buf.read_u32_le(buf.len() - 4) as usize;
    let end_of_meta_loc = buf.len() - 4;

    // ekey_loc can be negative - which means it will be before the detection buffer.
    let ekey_loc = end_of_meta_loc as i64 - key_size as i64;

    Ok(Detection {
        eof_position: ekey_loc,
        ekey_position: ekey_loc,
        ekey_len: key_size,
        song_id: "",
    })
}

fn detect_v2(buf: &[u8]) -> Result<Detection, DetectionError> {
    let meta_size = buf.read_u32_be(buf.len() - 8) as usize;
    let end_of_meta_loc = buf.len() - 8;

    // ekey_loc can be negative - which means it will be before the detection buffer.
    let ekey_loc = end_of_meta_loc as i64 - meta_size as i64;
    let search_start_idx = if ekey_loc > 0 { ekey_loc as usize } else { 0 };
    // Locate the end of ekey (where the comma is)...
    let ekey_end_loc = find_comma(buf, search_start_idx, end_of_meta_loc)
        .ok_or(DetectionError::CouldNotIdentifyEndOfEKey())?;
    let ekey_len = (ekey_end_loc as i64 - ekey_loc) as usize;

    // The song id come right after the key, seperated by a comma ","
    let song_id_loc = ekey_end_loc + 1;
    // Ignore if song id extraction failed.
    let song_id = find_comma(buf, song_id_loc, end_of_meta_loc)
        .and_then(|end| from_utf8(&buf[song_id_loc..end]).ok())
        .unwrap_or_default();

    Ok(Detection {
        eof_position: ekey_loc,
        ekey_position: ekey_loc,
        ekey_len,
        song_id,
    })
}

pub fn detect(buf: &[u8]) -> Result<Detection, DetectionError> {
    if buf.len() < 8 {
        return Err(DetectionError::BufferTooSmall());
    }

    // QMC2 v2: eof_magic is string "QTag"
    let eof_magic = buf.read_u32_le(buf.len() - 4);
    if eof_magic == MAGIC_QMC2_QTAG {
        return detect_v2(buf);
    }

    // QMC2 v1: eof_magic is actually a size.
    let len_v1 = eof_magic;
    // Known max size is 528 bytes (0x210), round it up.
    if 0 < len_v1 && len_v1 <= 0x300 {
        return detect_v1(buf);
    }

    if eof_magic == 0 {
        Err(DetectionError::ZerosAtEOF())
    } else {
        Err(DetectionError::UnknownMagicLE32(eof_magic))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detection_small_buffer_boundary_check() {
        assert_eq!(
            detect(&[0u8; 7]).unwrap_err(),
            DetectionError::BufferTooSmall()
        );

        assert_eq!(detect(&[0u8; 8]).unwrap_err(), DetectionError::ZerosAtEOF());
    }

    #[test]
    fn test_detect_v2_embedded() {
        let input = [
            // 10 bytes of attached metadata
            b'a', b'a', b'a', b'a', b',', // ekey
            b'1', b'8', b',', // song id
            b'2', b',', // version identifier?
            // 10 = 0x0A
            0x00, 0x00, 0x00, 0x0A, // size of metadata (big endian)
            b'Q', b'T', b'a', b'g', //  EOF Magic
        ];
        let result = detect(&input).unwrap();
        assert_eq!(
            result,
            Detection {
                eof_position: 0,
                ekey_position: 0,
                ekey_len: 4,
                song_id: "18",
            }
        );
    }

    #[test]
    fn test_detect_v2_ekey_before_buffer() {
        let input = [
            // 10 bytes of attached metadata (+16 bytes "before" the buffer)
            b'a', b'a', b'a', b'a', b',', // ekey
            b'2', b'7', b',', // song id
            b'2', b',', // version identifier?
            // 10 = 0x0A; +16 = 0x1A
            0x00, 0x00, 0x00, 0x1A, // size of metadata (big endian)
            b'Q', b'T', b'a', b'g', //  EOF Magic
        ];
        let result = detect(&input).unwrap();
        assert_eq!(
            result,
            Detection {
                eof_position: -16,
                ekey_position: -16,
                ekey_len: 20,
                song_id: "27",
            }
        );
    }

    #[test]
    fn test_detect_v2_work_without_song_id() {
        let input = [
            // 10 bytes of attached metadata (+16 bytes "before" the buffer)
            b'a', b'a', b'a', b'a', b',', // ekey
            b'-', b'-', b'-', // song id
            b'-', b'-', // version identifier?
            // 10 = 0x0A; +16 = 0x1A
            0x00, 0x00, 0x00, 0x1A, // size of metadata (big endian)
            b'Q', b'T', b'a', b'g', //  EOF Magic
        ];
        let result = detect(&input).unwrap();
        assert_eq!(
            result,
            Detection {
                eof_position: -16,
                ekey_position: -16,
                ekey_len: 20,
                song_id: "",
            }
        );
    }

    #[test]
    fn test_detect_fallback_to_v1() {
        let input = [
            b'a', b'a', b'a', b'a', // ekey
            // key size, little-endian
            4, 0, 0, 0,
        ];
        let result = detect(&input).unwrap();
        assert_eq!(
            result,
            Detection {
                eof_position: 0,
                ekey_position: 0,
                ekey_len: 4,
                song_id: "",
            }
        );
    }

    #[test]
    fn test_detect_fallback_to_v1_within_boundary() {
        let input = [
            b'a', b'a', b'a', b'a', // ekey
            // key size, little-endian
            0x00, 0x03, 0, 0,
        ];
        let result = detect(&input).unwrap();
        assert_eq!(
            result,
            Detection {
                eof_position: -0x0300 + 4,
                ekey_position: -0x0300 + 4,
                ekey_len: 0x300,
                song_id: "",
            }
        );
    }

    #[test]
    fn test_detect_fallback_to_v1_out_of_boundary() {
        let input = [
            b'a', b'a', b'a', b'a', // ekey
            // key size, little-endian (0x0301)
            0x01, 0x03, 0, 0,
        ];
        let result = detect(&input).unwrap_err();
        assert_eq!(result, DetectionError::UnknownMagicLE32(0x0301));
    }
}
