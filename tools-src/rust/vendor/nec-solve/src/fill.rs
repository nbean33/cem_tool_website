//! Filling the interaction matrix (`CMSET`, `CMWW`, `CMWS`, `CMSW`, `CMSS`).
//!
//! The unknowns are one basis-function amplitude per wire segment, then two surface-current
//! components per patch (along its `t1`, then its `t2`). The equations are, in the same order,
//! the tangential electric field at each segment centre (the EFIE), then two components of the
//! magnetic-field integral equation (MFIE) at each patch centre — the `t2` component first, then
//! `t1`, as NEC orders them.
//!
//! Wire-on-wire element `Z[i][j]` is the field tangential to observation segment `i`, at its
//! centre, due to basis function `j` carrying unit amplitude. Basis function `j` spans several
//! segments, so the element is a sum over every source segment it touches:
//!
//! ```text
//!     Z[i][j] = Σ_{segments J touched by basis j}  E_K(J→i)·a + E_S(J→i)·b + E_C(J→i)·c
//! ```
//!
//! where `E_K`, `E_S`, `E_C` are the tangential fields of a constant, `sin` and `cos` current on
//! segment `J` and `(a, b, c)` are basis `j`'s coefficients there. The other blocks:
//!
//! * wire rows, patch columns (`CMSW`): the electric field of each patch's unit currents along
//!   the wire; at a wire ending on a patch, the four quarter patches are integrated properly and
//!   the singular current flowing out of the junction adds to the wire's own column (`PCINT`);
//! * patch rows, wire columns (`CMWS`): `−t·H` of each segment's current shapes;
//! * patch rows, patch columns (`CMSS`): `−t·H` of each patch's currents, with the `∓½` jump of
//!   the MFIE on the diagonal.
//!
//! A loaded segment's equation also carries the voltage its load drops (`CMSET`): its row loses
//! `Z'·(a + c)` for every basis function on it, `Z'` the load per unit length.
//!
//! # Orientation
//!
//! NEC stores this matrix transposed — `CM(basis, observation)` — and its factor/solve routines
//! account for that. Here it is stored the conventional way, rows = equations and columns =
//! unknowns, so `Z · α = E` reads as written.
//!
//! # Parallelism
//!
//! Rows are filled independently in parallel. Filling by row rather than by source (as NEC's loop
//! does) means each thread writes only its own row, so no synchronization is needed.

use faer::Mat;
use nec_geom::{Connectivity, Junction};
use nec_kernel::hfield::hsfld;
use nec_kernel::surface::{hintg, pcint, unere};
use nec_kernel::{Contribution, ExtendedEnd, Ground, ImageGround, SurfacePatch, WireSeg, efld};
use num_complex::Complex64;
use rayon::prelude::*;

type C = Complex64;
const ZERO: C = C::new(0.0, 0.0);

/// `DATA PI/3.141592654D+0/` in `CMSW`: NEC's literal, kept for fidelity.
#[allow(clippy::approx_constant)]
const PI: f64 = 3.141_592_654;

/// Everything the fill needs about the structure at one frequency.
pub struct Structure<'a> {
    /// Wire segments, wavelengths.
    pub segs: &'a [WireSeg],
    /// Surface patches, wavelengths.
    pub patches: &'a [SurfacePatch],
    /// Per segment, every basis function with a component on it (`TRIO`).
    pub by_segment: &'a [Vec<Contribution>],
    /// Segment connectivity, for wire–surface junctions.
    pub connectivity: &'a [Connectivity],
    /// Distance beyond which wire interactions use the lumped approximation (`KH`).
    pub rkh: f64,
    /// The ground for the wires' electric field.
    pub ground: &'a Ground,
    /// The ground for magnetic fields and patch fields.
    pub image: &'a ImageGround,
    /// Extended thin-wire kernel end treatments, if `EK`.
    pub extended: Option<&'a [[ExtendedEnd; 2]]>,
    /// Load impedance per unit length on each segment (`ZARRAY`), as
    /// `LoadKind::per_unit_length` gives it.
    pub zarray: &'a [C],
}

impl Structure<'_> {
    /// The order of the system.
    #[must_use]
    pub fn unknowns(&self) -> usize {
        self.segs.len() + 2 * self.patches.len()
    }

    /// The patch a wire segment's end sits on, as `CMSW` finds it: the first of its four quarter
    /// patches (0-based) and `FSIGN`, −1 for end 1 and +1 for end 2. End 2 wins if both are.
    fn junction(&self, i: usize) -> Option<(usize, f64)> {
        let c = &self.connectivity[i];
        let mut found = None;
        if let Junction::Patch(p) = c.end1 {
            found = Some((p - 1, -1.0));
        }
        if let Junction::Patch(p) = c.end2 {
            found = Some((p - 1, 1.0));
        }
        found
    }

    /// Wire equation `i`: the tangential electric field at segment `i`'s centre, from the
    /// currents on segments `wires` and patches `patches` — every element whose basis function
    /// lies on those sources is complete.
    fn wire_row(&self, i: usize, wires: &[usize], patches: &[usize]) -> Vec<C> {
        let n = self.segs.len();
        let obs = &self.segs[i];
        let mut row = vec![ZERO; self.unknowns()];
        for &j in wires {
            let src = &self.segs[j];
            let f = efld(
                src,
                obs.center,
                obs.radius,
                i == j,
                self.rkh,
                self.ground,
                self.extended.map(|e| e[j]),
            );
            let (etk, ets, etc) = f.along(obs.dir);
            for c in &self.by_segment[j] {
                row[c.basis] += etk * c.a + ets * c.b + etc * c.c;
            }
        }
        let zl = self.zarray[i];
        if zl.re != 0.0 || zl.im != 0.0 {
            for c in &self.by_segment[i] {
                row[c.basis] -= (c.a + c.c) * zl;
            }
        }
        if self.patches.is_empty() {
            return row;
        }
        let dot = |v: [C; 3]| v[0] * obs.dir[0] + v[1] * obs.dir[1] + v[2] * obs.dir[2];
        let junction = self.junction(i);
        // At a junction, PCINT replaces the direct field of the four quarter patches.
        let quarters = junction.map(|(first, fsign)| {
            let e = pcint(&self.patches[first], obs.center, obs.dir);
            // The singular current carries basis function i's current at the junction end into
            // the surface: SBF(I, I) evaluated at s = ±Δ/2.
            let own = self.by_segment[i]
                .last()
                .expect("a segment carries its own basis function");
            let py = PI * obs.length * fsign;
            let (px, py) = (py.sin(), py.cos());
            row[i] += e[8] * fsign * (own.a + own.b * px + own.c * py);
            (first, e)
        });
        for &q in patches {
            let src = &self.patches[q];
            let (k, s) = (n + 2 * q, n + 2 * q + 1);
            match quarters {
                Some((first, e)) if (first..first + 4).contains(&q) => {
                    row[k] = e[q - first];
                    row[s] = e[q - first + 4];
                }
                _ => {
                    let [fk, fs] = unere(src, obs.center, false, self.image);
                    row[k] += dot(fk);
                    row[s] += dot(fs);
                }
            }
            if !matches!(self.image, ImageGround::None) {
                let [fk, fs] = unere(src, obs.center, true, self.image);
                row[k] += dot(fk);
                row[s] += dot(fs);
            }
        }
        row
    }

    /// Patch equations `2p` and `2p + 1`: `−t2·H` and `−t1·H` at patch `p`'s centre, signed by
    /// its `SALP`, from the currents on segments `wires` and patches `patches`.
    fn patch_rows(&self, p: usize, wires: &[usize], patches: &[usize]) -> [Vec<C>; 2] {
        let n = self.segs.len();
        let obs = &self.patches[p];
        let mut rows = [vec![ZERO; self.unknowns()], vec![ZERO; self.unknowns()]];
        let tangent = |t: [f64; 3]| [t[0] * obs.salp, t[1] * obs.salp, t[2] * obs.salp];
        let (t2, t1) = (tangent(obs.t2), tangent(obs.t1));
        let dot = |v: [C; 3], t: [f64; 3]| v[0] * t[0] + v[1] * t[1] + v[2] * t[2];
        for &j in wires {
            let src = &self.segs[j];
            let h = hsfld(src, obs.center, 0.0, self.image);
            for (row, t) in rows.iter_mut().zip([t2, t1]) {
                let (etk, ets, etc) = (-dot(h[0], t), -dot(h[1], t), -dot(h[2], t));
                for c in &self.by_segment[j] {
                    row[c.basis] += etk * c.a + ets * c.b + etc * c.c;
                }
            }
        }
        for &q in patches {
            let src = &self.patches[q];
            let [hk, hs] = hintg(src, obs.center, self.image);
            let mut g = [[-dot(hk, t2), -dot(hs, t2)], [-dot(hk, t1), -dot(hs, t1)]];
            if p == q {
                g[0][0] -= 0.5;
                g[1][1] += 0.5;
            }
            for (row, gi) in rows.iter_mut().zip(g) {
                row[n + 2 * q] = gi[0];
                row[n + 2 * q + 1] = gi[1];
            }
        }
        rows
    }
}

impl Structure<'_> {
    /// The rows of the matrix for equations `rows`, restricted to unknowns `cols`: only the
    /// sources those unknowns' basis functions lie on are visited, so a few columns of many rows
    /// cost a few columns' worth of field evaluations.
    #[must_use]
    pub fn block(&self, rows: &[usize], cols: &[usize]) -> Mat<C> {
        let n = self.segs.len();
        let mut wanted = vec![false; self.unknowns()];
        for &c in cols {
            wanted[c] = true;
        }
        let wires: Vec<usize> = (0..n)
            .filter(|&j| self.by_segment[j].iter().any(|c| wanted[c.basis]))
            .collect();
        let patches: Vec<usize> = (0..self.patches.len())
            .filter(|&q| wanted[n + 2 * q] || wanted[n + 2 * q + 1])
            .collect();
        let full: Vec<Vec<C>> = rows
            .par_iter()
            .map(|&r| {
                if r < n {
                    self.wire_row(r, &wires, &patches)
                } else {
                    let k = r - n;
                    let [a, b] = self.patch_rows(k / 2, &wires, &patches);
                    if k.is_multiple_of(2) { a } else { b }
                }
            })
            .collect();
        Mat::from_fn(rows.len(), cols.len(), |i, j| full[i][cols[j]])
    }
}

/// Fill the interaction matrix: `n + 2m` square for `n` segments and `m` patches.
#[must_use]
pub fn fill(st: &Structure<'_>) -> Mat<C> {
    let n = st.segs.len();
    let all_wires: Vec<usize> = (0..n).collect();
    let all_patches: Vec<usize> = (0..st.patches.len()).collect();
    let wires: Vec<Vec<C>> = (0..n)
        .into_par_iter()
        .map(|i| st.wire_row(i, &all_wires, &all_patches))
        .collect();
    let patches: Vec<[Vec<C>; 2]> = (0..st.patches.len())
        .into_par_iter()
        .map(|p| st.patch_rows(p, &all_wires, &all_patches))
        .collect();
    let size = st.unknowns();
    Mat::from_fn(size, size, |i, j| {
        if i < n {
            wires[i][j]
        } else {
            let k = i - n;
            patches[k / 2][k % 2][j]
        }
    })
}
