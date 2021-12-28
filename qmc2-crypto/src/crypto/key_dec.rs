extern crate base64;

use super::errors::CryptoError;
use super::tc_tea;

fn simple_make_key(seed: u8, size: usize) -> Vec<u8> {
    let mut result: Vec<u8> = vec![0u8; size];

    for (i, b) in result.iter_mut().enumerate() {
        // Some random math, then truncate to u8.
        let value = (seed as f32) + (i as f32) * 0.1;
        *b = (100.0 * value.tan().abs()) as u8;
    }

    result
}

pub fn parse_ekey(ekey: &str) -> Result<Vec<u8>, CryptoError> {
    let ekey_decoded = base64::decode(ekey).map_err(|_| CryptoError::EKeyParseError())?;

    if ekey_decoded.len() < 8 {
        return Err(CryptoError::EKeyParseError());
    }

    let simple_key_buf = simple_make_key(106, 8);

    let mut tea_key = [0u8; 16];
    for i in (0..16).step_by(2) {
        tea_key[i] = simple_key_buf[i / 2];
        tea_key[i + 1] = ekey_decoded[i / 2];
    }

    let mut raw_key = tc_tea::oi_symmetry_decrypt2(&ekey_decoded[8..], &tea_key)
        .map_err(|_| CryptoError::QMC2KeyDeriveError())?;
    let mut result = Vec::from(&ekey_decoded[0..8]);
    result.append(&mut raw_key);

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smk_test() {
        let expected = [0x69, 0x56, 0x46, 0x38, 0x2b, 0x20, 0x15, 0x0b].to_vec();
        let actual = simple_make_key(106, 8);
        assert_eq!(actual, expected);
    }
}
