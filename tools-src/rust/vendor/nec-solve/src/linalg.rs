//! LU factors that can be kept, written to a file and used again (NEC's `FACTRS`/`SOLVES`).
//!
//! A numerical Green's function is, above all, a factored matrix: the expensive part of a solve,
//! done once and saved so that later runs can add to the structure without repeating it. This
//! is that matrix, held as the packed factors of `P·A = L·U` (unit lower `L` below the diagonal,
//! `U` on and above it) with the row permutation.

use faer::dyn_stack::{MemBuffer, MemStack};
use faer::linalg::lu::partial_pivoting::factor::{lu_in_place, lu_in_place_scratch};
use faer::linalg::triangular_solve::{
    solve_unit_lower_triangular_in_place, solve_upper_triangular_in_place,
};
use faer::{Mat, Par};
use num_complex::Complex64;

type C = Complex64;

/// LU factors of a square complex matrix, with partial pivoting.
#[derive(Debug, Clone, PartialEq)]
pub struct Factors {
    /// `L` strictly below the diagonal (its unit diagonal implied), `U` on and above it.
    lu: Mat<C>,
    /// Row `i` of `P·A` is row `perm[i]` of `A`.
    perm: Vec<usize>,
}

impl Factors {
    /// Factor `a`.
    #[must_use]
    pub fn new(a: &Mat<C>) -> Self {
        let n = a.nrows();
        let mut lu = a.clone();
        let mut perm = vec![0usize; n];
        let mut perm_inv = vec![0usize; n];
        let par = parallelism(n);
        let req = lu_in_place_scratch::<usize, C>(n, n, par, Default::default());
        let mut buf = MemBuffer::new(req);
        lu_in_place(
            lu.as_mut(),
            &mut perm,
            &mut perm_inv,
            par,
            MemStack::new(&mut buf),
            Default::default(),
        );
        Self { lu, perm }
    }

    /// The order of the matrix.
    #[must_use]
    pub fn len(&self) -> usize {
        self.perm.len()
    }

    /// Whether the matrix is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.perm.is_empty()
    }

    /// Solve `A·X = B` for every column of `b`.
    #[must_use]
    pub fn solve(&self, b: &Mat<C>) -> Mat<C> {
        let mut x = Mat::from_fn(b.nrows(), b.ncols(), |i, j| b[(self.perm[i], j)]);
        let par = parallelism(self.len());
        solve_unit_lower_triangular_in_place(self.lu.as_ref(), x.as_mut(), par);
        solve_upper_triangular_in_place(self.lu.as_ref(), x.as_mut(), par);
        x
    }

    /// The factors as bytes: the order, the permutation, then the packed matrix column by
    /// column, every number little-endian.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let n = self.len();
        let mut out = Vec::with_capacity(8 + 8 * n + 16 * n * n);
        out.extend_from_slice(&(n as u64).to_le_bytes());
        for &p in &self.perm {
            out.extend_from_slice(&(p as u64).to_le_bytes());
        }
        for j in 0..n {
            for i in 0..n {
                let z = self.lu[(i, j)];
                out.extend_from_slice(&z.re.to_le_bytes());
                out.extend_from_slice(&z.im.to_le_bytes());
            }
        }
        out
    }

    /// Read factors written by [`Factors::to_bytes`], returning them and the bytes left over.
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Option<(Self, &[u8])> {
        let (n, mut rest) = take_u64(bytes)?;
        let n = usize::try_from(n).ok()?;
        let mut perm = Vec::with_capacity(n);
        for _ in 0..n {
            let (p, r) = take_u64(rest)?;
            let p = usize::try_from(p).ok().filter(|&p| p < n)?;
            perm.push(p);
            rest = r;
        }
        let need = n.checked_mul(n)?.checked_mul(16)?;
        if rest.len() < need {
            return None;
        }
        let (data, rest) = rest.split_at(need);
        let f =
            |k: usize| f64::from_le_bytes(data[8 * k..8 * k + 8].try_into().expect("eight bytes"));
        let lu = Mat::from_fn(n, n, |i, j| {
            let k = 2 * (j * n + i);
            C::new(f(k), f(k + 1))
        });
        Some((Self { lu, perm }, rest))
    }
}

/// Threads for the factorization and solve. Below about a thousand unknowns the coordination
/// costs more than the arithmetic (measured: 315 unknowns factor in 2 ms serially, 15 ms on 32
/// threads); above it a handful of threads help and more contend for memory bandwidth (2100
/// unknowns: 524 ms serially, 156 ms on 8 threads, 354 ms on 32).
#[cfg(target_arch = "wasm32")]
fn parallelism(_n: usize) -> Par {
    Par::Seq // cem_tool_website: single-threaded in the browser
}

#[cfg(not(target_arch = "wasm32"))]
fn parallelism(n: usize) -> Par {
    const MAX_THREADS: usize = 8;
    if n < 1024 {
        Par::Seq
    } else {
        let cores = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
        Par::rayon(cores.min(MAX_THREADS))
    }
}

fn take_u64(bytes: &[u8]) -> Option<(u64, &[u8])> {
    let (head, rest) = bytes.split_at_checked(8)?;
    Some((u64::from_le_bytes(head.try_into().ok()?), rest))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matrix(n: usize) -> Mat<C> {
        // Pseudo-random and full rank, with no diagonal dominance, so pivoting actually happens.
        Mat::from_fn(n, n, |i, j| {
            #[allow(clippy::cast_precision_loss)]
            let k = ((i * 7919 + j * 104_729) % 1013) as f64;
            C::new((k * 0.37).sin(), (k * 0.11).cos())
        })
    }

    #[test]
    fn solves_and_survives_a_round_trip_through_bytes() {
        let a = matrix(7);
        let f = Factors::new(&a);
        let b = Mat::from_fn(7, 2, |i, j| {
            #[allow(clippy::cast_precision_loss)]
            C::new(i as f64 + 1.0, j as f64 - 0.5)
        });
        let x = f.solve(&b);
        let r = &a * &x - &b;
        let worst = (0..7)
            .flat_map(|i| (0..2).map(move |j| (i, j)))
            .map(|(i, j)| r[(i, j)].norm())
            .fold(0.0, f64::max);
        assert!(worst < 1e-12, "residual {worst}");

        let bytes = f.to_bytes();
        let (g, rest) = Factors::from_bytes(&bytes).expect("reads");
        assert!(rest.is_empty());
        assert_eq!(g, f);
        assert!(Factors::from_bytes(&bytes[..bytes.len() - 1]).is_none());
    }
}
