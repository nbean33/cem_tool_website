//! Solving for the currents and reporting input impedance (`ETMNS`, `QDSRC`,
//! `FACTRS`/`SOLVES`, `CABC`, the source branch of `NETWK`).

use crate::excitation::{IncidentField, IncidentGround};
use crate::fill::{Structure, fill};
use crate::linalg::Factors;
use crate::network::{Network, solve_networks};
use crate::program::SegmentLoad;
use faer::Mat;
use nec_core::Frequency;
use nec_geom::{Geometry, Junction};
use nec_kernel::consts::TP;
use nec_kernel::hfield::hsfld;
use nec_kernel::{
    Contribution, ExtendedEnd, Ground, ImageGround, Piece, SurfacePatch, WireSeg, basis_function, basis_function_with, efld, extended_ends, extended_ends_off_matrix,
    pieces_by_segment, scale, scale_patches,
};
use num_complex::Complex64;

type C = Complex64;
const ZERO: C = C::new(0.0, 0.0);

/// How a voltage source drives its segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SourceKind {
    /// `EX 0`: an applied field `−V/Δ` along the segment (NEC's usual delta-gap source).
    #[default]
    AppliedField,
    /// `EX 5`: a voltage across end 1 of the segment, modelled by the jump it causes in the
    /// derivative of the current (`QDSRC`). NEC reports its input current at that end, not at the
    /// segment centre.
    CurrentSlopeDiscontinuity,
}

/// A voltage source on one segment (`EX` type 0 or 5).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VoltageSource {
    /// 0-based segment index.
    pub segment: usize,
    /// Source voltage, volts.
    pub voltage: C,
    /// How the voltage is applied.
    pub kind: SourceKind,
}

/// Options that change the fill, for one frequency.
#[derive(Debug, Clone, PartialEq)]
pub struct SolveOptions {
    /// Distance in wavelengths beyond which interactions use the lumped approximation (`KH`).
    pub rkh: f64,
    /// The ground at this frequency. [`crate::Program::options`] builds it from the `GN` card.
    pub ground: Ground,
    /// Segment loads (`LD`).
    pub loads: Vec<SegmentLoad>,
    /// Transmission lines and two-port networks (`TL`, `NT`).
    pub networks: Vec<Network>,
    /// An incident field (`EX` 1–4) to solve for instead of voltage sources. A deck with a
    /// sweep of incidence angles needs one solve per angle; see
    /// [`crate::Program::incident_fields`].
    pub incident: Option<IncidentField>,
    /// How the ground reflects an incident plane wave.
    pub incident_ground: IncidentGround,
    /// The extended thin-wire kernel (`EK`), for segments too fat for the ordinary one.
    pub extended_kernel: bool,
}

impl SolveOptions {
    /// The ground as NEC's image-based routines see it — the wires' magnetic field and every
    /// patch field. Finite grounds of every kind appear as reflection coefficients; `GN 3`, which
    /// fills the wire matrix as perfect ground, is finite here.
    #[must_use]
    pub fn image_ground(&self) -> ImageGround {
        match &self.ground {
            Ground::FreeSpace => ImageGround::None,
            Ground::Perfect => match self.incident_ground {
                IncidentGround::Finite(zrati) => ImageGround::Finite { zrati, screen: None },
                _ => ImageGround::Perfect,
            },
            Ground::Reflection { zrati, screen } => ImageGround::Finite {
                zrati: *zrati,
                screen: *screen,
            },
            Ground::Sommerfeld(g) => ImageGround::Finite { zrati: g.u, screen: None },
        }
    }
}

impl Default for SolveOptions {
    fn default() -> Self {
        Self {
            rkh: nec_kernel::consts::DEFAULT_RKH,
            ground: Ground::FreeSpace,
            loads: Vec::new(),
            networks: Vec::new(),
            incident: None,
            incident_ground: IncidentGround::None,
            extended_kernel: false,
        }
    }
}

/// The engine's "ANTENNA INPUT PARAMETERS" row for one source.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InputParameters {
    /// 1-based segment number, as NEC prints it.
    pub segment: usize,
    /// Wire tag of the source segment.
    pub tag: i32,
    /// Source voltage, volts.
    pub voltage: C,
    /// Input current, amperes: at the segment centre for an applied-field source, at end 1 for
    /// a current-slope-discontinuity source.
    pub current: C,
    /// `V / I`, ohms.
    pub impedance: C,
    /// `I / V`, siemens.
    pub admittance: C,
    /// Input power `½·Re(V·I*)`, watts.
    pub power: f64,
}

/// NEC's "POWER BUDGET", watts.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PowerBudget {
    /// Total input power from the voltage sources (`PIN`). Zero for an incident-field excitation.
    pub input: f64,
    /// Power dissipated in segment loads, wire conductivity included (`PLOSS`).
    pub structure_loss: f64,
    /// Power absorbed by transmission lines and networks (`PNLS`).
    pub network_loss: f64,
}

impl PowerBudget {
    /// Radiated power: input less structure and network losses.
    #[must_use]
    pub fn radiated(&self) -> f64 {
        self.input - self.network_loss - self.structure_loss
    }

    /// Radiated power as a percentage of input power.
    #[must_use]
    pub fn efficiency_percent(&self) -> f64 {
        100.0 * self.radiated() / self.input
    }
}

/// The result of one frequency's solution.
#[derive(Debug, Clone, PartialEq)]
pub struct Solution {
    /// The frequency solved.
    pub frequency: Frequency,
    /// Current at each segment's centre, amperes, indexed by 0-based segment.
    pub currents: Vec<C>,
    /// Each segment's current as `A + B·sin(ks) + C·cos(ks)` along it (NEC's `AIR`..`CII`), in
    /// the kernel's units: multiply by the wavelength in metres for amperes. Far fields are
    /// computed from these.
    pub coefficients: Vec<[C; 3]>,
    /// Surface current density on each patch, A/m, as Cartesian `(x, y, z)` components — NEC's
    /// `SURFACE PATCH CURRENTS` table. Its components along the patch's tangents are
    /// `J·t1` and `J·t2`.
    pub patch_currents: Vec<[C; 3]>,
    /// One entry per voltage source.
    pub inputs: Vec<InputParameters>,
    /// One entry per network or transmission-line port, as NEC's "STRUCTURE EXCITATION DATA AT
    /// NETWORK CONNECTION POINTS": the ports that are not sources first, then those that are.
    /// Voltage is the network's across the port segment, current the structure's through it,
    /// so the impedance is what the structure presents at that port.
    pub network_ports: Vec<InputParameters>,
    /// Input power and where it goes.
    pub power: PowerBudget,
}

impl Solution {
    /// Charge per unit length at each segment centre, coulombs per metre — NEC's `PQ` table,
    /// `(10⁻⁶/f_MHz)·j·B`: the continuity equation `q = (j/ω)·dI/ds` applied to the current's
    /// sine term, the only one with a slope at the centre.
    #[must_use]
    pub fn charge_densities(&self) -> Vec<C> {
        let fr = 1e-6 / self.frequency.mhz();
        self.coefficients.iter().map(|t| C::new(0.0, fr) * t[1]).collect()
    }
}

/// Errors from the solve.
#[derive(Debug, thiserror::Error)]
pub enum SolveError {
    /// The structure has no segments and no patches.
    #[error("structure has no wire segments or surface patches")]
    Empty,
    /// A source referred to a segment that does not exist.
    #[error("source on segment {0} but the structure has {1} segments")]
    BadSource(usize, usize),
}

/// The current a charge-discontinuity source injects: segment `is`'s basis function with end 1
/// freed and no end cap (`TBF(IS,0)` with `ICON1(IS) = 0`), scaled by `CURD`.
struct InjectedCurrent {
    pieces: Vec<Piece>,
    curd: C,
}

impl InjectedCurrent {
    /// `QDSRC`/`CABC`: `CURD = CCJ·V / ((ln(2s/a) − 1)·(B·cos(ks) + C·sin(ks))·λ)`, with `s` half
    /// the segment length, `a` its radius and `B`, `C` the freed basis function's centre piece.
    fn new(is: usize, v: C, segs: &[WireSeg], geom: &Geometry, lambda: f64) -> Self {
        let own = nec_geom::Connectivity {
            end1: Junction::Free,
            end2: geom.connectivity[is].end2.clone(),
        };
        let pieces = basis_function_with(is, segs, &own, false);
        let centre = pieces.last().copied().expect("a basis function has a centre piece");
        let s = segs[is].length * 0.5;
        let ccj = C::new(0.0, -0.016_666_666_67);
        let denom = ((2.0 * s / segs[is].radius).ln() - 1.0)
            * (centre.b * (TP * s).cos() + centre.c * (TP * s).sin())
            * lambda;
        Self {
            pieces,
            curd: ccj * v / denom,
        }
    }
}

/// A structure made ready for the fill at one frequency: kernel units, the basis functions on
/// every segment, the extended kernel's end treatments, and the loads per unit length.
pub(crate) struct Prepared {
    pub(crate) segs: Vec<WireSeg>,
    pub(crate) patches: Vec<SurfacePatch>,
    pub(crate) image: ImageGround,
    pub(crate) by_segment: Vec<Vec<Contribution>>,
    pub(crate) ext_ends: Option<Vec<[ExtendedEnd; 2]>>,
    /// `ZARRAY`: each segment's load per unit length (loads on the same segment add).
    pub(crate) zarray: Vec<C>,
}

impl Prepared {
    pub(crate) fn new(geom: &Geometry, frequency: Frequency, options: &SolveOptions) -> Self {
        let segs = scale(geom, frequency);
        let lambda = frequency.wavelength_m();
        let mut zarray = vec![ZERO; segs.len()];
        for l in &options.loads {
            let s = &segs[l.segment];
            zarray[l.segment] += l.kind.per_unit_length(s.length, s.radius, lambda);
        }
        Self {
            patches: scale_patches(geom, frequency),
            image: options.image_ground(),
            by_segment: pieces_by_segment(&segs, &geom.connectivity),
            // CMWW decides each source segment's end treatment once, for the extended kernel.
            ext_ends: options
                .extended_kernel
                .then(|| (0..segs.len()).map(|j| extended_ends(j, &segs, &geom.connectivity)).collect()),
            zarray,
            segs,
        }
    }

    pub(crate) fn structure<'a>(&'a self, geom: &'a Geometry, options: &'a SolveOptions) -> Structure<'a> {
        Structure {
            segs: &self.segs,
            patches: &self.patches,
            by_segment: &self.by_segment,
            connectivity: &geom.connectivity,
            rkh: options.rkh,
            ground: &options.ground,
            image: &self.image,
            extended: self.ext_ends.as_deref(),
            zarray: &self.zarray,
        }
    }
}

/// Refuse what cannot be solved: an empty structure, or a source on a segment that is not there.
pub(crate) fn check(geom: &Geometry, sources: &[VoltageSource]) -> Result<(), SolveError> {
    let n = geom.len();
    if geom.is_empty() {
        return Err(SolveError::Empty);
    }
    match sources.iter().find(|s| s.segment >= n) {
        Some(s) => Err(SolveError::BadSource(s.segment + 1, n)),
        None => Ok(()),
    }
}

/// Solve one frequency for the given voltage sources.
pub fn solve(
    geom: &Geometry,
    frequency: Frequency,
    sources: &[VoltageSource],
    options: SolveOptions,
) -> Result<Solution, SolveError> {
    check(geom, sources)?;
    let prep = Prepared::new(geom, frequency, &options);
    let factors = Factors::new(&fill(&prep.structure(geom, &options)));
    Ok(finish(geom, frequency, sources, &options, &prep, &|rhs| factors.solve(rhs)))
}

/// Everything after the matrix: the right-hand side, the currents from `solve_rhs` (which
/// solves the system for any right-hand side), networks, input parameters and the power budget.
#[allow(clippy::too_many_lines)]
pub(crate) fn finish(
    geom: &Geometry,
    frequency: Frequency,
    sources: &[VoltageSource],
    options: &SolveOptions,
    prep: &Prepared,
    solve_rhs: &dyn Fn(&Mat<C>) -> Mat<C>,
) -> Solution {
    let n = geom.len();
    let (segs, patches, image, zarray) = (&prep.segs, &prep.patches, &prep.image, &prep.zarray);
    let ext_ends = &prep.ext_ends;
    let size = n + 2 * patches.len();
    let lambda = frequency.wavelength_m();

    // ETMNS: the right-hand side is the negative of the incident field. An applied-field voltage
    // source is the voltage spread over the source segment's length in metres.
    let mut e: Mat<C> = Mat::zeros(size, 1);
    if let Some(field) = &options.incident {
        for (i, v) in field.rhs(segs, patches, lambda, options.incident_ground).into_iter().enumerate() {
            e[(i, 0)] = v;
        }
    }
    for s in sources.iter().filter(|s| s.kind == SourceKind::AppliedField) {
        e[(s.segment, 0)] -= s.voltage / (segs[s.segment].length * lambda);
    }
    // QDSRC: a charge-discontinuity source injects a known current; its field (and its drop
    // across any load) moves to the right-hand side.
    let injected: Vec<(usize, InjectedCurrent)> = sources
        .iter()
        .filter(|s| s.kind == SourceKind::CurrentSlopeDiscontinuity)
        .map(|s| (s.segment, InjectedCurrent::new(s.segment, s.voltage, segs, geom, lambda)))
        .collect();
    for (_, inj) in &injected {
        for p in &inj.pieces {
            let src = &segs[p.segment];
            for (i, obs) in segs.iter().enumerate() {
                let f = efld(
                    src,
                    obs.center,
                    obs.radius,
                    i == p.segment,
                    options.rkh,
                    &options.ground,
                    // QDSRC classifies the ends itself, not as CMWW does.
                    ext_ends
                        .as_ref()
                        .map(|_| extended_ends_off_matrix(p.segment, segs, &geom.connectivity)),
                );
                let (etk, ets, etc) = f.along(obs.dir);
                e[(i, 0)] -= (etk * p.a + ets * p.b + etc * p.c) * inj.curd;
            }
            // Its magnetic field drives the patch equations.
            for (q, pt) in patches.iter().enumerate() {
                let h = hsfld(src, pt.center, 0.0, image);
                for (row, t) in [(n + 2 * q, pt.t2), (n + 2 * q + 1, pt.t1)] {
                    let along = |v: [C; 3]| v[0] * t[0] + v[1] * t[1] + v[2] * t[2];
                    e[(row, 0)] += (along(h[0]) * p.a + along(h[1]) * p.b + along(h[2]) * p.c) * inj.curd * pt.salp;
                }
            }
            let zl = zarray[p.segment];
            if zl.re != 0.0 || zl.im != 0.0 {
                e[(p.segment, 0)] += zl * inj.curd * (p.a + p.c);
            }
        }
    }

    // CABC reconstructs currents from the basis functions themselves (TBF with the end cap) —
    // not from the fill's TRIO contributions, which differ on coincident wires.
    let tbf: Vec<Vec<Piece>> = (0..n).map(|i| basis_function(i, segs, &geom.connectivity, true)).collect();
    // SOLGF then CABC: per-segment coefficients (A, B, C) of the current, in kernel units. NEC
    // adds the injected currents of charge-discontinuity sources to every solution, including
    // NETWK's unit solves.
    let coefficients = |rhs: &Mat<C>| -> Vec<[C; 3]> {
        let alpha = solve_rhs(rhs);
        let mut abc = vec![[ZERO; 3]; n];
        let mut add = |pieces: &[Piece], amp: C| {
            for p in pieces {
                let t = &mut abc[p.segment];
                t[0] += amp * p.a;
                t[1] += amp * p.b;
                t[2] += amp * p.c;
            }
        };
        for (i, pieces) in tbf.iter().enumerate() {
            add(pieces, alpha[(i, 0)]);
        }
        for (_, inj) in &injected {
            add(&inj.pieces, inj.curd);
        }
        abc
    };
    // The current at a segment's centre: A + C (sin 0 = 0, cos 0 = 1).
    let centre_currents = |rhs: &Mat<C>| -> Vec<C> {
        coefficients(rhs).iter().map(|t| t[0] + t[2]).collect()
    };

    // NETWK: with networks, the excitation gains the network fields at their ports, and each
    // applied-field source that is also a port delivers extra current into the network.
    let mut network_current = vec![ZERO; n];
    let mut eq_ports: Vec<(usize, C)> = Vec::new();
    let mut src_ports: Vec<(usize, C)> = Vec::new();
    if !options.networks.is_empty() {
        let seg_len: Vec<f64> = segs.iter().map(|s| s.length).collect();
        let src: Vec<(usize, C)> = sources
            .iter()
            .filter(|s| s.kind == SourceKind::AppliedField)
            .map(|s| (s.segment, s.voltage))
            .collect();
        let distance = |a: usize, b: usize| {
            let (p, q) = (segs[a].center, segs[b].center);
            lambda * ((q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2) + (q[2] - p[2]).powi(2)).sqrt()
        };
        let sol = solve_networks(
            &options.networks,
            &src,
            &seg_len,
            distance,
            lambda,
            &e,
            centre_currents,
        );
        e = sol.rhs;
        for (seg, i) in sol.source_currents {
            network_current[seg] = i;
        }
        eq_ports = sol.eq_ports;
        src_ports = sol.src_ports;
    }

    let abc = coefficients(&e);
    let currents: Vec<C> = abc.iter().map(|t| (t[0] + t[2]) * lambda).collect();
    // CABC: patch currents from their t1, t2 components to Cartesian.
    let patch_currents: Vec<[C; 3]> = if patches.is_empty() {
        Vec::new()
    } else {
        let alpha = solve_rhs(&e);
        patches
            .iter()
            .enumerate()
            .map(|(q, pt)| {
                let (c1, c2) = (alpha[(n + 2 * q, 0)], alpha[(n + 2 * q + 1, 0)]);
                [c1 * pt.t1[0] + c2 * pt.t2[0], c1 * pt.t1[1] + c2 * pt.t2[1], c1 * pt.t1[2] + c2 * pt.t2[2]]
            })
            .collect()
    };

    let inputs: Vec<InputParameters> = sources
        .iter()
        .map(|s| {
            let i = match s.kind {
                SourceKind::AppliedField => currents[s.segment] + network_current[s.segment] * lambda,
                SourceKind::CurrentSlopeDiscontinuity => {
                    // NETWK label 57: the current at end 1, A − B·sin(πΔ) + C·cos(πΔ).
                    let [a, b, c] = abc[s.segment];
                    let half = segs[s.segment].length * TP * 0.5;
                    (a - b * half.sin() + c * half.cos()) * lambda
                }
            };
            InputParameters {
                segment: s.segment + 1,
                tag: geom.segments[s.segment].tag,
                voltage: s.voltage,
                current: i,
                impedance: s.voltage / i,
                admittance: i / s.voltage,
                power: 0.5 * (s.voltage * i.conj()).re,
            }
        })
        .collect();

    // The power budget, as the main program and NETWK accumulate it. Structure loss is
    // ½|I|²·R' summed over loaded segments, R' the load's resistance per unit length.
    let mut power = PowerBudget::default();
    for (i, zl) in zarray.iter().enumerate() {
        if zl.re.abs() >= 1e-20 {
            power.structure_loss += 0.5 * currents[i].norm_sqr() * zl.re * segs[i].length;
        }
    }
    // Network loss: the power each port delivers into the structure counts against the
    // network; a source port's whole input power counts for it.
    for &(seg, x) in &eq_ports {
        let v = x * segs[seg].length * lambda;
        power.network_loss -= 0.5 * (v * currents[seg].conj()).re;
    }
    for &(seg, v) in &src_ports {
        power.network_loss -= 0.5 * (v * currents[seg].conj()).re;
    }
    for input in &inputs {
        power.input += input.power;
        if src_ports.iter().any(|&(seg, _)| seg + 1 == input.segment) {
            power.network_loss += input.power;
        }
    }

    let port = |seg: usize, voltage: C| {
        let current = currents[seg];
        InputParameters {
            segment: seg + 1,
            tag: geom.segments[seg].tag,
            voltage,
            current,
            impedance: voltage / current,
            admittance: current / voltage,
            power: 0.5 * (voltage * current.conj()).re,
        }
    };
    let network_ports = eq_ports
        .iter()
        .map(|&(seg, x)| port(seg, x * segs[seg].length * lambda))
        .chain(src_ports.iter().map(|&(seg, v)| port(seg, v)))
        .collect();

    Solution {
        frequency,
        currents,
        coefficients: abc,
        patch_currents,
        inputs,
        network_ports,
        power,
    }
}
