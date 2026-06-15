//! Tiny deterministic PRNG (SplitMix64).
//!
//! Self-contained (no `rand` dependency) and fully seedable, so leaf
//! generation is reproducible — which also makes "determinism given a seed" a
//! clean invariant to state once we start proving.

use crate::Scalar;

pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in [0, 1).
    pub fn next_f64(&mut self) -> Scalar {
        (self.next_u64() >> 11) as Scalar / ((1u64 << 53) as Scalar)
    }

    /// Uniform in [lo, hi).
    pub fn range(&mut self, lo: Scalar, hi: Scalar) -> Scalar {
        lo + (hi - lo) * self.next_f64()
    }
}
