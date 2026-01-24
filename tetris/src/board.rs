use std::u64;

use crate::moves::Move;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Board {
    pub cols: [u64; 10],
}

impl Board {
    pub const fn new() -> Self {
        Self { cols: [0; 10] }
    }

    pub fn count(&self) -> u32 {
        self.cols.iter().fold(0, |a, c| a + c.count_ones())
    }

    pub fn height(&self, x: usize) -> u32 {
        64 - self.cols[x].leading_zeros()
    }

    pub fn heights(&self) -> [u32; 10] {
        let mut heights = [0; 10];

        for x in 0..10 {
            heights[x] = self.height(x);
        }

        heights
    }

    pub fn is_empty(&self) -> bool {
        self.cols == [0; 10]
    }

    pub fn has(&self, x: i8, y: i8) -> bool {
        match (x, y) {
            (0..10, 0..40) => self.cols[x as usize] & (1 << y) != 0,
            _ => true,
        }
    }

    pub fn set(&mut self, x: i8, y: i8) {
        self.cols[x as usize] |= 1 << y;
    }

    pub fn clear(&mut self, x: i8, y: i8) {
        self.cols[x as usize] &= !(1 << y);
    }

    pub fn place(&mut self, mv: &Move) {
        for (x, y) in mv.cells() {
            self.set(x, y);
        }
    }

    #[cfg(all(target_arch = "x86_64", target_feature = "bmi2"))]
    pub fn clear_lines(&mut self) -> u8 {
        let mask = self.cols.iter().fold(u64::MAX, |a, c| a & c);

        if mask == 0 {
            return 0;
        }

        for x in 0..10 {
            self.cols[x] = unsafe { std::arch::x86_64::_pext_u64(self.cols[x], !mask) };
        }

        mask.count_ones() as u8
    }

    #[cfg(not(all(target_arch = "x86_64", target_feature = "bmi2")))]
    pub fn clear_lines(&mut self) -> u8 {
        let mut mask = self.cols.iter().fold(u64::MAX, |a, c| a & c);

        if mask == 0 {
            return 0;
        }

        let shift = mask.trailing_zeros();

        mask >>= shift;

        for x in 0..10 {
            let lo = self.cols[x] & ((1 << shift) - 1);
            let mut hi = self.cols[x] >> shift;

            hi = match mask {
                0b0001 => hi >> 1,
                0b0011 => hi >> 2,
                0b0111 => hi >> 3,
                0b1111 => hi >> 4,
                0b0101 => ((hi >> 1) & 0b0001) | ((hi >> 3) << 1),
                0b1001 => ((hi >> 1) & 0b0011) | ((hi >> 4) << 2),
                0b1011 => ((hi >> 2) & 0b0001) | ((hi >> 4) << 1),
                0b1101 => ((hi >> 1) & 0b0001) | ((hi >> 4) << 1),
                _ => unreachable!(),
            };

            self.cols[x] = lo | (hi << shift);
        }

        mask.count_ones() as u8
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in (0..20).rev() {
            for x in 0..10 {
                write!(f, "{}", if self.has(x, y) { '#' } else { '.' })?;
            }

            write!(f, "\n")?;
        }

        Ok(())
    }
}
