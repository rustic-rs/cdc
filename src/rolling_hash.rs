use crate::{Polynom, Polynom64};

pub mod constants {
    use crate::Polynom64;

    /// Default irreducible modulo polynom.
    pub const MOD_POLYNOM: Polynom64 = 0x003D_A335_8B4D_C173;
}

/// A rolling hash implementation for 64 bit polynoms.
pub trait RollingHash64 {
    /// Resets the rolling hash.
    fn reset(&mut self);

    /// Attempt to prefill the window
    ///
    /// # Arguments
    ///
    /// * `iter` - The iterator to read from.
    fn prefill_window<I>(&mut self, iter: &mut I) -> usize
    where
        I: Iterator<Item = u8>;

    /// Combines a reset with a prefill in an optimized way.
    ///
    /// # Arguments
    ///
    /// * `iter` - The iterator to read from.
    fn reset_and_prefill_window<I>(&mut self, iter: &mut I) -> usize
    where
        I: Iterator<Item = u8>;

    /// Slides the window by byte.
    ///
    /// # Arguments
    ///
    /// * `byte` - The byte to slide in.
    fn slide(&mut self, byte: u8);

    /// Returns the current hash as a `Polynom64`.
    fn get_hash(&self) -> &Polynom64;
}

/// A rolling hash implementation for 64 bit polynoms from Rabin.
#[derive(Debug, Clone)]
pub struct Rabin64 {
    // Configuration
    /// Window size.
    window_size: usize, // The size of the data window used in the hash calculation.
    /// Window size mask.
    window_size_mask: usize, // = window_size - 1, supposing that it is an exponent of 2.

    // Precalculations
    /// The number of bits to shift the polynom to the left.
    polynom_shift: i32,

    /// Precalculated out table.
    out_table: [Polynom64; 256],
    /// Precalculated mod table.
    mod_table: [Polynom64; 256],

    // Current state
    /// The data window.
    window_data: Vec<u8>,
    /// The current window index.
    window_index: usize,
    /// The current hash.
    pub hash: Polynom64,
}

impl Rabin64 {
    /// Calculates the out table. The out table is used to remove the byte that is sliding out of the window.
    #[must_use]
    pub fn calculate_out_table(window_size: usize, mod_polynom: &Polynom64) -> [Polynom64; 256] {
        let mut out_table = [0; 256];
        for (b, elem) in out_table.iter_mut().enumerate() {
            let mut hash = (b as Polynom64).modulo(mod_polynom);
            for _ in 0..window_size - 1 {
                hash <<= 8;
                hash = hash.modulo(mod_polynom);
            }
            *elem = hash;
        }

        out_table
    }

    /// Calculates the mod table. The mod table is used to add the byte that is sliding into the window.
    #[must_use]
    pub fn calculate_mod_table(mod_polynom: &Polynom64) -> [Polynom64; 256] {
        let mut mod_table = [0; 256];
        let k = mod_polynom.degree();
        for (b, elem) in mod_table.iter_mut().enumerate() {
            let p: Polynom64 = (b as Polynom64) << k;
            *elem = p.modulo(mod_polynom) | p;
        }

        mod_table
    }

    /// Creates a new `Rabin64` rolling hash.
    #[must_use]
    pub fn new(window_size_nb_bits: u32) -> Self {
        Self::new_with_polynom(window_size_nb_bits, &constants::MOD_POLYNOM)
    }

    /// Creates a new `Rabin64` rolling hash with a specific modulo polynom.
    ///
    /// # Arguments
    ///
    /// * `window_size_nb_bits` - The number of bits of the window size.
    /// * `mod_polynom` - The modulo polynom.
    #[must_use]
    pub fn new_with_polynom(window_size_nb_bits: u32, mod_polynom: &Polynom64) -> Self {
        let window_size = 1 << window_size_nb_bits;

        let window_data = vec![0; window_size];

        Self {
            window_size,
            window_size_mask: window_size - 1,
            polynom_shift: mod_polynom.degree() - 8,
            out_table: Self::calculate_out_table(window_size, mod_polynom),
            mod_table: Self::calculate_mod_table(mod_polynom),
            window_data,
            window_index: 0,
            hash: 0,
        }
    }

    #[cfg(test)]
    pub(crate) fn hash_block(&mut self, bytes: &[u8], mod_polynom: Polynom64) {
        for v in bytes {
            self.hash <<= 8;
            self.hash |= Polynom64::from(*v);
            self.hash = self.hash.modulo(&mod_polynom);
        }
    }
}

impl RollingHash64 for Rabin64 {
    fn reset(&mut self) {
        self.window_data.clear();
        self.window_data.resize(self.window_size, 0);
        self.window_index = 0;
        self.hash = 0;

        // Not needed.
        // self.slide(1);
    }

    // Attempt to fills the window - 1 byte.
    fn prefill_window<I>(&mut self, iter: &mut I) -> usize
    where
        I: Iterator<Item = u8>,
    {
        let mut nb_bytes_read = 0;
        for _ in 0..self.window_size - 1 {
            match iter.next() {
                Some(b) => {
                    self.slide(b);
                    nb_bytes_read += 1;
                }
                None => break,
            }
        }

        nb_bytes_read
    }

    // Combines a reset with a prefill in an optimized way.
    fn reset_and_prefill_window<I>(&mut self, iter: &mut I) -> usize
    where
        I: Iterator<Item = u8>,
    {
        self.hash = 0;
        let mut nb_bytes_read = 0;
        for _ in 0..self.window_size - 1 {
            match iter.next() {
                Some(b) => {
                    // Take the old value out of the window and the hash.
                    // ... let's suppose that the buffer contains zeroes, do nothing.

                    // Put the new value in the window and in the hash.
                    self.window_data[self.window_index] = b;
                    let mod_index = (self.hash >> self.polynom_shift) & 255;
                    self.hash <<= 8;
                    self.hash |= Polynom64::from(b);
                    self.hash ^= self.mod_table[mod_index as usize];

                    // Move the windowIndex to the next position.
                    self.window_index = (self.window_index + 1) & self.window_size_mask;

                    nb_bytes_read += 1;
                }
                None => break,
            }
        }

        // Because we didn't overwrite that element in the loop above.
        self.window_data[self.window_index] = 0;

        nb_bytes_read
    }

    #[inline]
    fn slide(&mut self, byte: u8) {
        // Take the old value out of the window and the hash.
        let out_value = self.window_data[self.window_index];
        self.hash ^= self.out_table[out_value as usize];

        // Put the new value in the window and in the hash.
        self.window_data[self.window_index] = byte;
        let mod_index = (self.hash >> self.polynom_shift) & 255;
        self.hash <<= 8;
        self.hash |= Polynom64::from(byte);
        self.hash ^= self.mod_table[mod_index as usize];

        // Move the windowIndex to the next position.
        self.window_index = (self.window_index + 1) & self.window_size_mask;
    }

    #[inline]
    fn get_hash(&self) -> &Polynom64 {
        &self.hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::polynom::Polynom64;

    fn to_hex_string(polynoms: &[Polynom64], prefix: &str) -> String {
        let strs: Vec<String> = polynoms
            .iter()
            .map(|p| format!("{}{:016x} {}", prefix, p, 0))
            .collect();
        strs.join("\n")
    }

    #[test]
    fn print_tables() {
        let out_table = Rabin64::calculate_out_table(32, &constants::MOD_POLYNOM);
        let mod_table = Rabin64::calculate_mod_table(&constants::MOD_POLYNOM);
        println!("{}", to_hex_string(&out_table[..], "outTable "));
        println!("{}", to_hex_string(&mod_table[..], "modTable "));
    }

    #[test]
    fn rabin_hash() {
        use std::cmp::max;

        // Random meaningless data.
        let data = [
            17u8, 28, 53, 64, 175, 216, 27, 208, 109, 130, 143, 35, 93, 244, 45, 18, 64, 193, 204,
            59, 169, 139, 53, 59, 55, 65, 242, 73, 60, 198, 45, 22, 56, 90, 81, 181,
        ];

        let mut rabin1 = Rabin64::new(5);
        let mut rabin2 = Rabin64::new(5);

        // Block by block, no optimization, used raw modulo formula.
        for i in 0..data.len() {
            let block = &data[(max(31, i) - 31)..=i];
            rabin1.reset();
            rabin1.hash_block(block, constants::MOD_POLYNOM);

            rabin2.slide(data[i]);

            //println!("{:02} {:02} {:016x} {:016x} {:?}", i, block.len(), rabin1.hash, rabin2.hash, block);
            assert_eq!(rabin1.hash, rabin2.hash);
        }
    }
}
