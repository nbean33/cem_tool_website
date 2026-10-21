//! Near fields on a plane, for display: NEC's `NE`/`NH` evaluated on a grid of points.

use crate::{EngineError, Model, Result};
use nec_solve::{NearFieldRequest, Solution};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

/// Which coordinate plane a slice lies in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlicePlane {
    /// Horizontal: `u = x`, `v = y`, at `z = offset`.
    Xy,
    /// Vertical along x: `u = x`, `v = z`, at `y = offset`.
    Xz,
    /// Vertical along y: `u = y`, `v = z`, at `x = offset`.
    Yz,
}

/// A rectangle of a coordinate plane, sampled on an `n_u × n_v` grid (metres).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SliceRequest {
    /// The plane.
    pub plane: SlicePlane,
    /// The plane's position along its normal.
    pub offset: f64,
    /// First and last `u`.
    pub u: [f64; 2],
    /// First and last `v`.
    pub v: [f64; 2],
    /// Points along `u` (at least 2).
    pub n_u: usize,
    /// Points along `v` (at least 2).
    pub n_v: usize,
    /// The magnetic field (`NH`) rather than the electric (`NE`).
    pub magnetic: bool,
}

/// Field magnitudes on a slice.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Slice {
    /// `|F| = √(|Fx|² + |Fy|² + |Fz|²)` from peak phasors — volts per metre, or amperes per
    /// metre — row by row: `n_u` values at the first `v`, then the next.
    pub values: Vec<f64>,
    /// The largest value.
    pub max: f64,
}

/// The most points a slice may ask for.
pub const MAX_SLICE_POINTS: usize = 40_000;

impl Model {
    /// The near field of solution `sol` on a slice, each row of the grid computed in parallel.
    ///
    /// # Errors
    /// The grid is degenerate or too large, or the deck cannot be solved.
    pub fn near_field_slice(&self, sol: &Solution, req: &SliceRequest) -> Result<Slice> {
        if req.n_u < 2 || req.n_v < 2 || req.n_u * req.n_v > MAX_SLICE_POINTS {
            return Err(EngineError::Solve(format!(
                "a near-field slice needs between 2×2 and {MAX_SLICE_POINTS} points"
            )));
        }
        let prog = self.program()?;
        #[allow(clippy::cast_precision_loss)]
        let step = |r: [f64; 2], n: usize| (r[1] - r[0]) / (n - 1) as f64;
        let (du, dv) = (step(req.u, req.n_u), step(req.v, req.n_v));
        let rows: Vec<Vec<f64>> = (0..req.n_v)
            .into_par_iter()
            .map(|j| {
                #[allow(clippy::cast_precision_loss)]
                let v = req.v[0] + dv * j as f64;
                let (start, step) = match req.plane {
                    SlicePlane::Xy => ([req.u[0], v, req.offset], [du, 0.0, 0.0]),
                    SlicePlane::Xz => ([req.u[0], req.offset, v], [du, 0.0, 0.0]),
                    SlicePlane::Yz => ([req.offset, req.u[0], v], [0.0, du, 0.0]),
                };
                let count = match req.plane {
                    SlicePlane::Xy | SlicePlane::Xz => [req.n_u, 1, 1],
                    SlicePlane::Yz => [1, req.n_u, 1],
                };
                let r = NearFieldRequest {
                    magnetic: req.magnetic,
                    spherical: false,
                    count,
                    start,
                    step,
                };
                prog.near_field(&self.geom, sol, &r)
                    .iter()
                    .map(|(_, f)| (f[0].norm_sqr() + f[1].norm_sqr() + f[2].norm_sqr()).sqrt())
                    .collect()
            })
            .collect();
        let values: Vec<f64> = rows.into_iter().flatten().collect();
        let max = values
            .iter()
            .copied()
            .filter(|v| v.is_finite())
            .fold(0.0, f64::max);
        Ok(Slice { values, max })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testdata::DIPOLE;

    #[test]
    fn a_slice_matches_single_points_and_peaks_near_the_wire() {
        let m = Model::new(DIPOLE).expect("builds");
        let prog = m.program().expect("program");
        let f = prog.frequencies[0];
        let sol = m.solve(f, prog.options(f)).expect("solves");
        // The dipole lies along y; a horizontal slice through it.
        let req = SliceRequest {
            plane: SlicePlane::Xy,
            offset: 0.0,
            u: [-0.5, 0.5],
            v: [-0.5, 0.5],
            n_u: 11,
            n_v: 11,
            magnetic: false,
        };
        let s = m.near_field_slice(&sol, &req).expect("slice");
        assert_eq!(s.values.len(), 121);
        // Point (u, v) = (0.3, -0.4) is index j·11 + i with i = 8, j = 1.
        let one = prog.near_field(
            &m.geom,
            &sol,
            &NearFieldRequest {
                magnetic: false,
                spherical: false,
                count: [1, 1, 1],
                start: [0.3, -0.4, 0.0],
                step: [0.0; 3],
            },
        );
        let e = one[0].1;
        let mag = (e[0].norm_sqr() + e[1].norm_sqr() + e[2].norm_sqr()).sqrt();
        assert!((s.values[11 + 8] - mag).abs() < 1e-9 * mag);
        // Strongest beside the wire, at its ends (the charge), not far out at the corners.
        assert!(s.values[0] < 0.2 * s.max);
    }
}
