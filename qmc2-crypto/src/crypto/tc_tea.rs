//! tc_tea (TenCent's TEA) a variant of the standard TEA (Tiny Encryption Algorithm).
//! Notably, it uses a different round number and adds a tweaked CBC mode.

use super::errors::CryptoError;
use super::utils_stream::StreamHelper;

const ROUNDS: u32 = 16;
const SALT_LEN: usize = 2;
const ZERO_LEN: usize = 7;
const MINIMUM_ENCRYPTED_TEA_LEN: usize = 1 + SALT_LEN + ZERO_LEN;
const DELTA: u32 = 0x9e3779b9;

#[inline]
/// Perform a single round of wrapping arithmetics
fn tea_single_round(value: u32, sum: u32, key1: u32, key2: u32) -> u32 {
    // z -= ((y << 4) + k[2]) ^ (y + sum) ^ ((y >> 5) + k[3]);
    // y -= ((z << 4) + k[0]) ^ (z + sum) ^ ((z >> 5) + k[1]);
    value.wrapping_shl(4).wrapping_add(key1)
        ^ sum.wrapping_add(value)
        ^ value.wrapping_shr(5).wrapping_add(key2)
}

#[inline]
/// Perform a single operation of tea's ecb decryption.
fn tea_decrypt_ecb(block: &mut [u8], key: &[u8; 16]) {
    let mut k = [0u32; 4];
    for (i, k) in k.iter_mut().enumerate() {
        *k = key.read_u32_be(i * 4);
    }

    let mut y = block.read_u32_be(0);
    let mut z = block.read_u32_be(4);
    let mut sum = DELTA.wrapping_mul(ROUNDS);

    for _ in 0..ROUNDS {
        z = z.wrapping_sub(tea_single_round(y, sum, k[2], k[3]));
        y = y.wrapping_sub(tea_single_round(z, sum, k[0], k[1]));

        sum = sum.wrapping_sub(DELTA);
    }

    block.write_u32_be(0, y);
    block.write_u32_be(4, z);
}

/// Decrypts a byte array containing the following:
/// * PadLen  (1 byte)
/// * Padding (variable, 0-7byte)
/// * Salt    (2 bytes)
/// * Body    (? bytes)
/// * Zero    (7 bytes)
/// PadLen/Padding/Salt is random bytes. Minium of 3 bytes.
/// PadLen is taken from the last 3 bit of the first byte.
pub fn oi_symmetry_decrypt2(input: &[u8], key: &[u8; 16]) -> Result<Vec<u8>, CryptoError> {
    let len = input.len();
    if (len < MINIMUM_ENCRYPTED_TEA_LEN) || (len % 8 != 0) {
        return Err(CryptoError::TEAInputSizeError(
            len,
            MINIMUM_ENCRYPTED_TEA_LEN,
        ));
    }

    let mut decrypted_buf = input.to_vec();

    // Decrypt blocks
    tea_decrypt_ecb(&mut decrypted_buf[0..8], key);
    for i in (8..len).step_by(8) {
        for j in i..i + 8 {
            decrypted_buf[j] ^= decrypted_buf[j - 8];
        }
        tea_decrypt_ecb(&mut decrypted_buf[i..i + 8], key);
    }
    // Finalise. First block xor with ZERO iv, so we can skip.
    for i in (8..len).step_by(8) {
        for j in i..i + 8 {
            decrypted_buf[j] ^= input[j - 8];
        }
    }

    let pad_size = usize::from(decrypted_buf[0] & 0b111);

    // Prefixed with "pad_size", "padding", "salt"
    let start_loc = 1 + pad_size + SALT_LEN;
    let end_loc = len - ZERO_LEN;
    let zeros = &decrypted_buf[end_loc..];

    // I know this is not constant time comparison, but anyway...
    if zeros.iter().all(|&x| x == 0) {
        Ok(decrypted_buf[start_loc..end_loc].to_vec())
    } else {
        Err(CryptoError::TEAZeroVerificationError())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tc_tea_basic() {
        let input: [u8; 24] = [
            0x91, 0x09, 0x51, 0x62, 0xe3, 0xf5, 0xb6, 0xdc, //
            0x6b, 0x41, 0x4b, 0x50, 0xd1, 0xa5, 0xb8, 0x4e, //
            0xc5, 0x0d, 0x0c, 0x1b, 0x11, 0x96, 0xfd, 0x3c, //
        ];
        let key = [
            0x31u8, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, //
            0x41u8, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, //
        ];

        let result = oi_symmetry_decrypt2(&input, &key);
        assert_eq!(result.unwrap(), [1u8, 2, 3, 4, 5, 6, 7, 8]);
    }
}
