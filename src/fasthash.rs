/// A hash function producing a 32 bit hash for `k`, using `seed` as the initial
/// hasher state.
///
/// This implementation makes use of the [`_mm_crc32_u32`] intrinsic available
/// on x86_46 platforms that support SSE4.2 or higher.
///
/// The non-simd fallback implementation uses the [Fowler–Noll–Vo hash] and can
/// be used by disabling the `simd` crate feature.
///
/// [`_mm_crc32_u32`]: https://software.intel.com/sites/landingpage/IntrinsicsGuide/#text=_mm_crc32_u32&expand=1287
/// [Fowler–Noll–Vo hash]: http://www.isthe.com/chongo/tech/comp/fnv/index.html
#[cfg(all(target_arch = "x86_64", target_feature = "sse4.2", feature = "simd"))]
pub fn fasthash(k: u32, seed: u32) -> u32 {
    unsafe { std::arch::x86_64::_mm_crc32_u32(seed, k) }
}

/// A hash function producing a 32 bit hash for `k`, using `seed` as the initial
/// hasher state.
///
/// This is a fallback implementation for platforms that do not support the
/// [`_mm_crc32_u32`] intrinsic. It makes use of the [Fowler–Noll–Vo hash]
/// function which is extremely quick at hashing small amounts of data.
///
/// [`_mm_crc32_u32`]: https://software.intel.com/sites/landingpage/IntrinsicsGuide/#text=_mm_crc32_u32&expand=1287
/// [Fowler–Noll–Vo hash]: http://www.isthe.com/chongo/tech/comp/fnv/index.html
#[cfg(not(all(target_arch = "x86_64", target_feature = "sse4.2", feature = "simd")))]
pub fn fasthash(k: u32, seed: u32) -> u32 {
    use fnv::FnvHasher;
    use std::hash::Hasher;

    let mut h = FnvHasher::with_key(seed.into());
    h.write_u32(k);
    h.finish() as u32 // Truncate down to u32, discarding 32 bits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_impl() {
        let a = fasthash(42, 24);
        let b = fasthash(13, 31);

        assert_ne!(a, b);
    }
}
