//! Maximum coupling between pairs of segments (`CP`; NEC's `COUPLE`).
//!
//! Each listed segment is driven alone in turn; the currents it induces at the others give the
//! two-port admittances `Y11`, `Y22` and `Y12` (averaged both ways, which reciprocity makes equal
//! in principle). From those, NEC reports the largest power transfer achievable with the second
//! segment terminated in its best load, that load, and the input impedance seen at the first.

use crate::solve::{Solution, SolveError, SolveOptions, SourceKind, VoltageSource, solve};
use nec_core::Frequency;
use nec_geom::Geometry;
use num_complex::Complex64;

type C = Complex64;

/// One pair's row of NEC's "ISOLATION DATA".
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CouplingRow {
    /// First segment, 0-based.
    pub first: usize,
    /// Second segment, 0-based.
    pub second: usize,
    /// `C = |Y12²| / (2·Re Y11·Re Y22 − Re Y12²)`. Outside `[0, 1]` NEC reports an error instead
    /// of the figures below, which are then `None`.
    pub c: f64,
    /// The figures, when `C` is in range.
    pub result: Option<MaximumCoupling>,
}

/// The best the pair can do.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaximumCoupling {
    /// Maximum coupling, dB.
    pub coupling_db: f64,
    /// The load on the second segment that achieves it, ohms.
    pub load: C,
    /// The input impedance at the first segment with that load, ohms.
    pub input: C,
}

/// The self- and mutual admittances of the listed segments and the solutions that gave them.
#[derive(Debug, Clone)]
pub struct Coupling {
    /// One row per pair `(i, j)`, `i < j`, in NEC's order.
    pub rows: Vec<CouplingRow>,
    /// The solution with only segment `k` driven, for each listed segment.
    pub solutions: Vec<Solution>,
}

/// `DB10`: `10·log10(x)`, or −999.99 for `x < 1e-20`.
fn db10(x: f64) -> f64 {
    if x < 1e-20 { -999.99 } else { 10.0 * x.log10() }
}

/// Drive each of `segments` (0-based) alone with 1 V and compute NEC's isolation data.
pub fn coupling(geom: &Geometry, f: Frequency, segments: &[usize], options: &SolveOptions) -> Result<Coupling, SolveError> {
    let mut y = Vec::with_capacity(segments.len());
    let mut solutions = Vec::with_capacity(segments.len());
    for &s in segments {
        let source = VoltageSource {
            segment: s,
            voltage: C::new(1.0, 0.0),
            kind: SourceKind::AppliedField,
        };
        let sol = solve(geom, f, &[source], options.clone())?;
        // `CUR(J)*WLAM/ZIN`: the centre current over the source voltage.
        y.push(segments.iter().map(|&k| sol.currents[k]).collect::<Vec<C>>());
        solutions.push(sol);
    }
    let mut rows = Vec::new();
    for i in 0..segments.len() {
        for j in i + 1..segments.len() {
            let (y11, y22) = (y[i][i], y[j][j]);
            let y12 = 0.5 * (y[i][j] + y[j][i]);
            let yin = y12 * y12;
            let dbc = yin.norm();
            let c = dbc / (2.0 * y11.re * y22.re - yin.re);
            let result = (0.0..=1.0).contains(&c).then(|| {
                let gmax = if c < 0.01 { 0.5 * (c + 0.25 * c * c * c) } else { (1.0 - (1.0 - c * c).sqrt()) / c };
                let rho = gmax * yin.conj() / dbc;
                let yl = ((1.0 - rho) / (1.0 + rho) + 1.0) * y22.re - y22;
                let yin = y11 - yin / (y22 + yl);
                MaximumCoupling {
                    coupling_db: db10(gmax),
                    load: 1.0 / yl,
                    input: 1.0 / yin,
                }
            });
            rows.push(CouplingRow {
                first: segments[i],
                second: segments[j],
                c,
                result,
            });
        }
    }
    Ok(Coupling { rows, solutions })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::read_program;

    #[test]
    fn two_parallel_dipoles_couple_strongly_and_reciprocally() {
        let src = "CE\nGW 1 9 0 -.25 0 0 .25 0 .001\nGW 2 9 .2 -.25 0 .2 .25 0 .001\nGE 0\n\
                   EX 0 1 5 0 1 0\nFR 0 1 0 0 299.8 0\nEN\n";
        let mut d = nec_cards::parse_deck(src).expect("parses");
        d.resolve().expect("resolves");
        let g = Geometry::build(&d).expect("builds");
        let p = read_program(&d, &g).expect("program");
        let f = p.frequencies[0];
        let c = coupling(&g, f, &[4, 13], &p.options(f)).expect("solves");
        assert_eq!(c.rows.len(), 1);
        let r = c.rows[0].result.expect("in range");
        // Two parallel half-wave dipoles 0.2 λ apart couple within a few dB; the engine gate
        // checks the figures themselves.
        assert!(r.coupling_db < 0.0 && r.coupling_db > -15.0, "{r:?}");
        // By symmetry the matched input and load impedances agree.
        assert!((r.input - r.load.conj()).norm() / r.input.norm() < 1e-6, "{r:?}");
    }
}
