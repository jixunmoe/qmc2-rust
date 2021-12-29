use super::errors::CryptoError;

fn simple_make_key(seed: u8, size: usize) -> Box<[u8]> {
    let mut result = vec![0u8; size].into_boxed_slice();

    for (i, b) in result.iter_mut().enumerate() {
        // Some random math, then truncate to u8.
        let value = (seed as f32) + (i as f32) * 0.1;
        *b = (100.0 * value.tan().abs()) as u8;
    }

    result
}

fn derive_tea_key(ekey_header: &[u8]) -> Box<[u8]> {
    let simple_key_buf = simple_make_key(106, 8);

    let mut tea_key = [0u8; 16];
    for i in (0..16).step_by(2) {
        tea_key[i] = simple_key_buf[i / 2];
        tea_key[i + 1] = ekey_header[i / 2];
    }

    Box::from(tea_key)
}

pub fn parse_ekey(ekey: &str) -> Result<Box<[u8]>, CryptoError> {
    let ekey_decoded = base64::decode(ekey).map_err(|_| CryptoError::EKeyParseError)?;

    if ekey_decoded.len() < 8 {
        return Err(CryptoError::EKeyParseError);
    }

    let (header, body) = ekey_decoded.split_at(8);
    let tea_key = derive_tea_key(header);
    let body = tc_tea::decrypt(body, &tea_key).ok_or(CryptoError::QMC2KeyDeriveError)?;

    Ok([header, &*body].concat().into())
}

pub fn generate_ekey<T: AsRef<[u8]>>(key: T) -> String {
    // Generate encrypted version of the key...
    let (key_header, key_body) = key.as_ref().split_at(8);
    debug_assert_eq!(key_header.len(), 8);

    let tea_key = derive_tea_key(key_header);
    debug_assert_eq!(tea_key.len(), 16);

    let encrypted_body = tc_tea::encrypt(key_body, tea_key).unwrap();
    let ekey_encoded = [key_header, &*encrypted_body].concat();

    base64::encode(ekey_encoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_key() {
        let expected = [0x69, 0x56, 0x46, 0x38, 0x2b, 0x20, 0x15, 0x0b];
        let actual = simple_make_key(106, 8);
        assert_eq!(actual.to_vec(), expected);
    }

    #[test]
    fn test_derive_tea_key() {
        let ekey = [0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8];
        let expected = [
            0x69, 0xf1, //
            0x56, 0xf2, //
            0x46, 0xf3, //
            0x38, 0xf4, //
            0x2b, 0xf5, //
            0x20, 0xf6, //
            0x15, 0xf7, //
            0x0b, 0xf8, //
        ];
        let actual = derive_tea_key(&ekey);
        assert_eq!(actual.to_vec(), expected);
    }

    #[test]
    fn test_generate_ekey() {
        let expected_key = b"12345678...test data by Jixun";
        let ekey = generate_ekey(&expected_key);
        let actual = parse_ekey(&ekey).unwrap();
        assert_eq!(
            std::str::from_utf8(&*actual).unwrap(),
            std::str::from_utf8(expected_key).unwrap()
        );
    }

    #[test]
    fn test_parse_ekey() {
        let expected_key = "This is a test key for test purpose :D";
        let ekey = "VGhpcyBpcyBHFWEh4cjZ1Vi7rJ56XeoPlqGM1sxBGPg7mt89umKclFBr9iqfmFdS";
        let decoded_key = parse_ekey(ekey).unwrap();
        assert_eq!(std::str::from_utf8(&*decoded_key).unwrap(), expected_key);
    }
}
