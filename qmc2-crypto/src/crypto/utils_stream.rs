pub trait StreamHelper {
    fn read_u32_be(&self, offset: usize) -> u32;
    fn read_u32_le(&self, offset: usize) -> u32;
    fn write_u32_be(&mut self, offset: usize, value: u32);
}

impl StreamHelper for [u8] {
    #[inline]
    fn read_u32_be(&self, offset: usize) -> u32 {
        (u32::from(self[offset]) << 24)
            | (u32::from(self[offset + 1]) << 16)
            | (u32::from(self[offset + 2]) << 8)
            | (u32::from(self[offset + 3]))
    }

    #[inline]
    fn read_u32_le(&self, offset: usize) -> u32 {
        (u32::from(self[offset + 3]) << 24)
            | (u32::from(self[offset + 2]) << 16)
            | (u32::from(self[offset + 1]) << 8)
            | (u32::from(self[offset]))
    }

    #[inline]
    fn write_u32_be(&mut self, offset: usize, value: u32) {
        self[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_u32_be_test() {
        let v1 = [1, 2, 3, 4];
        let v2 = [0x7f, 0xff, 0xee, 0xdd, 0xcc];
        assert_eq!(v1.read_u32_be(0), 0x01020304);
        assert_eq!(v2.read_u32_be(1), 0xffeeddcc);
    }

    #[test]
    fn read_u32_le_test() {
        let v1 = [1, 2, 3, 4];
        let v2 = [0x7f, 0xff, 0xee, 0xdd, 0xcc];
        assert_eq!(v1.read_u32_le(0), 0x04030201);
        assert_eq!(v2.read_u32_le(1), 0xccddeeff);
    }

    #[test]
    fn test_write_u32_be() {
        let v2 = &mut [0x7fu8, 0xff, 0xee, 0xdd, 0xcc];
        v2.write_u32_be(0, 0x01020304);
        assert_eq!(v2, &[1u8, 2, 3, 4, 0xcc]);
    }
}
