//! Reading the program-control cards: `FR`, `EX`, `KH`, `EK`, `GN`, `GD`, `LD`, `TL`, `NT`,
//! `RP`, `NE`, `NH` and `WG`.
//!
//! An `NX` card in a deck that was not first split into its runs (see [`nec_cards::Deck::runs`]) is
//! reported as unsupported rather than ignored: ignoring it would silently solve the wrong
//! structure and report the result as if it were right.

use crate::excitation::{IncidentField, IncidentGround, Polarization};
use crate::farfield::{FarFieldGround, GainBasis, Pattern, PatternRequest};
use crate::nearfield::{NearFieldContext, NearFieldRequest};
use crate::solve::Solution;
use crate::network::{Network, NetworkKind};
use crate::solve::{SolveOptions, SourceKind, VoltageSource};
use nec_cards::{Deck, Item};
use nec_core::Frequency;
use nec_geom::Geometry;
use nec_kernel::cmath::cdiv;
use nec_kernel::sommerfeld::SommerfeldGround;
use nec_kernel::somnec::SommerfeldGrid;
use nec_kernel::load::{LoadKind, Rlc};
use nec_kernel::{Ground, RadialScreen};
use std::sync::Arc;
use num_complex::Complex64;

/// What a deck asks the solver to do.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    /// Frequencies to solve, in order.
    pub frequencies: Vec<Frequency>,
    /// Applied-field voltage sources.
    pub sources: Vec<VoltageSource>,
    /// Lumped-approximation distance in wavelengths (`KH`), or NEC's default of 1.
    pub rkh: f64,
    /// The ground (`GN`), or free space.
    pub ground: GroundSpec,
    /// Segment loads (`LD`), one entry per loaded segment per card.
    pub loads: Vec<SegmentLoad>,
    /// Transmission lines and two-port networks (`TL`, `NT`), in deck order.
    pub networks: Vec<Network>,
    /// An incident-field excitation (`EX` 1–4), which replaces the voltage sources.
    pub incident: Option<IncidentSpec>,
    /// The extended thin-wire kernel (`EK`).
    pub extended_kernel: bool,
    /// Radiation patterns requested (`RP`), in deck order; each is computed for every solution.
    pub patterns: Vec<PatternRequest>,
    /// Near fields requested (`NE`, `NH`), in deck order.
    pub near_fields: Vec<NearFieldRequest>,
    /// The second ground medium for far fields — `[εr2, σ2, edge distance m, height m]` — from
    /// `GD`, or from `GN` fields 3–6 when `GN` gives no radial screen.
    pub second_medium: Option<[f64; 4]>,
    /// `WG`: write a numerical Green's function of this structure, to the file named.
    pub write_ngf: Option<String>,
    /// `CP`: segments (0-based, up to five) whose mutual coupling is wanted; see
    /// [`crate::coupling()`].
    pub coupling: Vec<usize>,
}

/// An `EX` card of type 1–4, as written: `EX type NTHETA NPHI . F1 .. F6`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IncidentSpec {
    /// 1–3: plane wave (linear, right elliptic, left elliptic); 4: elementary current source.
    pub kind: i32,
    /// Number of incidence angles in θ (plane waves).
    pub n_theta: usize,
    /// Number of incidence angles in φ (plane waves).
    pub n_phi: usize,
    /// `F1`..`F6`.
    pub f: [f64; 6],
}

impl Program {
    /// The incident fields to solve for, one solution each, in the order NEC solves them: θ
    /// stepping fastest, then φ, the angles accumulated by repeated addition as NEC does.
    /// Empty when the deck is excited by voltage sources.
    #[must_use]
    pub fn incident_fields(&self) -> Vec<IncidentField> {
        let Some(spec) = self.incident else {
            return Vec::new();
        };
        let f = spec.f;
        if spec.kind == 4 {
            return vec![IncidentField::CurrentElement {
                position: [f[0], f[1], f[2]],
                alpha: f[3],
                beta: f[4],
                moment: f[5],
            }];
        }
        let polarization = match spec.kind {
            1 => Polarization::Linear,
            2 => Polarization::RightElliptic,
            _ => Polarization::LeftElliptic,
        };
        let mut out = Vec::new();
        let mut phi = f[1];
        for _ in 0..spec.n_phi.max(1) {
            let mut theta = f[0];
            for _ in 0..spec.n_theta.max(1) {
                out.push(IncidentField::PlaneWave {
                    polarization,
                    theta,
                    phi,
                    eta: f[2],
                    axial_ratio: f[5],
                });
                theta += f[3];
            }
            phi += f[4];
        }
        out
    }
}

/// One `LD` card's load on one segment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SegmentLoad {
    /// 0-based segment index.
    pub segment: usize,
    /// The load.
    pub kind: LoadKind,
}

/// Why a deck cannot be solved by this build.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ProgramError {
    /// A card the free-space wire solver does not implement.
    #[error("line {line}: {mnemonic} {what} is not supported yet")]
    Unsupported {
        /// Source line.
        line: usize,
        /// The card.
        mnemonic: String,
        /// What about it is unsupported.
        what: String,
    },
    /// No excitation.
    #[error("no EX card: nothing excites the structure")]
    NoSource,
    /// An `EX` card addresses a segment that does not exist.
    #[error("line {line}: EX tag {tag} segment {segment} does not exist")]
    BadSegment {
        /// Source line.
        line: usize,
        /// Tag addressed.
        tag: i32,
        /// Segment addressed.
        segment: i32,
    },
    /// A card a run built on a numerical Green's function cannot have: the frequency, the ground
    /// and the loads of its structure are fixed by the Green's function.
    #[error("line {line}: {what} is not allowed with a numerical Green's function")]
    NotWithNgf {
        /// Source line.
        line: usize,
        /// What was refused.
        what: String,
    },
    /// A malformed card or out-of-range value.
    #[error("{0}")]
    Bad(String),
}

/// Interpret the program-control part of a resolved deck.
///
/// When several `FR` cards appear the last one before execution wins; `EX` cards accumulate.
pub fn read_program(deck: &Deck, geom: &Geometry) -> Result<Program, ProgramError> {
    read_program_on(deck, geom, None)
}

/// [`read_program`] for a run built on a numerical Green's function (`GF`), whose program is
/// `base`. As in NEC the run inherits the Green's function's frequency, ground and loads, and may
/// not change them: `FR` and `GN` cards are refused, and so are loads on the base's segments.
#[allow(clippy::too_many_lines)]
pub fn read_program_on(deck: &Deck, geom: &Geometry, base: Option<&Program>) -> Result<Program, ProgramError> {
    let mut frequencies: Option<Vec<Frequency>> = None;
    let mut sources: Vec<VoltageSource> = Vec::new();
    let mut rkh = nec_kernel::consts::DEFAULT_RKH;
    let mut ground = GroundSpec::FreeSpace;
    let mut loads: Vec<SegmentLoad> = Vec::new();
    let mut networks: Vec<Network> = Vec::new();
    let mut incident: Option<IncidentSpec> = None;
    let mut extended_kernel = false;
    let mut patterns: Vec<PatternRequest> = Vec::new();
    let mut near_fields: Vec<NearFieldRequest> = Vec::new();
    let mut second_medium: Option<[f64; 4]> = None;
    // NEC's IFLOW: the last program card that sets it. Consecutive LD cards accumulate; an LD
    // after any other such card starts a new set (NLOAD = 0). Only PT, PQ and PL leave it alone.
    let mut last_flow = String::new();
    let mut past_geometry = false;
    let mut write_ngf: Option<String> = None;
    let mut coupling: Vec<usize> = Vec::new();
    let refuse = |card: &nec_cards::Card, what: &str| -> Result<(), ProgramError> {
        if base.is_some() {
            Err(ProgramError::NotWithNgf {
                line: card.line,
                what: what.to_string(),
            })
        } else {
            Ok(())
        }
    };

    let unsupported = |line: usize, m: &str, what: &str| ProgramError::Unsupported {
        line,
        mnemonic: m.to_string(),
        what: what.to_string(),
    };
    let bad = |e: nec_cards::CardError| ProgramError::Bad(e.to_string());

    for item in &deck.items {
        let Item::Card(card) = item else { continue };
        if !past_geometry {
            if card.mnemonic == "GE" {
                past_geometry = true;
            }
            continue;
        }
        let continues_loads = last_flow == "LD";
        // NT and TL share IFLOW = 6: consecutive ones accumulate, one after anything else
        // starts a new set (NONET = 0).
        let continues_networks = last_flow == "NT" || last_flow == "TL";
        // EX cards group the same way (IFLOW = 5): an EX after any other card starts a new list
        // of sources (NSANT = NVQD = 0).
        let continues_sources = last_flow == "EX";
        let last_flow_before_cp = last_flow.clone();
        if !matches!(card.mnemonic.as_str(), "PT" | "PQ" | "PL") {
            last_flow.clone_from(&card.mnemonic);
        }
        match card.mnemonic.as_str() {
            "FR" => {
                refuse(card, "an FR card")?;
                let kind = card.int(0).map_err(bad)?;
                let count = card.int(1).map_err(bad)?.max(1);
                let start = card.real(4).map_err(bad)?;
                let step = card.real_or(5, 0.0).map_err(bad)?;
                let mut list = Vec::new();
                let mut f = start;
                for k in 0..count {
                    // Linear: F + k·Δ. Logarithmic: NEC multiplies cumulatively, F·Δ^k.
                    let mhz = if kind == 1 {
                        if k > 0 {
                            f *= step;
                        }
                        f
                    } else {
                        start + f64::from(k) * step
                    };
                    list.push(
                        Frequency::from_mhz(mhz)
                            .map_err(|e| ProgramError::Bad(format!("line {}: {e}", card.line)))?,
                    );
                }
                frequencies = Some(list);
            }
            "EX" => {
                // Starting a new EX set clears the sources; `incident` is set by both branches
                // below.
                if !continues_sources {
                    sources.clear();
                }
                let kind = card.int(0).map_err(bad)?;
                if (1..=4).contains(&kind) {
                    // An incident field replaces any voltage sources (NSANT = NVQD = 0).
                    sources.clear();
                    let count = |i: usize| -> Result<usize, ProgramError> {
                        Ok(usize::try_from(card.int(i).map_err(bad)?.max(0)).unwrap_or(0))
                    };
                    let mut f = [0.0; 6];
                    for (k, v) in f.iter_mut().enumerate() {
                        *v = card.real_or(4 + k, 0.0).map_err(bad)?;
                    }
                    incident = Some(IncidentSpec {
                        kind,
                        n_theta: count(1)?,
                        n_phi: count(2)?,
                        f,
                    });
                    continue;
                }
                incident = None;
                if kind == 6 {
                    return Err(unsupported(
                        card.line,
                        "EX",
                        "type 6 is a 4nec2 current source; lower the deck with \
                         nec_cards::lower_4nec2 first",
                    ));
                }
                let source_kind = match kind {
                    0 => SourceKind::AppliedField,
                    5 => SourceKind::CurrentSlopeDiscontinuity,
                    _ => {
                        return Err(unsupported(
                            card.line,
                            "EX",
                            &format!("type {kind} (only voltage sources, types 0 and 5)"),
                        ));
                    }
                };
                let tag = card.int(1).map_err(bad)?;
                let seg = card.int(2).map_err(bad)?;
                let position = usize::try_from(seg).ok().filter(|&p| p > 0);
                let s = position
                    .and_then(|p| geom.resolve_segment(tag, p))
                    .ok_or(ProgramError::BadSegment {
                        line: card.line,
                        tag,
                        segment: seg,
                    })?;
                let mut voltage = Complex64::new(
                    card.real_or(4, 0.0).map_err(bad)?,
                    card.real_or(5, 0.0).map_err(bad)?,
                );
                // NEC's main program: IF (ABS(VSANT).LT.1.D-20) VSANT=(1.,0.), and the same for
                // EX 5. A source written as 0 V is driven at 1 V — so a deck that "switches off"
                // a feed with EX 0 tag seg 0 0 0 still drives it (HFmultiband\5BWIDE-1.nec).
                if voltage.norm() < 1e-20 {
                    voltage = Complex64::new(1.0, 0.0);
                }
                sources.push(VoltageSource {
                    segment: s.index - 1,
                    voltage,
                    kind: source_kind,
                });
            }
            "KH" => rkh = card.real_or(4, rkh).map_err(bad)?,
            "GN" => {
                refuse(card, "a GN card")?;
                ground = read_gn(card).map_err(|e| match e {
                    GnError::Card(e) => bad(e),
                    GnError::Unsupported(what) => unsupported(card.line, "GN", &what),
                })?;
                // NEC's main program, label 23: without a radial screen, GN's fields 3-6 are the
                // second medium for far fields.
                let iperf = card.real_or(0, 0.0).map_err(bad)?;
                let nradl = card.real_or(1, 0.0).map_err(bad)?;
                if iperf > -0.5 && nradl.abs() < 0.5 {
                    let mut m = [0.0; 4];
                    for (k, v) in m.iter_mut().enumerate() {
                        *v = card.real_or(6 + k, 0.0).map_err(bad)?;
                    }
                    second_medium = Some(m);
                }
            }
            "GD" => {
                let mut m = [0.0; 4];
                for (k, v) in m.iter_mut().enumerate() {
                    *v = card.real_or(4 + k, 0.0).map_err(bad)?;
                }
                second_medium = Some(m);
            }
            "RP" => {
                #[allow(clippy::cast_possible_truncation)]
                let int = |i: usize| card.real_or(i, 0.0).map(|v| v.trunc() as i32).map_err(bad);
                let mut f = [0.0; 6];
                for (k, v) in f.iter_mut().enumerate() {
                    *v = card.real_or(4 + k, 0.0).map_err(bad)?;
                }
                patterns.push(PatternRequest::from_fields(int(0)?, int(1)?, int(2)?, int(3)?, f));
            }
            "LD" => {
                if !continues_loads {
                    loads.clear();
                }
                read_ld(card, geom, &mut loads).map_err(|e| match e {
                    LdError::Card(e) => bad(e),
                    LdError::Unsupported(what) => unsupported(card.line, "LD", &what),
                    LdError::Bad(what) => ProgramError::Bad(format!("line {}: {what}", card.line)),
                })?;
            }
            // EK turns the extended thin-wire kernel on; EK -1 turns it off again.
            "EK" => extended_kernel = card.real_or(0, 0.0).map_err(bad)? > -0.5,
            m @ ("TL" | "NT") => {
                if !continues_networks {
                    networks.clear();
                }
                if let Some(net) = read_network(card, m == "TL", geom)? {
                    networks.push(net);
                }
            }
            "WG" => {
                refuse(card, "writing another one (WG)")?;
                write_ngf = Some(nec_cards::ngf_file_name(card));
            }
            // Consecutive CP cards accumulate (IFLOW = 2); each names up to two segments.
            "CP" => {
                if last_flow_before_cp != "CP" {
                    coupling.clear();
                }
                for k in [0, 2] {
                    let seg = card.int(k + 1).map_err(bad)?;
                    if seg == 0 {
                        break;
                    }
                    let tag = card.int(k).map_err(bad)?;
                    let s = usize::try_from(seg)
                        .ok()
                        .and_then(|p| geom.resolve_segment(tag, p))
                        .ok_or(ProgramError::BadSegment {
                            line: card.line,
                            tag,
                            segment: seg,
                        })?;
                    coupling.push(s.index - 1);
                }
                if coupling.len() > 5 {
                    return Err(ProgramError::Bad(format!("line {}: CP names more than five segments", card.line)));
                }
            }
            m @ "NX" => {
                return Err(unsupported(card.line, m, "card"));
            }
            // Output requests (execute, pattern, near field, print charges/currents) do not
            // change the solution.
            m @ ("NE" | "NH") => {
                #[allow(clippy::cast_possible_truncation)]
                let int = |i: usize| card.real_or(i, 0.0).map(|v| v.trunc() as i64).map_err(bad);
                let count = |i: usize| int(i).map(|v| usize::try_from(v.max(1)).unwrap_or(1));
                let mut f = [0.0; 6];
                for (k, v) in f.iter_mut().enumerate() {
                    *v = card.real_or(4 + k, 0.0).map_err(bad)?;
                }
                near_fields.push(NearFieldRequest {
                    magnetic: m == "NH",
                    spherical: int(0)? == 1,
                    count: [count(1)?, count(2)?, count(3)?],
                    start: [f[0], f[1], f[2]],
                    step: [f[3], f[4], f[5]],
                });
            }
            "XQ" | "PQ" | "PT" | "EN" => {}
            other => return Err(unsupported(card.line, other, "card")),
        }
    }

    // With no FR card NEC solves once at its initial FMHZ = CVEL, 299.8 MHz (one wavelength of
    // one metre, as near as its constant gets).
    let frequencies = match (base, frequencies) {
        (Some(b), _) => b.frequencies.first().copied().into_iter().collect(),
        (None, Some(f)) => f,
        (None, None) => vec![Frequency::from_mhz(nec_core::CVEL_MHZ_M).expect("CVEL is a valid frequency")],
    };
    if let Some(b) = base {
        ground = b.ground;
        if let Some(l) = loads.iter().find(|l| l.segment < geom.base_segments) {
            return Err(ProgramError::NotWithNgf {
                line: 0,
                what: format!("loading segment {} of the Green's function", l.segment + 1),
            });
        }
        // The base's own loads stay on its segments.
        let mut all = b.loads.clone();
        all.append(&mut loads);
        loads = all;
    }
    // A run that only writes a Green's function needs no excitation.
    if sources.is_empty() && incident.is_none() && write_ngf.is_none() {
        return Err(ProgramError::NoSource);
    }
    Ok(Program {
        frequencies,
        sources,
        rkh,
        ground,
        loads,
        networks,
        incident,
        extended_kernel,
        patterns,
        near_fields,
        second_medium,
        write_ngf,
        coupling,
    })
}

/// The ground a deck asks for, independent of frequency.
///
/// Read from the `GN` card's `IPERF` (`I1`). Only `GN` selects ground: `GE` decides which wire
/// ends are bonded to the ground plane, which is connectivity, not the field calculation.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum GroundSpec {
    /// No ground (no `GN`, or `GN -1`).
    #[default]
    FreeSpace,
    /// `GN 1`: a perfectly conducting plane at `z = 0`.
    Perfect,
    /// `GN 0`: finite ground by the reflection-coefficient approximation.
    Reflection {
        /// Relative permittivity.
        epsr: f64,
        /// Conductivity in S/m, as written. Negative is NEC's alternate form; see
        /// [`Program::options`].
        sigma: f64,
        /// A radial-wire ground screen, when `GN` gives a number of radials.
        screen: Option<ScreenSpec>,
    },
    /// `GN 2`: finite ground by the Sommerfeld integrals (NEC's Sommerfeld/Norton method).
    Sommerfeld {
        /// Relative permittivity.
        epsr: f64,
        /// Conductivity in S/m, as written; negative as for [`GroundSpec::Reflection`].
        sigma: f64,
    },
    /// `GN 3`, which NEC-2 does not document. The engine prints "FINITE GROUND. REFLECTION
    /// COEFFICIENT APPROXIMATION" for it, but `EFLD` tests `IPERF > 0` and so fills the matrix
    /// exactly as for perfect ground; only the far-field routines treat it as finite ground.
    /// Confirmed against the reference engine: a vertical dipole gives the same impedance under
    /// `GN 1` and `GN 3`.
    PerfectNearField {
        /// Relative permittivity (used by far-field patterns only).
        epsr: f64,
        /// Conductivity in S/m (used by far-field patterns only).
        sigma: f64,
    },
}

/// A radial-wire ground screen as written on the `GN` card, in metres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenSpec {
    /// Number of radials (`NRADL`).
    pub count: u32,
    /// Radial length, metres.
    pub length_m: f64,
    /// Radial wire radius, metres.
    pub radius_m: f64,
}

impl Program {
    /// The ground as far-field patterns see it at `frequency`.
    #[must_use]
    pub fn far_field_ground(&self, frequency: Frequency) -> FarFieldGround {
        let wlam = frequency.wavelength_m();
        let incident = self.options(frequency).incident_ground;
        let (present, perfect, zrati) = match incident {
            IncidentGround::None => (false, false, Complex64::new(1.0, 0.0)),
            IncidentGround::Perfect => (true, true, Complex64::new(1.0, 0.0)),
            IncidentGround::Finite(z) => (true, false, z),
        };
        let screen = match self.ground {
            GroundSpec::Reflection { screen: Some(s), .. } => {
                Some(RadialScreen::new(s.count, s.length_m, s.radius_m, wlam))
            }
            _ => None,
        };
        FarFieldGround {
            present,
            perfect,
            zrati,
            second: self
                .second_medium
                .map(|[e, s, cl, ch]| FarFieldGround::second_medium(e, s, cl, ch, wlam)),
            screen,
        }
    }

    /// The radiation pattern `request` asks for, from solution `sol` of `geom`.
    #[must_use]
    pub fn pattern(&self, geom: &Geometry, sol: &Solution, request: &PatternRequest) -> Pattern {
        let segs = nec_kernel::scale(geom, sol.frequency);
        let patches = nec_kernel::scale_patches(geom, sol.frequency);
        crate::farfield::pattern(
            request,
            &segs,
            &sol.coefficients,
            &crate::farfield::Surface {
                patches: &patches,
                currents: &sol.patch_currents,
            },
            sol.frequency.wavelength_m(),
            &self.far_field_ground(sol.frequency),
            self.gain_basis(sol, request),
        )
    }

    /// The near field `request` asks for, from solution `sol` of `geom`: one `(point in metres,
    /// [x, y, z] field)` per observation point — volts per metre for `NE`, amperes per metre for
    /// `NH`.
    #[must_use]
    pub fn near_field(
        &self,
        geom: &Geometry,
        sol: &Solution,
        request: &NearFieldRequest,
    ) -> Vec<([f64; 3], [Complex64; 3])> {
        let f = sol.frequency;
        let wlam = f.wavelength_m();
        let segs = nec_kernel::scale(geom, f);
        let patches = nec_kernel::scale_patches(geom, f);
        let options = self.options(f);
        let image = options.image_ground();
        let ext: Option<Vec<[nec_kernel::ExtendedEnd; 2]>> = options
            .extended_kernel
            .then(|| (0..segs.len()).map(|j| nec_kernel::extended_ends_off_matrix(j, &segs, &geom.connectivity)).collect());
        let far = self.far_field_ground(f);
        let ctx = NearFieldContext {
            segs: &segs,
            abc: &sol.coefficients,
            patches: &patches,
            patch_currents: &sol.patch_currents,
            image: &image,
            ground: &options.ground,
            far_ground: &far,
            extended: ext.as_deref(),
            rkh: options.rkh,
        };
        request
            .points()
            .into_iter()
            .map(|p| {
                let pw = [p[0] / wlam, p[1] / wlam, p[2] / wlam];
                (p, if request.magnetic { ctx.magnetic(pw) } else { ctx.electric(pw) })
            })
            .collect()
    }

    /// How to turn `sol`'s fields into gains for `request`: relative to input power for voltage
    /// sources (or to radiated power for directive gain), as a scattering cross-section for an
    /// incident plane wave.
    #[must_use]
    pub fn gain_basis(&self, sol: &Solution, request: &PatternRequest) -> GainBasis {
        let wlam = sol.frequency.wavelength_m();
        match self.incident {
            Some(spec) if spec.kind <= 3 => GainBasis::for_plane_wave(spec.f[5]),
            Some(spec) => {
                let input = GainBasis::current_element_power(spec.f[5], wlam);
                let radiated = input - sol.power.structure_loss - sol.power.network_loss;
                GainBasis::from_power(wlam, input, radiated, request.directive)
            }
            None => GainBasis::from_power(wlam, sol.power.input, sol.power.radiated(), request.directive),
        }
    }

    /// The fill options for one frequency, with the ground evaluated there.
    ///
    /// A negative conductivity follows NEC's main program: on the first solve it is replaced by
    /// `|σ| / (59.96·λ)` at that frequency — so `ε_c` has imaginary part `−|σ|` there — and the
    /// replaced, positive value is kept for every later frequency. The first solve is taken to
    /// be the first frequency of the sweep.
    #[must_use]
    pub fn options(&self, frequency: Frequency) -> SolveOptions {
        let lambda = frequency.wavelength_m();
        let conductivity = |sigma: f64| {
            if sigma < 0.0 {
                let first = self.frequencies.first().map_or(lambda, |f| f.wavelength_m());
                -sigma / (nec_kernel::ground::SIG_TO_EPS * first)
            } else {
                sigma
            }
        };
        // ETMNS reflects a plane wave off perfect ground only for IPERF = 1; every other ground,
        // GN 3 included, uses ZRATI = 1/sqrt(eps_c).
        let incident_ground = match self.ground {
            GroundSpec::FreeSpace => IncidentGround::None,
            GroundSpec::Perfect => IncidentGround::Perfect,
            GroundSpec::Reflection { epsr, sigma, .. }
            | GroundSpec::Sommerfeld { epsr, sigma }
            | GroundSpec::PerfectNearField { epsr, sigma } => {
                let epsc = nec_kernel::ground::complex_permittivity(epsr, conductivity(sigma), lambda);
                IncidentGround::Finite(nec_kernel::cmath::rdiv(1.0, nec_kernel::cmath::csqrt(epsc)))
            }
        };
        let ground = match self.ground {
            GroundSpec::FreeSpace => Ground::FreeSpace,
            GroundSpec::Perfect | GroundSpec::PerfectNearField { .. } => Ground::Perfect,
            GroundSpec::Sommerfeld { epsr, sigma } => {
                let sigma_now = conductivity(sigma);
                let epsc = nec_kernel::ground::complex_permittivity(epsr, sigma_now, lambda);
                // NEC builds the grid at each frequency, except for a negative conductivity:
                // then only at the first (LLNEG), and later frequencies reuse it while its
                // permittivity is within 1e-3 of the current one. Beyond that NEC stops with
                // "ERROR IN GROUND PARAMETERS" (confirmed on the reference engine with a
                // 280-320 MHz sweep); here the grid is rebuilt for the current frequency
                // instead, exactly as for a positive conductivity.
                let reused = self
                    .frequencies
                    .first()
                    .filter(|_| sigma < 0.0)
                    .map(|first| SommerfeldGrid::new(first.mhz(), epsr, sigma_now))
                    .filter(|g| cdiv(g.epscf - epsc, epsc).norm() < 1e-3);
                let grid =
                    reused.unwrap_or_else(|| SommerfeldGrid::new(frequency.mhz(), epsr, sigma_now));
                Ground::Sommerfeld(SommerfeldGround::new(epsc, Arc::new(grid)))
            }
            GroundSpec::Reflection { epsr, sigma, screen } => Ground::reflection(
                epsr,
                conductivity(sigma),
                lambda,
                screen.map(|s| RadialScreen::new(s.count, s.length_m, s.radius_m, lambda)),
            ),
        };
        SolveOptions {
            rkh: self.rkh,
            ground,
            loads: self.loads.clone(),
            networks: self.networks.clone(),
            incident: None,
            incident_ground,
            extended_kernel: self.extended_kernel,
        }
    }
}

enum GnError {
    Card(nec_cards::CardError),
    Unsupported(String),
}

/// `GN I1 I2 . . EPSR SIG F3 F4 F5 F6`.
fn read_gn(card: &nec_cards::Card) -> Result<GroundSpec, GnError> {
    let iperf = card.int(0).map_err(GnError::Card)?;
    let real = |i: usize| card.real_or(i, 0.0).map_err(GnError::Card);
    match iperf {
        -1 => Ok(GroundSpec::FreeSpace),
        1 => Ok(GroundSpec::Perfect),
        0 => {
            #[allow(clippy::cast_possible_truncation)]
            let nradl = real(1)?.trunc() as i32;
            let screen = if nradl > 0 {
                Some(ScreenSpec {
                    count: u32::try_from(nradl).unwrap_or(u32::MAX),
                    length_m: real(6)?,
                    radius_m: real(7)?,
                })
            } else {
                None
            };
            Ok(GroundSpec::Reflection {
                epsr: real(4)?,
                sigma: real(5)?,
                screen,
            })
        }
        3 => Ok(GroundSpec::PerfectNearField {
            epsr: real(4)?,
            sigma: real(5)?,
        }),
        2 => {
            // NEC stops: "RADIAL WIRE G. S. APPROXIMATION MAY NOT BE USED WITH SOMMERFELD
            // GROUND OPTION".
            if real(1)? > 0.0 {
                return Err(GnError::Unsupported(
                    "a radial-wire screen with Sommerfeld ground".to_string(),
                ));
            }
            Ok(GroundSpec::Sommerfeld {
                epsr: real(4)?,
                sigma: real(5)?,
            })
        }
        other => Err(GnError::Unsupported(format!("ground type {other}"))),
    }
}

enum LdError {
    Card(nec_cards::CardError),
    Unsupported(String),
    Bad(String),
}

/// `LD TYPE TAG FROM TO  R L C`: append the card's load on each segment it selects.
///
/// Segment selection follows NEC's main program and `LOAD`: tag 0 with no range loads every
/// segment; tag 0 with a range loads those absolute segment numbers; a non-zero tag loads its
/// segments numbered `FROM..=TO` within the tag (all of them when `FROM` is 0). A zero `TO`
/// means `TO = FROM`.
///
/// Type −1 adds nothing. It clears the earlier loads only the way any `LD` does, by starting a
/// new set after a non-`LD` card; directly after other `LD` cards NEC appends it as a load of
/// type −1, which `LOAD`'s computed `GO TO` turns into a zero series load, and the earlier loads
/// stay. Confirmed on the reference engine (a 1 kΩ load followed by `LD -1` still reads 1051 Ω).
fn read_ld(
    card: &nec_cards::Card,
    geom: &Geometry,
    loads: &mut Vec<SegmentLoad>,
) -> Result<(), LdError> {
    #[allow(clippy::cast_possible_truncation)]
    let int = |i: usize| card.real_or(i, 0.0).map(|v| v.trunc() as i64).map_err(LdError::Card);
    let real = |i: usize| card.real_or(i, 0.0).map_err(LdError::Card);
    let kind_code = int(0)?;
    if kind_code == -1 {
        return Ok(());
    }
    let (r, l, c) = (real(4)?, real(5)?, real(6)?);
    let kind = match kind_code {
        0 => LoadKind::SeriesRlc(Rlc { r, l, c }),
        1 => LoadKind::ParallelRlc(Rlc { r, l, c }),
        2 => LoadKind::SeriesRlcPerMetre(Rlc { r, l, c }),
        3 => LoadKind::ParallelRlcPerMetre(Rlc { r, l, c }),
        4 => LoadKind::Impedance { r, x: l },
        5 => LoadKind::Conductivity { sigma: r },
        6 | 7 => {
            return Err(LdError::Unsupported(format!(
                "type {kind_code} is a 4nec2 extension; lower the deck with \n                 nec_cards::lower_4nec2 first"
            )));
        }
        other => return Err(LdError::Bad(format!("LD type {other} does not exist"))),
    };

    let tag = int(1)?;
    let from = int(2)?;
    let mut to = int(3)?;
    if to == 0 {
        to = from;
    }
    if to < from {
        return Err(LdError::Bad(format!("LD segment range {from} to {to} is reversed")));
    }
    let before = loads.len();
    if tag == 0 {
        #[allow(clippy::cast_possible_wrap)]
        let n = geom.len() as i64;
        let (lo, hi) = if from == 0 { (1, n) } else { (from, to) };
        if lo < 1 || hi > n {
            return Err(LdError::Bad(format!("LD segments {lo} to {hi}, but there are {n}")));
        }
        for s in lo..=hi {
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            loads.push(SegmentLoad { segment: (s - 1) as usize, kind });
        }
    } else {
        let mut ichk = 0;
        for (i, seg) in geom.segments.iter().enumerate() {
            if i64::from(seg.tag) != tag {
                continue;
            }
            ichk += 1;
            if from == 0 || (from..=to).contains(&ichk) {
                loads.push(SegmentLoad { segment: i, kind });
            }
        }
    }
    if loads.len() == before {
        // NEC: "LOADING DATA CARD ERROR, NO SEGMENT HAS AN ITAG = ..."
        return Err(LdError::Bad(format!("LD selects no segment (tag {tag}, {from} to {to})")));
    }
    Ok(())
}

/// `NT TAG1 SEG1 TAG2 SEG2  Y11R Y11I Y12R Y12I Y22R Y22I` or
/// `TL TAG1 SEG1 TAG2 SEG2  Z0 LENGTH Y1R Y1I Y2R Y2I`.
///
/// A port segment of -1 on port 1 clears the networks without adding one (NEC tests it only
/// when the card starts a new set; within a set it would fail to resolve). A negative `Z0`
/// is a crossed line.
fn read_network(
    card: &nec_cards::Card,
    is_line: bool,
    geom: &Geometry,
) -> Result<Option<Network>, ProgramError> {
    let bad = |e: nec_cards::CardError| ProgramError::Bad(e.to_string());
    let real = |i: usize| card.real_or(i, 0.0).map_err(bad);
    #[allow(clippy::cast_possible_truncation)]
    let int = |i: usize| real(i).map(|v| v.trunc() as i32);
    let (tag1, seg1, tag2, seg2) = (int(0)?, int(1)?, int(2)?, int(3)?);
    if seg1 == -1 {
        return Ok(None);
    }
    let resolve = |tag: i32, seg: i32| {
        usize::try_from(seg)
            .ok()
            .filter(|&p| p > 0)
            .and_then(|p| geom.resolve_segment(tag, p))
            .map(|s| s.index - 1)
            .ok_or(ProgramError::BadSegment {
                line: card.line,
                tag,
                segment: seg,
            })
    };
    let port1 = resolve(tag1, seg1)?;
    let port2 = resolve(tag2, seg2)?;
    let f: Vec<f64> = (4..10).map(real).collect::<Result<_, _>>()?;
    let kind = if is_line {
        NetworkKind::Line {
            z0: f[0].abs(),
            length_m: f[1],
            crossed: f[0] <= 0.0,
            shunt1: Complex64::new(f[2], f[3]),
            shunt2: Complex64::new(f[4], f[5]),
        }
    } else {
        NetworkKind::Admittance {
            y11: Complex64::new(f[0], f[1]),
            y12: Complex64::new(f[2], f[3]),
            y22: Complex64::new(f[4], f[5]),
        }
    };
    Ok(Some(Network { port1, port2, kind }))
}
