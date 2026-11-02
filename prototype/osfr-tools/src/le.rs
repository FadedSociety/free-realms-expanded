//! Little-endian field reads that cannot panic (L109, hygiene pass). A buffer that ends first
//! gives `None`, which the caller turns into its own refusal. They replace the
//! `bytes[a..b].try_into().unwrap()` idiom: 283 of the 562 production unwraps measured on
//! 2026-11-01 (`lab/reports/hygiene-20261101`).

/// The `N` bytes at `at`, or `None` when the buffer ends first.
pub fn bytes<const N: usize>(buffer: &[u8], at: usize) -> Option<[u8; N]> {
    buffer.get(at..)?.first_chunk().copied()
}

pub fn u16(buffer: &[u8], at: usize) -> Option<u16> {
    bytes(buffer, at).map(u16::from_le_bytes)
}

pub fn u32(buffer: &[u8], at: usize) -> Option<u32> {
    bytes(buffer, at).map(u32::from_le_bytes)
}

pub fn i32(buffer: &[u8], at: usize) -> Option<i32> {
    bytes(buffer, at).map(i32::from_le_bytes)
}

pub fn u64(buffer: &[u8], at: usize) -> Option<u64> {
    bytes(buffer, at).map(u64::from_le_bytes)
}

pub fn f32(buffer: &[u8], at: usize) -> Option<f32> {
    bytes(buffer, at).map(f32::from_le_bytes)
}

#[cfg(test)]
mod tests {
    #[test]
    fn reads_stop_at_the_end_of_the_buffer() {
        let buffer = [1, 0, 0, 0, 2, 0];
        assert_eq!(super::u32(&buffer, 0), Some(1));
        assert_eq!(super::u16(&buffer, 4), Some(2));
        assert_eq!(super::u32(&buffer, 3), None);
        assert_eq!(super::u16(&buffer, 6), None);
        assert_eq!(super::u16(&buffer, usize::MAX), None);
    }
}
