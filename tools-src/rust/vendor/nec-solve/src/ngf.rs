//! The numerical Green's function (`WG`, `GF`): solving a structure once and reusing the work.
//!
//! A `WG` card saves a structure at one frequency together with its factored interaction matrix
//! — the expensive part of a solve. A later run that starts with `GF` adds wires and patches to
//! that structure and solves the whole, factoring only what is new. NEC's manual describes the
//! typical use: a ground radial system or a large fixed structure, with an antenna on it that
//! is changed and re-solved many times.
//!
//! # The solution is the full structure's
//!
//! Splitting the unknowns into the saved structure's (`o`) and the new ones (`n`), the combined
//! system is
//!
//! ```text
//!     [ Z_oo  Z_on ] [x_o]   [b_o]
//!     [ Z_no  Z_nn ] [x_n] = [b_n]
//! ```
//!
//! `Z_oo` is the saved matrix `A` — except in the columns of saved basis functions that change
//! because a new wire joins their junctions (NEC's `ISCON` segments), and every such column is
//! recomputed in full: `Z_oo = A + Δ·Sᵀ`, `Δ` the differences in those columns and `S` selecting
//! them. As NEC does with its "modified basis functions", those columns become extra unknowns
//! `y = Sᵀ·x_o`, which bordering keeps `A` intact:
//!
//! ```text
//!     [ A     Δ    Z_on ] [x_o]   [b_o]
//!     [ −Sᵀ   I    0    ] [ y ] = [ 0 ]
//!     [ Z_no  0    Z_nn ] [x_n]   [b_n]
//! ```
//!
//! and the Schur complement `D − C·A⁻¹·B` of the saved block is all that is factored anew. The
//! result is the full structure's solution, to rounding: the tests here check it against a
//! direct solve of the combined structure.
//!
//! The new columns use the new run's `KH` and `EK`, while `A` keeps the saved run's — as in NEC.
//!
//! A new wire ending on a saved patch (NEC's `NPCON`) needs that patch split into quarters, which
//! the saved matrix cannot accommodate. The quarters join the new unknowns, and the saved patch
//! is switched off by two more border rows and columns per patch: its two currents are held at
//! zero, and its two equations are released by a free unknown each.

use crate::fill::fill;
use crate::linalg::Factors;
use crate::program::{Program, read_program};
use crate::solve::{Prepared, Solution, SolveError, SolveOptions, VoltageSource, check, finish};
use faer::Mat;
use nec_core::Frequency;
use nec_geom::Geometry;
use num_complex::Complex64;

type C = Complex64;

/// The first line of an NGF file written here. NEC's own NGF files are Fortran unformatted
/// records and cannot be read; they are recognized by not starting with this.
const MAGIC: &[u8] = b"nec-rs numerical Green's function v1\n";

/// Why a numerical Green's function could not be made or used.
#[derive(Debug, thiserror::Error)]
pub enum NgfError {
    /// The run writing it could not be read.
    #[error("{0}")]
    Deck(String),
    /// The structure could not be solved.
    #[error(transparent)]
    Solve(#[from] SolveError),
    /// The file is not one this program wrote.
    #[error("not a numerical Green's function file written by nec-rs{0}")]
    Format(&'static str),
}

/// A structure, its program, and its factored matrix at one frequency.
#[derive(Debug, Clone)]
pub struct GreensFunction {
    /// The run that wrote it, as resolved numeric deck text: rebuilds the structure and program.
    pub deck: String,
    /// The structure.
    pub geometry: Geometry,
    /// Its program: frequency, ground, loads, `KH`, `EK`.
    pub program: Program,
    /// The frequency it was solved at.
    pub frequency: Frequency,
    factors: Factors,
}

fn rebuild(deck: &str) -> Result<(Geometry, Program), NgfError> {
    let err = |e: &dyn std::fmt::Display| NgfError::Deck(e.to_string());
    let mut d = nec_cards::parse_deck(deck).map_err(|e| err(&e))?;
    d.resolve().map_err(|e| err(&e))?;
    let mut d = nec_cards::lower_4nec2(&d).map_err(|e| err(&e))?;
    let geometry = Geometry::build(&d).map_err(|e| err(&e))?;
    d.resolve_percentages(|tag| geometry.by_tag(tag).count()).map_err(|e| err(&e))?;
    let program = read_program(&d, &geometry).map_err(|e| err(&e))?;
    Ok((geometry, program))
}

impl GreensFunction {
    /// Solve the structure of `deck` (resolved numeric deck text, as written for the engine) at
    /// its first frequency and keep the factored matrix.
    pub fn compute(deck: &str) -> Result<Self, NgfError> {
        let (geometry, program) = rebuild(deck)?;
        check(&geometry, &[])?;
        let frequency = *program
            .frequencies
            .first()
            .ok_or(NgfError::Deck("no frequency".into()))?;
        let options = program.options(frequency);
        let prep = Prepared::new(&geometry, frequency, &options);
        let factors = Factors::new(&fill(&prep.structure(&geometry, &options)));
        Ok(Self {
            deck: deck.to_string(),
            geometry,
            program,
            frequency,
            factors,
        })
    }

    /// The file contents: [`MAGIC`], the deck's length and text, then the factors.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = MAGIC.to_vec();
        out.extend_from_slice(&(self.deck.len() as u64).to_le_bytes());
        out.extend_from_slice(self.deck.as_bytes());
        out.extend_from_slice(&self.factors.to_bytes());
        out
    }

    /// Read a file written by [`GreensFunction::to_bytes`].
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, NgfError> {
        let rest = bytes
            .strip_prefix(MAGIC)
            .ok_or(NgfError::Format(" (NEC's own NGF files cannot be read)"))?;
        let (len, rest) = rest
            .split_at_checked(8)
            .ok_or(NgfError::Format(": truncated"))?;
        let len = usize::try_from(u64::from_le_bytes(len.try_into().expect("eight bytes")))
            .map_err(|_| NgfError::Format(": bad length"))?;
        let (deck, rest) = rest
            .split_at_checked(len)
            .ok_or(NgfError::Format(": truncated"))?;
        let deck = std::str::from_utf8(deck)
            .map_err(|_| NgfError::Format(": bad text"))?
            .to_string();
        let (factors, rest) = Factors::from_bytes(rest).ok_or(NgfError::Format(": bad matrix"))?;
        if !rest.is_empty() {
            return Err(NgfError::Format(": trailing data"));
        }
        let (geometry, program) = rebuild(&deck)?;
        if factors.len() != geometry.unknowns() {
            return Err(NgfError::Format(": the matrix does not fit the structure"));
        }
        let frequency = *program
            .frequencies
            .first()
            .ok_or(NgfError::Format(": no frequency"))?;
        Ok(Self {
            deck,
            geometry,
            program,
            frequency,
            factors,
        })
    }

    /// Solve the structure `geom`, built on this Green's function with
    /// [`Geometry::build_on`], for the given sources. `options` are the new run's (its program
    /// read with [`crate::read_program_on`] supplies the inherited ground and loads).
    #[allow(clippy::too_many_lines, clippy::similar_names)]
    pub fn solve(
        &self,
        geom: &Geometry,
        sources: &[VoltageSource],
        options: &SolveOptions,
    ) -> Result<Solution, SolveError> {
        check(geom, sources)?;
        let f = self.frequency;
        let (n, m) = (geom.len(), geom.patches.len());
        let (n1, m1) = (geom.base_segments, geom.base_patches);
        let no = n1 + 2 * m1;
        // Combined index of the saved system's unknown (and equation) k.
        let comb = |k: usize| if k < n1 { k } else { n + (k - n1) };
        let old: Vec<usize> = (0..no).map(comb).collect();
        let new: Vec<usize> = (n1..n).chain(n + 2 * m1..n + 2 * m).collect();
        let nn = new.len();

        let prep = Prepared::new(geom, f, options);
        let base_options = self.program.options(f);
        let base = Prepared::new(&self.geometry, f, &base_options);

        // Saved basis functions recomputed in full: every one whose column is no longer what the
        // saved matrix holds. That is NEC's ISCON set (segments sharing a junction with a new
        // segment or ending on a new patch) and more: a saved segment whose connections changed
        // some other way — a GE 1 run bonding a saved wire end to ground — or a basis function
        // lying on such a segment. NEC keeps those saved columns while rebuilding the currents
        // with the new connections, which makes its answer differ from a direct solve of the same
        // structure (12% on `Objects\Gr_func.nec`); here the result is the direct solve's.
        let changed_seg: Vec<bool> = (0..n1)
            .map(|j| geom.connectivity[j] != self.geometry.connectivity[j])
            .collect();
        let column = |by: &[Vec<nec_kernel::Contribution>], j: usize| -> Vec<(usize, [f64; 3])> {
            (0..n1)
                .flat_map(|seg| {
                    by[seg]
                        .iter()
                        .filter(move |c| c.basis == j)
                        .map(move |c| (seg, [c.a, c.b, c.c]))
                })
                .collect()
        };
        let s: Vec<usize> = (0..n1)
            .filter(|&j| {
                column(&prep.by_segment, j) != column(&base.by_segment, j)
                    || prep.by_segment[n1..]
                        .iter()
                        .any(|cs| cs.iter().any(|c| c.basis == j))
                    || base
                        .by_segment
                        .iter()
                        .enumerate()
                        .any(|(seg, cs)| changed_seg[seg] && cs.iter().any(|c| c.basis == j))
            })
            .collect();
        let k = s.len();
        // The superseded patches' unknowns (and equations) in the saved system.
        let held: Vec<usize> = geom
            .superseded_patches
            .iter()
            .flat_map(|&q| [n1 + 2 * q, n1 + 2 * q + 1])
            .collect();
        let h = held.len();

        let combined = prep.structure(geom, options);
        // Saved rows: the changed columns as they are now, and every new column.
        let cols: Vec<usize> = s.iter().copied().chain(new.iter().copied()).collect();
        let upper = combined.block(&old, &cols);
        // The changed columns as the saved matrix has them.
        let was = base
            .structure(&self.geometry, &base_options)
            .block(&(0..no).collect::<Vec<_>>(), &s);
        // New rows, every column.
        let all: Vec<usize> = (0..n + 2 * m).collect();
        let lower = combined.block(&new, &all);

        // Border unknowns: y (k), the released equations' free unknowns (h), then the new ones.
        // B = [Δ | E_held | Z_on], C = [−Sᵀ ; E_heldᵀ ; Z_no], D = diag(I, 0, Z_nn).
        let one = C::new(1.0, 0.0);
        let zero = C::new(0.0, 0.0);
        let m2 = k + h + nn;
        let b = Mat::from_fn(no, m2, |i, j| {
            if j < k {
                upper[(i, j)] - was[(i, j)]
            } else if j < k + h {
                if held[j - k] == i { one } else { zero }
            } else {
                upper[(i, j - h)]
            }
        });
        let c = Mat::from_fn(m2, no, |i, j| {
            if i < k {
                if j == s[i] { -one } else { zero }
            } else if i < k + h {
                if held[i - k] == j { one } else { zero }
            } else {
                lower[(i - k - h, old[j])]
            }
        });
        let d = Mat::from_fn(m2, m2, |i, j| {
            if i < k && j < k {
                if i == j { one } else { zero }
            } else if i >= k + h && j >= k + h {
                lower[(i - k - h, new[j - k - h])]
            } else {
                zero
            }
        });
        let w = self.factors.solve(&b);
        let schur = Factors::new(&(&d - &c * &w));

        let solve_rhs = |rhs: &Mat<C>| -> Mat<C> {
            let bo = Mat::from_fn(no, rhs.ncols(), |i, j| rhs[(old[i], j)]);
            let u = self.factors.solve(&bo);
            let r2 = Mat::from_fn(m2, rhs.ncols(), |i, j| if i < k + h { zero } else { rhs[(new[i - k - h], j)] });
            let z2 = schur.solve(&(&r2 - &c * &u));
            let xo = &u - &w * &z2;
            let mut x = Mat::zeros(rhs.nrows(), rhs.ncols());
            for col in 0..rhs.ncols() {
                for (i, &r) in old.iter().enumerate() {
                    x[(r, col)] = xo[(i, col)];
                }
                for (i, &r) in new.iter().enumerate() {
                    x[(r, col)] = z2[(k + h + i, col)];
                }
            }
            x
        };
        Ok(finish(geom, f, sources, options, &prep, &solve_rhs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::read_program_on;
    use crate::solve::solve;

    fn build(src: &str, base: Option<&GreensFunction>) -> (Geometry, Program) {
        let mut d = nec_cards::parse_deck(src).expect("parses");
        d.resolve().expect("resolves");
        let g = Geometry::build_on(&d, base.map(|b| &b.geometry)).expect("builds");
        let p = read_program_on(&d, &g, base.map(|b| &b.program)).expect("program");
        (g, p)
    }

    /// Four radials over Sommerfeld ground, saved; then a loaded vertical joined to them at the
    /// centre — which changes the radials' basis functions — solved on the saved function and
    /// directly, as one structure.
    #[test]
    fn a_green_s_function_solve_is_the_full_structure_s() {
        let radials = "CE\nGW 10 8 0 0 0.1 10 0 0.1 0.001\nGR 1 4\nGE -1\n\
                       GN 2 0 0 0 13 0.005\nLD 5 0 0 0 5.8e7\nFR 0 1 0 0 7.1\nWG\nEN\n";
        let gf = GreensFunction::compute(radials).expect("computes");
        let gf = GreensFunction::from_bytes(&gf.to_bytes()).expect("round trip");
        let antenna = "CE\nGF\nGW 1 10 0 0 0.1 0 0 10.1 0.002\nGE -1\nLD 0 1 3 3 10 0 0\nEX 0 1 1 0 1 0\nEN\n";
        let (g, p) = build(antenna, Some(&gf));
        assert_eq!((g.base_segments, g.len()), (32, 42));
        let f = gf.frequency;
        let via_ngf = gf.solve(&g, &p.sources, &p.options(f)).expect("solves");

        // The same structure in one piece.
        let whole = "CE\nGW 10 8 0 0 0.1 10 0 0.1 0.001\nGR 1 4\n\
                     GW 1 10 0 0 0.1 0 0 10.1 0.002\nGE -1\nGN 2 0 0 0 13 0.005\nLD 5 0 1 32 5.8e7\n\
                     LD 0 1 3 3 10 0 0\nFR 0 1 0 0 7.1\nEX 0 1 1 0 1 0\nEN\n";
        let (gw, pw) = build(whole, None);
        assert_eq!(gw.len(), 42);
        assert_eq!(pw.loads.len(), 33);
        let direct = solve(&gw, f, &pw.sources, pw.options(f)).expect("solves");
        let (a, b) = (via_ngf.inputs[0].impedance, direct.inputs[0].impedance);
        assert!((a - b).norm() / b.norm() < 1e-10, "NGF {a} vs direct {b}");
        let worst = via_ngf
            .currents
            .iter()
            .zip(&direct.currents)
            .map(|(x, y)| (x - y).norm())
            .fold(0.0, f64::max);
        assert!(worst < 1e-12, "currents differ by {worst}");
        assert!((via_ngf.power.structure_loss - direct.power.structure_loss).abs() < 1e-12);
    }

    /// A saved patch plate, and a monopole standing on one of its patches: the patch is
    /// superseded by quarters on the NGF side and split in place on the direct side, and both
    /// give the same structure.
    #[test]
    fn a_new_wire_may_end_on_a_saved_patch() {
        let plate = "CE\nSM 4 4 -.5 -.5 0 .5 -.5 0\nSC 0 0 .5 .5 0\nGE 0\nFR 0 1 0 0 300\nWG\nEN\n";
        let gf = GreensFunction::compute(plate).expect("computes");
        let (g, p) = build("CE\nGF\nGW 1 6 .125 .125 0 .125 .125 .25 .002\nGE 0\nEX 0 1 1 0 1\nEN\n", Some(&gf));
        assert_eq!(g.superseded_patches, vec![10]);
        assert_eq!(g.patches.len(), 20);
        let f = gf.frequency;
        let via_ngf = gf.solve(&g, &p.sources, &p.options(f)).expect("solves");
        assert!(via_ngf.patch_currents[10].iter().all(|j| j.norm() < 1e-12), "superseded patch carries no current");

        let whole = "CE\nSM 4 4 -.5 -.5 0 .5 -.5 0\nSC 0 0 .5 .5 0\nGW 1 6 .125 .125 0 .125 .125 .25 .002\nGE 0\n\
                     FR 0 1 0 0 300\nEX 0 1 1 0 1\nEN\n";
        let (gw, pw) = build(whole, None);
        let direct = solve(&gw, f, &pw.sources, pw.options(f)).expect("solves");
        let (a, b) = (via_ngf.inputs[0].impedance, direct.inputs[0].impedance);
        assert!((a - b).norm() / b.norm() < 1e-9, "NGF {a} vs direct {b}");
    }

    #[test]
    fn a_ngf_run_may_not_change_frequency_or_ground() {
        let gf =
            GreensFunction::compute("CE\nGW 1 5 0 0 0 1 0 0 1e-3\nGE 0\nFR 0 1 0 0 100\nWG\nEN\n")
                .expect("computes");
        let mut d = nec_cards::parse_deck(
            "CE\nGF\nGW 2 3 1 0 0 2 0 0 1e-3\nGE 0\nFR 0 1 0 0 50\nEX 0 2 1 0 1\nEN\n",
        )
        .expect("parses");
        d.resolve().expect("resolves");
        let g = Geometry::build_on(&d, Some(&gf.geometry)).expect("builds");
        assert!(matches!(
            read_program_on(&d, &g, Some(&gf.program)),
            Err(crate::ProgramError::NotWithNgf { .. })
        ));
        assert!(GreensFunction::from_bytes(b"\x04\x00\x00\x00junk").is_err());
    }
}
