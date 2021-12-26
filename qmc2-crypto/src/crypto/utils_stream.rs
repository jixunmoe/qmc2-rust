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
        let v1: &[u8] = &[1, 2, 3, 4];
        let v2: &[u8] = &[0x7f, 0xff, 0xee, 0xdd, 0xcc];
        assert_eq!(0x01020304, v1.read_u32_be(0));
        assert_eq!(0xffeeddcc, v2.read_u32_be(1));
    }
}
