//! A small, seedable pseudo-random generator, so runs repeat exactly.

/// `SplitMix64` (Steele, Lea & Flood, 2014): one 64-bit state, passes `BigCrush`, and is
/// more than enough to pick mutation partners and crossover masks.
#[derive(Debug, Clone)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// A generator from a seed; equal seeds give equal sequences.
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// The next 64 random bits.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// Uniform in `[0, 1)`, from the top 53 bits.
    pub fn uniform(&mut self) -> f64 {
        // Exact: both values fit in the 53-bit mantissa.
        #[allow(clippy::cast_precision_loss)]
        let (top, scale) = ((self.next_u64() >> 11) as f64, (1u64 << 53) as f64);
        top / scale
    }

    /// Uniform in `0..n`, for `n > 0`.
    pub fn below(&mut self, n: usize) -> usize {
        let n64 = u64::try_from(n).unwrap_or(u64::MAX);
        // Lemire's multiply-shift; the bias is below 2⁻³² for any population size used here.
        let wide = u128::from(self.next_u64()) * u128::from(n64);
        usize::try_from(wide >> 64).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeatable_and_in_range() {
        let (mut a, mut b) = (Rng::new(1), Rng::new(1));
        for _ in 0..1000 {
            assert_eq!(a.next_u64(), b.next_u64());
            let u = a.uniform();
            assert!((0.0..1.0).contains(&u));
            assert!(a.below(7) < 7);
            b.uniform();
            b.below(7);
        }
    }
}
