use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

/// Compute the number of u64 words needed for a board of given dimensions.
pub const fn nw_for_board(width: u8, height: u8) -> usize {
    ((width as u16 * height as u16) as usize).div_ceil(64)
}

/// A fixed-size bitboard parameterized by the number of u64 words.
/// `NW` = number of active words = ceil(width*height / 64).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Bitboard<const NW: usize> {
    words: [u64; NW],
}

impl<const NW: usize> Bitboard<NW> {
    /// All bits zero.
    #[inline]
    pub const fn empty() -> Self {
        Bitboard { words: [0; NW] }
    }

    /// Single bit set at `index`.
    #[inline]
    pub fn single(index: usize) -> Self {
        debug_assert!(index < NW * 64);
        let mut bb = Self::empty();
        bb.words[index / 64] = 1u64 << (index % 64);
        bb
    }

    /// Construct from raw words.
    #[inline]
    pub const fn from_words(words: [u64; NW]) -> Self {
        Bitboard { words }
    }

    /// Test whether bit `index` is set.
    #[inline]
    pub fn get(&self, index: usize) -> bool {
        debug_assert!(index < NW * 64);
        (self.words[index / 64] >> (index % 64)) & 1 != 0
    }

    /// Set bit `index` to 1.
    #[inline]
    pub fn set(&mut self, index: usize) {
        debug_assert!(index < NW * 64);
        self.words[index / 64] |= 1u64 << (index % 64);
    }

    /// Clear bit `index` to 0.
    #[inline]
    pub fn clear(&mut self, index: usize) {
        debug_assert!(index < NW * 64);
        self.words[index / 64] &= !(1u64 << (index % 64));
    }

    /// True if no bits are set.
    #[inline]
    pub fn is_empty(&self) -> bool {
        let mut i = 0;
        while i < NW {
            if self.words[i] != 0 {
                return false;
            }
            i += 1;
        }
        true
    }

    /// True if any bit is set.
    #[inline]
    pub fn is_nonzero(&self) -> bool {
        let mut i = 0;
        while i < NW {
            if self.words[i] != 0 {
                return true;
            }
            i += 1;
        }
        false
    }

    /// Population count — number of set bits.
    #[inline]
    pub fn count(&self) -> u32 {
        let mut total = 0u32;
        let mut i = 0;
        while i < NW {
            total += self.words[i].count_ones();
            i += 1;
        }
        total
    }

    /// Index of the lowest set bit, or `None` if empty.
    #[inline]
    pub fn lowest_bit_index(&self) -> Option<usize> {
        let mut i = 0;
        while i < NW {
            let w = self.words[i];
            if w != 0 {
                return Some(i * 64 + w.trailing_zeros() as usize);
            }
            i += 1;
        }
        None
    }

    /// Shift all bits left (toward higher indices) by `n` positions.
    /// Bits shifted beyond NW*64-1 are lost.
    #[inline]
    pub fn shift_left(&self, n: usize) -> Self {
        if n == 0 {
            return *self;
        }
        if n >= NW * 64 {
            return Self::empty();
        }
        let word_shift = n / 64;
        let bit_shift = n % 64;
        let mut out = [0u64; NW];

        if bit_shift == 0 {
            out[word_shift..NW].copy_from_slice(&self.words[..(NW - word_shift)]);
        } else {
            let mut i = word_shift;
            while i < NW {
                out[i] = self.words[i - word_shift] << bit_shift;
                if i > word_shift {
                    out[i] |= self.words[i - word_shift - 1] >> (64 - bit_shift);
                }
                i += 1;
            }
        }
        Bitboard { words: out }
    }

    /// Shift all bits right (toward lower indices) by `n` positions.
    /// Bits shifted below 0 are lost.
    #[inline]
    pub fn shift_right(&self, n: usize) -> Self {
        if n == 0 {
            return *self;
        }
        if n >= NW * 64 {
            return Self::empty();
        }
        let word_shift = n / 64;
        let bit_shift = n % 64;
        let mut out = [0u64; NW];

        if bit_shift == 0 {
            out[..(NW - word_shift)].copy_from_slice(&self.words[word_shift..]);
        } else {
            let mut i = 0;
            while i < NW - word_shift {
                out[i] = self.words[i + word_shift] >> bit_shift;
                if i + word_shift + 1 < NW {
                    out[i] |= self.words[i + word_shift + 1] << (64 - bit_shift);
                }
                i += 1;
            }
        }
        Bitboard { words: out }
    }

    /// `self & !rhs` — bits in self that are not in rhs.
    #[inline]
    pub fn andnot(self, rhs: Bitboard<NW>) -> Bitboard<NW> {
        let mut out = [0u64; NW];
        let mut i = 0;
        while i < NW {
            out[i] = self.words[i] & !rhs.words[i];
            i += 1;
        }
        Bitboard { words: out }
    }

    /// Iterate over indices of set bits.
    #[inline]
    pub fn iter_ones(&self) -> BitIterator<NW> {
        BitIterator {
            words: self.words,
            word_index: 0,
        }
    }
}

impl<const NW: usize> BitAnd for Bitboard<NW> {
    type Output = Bitboard<NW>;
    #[inline]
    fn bitand(self, rhs: Bitboard<NW>) -> Bitboard<NW> {
        let mut out = [0u64; NW];
        let mut i = 0;
        while i < NW {
            out[i] = self.words[i] & rhs.words[i];
            i += 1;
        }
        Bitboard { words: out }
    }
}

impl<const NW: usize> BitAnd for &Bitboard<NW> {
    type Output = Bitboard<NW>;
    #[inline]
    fn bitand(self, rhs: &Bitboard<NW>) -> Bitboard<NW> {
        let mut out = [0u64; NW];
        let mut i = 0;
        while i < NW {
            out[i] = self.words[i] & rhs.words[i];
            i += 1;
        }
        Bitboard { words: out }
    }
}

impl<const NW: usize> BitAndAssign for Bitboard<NW> {
    #[inline]
    fn bitand_assign(&mut self, rhs: Bitboard<NW>) {
        let mut i = 0;
        while i < NW {
            self.words[i] &= rhs.words[i];
            i += 1;
        }
    }
}

impl<const NW: usize> BitOr for Bitboard<NW> {
    type Output = Bitboard<NW>;
    #[inline]
    fn bitor(self, rhs: Bitboard<NW>) -> Bitboard<NW> {
        let mut out = [0u64; NW];
        let mut i = 0;
        while i < NW {
            out[i] = self.words[i] | rhs.words[i];
            i += 1;
        }
        Bitboard { words: out }
    }
}

impl<const NW: usize> BitOrAssign for Bitboard<NW> {
    #[inline]
    fn bitor_assign(&mut self, rhs: Bitboard<NW>) {
        let mut i = 0;
        while i < NW {
            self.words[i] |= rhs.words[i];
            i += 1;
        }
    }
}

impl<const NW: usize> Not for Bitboard<NW> {
    type Output = Bitboard<NW>;
    #[inline]
    fn not(self) -> Bitboard<NW> {
        let mut out = [0u64; NW];
        let mut i = 0;
        while i < NW {
            out[i] = !self.words[i];
            i += 1;
        }
        Bitboard { words: out }
    }
}

/// Iterator over set-bit indices in a `Bitboard`.
pub struct BitIterator<const NW: usize> {
    words: [u64; NW],
    word_index: u8,
}

impl<const NW: usize> Iterator for BitIterator<NW> {
    type Item = usize;
    #[inline]
    fn next(&mut self) -> Option<usize> {
        while (self.word_index as usize) < NW {
            let wi = self.word_index as usize;
            let w = self.words[wi];
            if w != 0 {
                let bit = w.trailing_zeros() as usize;
                // Clear lowest set bit
                self.words[wi] = w & (w - 1);
                return Some(wi * 64 + bit);
            }
            self.word_index += 1;
        }
        None
    }
}

/// Precomputed masks for a given board geometry. Created once per Game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoardGeometry<const NW: usize> {
    pub width: u8,
    pub height: u8,
    pub area: u16,
    /// Mask with 1s at all valid board positions (indices 0..area).
    pub board_mask: Bitboard<NW>,
    /// board_mask minus column 0; applied after shift_left(1) to prevent wrap from col w-1 to col 0.
    pub not_col0: Bitboard<NW>,
    /// board_mask minus last column; applied after shift_right(1) to prevent wrap from col 0 to col w-1.
    pub not_col_last: Bitboard<NW>,

    // Connect 4 specific masks
    /// One mask per column — bits set for all rows in that column. Only indices 0..width are valid.
    pub column_masks: [Bitboard<NW>; 32],
    /// Bits set for the top row only (for board-full check).
    pub top_row_mask: Bitboard<NW>,
    /// Bits set for the bottom row only (row 0).
    pub bottom_row_mask: Bitboard<NW>,
}

impl<const NW: usize> BoardGeometry<NW> {
    /// Build geometry for a `width × height` board.
    pub fn new(width: u8, height: u8) -> Self {
        debug_assert!((2..=32).contains(&width));
        debug_assert!((2..=32).contains(&height));
        let area = width as u16 * height as u16;
        assert!(
            NW == (area as usize).div_ceil(64),
            "NW={} does not match board {}x{} (need {})",
            NW,
            width,
            height,
            (area as usize).div_ceil(64)
        );
        let w = width as usize;
        let h = height as usize;

        let mut board_mask = Bitboard::empty();
        for i in 0..area as usize {
            board_mask.set(i);
        }

        let mut not_col0 = board_mask;
        for row in 0..h {
            not_col0.clear(row * w); // column 0
        }

        let mut not_col_last = board_mask;
        for row in 0..h {
            not_col_last.clear(row * w + w - 1); // last column
        }

        // Column masks
        let mut column_masks = [Bitboard::empty(); 32];
        for (col, mask) in column_masks.iter_mut().enumerate().take(w) {
            *mask = Bitboard::empty();
            for row in 0..h {
                mask.set(row * w + col);
            }
        }

        // Top row mask
        let mut top_row_mask = Bitboard::empty();
        for col in 0..w {
            top_row_mask.set((h - 1) * w + col);
        }

        // Bottom row mask
        let mut bottom_row_mask = Bitboard::empty();
        for col in 0..w {
            bottom_row_mask.set(col);
        }

        BoardGeometry {
            width,
            height,
            area,
            board_mask,
            not_col0,
            not_col_last,
            column_masks,
            top_row_mask,
            bottom_row_mask,
        }
    }

    /// Compute the set of all orthogonal neighbors of every bit in `bb`.
    #[inline]
    pub fn neighbors(&self, bb: &Bitboard<NW>) -> Bitboard<NW> {
        let w = self.width as usize;

        let right = bb.shift_left(1) & self.not_col0;
        let left = bb.shift_right(1) & self.not_col_last;
        let up = bb.shift_left(w);
        let down = bb.shift_right(w);

        (right | left | up | down) & self.board_mask
    }

    /// Flood-fill from `seed` through `mask`. Returns the connected component
    /// of `seed` within `mask`.
    #[inline]
    pub fn flood_fill(&self, seed: Bitboard<NW>, mask: Bitboard<NW>) -> Bitboard<NW> {
        let mut filled = seed & mask;
        loop {
            let nbrs = self.neighbors(&filled);
            let expanded = (filled | nbrs) & mask;
            if expanded == filled {
                return filled;
            }
            filled = expanded;
        }
    }

    /// Check if a player's bitboard has four in a row in any direction.
    ///
    /// For each direction, we do 3 shift-and-AND steps, applying a column mask
    /// at each step to prevent bits from wrapping across row boundaries.
    #[inline]
    pub fn has_four_in_a_row(&self, bb: &Bitboard<NW>) -> bool {
        let w = self.width as usize;

        // Horizontal: shift_left(1) moves col c to col c+1.
        // A bit at col w-1 would wrap to col 0 of the next row, so mask with not_col0.
        {
            let s1 = bb.shift_left(1) & self.not_col0;
            let h2 = *bb & s1;
            let s2 = s1.shift_left(1) & self.not_col0;
            let h3 = h2 & s2;
            let s3 = s2.shift_left(1) & self.not_col0;
            let h4 = h3 & s3;
            if h4.is_nonzero() {
                return true;
            }
        }

        // Vertical: shift_left(w) moves row r to row r+1.
        // No column wrapping possible — pure row shift.
        {
            let s1 = bb.shift_left(w);
            let v2 = *bb & s1;
            let s2 = s1.shift_left(w);
            let v3 = v2 & s2;
            let s3 = s2.shift_left(w);
            let v4 = v3 & s3;
            if v4.is_nonzero() {
                return true;
            }
        }

        // Diagonal / (ascending): shift_left(w+1) moves (row, col) to (row+1, col+1).
        // The +1 col part can wrap, so mask with not_col0.
        {
            let s1 = bb.shift_left(w + 1) & self.not_col0;
            let d2 = *bb & s1;
            let s2 = s1.shift_left(w + 1) & self.not_col0;
            let d3 = d2 & s2;
            let s3 = s2.shift_left(w + 1) & self.not_col0;
            let d4 = d3 & s3;
            if d4.is_nonzero() {
                return true;
            }
        }

        // Diagonal \ (descending): shift_left(w-1) moves (row, col) to (row+1, col-1).
        // The -1 col part can wrap (col 0 -> col w-1 of same row before the row shift),
        // so mask with not_col_last.
        {
            let s1 = bb.shift_left(w - 1) & self.not_col_last;
            let d2 = *bb & s1;
            let s2 = s1.shift_left(w - 1) & self.not_col_last;
            let d3 = d2 & s2;
            let s3 = s2.shift_left(w - 1) & self.not_col_last;
            let d4 = d3 & s3;
            if d4.is_nonzero() {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let bb = Bitboard::<2>::empty();
        assert!(bb.is_empty());
        assert_eq!(bb.count(), 0);
        assert!(bb.lowest_bit_index().is_none());
    }

    #[test]
    fn test_single() {
        let bb = Bitboard::<16>::single(0);
        assert!(bb.get(0));
        assert!(!bb.get(1));
        assert_eq!(bb.count(), 1);
        assert_eq!(bb.lowest_bit_index(), Some(0));

        let bb2 = Bitboard::<16>::single(63);
        assert!(bb2.get(63));
        assert!(!bb2.get(62));
        assert!(!bb2.get(64));

        let bb3 = Bitboard::<16>::single(64);
        assert!(bb3.get(64));
        assert!(!bb3.get(63));

        let bb4 = Bitboard::<16>::single(1023);
        assert!(bb4.get(1023));
        assert_eq!(bb4.count(), 1);
    }

    #[test]
    fn test_set_clear() {
        let mut bb = Bitboard::<2>::empty();
        bb.set(100);
        assert!(bb.get(100));
        assert_eq!(bb.count(), 1);
        bb.clear(100);
        assert!(!bb.get(100));
        assert!(bb.is_empty());
    }

    #[test]
    fn test_bitwise_ops() {
        let a = Bitboard::<1>::single(5) | Bitboard::<1>::single(10);
        let b = Bitboard::<1>::single(10) | Bitboard::<1>::single(20);

        let and = a & b;
        assert!(and.get(10));
        assert!(!and.get(5));
        assert!(!and.get(20));

        let or = a | b;
        assert!(or.get(5));
        assert!(or.get(10));
        assert!(or.get(20));
    }

    #[test]
    fn test_shift_left() {
        let bb = Bitboard::<16>::single(0);
        let shifted = bb.shift_left(1);
        assert!(shifted.get(1));
        assert!(!shifted.get(0));

        // Cross word boundary: 63 -> 64
        let bb2 = Bitboard::<16>::single(63);
        let shifted2 = bb2.shift_left(1);
        assert!(shifted2.get(64));
        assert!(!shifted2.get(63));

        // Cross word boundary: 127 -> 128
        let bb3 = Bitboard::<16>::single(127);
        let shifted3 = bb3.shift_left(1);
        assert!(shifted3.get(128));
        assert!(!shifted3.get(127));
    }

    #[test]
    fn test_shift_right() {
        let bb = Bitboard::<16>::single(1);
        let shifted = bb.shift_right(1);
        assert!(shifted.get(0));
        assert!(!shifted.get(1));

        // Cross word boundary: 64 -> 63
        let bb2 = Bitboard::<16>::single(64);
        let shifted2 = bb2.shift_right(1);
        assert!(shifted2.get(63));
        assert!(!shifted2.get(64));

        // Shift from 0 -> lost
        let bb3 = Bitboard::<16>::single(0);
        let shifted3 = bb3.shift_right(1);
        assert!(shifted3.is_empty());
    }

    #[test]
    fn test_shift_by_width() {
        // Simulate shift by width=9 (row shift on 9x9 board)
        let bb = Bitboard::<2>::single(4); // col=4, row=0
        let shifted = bb.shift_left(9);
        assert!(shifted.get(13)); // col=4, row=1
        assert!(!shifted.get(4));
    }

    #[test]
    fn test_iter_ones() {
        let bb = Bitboard::<4>::single(3) | Bitboard::<4>::single(64) | Bitboard::<4>::single(200);
        let indices: Vec<usize> = bb.iter_ones().collect();
        assert_eq!(indices, vec![3, 64, 200]);
    }

    #[test]
    fn test_iter_ones_empty() {
        let bb = Bitboard::<2>::empty();
        let indices: Vec<usize> = bb.iter_ones().collect();
        assert!(indices.is_empty());
    }

    #[test]
    fn test_not() {
        let bb = Bitboard::<1>::single(5);
        let notbb = !bb;
        assert!(!notbb.get(5));
        assert!(notbb.get(0));
        assert!(notbb.get(6));
    }

    #[test]
    fn test_andnot() {
        let a = Bitboard::<1>::single(0) | Bitboard::single(5) | Bitboard::single(10);
        let b = Bitboard::<1>::single(5) | Bitboard::single(20);
        let result = a.andnot(b);
        assert!(result.get(0));
        assert!(!result.get(5));
        assert!(result.get(10));
        assert!(!result.get(20));
    }

    #[test]
    fn test_assign_ops() {
        let mut bb = Bitboard::<1>::single(1);
        bb |= Bitboard::single(2);
        assert!(bb.get(1));
        assert!(bb.get(2));

        bb &= Bitboard::single(2);
        assert!(!bb.get(1));
        assert!(bb.get(2));
    }

    #[test]
    fn test_nw_values() {
        assert_eq!(nw_for_board(2, 2), 1); // 4 bits
        assert_eq!(nw_for_board(5, 5), 1); // 25 bits
        assert_eq!(nw_for_board(7, 6), 1); // 42 bits (standard Connect 4)
        assert_eq!(nw_for_board(8, 8), 1); // 64 bits
        assert_eq!(nw_for_board(9, 9), 2); // 81 bits
        assert_eq!(nw_for_board(19, 19), 6); // 361 bits
        assert_eq!(nw_for_board(32, 32), 16); // 1024 bits
    }

    #[test]
    fn test_geometry_7x6() {
        let geo = BoardGeometry::<{ nw_for_board(7, 6) }>::new(7, 6);
        assert_eq!(geo.area, 42u16);
        assert_eq!(geo.board_mask.count(), 42);

        // Column masks
        for col in 0..7 {
            assert_eq!(geo.column_masks[col].count(), 6); // 6 rows per column
        }

        // Top row mask
        assert_eq!(geo.top_row_mask.count(), 7); // 7 columns in top row

        // Bottom row mask
        assert_eq!(geo.bottom_row_mask.count(), 7);
    }

    #[test]
    fn test_has_four_horizontal() {
        let geo = BoardGeometry::<{ nw_for_board(7, 6) }>::new(7, 6);
        let w = 7usize;

        // Place 4 consecutive in row 0: cols 0,1,2,3
        let mut bb = Bitboard::empty();
        for col in 0..4 {
            bb.set(0 * w + col); // row 0
        }
        assert!(geo.has_four_in_a_row(&bb));

        // Only 3 consecutive — no win
        let mut bb3 = Bitboard::empty();
        for col in 0..3 {
            bb3.set(0 * w + col);
        }
        assert!(!geo.has_four_in_a_row(&bb3));
    }

    #[test]
    fn test_has_four_vertical() {
        let geo = BoardGeometry::<{ nw_for_board(7, 6) }>::new(7, 6);
        let w = 7usize;

        // Place 4 consecutive in col 0: rows 0,1,2,3
        let mut bb = Bitboard::empty();
        for row in 0..4 {
            bb.set(row * w + 0);
        }
        assert!(geo.has_four_in_a_row(&bb));
    }

    #[test]
    fn test_has_four_diagonal_ascending() {
        let geo = BoardGeometry::<{ nw_for_board(7, 6) }>::new(7, 6);
        let w = 7usize;

        // Ascending diagonal: (0,0), (1,1), (2,2), (3,3)
        let mut bb = Bitboard::empty();
        for i in 0..4 {
            bb.set(i * w + i); // row i, col i
        }
        assert!(geo.has_four_in_a_row(&bb));
    }

    #[test]
    fn test_has_four_diagonal_descending() {
        let geo = BoardGeometry::<{ nw_for_board(7, 6) }>::new(7, 6);
        let w = 7usize;

        // Descending diagonal: (0,3), (1,2), (2,1), (3,0)
        let mut bb = Bitboard::empty();
        for i in 0..4 {
            bb.set(i * w + (3 - i)); // row i, col 3-i
        }
        assert!(geo.has_four_in_a_row(&bb));
    }

    #[test]
    fn test_no_wraparound_horizontal() {
        let geo = BoardGeometry::<{ nw_for_board(7, 6) }>::new(7, 6);
        let w = 7usize;

        // Place pieces at end of row 0 and start of row 1
        // cols 5,6 of row 0 + cols 0,1 of row 1 — should NOT be a win
        let mut bb = Bitboard::empty();
        bb.set(0 * w + 5);
        bb.set(0 * w + 6);
        bb.set(1 * w + 0);
        bb.set(1 * w + 1);
        assert!(!geo.has_four_in_a_row(&bb));
    }

    #[test]
    fn test_no_wraparound_diagonal() {
        let geo = BoardGeometry::<{ nw_for_board(7, 6) }>::new(7, 6);
        let w = 7usize;

        // Ascending diagonal that would wrap: (0,5), (1,6), (2,0), (3,1)
        // This should NOT be detected as a win
        let mut bb = Bitboard::empty();
        bb.set(0 * w + 5);
        bb.set(1 * w + 6);
        bb.set(2 * w + 0);
        bb.set(3 * w + 1);
        assert!(!geo.has_four_in_a_row(&bb));
    }
}
