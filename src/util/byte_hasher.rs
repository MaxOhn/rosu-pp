use std::hash::{BuildHasher, Hasher};

#[derive(Copy, Clone, Default)]
pub(crate) struct ByteHasher;

impl BuildHasher for ByteHasher {
    type Hasher = ByteHash;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        ByteHash { byte: 0 }
    }
}

pub(crate) struct ByteHash {
    byte: u8,
}

impl Hasher for ByteHash {
    #[inline]
    fn finish(&self) -> u64 {
        self.byte as u64
    }

    #[inline]
    fn write(&mut self, _: &[u8]) {
        unreachable!()
    }

    #[inline]
    fn write_u8(&mut self, byte: u8) {
        self.byte = byte;
    }
}

#[cfg(test)]
mod tests {
    use std::hash::{BuildHasher, Hash};

    use super::ByteHasher;

    #[test]
    fn hashes_byte() {
        let mut state = ByteHasher.build_hasher();
        42_u8.hash(&mut state);
    }

    #[test]
    #[should_panic]
    fn doesnt_hash_int() {
        let mut state = ByteHasher.build_hasher();
        42_i32.hash(&mut state);
    }
}
