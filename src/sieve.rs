use super::bitset::BitSet;

/// A trait for prime sieves.
pub trait Sieve {
    fn prime_pi(n: usize) -> usize;
}

/// The sieve of Eratosthenes.
pub struct Eratosthenes;

impl Eratosthenes {
    pub fn gen_table(n: usize) -> BitSet {
        let mut table = BitSet::new(n + 1, true);
        unsafe {
            table.clear(0);
            table.clear(1);
        }

        let (mut i, mut i_squared) = (2, 4);

        while i_squared <= n {
            let is_prime = unsafe { table.get(i) };
            if is_prime {
                let mut j = i_squared;
                loop {
                    unsafe { table.clear(j) }
                    j += i;
                    if j > n {
                        break;
                    }
                }
            }
            // (i+1)^2=i^2+2i+1
            i_squared += (i << 1) | 1;
            i += 1;
        }
        table
    }
}

impl Sieve for Eratosthenes {
    fn prime_pi(n: usize) -> usize {
        if n < 2 {
            return 0;
        }
        Self::gen_table(n).count_ones()
    }
}

/// The segmented sieve of Eratosthenes.
///
/// TODO: Find the threshold of segmentation.
pub struct SegmentedEratosthenes;

impl Sieve for SegmentedEratosthenes {
    fn prime_pi(n: usize) -> usize {
        if n < 2 {
            return 0;
        }
        let mut res = 0;
        let mut seg_len = int_sqrt(n);

        let mut seg = Eratosthenes::gen_table(seg_len);
        let primes = collect_primes(&seg, seg_len);
        res += primes.len();

        unsafe { seg.truncate(seg_len) }
        seg.set_all();

        let (mut low, mut high) = (seg_len + 1, seg_len << 1);
        loop {
            let max = int_sqrt(high) as u32;
            unsafe { for_each_max(&primes, max, |p| mark_non_primes(&mut seg, p, low, seg_len)) }
            res += seg.count_ones();

            low = high + 1;
            if low > n {
                break;
            }
            high = {
                let next = high + seg_len;
                if next > n {
                    seg_len = n - low + 1;
                    unsafe { seg.truncate(seg_len) }
                    n
                } else {
                    next
                }
            };
            seg.set_all();
        }
        res
    }
}

/// Returns the integer square root of `n`.
///
/// Reference: [Wikipedia](https://en.wikipedia.org/wiki/Integer_square_root)
#[inline]
fn int_sqrt(n: usize) -> usize {
    let mut x0 = n >> 1;
    if x0 != 0 {
        let mut x1 = (x0 + n / x0) >> 1;
        while x1 < x0 {
            x0 = x1;
            x1 = (x0 + n / x0) >> 1;
        }
        x0
    } else {
        n
    }
}

/// Collects the primes, i.e. the ones in a segment.
///
/// A dummy element is added at the end of the result (which is
/// not safely accessible), in order to avoid bounds check.
#[inline]
fn collect_primes(seg: &BitSet, seg_len: usize) -> Vec<u32> {
    let len = seg.count_ones();
    let mut res = Vec::with_capacity(len + 1);
    unsafe {
        let mut ptr = res.as_mut_ptr();
        for p in seg.iter_ones() {
            *ptr = p as u32;
            ptr = ptr.add(1);
        }
        *ptr = (seg_len + 1) as u32;
        res.set_len(len);
    }
    res
}

/// Calls `f` with each element <= `max` in the result of `collect_primes`.
#[inline]
unsafe fn for_each_max(primes: &[u32], max: u32, mut f: impl FnMut(u32)) {
    let mut ptr = primes.as_ptr();
    loop {
        let cur = *ptr;
        if cur > max {
            break;
        }
        f(cur);
        ptr = ptr.add(1);
    }
}

/// Marks multiples of `p` as non-primes in a segment.
#[inline]
fn mark_non_primes(seg: &mut BitSet, p: u32, low: usize, seg_len: usize) {
    let p = p as usize;
    let mut p_mul = low / p * p;
    if p_mul < low {
        p_mul += p;
    }

    let mut i = p_mul - low;
    while i < seg_len {
        unsafe { seg.clear(i) }
        i += p;
    }
}
