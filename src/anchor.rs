use std::ops::Deref;

use crate::fasthash;

use super::range_map;

pub(crate) enum Bucket {
    Original(u16),
    Remapped(u16)
}

impl Deref for Bucket {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        match self {
            Bucket::Original(v) => v,
            Bucket::Remapped(v) => v
        }
    }
}

/// Anchor is an implementation of Algorithm 3 from the AnchorHash paper.
///
/// This type is responsible for the consistent mapping of keys to buckets, and
/// managing the state of the buckets by adding and removing.
#[derive(Debug, Clone)]
#[allow(non_snake_case)]
pub(crate) struct Anchor {
    capacity: u16,

    // A contains the set of all buckets within the Anchor (either working, or
    // unused), and is said to be of size `a`.
    //
    // For b ∈ {0, 1, ..., a−1} all values of A[b] equal either 0 for a working
    // bucket (A[b] = 0 if b ∈ W) or A[b] equals the size of W immediately after
    // b is removed (A[b] = |Wb| if b ∈ R).
    A: Vec<u16>,

    // R is a LIFO stack tracking the order of removed buckets.
    //
    // When a bucket is removed from the Anchor, it is pushed to R. When a new
    // bucket is to be added, the last removed bucket is popped from R
    // preserving the LIFO order of bucket removal.
    R: Vec<u16>,

    // The number of working buckets (|W|).
    N: u16,

    // The array of working buckets in order.
    W: Vec<u16>,

    // K stores the successor for each removed bucket b (i.e. the bucket that
    // replaced it in W).
    K: Vec<u16>,

    // L stores the most recent location for each bucket within W.
    L: Vec<u16>,
}

impl Anchor {
    /// Initialise a new Anchor with a maximum of `capacity` resources and mark
    /// `working` number of buckets as active.
    ///
    /// # Panics
    ///
    /// This method panics if `working > capacity`.
    pub(crate) fn new(capacity: u16, working: u16) -> Self {
        assert!(
            working <= capacity,
            "working bucket count must not exceed capacity"
        );

        let mut anchor = Self {
            capacity,
            A: vec![0; capacity as _],
            R: (working..capacity).rev().collect(),
            N: working,

            K: (0..capacity).into_iter().collect(),
            L: (0..capacity).into_iter().collect(),
            W: (0..capacity).into_iter().collect(),
        };

        for b in working..capacity {
            anchor.A[b as usize] = b;
        }

        anchor
    }

    /// Resolve the hash `k` to a bucket.
    ///
    /// ```text
    /// GETBUCKET(k)
    ///   b←hash(k) mod a               // can use k if calling through wrapper as it is already hash(key)
    ///   while A[b]>0 do               // b is removed
    ///     h←h_b(k)                    // h←hash(b,k) mod A[b] OR k←rand(seed=k), h←k mod A[b]
    ///     while A[h]≥A[b] do          // W_b[h] != h, b removed prior to h
    ///       h←K[h]                    // search for W_b[h]
    ///     b←h                         // b←H_W_b(k)
    ///   return b
    /// ```
    pub(crate) fn get_bucket(&self, k: u32) -> Bucket {
        // Map the (already hashed) key into the range [0, capacity)
        let mut b = range_map(k, self.capacity as u32) as usize;
        let mut remapped = false;

        // While b is removed
        while self.A[b] > 0 {
            // Map to a bucket in Wb (W just after b has been removed).
            //
            // Incorporate both the key, and the bucket it mapped to for better
            // balance.
            //
            //  h ← hash(b, k) mod A[b]
            let bs = fasthash(b as u32, k);
            let mut h = range_map(bs, self.A[b] as u32);

            // Wb[h] != h (b removed prior to h)
            while self.A[h as usize] >= self.A[b] {
                remapped = true;
                // search for Wb[h]
                h = self.K[h as usize] as _;
            }

            // b ← HWb(k)
            b = h as _;
        }
        if remapped {
            Bucket::Remapped(b as _)
        } else {
            Bucket::Original(b as _)
        }
    }

    /// Add a new bucket to the anchor.
    ///
    /// This method returns `None` if the capacity of the Anchor has been
    /// reached.
    ///
    /// ```text
    /// ADDBUCKET( )
    ///   b←R.pop()
    ///   A[b]←0                        // W←W ∪ {b}, delete W_b
    ///   L[W[N]]←N
    ///   W[L[b]]←K[b]←b
    ///   N←N+ 1
    ///   return b
    /// ```
    pub(crate) fn add_bucket(&mut self) -> Option<u16> {
        // Restore the last removed bucket
        let b = self.R.pop()? as usize;

        // W ← W ∪ {b}, delete Wb
        self.A[b] = 0;

        // L[W[N]] ← N
        self.L[self.W[self.N as usize] as usize] = self.N;

        // W[L[b]] ← K[b] ← b
        self.W[self.L[b] as usize] = b as _;
        self.K[b] = b as _;

        // N ← N + 1
        self.N += 1;

        Some(b as u16)
    }

    /// Remove bucket b from the Anchor.
    ///
    /// # Panics
    ///
    /// This method panics if `b` is not an active bucket.
    ///
    /// ```text
    /// REMOVEBUCKET(b)
    ///   R.push(b)
    ///   N←N−1
    ///   A[b]←N                        // W_b←W\b, A[b]←|W_b|
    ///   W[L[b]]←K[b]←W[N]
    ///   L[W[N]]←L[b]
    /// ```
    pub(crate) fn remove_bucket(&mut self, b: u16) {
        // Can only remove in-use buckets
        assert_eq!(self.A[b as usize], 0);

        self.R.push(b);
        let b = b as usize;

        // N ← N − 1
        self.N -= 1;

        // Wb ← W\b, A[b] ← |W_b|
        self.A[b] = self.N;

        // W[L[b]] ← K[b] ← W[N]
        self.W[self.L[b] as usize] = self.W[self.N as usize];
        self.K[b] = self.W[self.N as usize];

        // L[W[N]] ← L[b]
        self.L[self.W[self.N as usize] as usize] = self.L[b];
    }

    // Return the set of working buckets.
    #[cfg(test)]
    pub(crate) fn working_buckets(&self) -> Vec<u16> {
        // A[0] == 0 at init with an empty Anchor.
        if self.N == 0 {
            return Vec::new();
        }
        let w = self
            .A
            .iter()
            .enumerate()
            .filter(|(_i, &v)| v == 0)
            .map(|(i, _v)| i as u16)
            .collect::<Vec<u16>>();

        w
    }
}

#[cfg(test)]
mod tests {
    use std::{
        cmp::{max, min},
        collections::HashSet,
    };

    use super::*;
    use hashbrown::HashMap;
    use quickcheck_macros::quickcheck;

    #[test]
    fn test_init_empty() {
        const WANT_SIZE: usize = 20;

        let a = Anchor::new(WANT_SIZE as _, 0);
        assert_eq!(a.A.len(), WANT_SIZE);
        assert!(a.A.iter().enumerate().all(|(i, &v)| i == v as usize));

        assert_eq!(a.R.len(), WANT_SIZE); // Fully unused
        assert_eq!(a.N, 0);

        assert_eq!(a.K.len(), WANT_SIZE);
        assert_eq!(a.L.len(), WANT_SIZE);
        assert_eq!(a.W.len(), WANT_SIZE);

        for i in 0..WANT_SIZE {
            assert_eq!(a.K[i], i as u16);
            assert_eq!(a.L[i], i as u16);
            assert_eq!(a.W[i], i as u16);
        }
    }

    #[test]
    fn test_init_populated() {
        const WANT_SIZE: usize = 20;
        const WORKING: usize = 15;

        let a = Anchor::new(WANT_SIZE as _, WORKING as _);
        assert_eq!(a.A.len(), WANT_SIZE);

        // Assert all working buckets are 0
        assert!(a.A.iter().take(WORKING).all(|&v| v == 0));

        // Assert all non-working buckets are populated with their bucket index
        for (&i, v) in a.A.iter().skip(WORKING).zip(WORKING..) {
            assert_eq!(i, v as u16);
        }

        // Assert the stack contains the 5 end buckets
        assert_eq!(a.R, vec![19, 18, 17, 16, 15]);
        assert_eq!(a.R.len(), WANT_SIZE - WORKING);

        // Assert N contains the number of working buckets
        assert_eq!(a.N, WORKING as u16);

        // Assert the sizes of K, L and W match, before checking their values
        // below
        assert_eq!(a.K.len(), WANT_SIZE);
        assert_eq!(a.L.len(), WANT_SIZE);
        assert_eq!(a.W.len(), WANT_SIZE);

        for i in 0..WANT_SIZE {
            assert_eq!(a.K[i], i as u16);
            assert_eq!(a.L[i], i as u16);
            assert_eq!(a.W[i], i as u16);
        }
    }

    #[test]
    fn test_add_bucket_full_anchor() {
        const SIZE: u16 = 20;
        let mut a = Anchor::new(SIZE, SIZE);
        if a.add_bucket().is_some() {
            panic!("adding bucket to full anchor should fail");
        }
    }

    #[quickcheck]
    fn test_get_returns_working_buckets(mut keys: Vec<u16>) -> bool {
        let num_buckets = match keys.pop() {
            Some(0) => return true,
            Some(v) => v,
            None => return true,
        };

        let mut a = Anchor::new(num_buckets, 0);

        // Add num_buckets to a, recording the buckets returned in working.
        let mut working = HashSet::with_capacity(num_buckets as _);
        for _i in 0..num_buckets {
            let b = a.add_bucket().unwrap();
            working.insert(b);
        }

        // Now start hashing keys. They MUST hash to working buckets.
        for k in keys {
            let got = a.get_bucket(k as _);
            if !working.contains(&got) {
                return false;
            }

            // Remove the bucket to drive the removal logic.
            if working.len() > 1 {
                a.remove_bucket(*got);
                assert!(working.remove(&got));
            }
        }

        true
    }

    #[test]
    fn test_bucket_balance() {
        use rand::prelude::*;
        let mut rng = rand::thread_rng();

        /// The number of working buckets.
        ///
        /// MUST be (significantly) smaller than the number of keys.
        const WORKING_BUCKETS: u16 = 10;

        /// The number of keys to hash into the working buckets in order to
        /// measure the balance.
        const KEYS: usize = 10_000;

        let a = Anchor::new(200, WORKING_BUCKETS);

        // Record which buckets see hits
        let mut seen = HashMap::new();
        for _ in 0..KEYS {
            let k = rng.gen();
            let got = a.get_bucket(k);
            let counter = seen.entry(*got).or_insert(0);
            *counter += 1;
        }

        // All working buckets must be used
        assert_eq!(seen.len(), WORKING_BUCKETS as _);

        // All buckets are roughly balanced
        let mut got_min = 0;
        let mut got_max = 0;
        for (k, v) in seen.into_iter() {
            println!("bucket {} has {} hits", k, v);
            got_min = min(v, got_min);
            got_max = max(v, got_max);
        }

        if (f64::from(got_max) * 0.9) < f64::from(got_min) {
            panic!(
                "expected max bucket hits ({}) to be within 10% of min bucket hits ({})",
                got_max, got_min
            );
        }
    }
}
