use crate::crypto::utils_stream::StreamHelper;

use super::errors::CommaNotFound;
use super::errors::DetectionError;

pub struct Detection<'a> {
    pub eof_position: i64,
    pub ekey_position: i64,
    pub ekey_len: usize,
    pub song_id: &'a str,
}

// 'QTag' in LittleEndian
const MAGIC_QMC2_QTAG: u32 = 0x67615451;

fn find_comma(buf: &[u8], start: usize, end: usize) -> Result<usize, CommaNotFound> {
    for (i, &byte) in buf[start..end].iter().enumerate() {
        if byte == b',' {
            return Ok(i + start);
        }
    }

    Err(CommaNotFound {})
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
        .map_err(|_| DetectionError::CouldNotIdentifyEndOfEKey())?;
    let ekey_len = (ekey_end_loc as i64 - ekey_loc) as usize;

    // The song id come right after the key, seperated by a comma ","
    let song_id_loc = ekey_end_loc + 1;
    // Ignore if song id extraction failed.
    let song_id = find_comma(buf, song_id_loc, end_of_meta_loc)
        .ok()
        .and_then(|end| std::str::from_utf8(&buf[song_id_loc..end]).ok())
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
    if 0 < len_v1 && len_v1 <= 0x300 {
        return detect_v1(buf);
    }

    if eof_magic == 0 {
        Err(DetectionError::ZerosAtEOF())
    } else {
        Err(DetectionError::UnknownMagicLE32(eof_magic))
    }
}
