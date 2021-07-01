use std::{marker::PhantomData, mem, ptr};

const BITS_PER_WORD: usize = mem::size_of::<usize>() * 8;
const BIT_INDEX_MASK: usize = BITS_PER_WORD - 1;
const WORD_INDEX_SHIFTS: u32 = BIT_INDEX_MASK.count_ones();

/// A heavily optimized bitset for prime sieve.
pub struct BitSet {
    /// The words storing bits.
    words: Vec<usize>,
    /// The last word with its used bits set.
    last_word_set: usize,
}

impl BitSet {
    /// Creates a new `BitSet` with the given length and initial value.
    #[inline]
    pub fn new(len: usize, initial_v: bool) -> Self {
        assert!(len != 0, "empty bitset");

        let words = ((len - 1) >> WORD_INDEX_SHIFTS) + 1;
        let last_word_set = {
            let last_bit_i = (len - 1) & BIT_INDEX_MASK;
            !((!0 - 1) << last_bit_i)
        };

        let words = if initial_v {
            let mut v = Vec::with_capacity(words);

            let ptr: *mut usize = v.as_mut_ptr();
            unsafe {
                ptr::write_bytes(ptr, 0xFF, words - 1);
                *ptr.add(words - 1) = last_word_set;
                v.set_len(words);
            }
            v
        } else {
            vec![0; words]
        };
        Self {
            words,
            last_word_set,
        }
    }

    #[inline]
    fn locate(&self, i: usize) -> (usize, usize) {
        let word_i = i >> WORD_INDEX_SHIFTS;
        debug_assert!(word_i < self.words.len(), "index out of bounds");
        let mask = 1 << (i & BIT_INDEX_MASK);
        debug_assert!(self.last_word_set & mask != 0, "index out of bounds");
        (word_i, mask)
    }

    /// Sets a bit.
    #[inline]
    pub unsafe fn set(&mut self, i: usize) {
        let (word_i, mask) = self.locate(i);
        *self.words.get_unchecked_mut(word_i) |= mask;
    }

    /// Clears a bit.
    #[inline]
    pub unsafe fn clear(&mut self, i: usize) {
        let (word_i, mask) = self.locate(i);
        *self.words.get_unchecked_mut(word_i) &= !mask;
    }

    /// Gets a bit.
    #[inline]
    pub unsafe fn get(&self, i: usize) -> bool {
        let (word_i, mask) = self.locate(i);
        *self.words.get_unchecked(word_i) & mask != 0
    }

    /// Sets all bits.
    #[inline]
    pub fn set_all(&mut self) {
        let ptr = self.words.as_mut_ptr();
        let len = self.words.len();
        unsafe {
            ptr::write_bytes(ptr, 0xFF, len - 1);
            *ptr.add(len - 1) = self.last_word_set;
        }
    }

    /// Clears all bits.
    #[inline]
    pub fn clear_all(&mut self) {
        self.words.fill(0);
    }

    /// Shortens the bitset.
    ///
    /// # Safety
    /// - `len` must be non-zero, and less than or equal to the length of the bitset.
    /// - All bits should be cleared or set afterwards, since some unused bits might still be set after truncating.
    #[inline]
    pub unsafe fn truncate(&mut self, len: usize) {
        debug_assert!(len != 0, "empty bitset");
        let words = ((len - 1) >> WORD_INDEX_SHIFTS) + 1;
        debug_assert!(words <= self.words.len(), "index out of bounds");

        let last_word_set = {
            let last_bit_i = (len - 1) & BIT_INDEX_MASK;
            !((!0 - 1) << last_bit_i)
        };

        self.words.truncate(words);
        self.last_word_set = last_word_set;
    }

    /// Returns the number of ones in the bitset.
    #[inline]
    pub fn count_ones(&self) -> usize {
        self.words
            .iter()
            .map(|word| word.count_ones() as usize)
            .sum()
    }

    /// Returns an iterator over the indexes of ones in the bitset.
    #[inline]
    pub fn iter_ones(&self) -> IterOnes<'_> {
        let ptr = self.words.as_ptr();
        IterOnes {
            ptr,
            end: unsafe { ptr.add(self.words.len()) },
            word: unsafe { *ptr },
            i: 0,
            _marker: PhantomData,
        }
    }
}

/// An iterator over the indexes of ones in a bitset.
///
/// Reference: [Really fast bitset decoding for “average” densities, Daniel Lemire][1]
///
/// [1]: https://lemire.me/blog/2019/05/03/really-fast-bitset-decoding-for-average-densities/
pub struct IterOnes<'a> {
    ptr: *const usize,
    end: *const usize,
    word: usize,
    i: usize,
    _marker: PhantomData<&'a usize>,
}

impl<'a> Iterator for IterOnes<'a> {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<usize> {
        while self.word == 0 {
            unsafe {
                let next = self.ptr.add(1);
                if next == self.end {
                    return None;
                }
                self.ptr = next;
                self.word = *next;
            }
            self.i += 1 << WORD_INDEX_SHIFTS;
        }

        let res = self.i | self.word.trailing_zeros() as usize;
        self.word &= self.word - 1;
        Some(res)
    }
}
