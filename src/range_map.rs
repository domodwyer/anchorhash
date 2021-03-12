/// An efficient modulo-like operation mapping `v` into the range `[0, max)` for
/// modern 64-bit CPUs.
///
/// Algorithm taken from Daniel Lemire's [`Fast Random Integer Generation in an
/// Interval`] without the rejection method, therefore accepting a bias in the
/// result.
///
/// Benchmarks (included in this crate) showed this implementation to be ~70%
/// faster than the (already very fast) modulo implementation.
///
/// [`Fast Random Integer Generation in an Interval`]: https://arxiv.org/abs/1805.10941
#[cfg(all(target_pointer_width = "64", feature = "fastmod"))]
pub fn range_map(v: u32, max: u32) -> u32 {
    debug_assert_ne!(max, 0);
    ((v as u64 * max as u64) >> 32) as u32
}

/// A 32-bit replacement for Daniel Lemire's [`Fast Random Integer Generation in
/// an Interval`] used on 64-bit CPUs.
///
/// Computed as `v % max`, including the result bias.
///
/// [`Fast Random Integer Generation in an Interval`]: https://arxiv.org/abs/1805.10941
#[cfg(not(all(target_pointer_width = "64", feature = "fastmod")))]
pub fn range_map(v: u32, max: u32) -> u32 {
    v % max
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn test_range_map(a: u32, b: u32) -> bool {
        if b == 0 {
            return true;
        }
        let got = range_map(a, b);
        (0..b).contains(&got)
    }
}
