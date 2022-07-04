use std::hash::{BuildHasher, Hasher};

#[derive(Copy, Clone, Default)]
pub(crate) struct BuildByteHasher;

impl BuildHasher for BuildByteHasher {
    type Hasher = ByteHasher;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        ByteHasher { byte: 0 }
    }
}

pub(crate) struct ByteHasher {
    byte: u8,
}

impl Hasher for ByteHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.byte as u64
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        // Only use this hasher for single bytes
        debug_assert_eq!(bytes.len(), 1);
        self.byte = bytes[0];
    }

    #[inline]
    fn write_u8(&mut self, byte: u8) {
        self.byte = byte;
    }
}
