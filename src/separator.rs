use crate::{Rabin64, RollingHash64};

/// A separator is a part of a stream of data that is separated by a separator.
#[derive(Debug, Clone, Copy)]
pub struct Separator {
    /// The index of the separator in the stream.
    pub index: u64,

    /// The hash of the separator.
    pub hash: u64,
}

/// An iterator that separates data.
#[derive(Debug)]
pub struct SeparatorIter<I, F> {
    iter: I,
    predicate: F,
    rabin: Rabin64,
    index: u64,
}

impl<I> SeparatorIter<I, fn(u64) -> bool>
where
    I: Iterator<Item = u8>,
{
    /// Creates a new `SeparatorIter`.
    ///
    /// # Arguments
    ///
    /// * `iter` - The iterator to separate.
    pub fn new(iter: I) -> Self {
        #[inline]
        fn default_predicate(x: u64) -> bool {
            const BITMASK: u64 = (1u64 << 13) - 1;
            x & BITMASK == BITMASK
        }

        // window_size: 1 << 6 == 64 bytes
        let separator_size_nb_bits = 6;

        Self::custom_new(iter, separator_size_nb_bits, default_predicate)
    }
}

impl<I, F> SeparatorIter<I, F>
where
    I: Iterator<Item = u8>,
    F: Fn(u64) -> bool,
{
    /// Creates a new `SeparatorIter`.
    ///
    /// # Arguments
    ///
    /// * `iter` - The iterator to separate.
    /// * `separator_size_nb_bits` - The number of bits of the separator size.
    /// * `predicate` - The predicate used to determine if a separator is a separator boundary.
    pub fn custom_new(mut iter: I, separator_size_nb_bits: u32, predicate: F) -> Self {
        let mut rabin = Rabin64::new(separator_size_nb_bits);
        let index = rabin.reset_and_prefill_window(&mut iter) as u64;

        Self {
            iter,
            predicate,
            rabin,
            index,
        }
    }
}

impl<I, F> Iterator for SeparatorIter<I, F>
where
    I: Iterator<Item = u8>,
    F: Fn(u64) -> bool,
{
    type Item = Separator;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(byte) = self.iter.next() {
            self.rabin.slide(byte);
            self.index += 1;
            if (self.predicate)(self.rabin.hash) {
                let separator = Separator {
                    index: self.index,
                    hash: self.rabin.hash,
                };

                // Note: We skip subsequent separators which may overlap the current one.
                self.index += self.rabin.reset_and_prefill_window(&mut self.iter) as u64;

                return Some(separator);
            }
        }

        None
    }
}

/// Converts a separator's hash to a level.
#[derive(Debug, Clone, Copy)]
pub struct HashToLevel {
    lvl0_nb_bits: u32,
    lvlup_nb_bits: u32,
    lvlup_bitmask: u64,
}

impl Default for HashToLevel {
    fn default() -> Self {
        Self::custom_new(13, 3)
    }
}

impl HashToLevel {
    /// Creates a new `HashToLevel`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `HashToLevel` with custom parameters.
    ///
    /// # Arguments
    ///
    /// * `lvl0_nb_bits` - The number of bits of the level 0.
    /// * `lvlup_nb_bits` - The number of bits of the level up.
    #[must_use]
    pub fn custom_new(lvl0_nb_bits: u32, lvlup_nb_bits: u32) -> Self {
        Self {
            lvl0_nb_bits,
            lvlup_nb_bits,
            lvlup_bitmask: (1u64 << lvlup_nb_bits) - 1,
        }
    }

    /// Converts a separator's hash to a level.
    ///
    /// # Arguments
    ///
    /// * `hash` - The separator's hash.
    #[must_use]
    pub fn to_level(&self, hash: u64) -> usize {
        let mut level = 0usize;
        let mut h = hash >> self.lvl0_nb_bits;
        while h & self.lvlup_bitmask == self.lvlup_bitmask {
            level += 1;
            h >>= self.lvlup_nb_bits;
        }

        level
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn hash_to_level() {
        let converter = HashToLevel::custom_new(4, 2);

        for n in 0..4 {
            assert_eq!(converter.to_level((9u64 << n) - 1), 0);
        }
        for n in 4..6 {
            assert_eq!(converter.to_level((9u64 << n) - 1), 0);
        }
        for n in 6..8 {
            assert_eq!(converter.to_level((9u64 << n) - 1), 1);
        }
        for n in 8..10 {
            assert_eq!(converter.to_level((9u64 << n) - 1), 2);
        }
        for n in 10..12 {
            assert_eq!(converter.to_level((9u64 << n) - 1), 3);
        }
        for n in 12..14 {
            assert_eq!(converter.to_level((9u64 << n) - 1), 4);
        }
    }
}
