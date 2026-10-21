//! Impedance-matching networks between a source and the antenna: lumped L, Pi and T networks
//! with lossy components, and transmission-line matches (a parallel or series stub, or a
//! series section), as 4nec2's Matching (F10) and Stub forms offer them.
//!
//! A network is a cascade of [`Stage`]s from the source to the antenna. Its behaviour at any
//! frequency is the product of the stages' ABCD (chain) matrices, which converts to the
//! short-circuit admittances NEC's `NT` card takes. The sign convention is the one NEC's own
//! `TL` card uses (`Y11 = −j·cot βl / Z0`, `Y12 = +j / (Z0 sin βl)`), so a line stage and a `TL`
//! card agree.
//!
//! [`apply`] writes a network into a deck the way 4nec2 does: the feed's voltage source moves to
//! a far dummy wire (tag `9800 + n`, which 4nec2 hides), and an `NT` card joins that wire to
//! the original feed segment. The `NT` holds the admittances at the design frequency, so
//! 4nec2 and the reference engines see a working deck; its trailing comment carries the
//! network itself, from which [`crate::Model`] recomputes the admittances at every frequency it
//! solves, so a sweep through a matched antenna stays right off the design frequency.

use nec_cards::edit::{cards, delete_line, geometry_end, insert_card, replace_card};
use nec_cards::format_value;
use nec_core::constants::CVEL_MHZ_M;
use num_complex::Complex64;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

type C = Complex64;

const J: C = C::new(0.0, 1.0);
const ONE: C = C::new(1.0, 0.0);
const ZERO: C = C::new(0.0, 0.0);

/// The marker that starts a matching network's description in an `NT` card's comment.
pub const MARKER: &str = "nec-rs match:";

/// Base tag for the dummy wires carrying a matched source; 4nec2 hides tags from 9800.
pub const MATCH_TAG_BASE: i32 = 9800;

/// A lumped component. A `q` of zero means lossless (an "infinite Q", as 4nec2 enters it).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Component {
    /// An inductor, henries.
    Inductor {
        /// Inductance.
        henry: f64,
        /// Quality factor; 0 for lossless.
        q: f64,
    },
    /// A capacitor, farads.
    Capacitor {
        /// Capacitance.
        farad: f64,
        /// Quality factor; 0 for lossless.
        q: f64,
    },
}

impl Component {
    /// Impedance at angular frequency `w`: `R + jX` with `R = |X| / Q`.
    #[must_use]
    pub fn impedance(&self, w: f64) -> C {
        let (x, q) = match *self {
            Self::Inductor { henry, q } => (w * henry, q),
            Self::Capacitor { farad, q } => (-1.0 / (w * farad), q),
        };
        let r = if q > 0.0 { x.abs() / q } else { 0.0 };
        C::new(r, x)
    }

    /// The lossless component with reactance `x` ohms at `w` (an inductor for `x > 0`).
    fn for_reactance(x: f64, w: f64, q_l: f64, q_c: f64) -> Self {
        if x > 0.0 {
            Self::Inductor { henry: x / w, q: q_l }
        } else {
            Self::Capacitor { farad: -1.0 / (w * x), q: q_c }
        }
    }

    /// The lossless component with susceptance `b` siemens at `w` (a capacitor for `b > 0`).
    fn for_susceptance(b: f64, w: f64, q_l: f64, q_c: f64) -> Self {
        if b > 0.0 {
            Self::Capacitor { farad: b / w, q: q_c }
        } else {
            Self::Inductor { henry: -1.0 / (w * b), q: q_l }
        }
    }
}

/// A transmission line type.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Line {
    /// Characteristic impedance, ohms.
    pub z0: f64,
    /// Velocity factor.
    pub vf: f64,
}

impl Line {
    /// Electrical length in radians of `length_m` metres at `mhz`.
    fn beta_l(&self, length_m: f64, mhz: f64) -> f64 {
        2.0 * PI * length_m * mhz / (CVEL_MHZ_M * self.vf)
    }

    /// Physical length in metres of `theta` radians at `mhz`.
    fn length(&self, theta: f64, mhz: f64) -> f64 {
        theta * CVEL_MHZ_M * self.vf / (2.0 * PI * mhz)
    }
}

/// One stage of a network, in order from the source.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "stage", rename_all = "snake_case")]
pub enum Stage {
    /// A component in series with the line.
    Series {
        /// The component.
        part: Component,
    },
    /// A component across the line.
    Shunt {
        /// The component.
        part: Component,
    },
    /// A length of line.
    Line {
        /// The line type.
        line: Line,
        /// Physical length, metres.
        length_m: f64,
    },
    /// A stub: open or shorted at its far end, across the line (parallel) or in series with one
    /// conductor.
    Stub {
        /// The stub's line type.
        line: Line,
        /// Physical length, metres.
        length_m: f64,
        /// Shorted (rather than open) at its far end.
        short: bool,
        /// In series with one conductor (rather than across the line).
        series: bool,
    },
}

type Abcd = [[C; 2]; 2];

fn mul(a: &Abcd, b: &Abcd) -> Abcd {
    [
        [a[0][0] * b[0][0] + a[0][1] * b[1][0], a[0][0] * b[0][1] + a[0][1] * b[1][1]],
        [a[1][0] * b[0][0] + a[1][1] * b[1][0], a[1][0] * b[0][1] + a[1][1] * b[1][1]],
    ]
}

impl Stage {
    fn abcd(&self, mhz: f64) -> Abcd {
        let w = 2.0 * PI * mhz * 1e6;
        match *self {
            Self::Series { part } => [[ONE, part.impedance(w)], [ZERO, ONE]],
            Self::Shunt { part } => [[ONE, ZERO], [ONE / part.impedance(w), ONE]],
            Self::Line { line, length_m } => {
                let bl = line.beta_l(length_m, mhz);
                let (s, c) = bl.sin_cos();
                [[C::new(c, 0.0), J * line.z0 * s], [J * s / line.z0, C::new(c, 0.0)]]
            }
            Self::Stub { line, length_m, short, series } => {
                let t = line.beta_l(length_m, mhz).tan();
                // Input impedance of the stub: jZ0·tan βl shorted, −jZ0·cot βl open.
                let z = if short { J * line.z0 * t } else { -J * line.z0 / t };
                if series {
                    [[ONE, z], [ZERO, ONE]]
                } else {
                    [[ONE, ZERO], [ONE / z, ONE]]
                }
            }
        }
    }
}

/// A matching network: stages from the source (NEC port 1) to the antenna (port 2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Network {
    /// What it is, for people: "L: series L, shunt C (low-pass)".
    pub label: String,
    /// The stages, source first.
    pub stages: Vec<Stage>,
}

impl Network {
    /// The chain matrix at `mhz`.
    #[must_use]
    pub fn abcd(&self, mhz: f64) -> [[C; 2]; 2] {
        self.stages
            .iter()
            .fold([[ONE, ZERO], [ZERO, ONE]], |m, s| mul(&m, &s.abcd(mhz)))
    }

    /// The short-circuit admittances `(Y11, Y12, Y22)` at `mhz` for an `NT` card, port 1 at the
    /// source; `None` when the network has no series path (`B = 0`), which a matching network
    /// never has.
    #[must_use]
    pub fn y_params(&self, mhz: f64) -> Option<(C, C, C)> {
        let [[a, b], [c, d]] = self.abcd(mhz);
        if b.norm() < 1e-300 {
            return None;
        }
        let det = a * d - b * c;
        Some((d / b, -det / b, a / b))
    }

    /// The impedance the source sees with `z_load` on the far side, at `mhz`.
    #[must_use]
    pub fn input_impedance(&self, z_load: C, mhz: f64) -> C {
        let [[a, b], [c, d]] = self.abcd(mhz);
        (a * z_load + b) / (c * z_load + d)
    }

    /// The fraction of the input power lost in the network, with `z_load` on the far side.
    #[must_use]
    pub fn loss_fraction(&self, z_load: C, mhz: f64) -> f64 {
        let [[a, b], [c, d]] = self.abcd(mhz);
        // Unit load current: V2 = Z_L, I2 = 1; then V1 = A·V2 + B·I2, I1 = C·V2 + D·I2.
        let (v2, i2) = (z_load, ONE);
        let (v1, i1) = (a * v2 + b * i2, c * v2 + d * i2);
        let p_in = 0.5 * (v1 * i1.conj()).re;
        let p_out = 0.5 * (v2 * i2.conj()).re;
        if p_in > 0.0 { 1.0 - p_out / p_in } else { 0.0 }
    }
}

/// Component quality for a design: `0` means lossless.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Quality {
    /// Inductor Q.
    pub q_l: f64,
    /// Capacitor Q.
    pub q_c: f64,
}

impl Default for Quality {
    /// 4nec2's defaults: coils Q 250, capacitors lossless.
    fn default() -> Self {
        Self { q_l: 250.0, q_c: 0.0 }
    }
}

const TINY: f64 = 1e-9;

fn series(x: f64, w: f64, q: Quality) -> Option<Stage> {
    (x.abs() > TINY).then(|| Stage::Series { part: Component::for_reactance(x, w, q.q_l, q.q_c) })
}

fn shunt(b: f64, w: f64, q: Quality) -> Option<Stage> {
    (b.abs() > TINY * 1e-6).then(|| Stage::Shunt { part: Component::for_susceptance(b, w, q.q_l, q.q_c) })
}

fn describe(stages: &[Stage], family: &str) -> String {
    let names: Vec<String> = stages
        .iter()
        .map(|s| match s {
            Stage::Series { part: Component::Inductor { .. } } => "series L".into(),
            Stage::Series { part: Component::Capacitor { .. } } => "series C".into(),
            Stage::Shunt { part: Component::Inductor { .. } } => "shunt L".into(),
            Stage::Shunt { part: Component::Capacitor { .. } } => "shunt C".into(),
            Stage::Line { .. } => "line".into(),
            Stage::Stub { short, series, .. } => {
                format!("{} {} stub", if *series { "series" } else { "parallel" }, if *short { "shorted" } else { "open" })
            }
        })
        .collect();
    let lumped: Vec<&Stage> = stages.iter().filter(|s| matches!(s, Stage::Series { .. } | Stage::Shunt { .. })).collect();
    let low = lumped.iter().all(|s| {
        matches!(s, Stage::Series { part: Component::Inductor { .. } } | Stage::Shunt { part: Component::Capacitor { .. } })
    });
    let high = lumped.iter().all(|s| {
        matches!(s, Stage::Series { part: Component::Capacitor { .. } } | Stage::Shunt { part: Component::Inductor { .. } })
    });
    let pass = if lumped.is_empty() {
        ""
    } else if low {
        " (low-pass)"
    } else if high {
        " (high-pass)"
    } else {
        ""
    };
    format!("{family}: {}{pass}", names.join(", "))
}

/// The two halves of an L network with the shunt element across the load: `(series X, shunt B)`
/// pairs (one per sign) turning `z_load` into `r_target`, if the load's conductance allows it.
fn shunt_at_load(z_load: C, r_target: f64) -> Vec<(f64, f64)> {
    let y = ONE / z_load;
    let (g, b_l) = (y.re, y.im);
    let d = g / r_target - g * g;
    if g <= 0.0 || d < -1e-15 {
        return Vec::new();
    }
    let root = d.max(0.0).sqrt();
    let mut out = Vec::new();
    for b_total in [root, -root] {
        let b = b_total - b_l;
        // Impedance after the shunt: 1 / (G + jB_total) = R_target + jX'.
        let x_after = (ONE / C::new(g, b_total)).im;
        out.push((-x_after, b));
        if root == 0.0 {
            break;
        }
    }
    out
}

/// The two halves of an L network with the series element at the load: `(shunt B, series X)`
/// pairs turning `z_load` into `r_target`, if the load's resistance allows it.
fn series_at_load(z_load: C, r_target: f64) -> Vec<(f64, f64)> {
    let (r_l, x_l) = (z_load.re, z_load.im);
    let d = r_l * r_target - r_l * r_l;
    if r_l <= 0.0 || d < -1e-12 {
        return Vec::new();
    }
    let root = d.max(0.0).sqrt();
    let mut out = Vec::new();
    for x_total in [root, -root] {
        let x = x_total - x_l;
        let b_after = (ONE / C::new(r_l, x_total)).im;
        out.push((-b_after, x));
        if root == 0.0 {
            break;
        }
    }
    out
}

/// Every L network matching `z_load` to a real `z0` at `mhz` (up to four).
#[must_use]
pub fn design_l(z_load: C, z0: f64, mhz: f64, q: Quality) -> Vec<Network> {
    let w = 2.0 * PI * mhz * 1e6;
    let mut out = Vec::new();
    for (x, b) in shunt_at_load(z_load, z0) {
        let stages: Vec<Stage> = [series(x, w, q), shunt(b, w, q)].into_iter().flatten().collect();
        out.push(Network { label: describe(&stages, "L"), stages });
    }
    for (b, x) in series_at_load(z_load, z0) {
        let stages: Vec<Stage> = [shunt(b, w, q), series(x, w, q)].into_iter().flatten().collect();
        out.push(Network { label: describe(&stages, "L"), stages });
    }
    out
}

/// The lowest loaded Q a Pi network can have for this match.
#[must_use]
pub fn pi_min_q(z_load: C, z0: f64) -> f64 {
    let rp = z_load.norm_sqr() / z_load.re;
    let (hi, lo) = (z0.max(rp), z0.min(rp));
    (hi / lo - 1.0).max(0.0).sqrt()
}

/// The lowest loaded Q a T network can have for this match.
#[must_use]
pub fn tee_min_q(z_load: C, z0: f64) -> f64 {
    let (hi, lo) = (z0.max(z_load.re), z0.min(z_load.re));
    (hi / lo - 1.0).max(0.0).sqrt()
}

/// Every Pi network (shunt, series, shunt) of loaded Q `q_net` (raised to the minimum when lower)
/// matching `z_load` to `z0`: two L sections through a virtual resistance below both ends.
#[must_use]
pub fn design_pi(z_load: C, z0: f64, mhz: f64, q_net: f64, q: Quality) -> Vec<Network> {
    let w = 2.0 * PI * mhz * 1e6;
    let rp = z_load.norm_sqr() / z_load.re;
    let qn = q_net.max(pi_min_q(z_load, z0));
    let r_v = (z0.max(rp) / (1.0 + qn * qn)).min(z0).min(rp);
    let mut out = Vec::new();
    for (x1, b1) in shunt_at_load(z_load, r_v) {
        for (b2, x2) in series_at_load(C::new(r_v, 0.0), z0) {
            let stages: Vec<Stage> = [shunt(b2, w, q), series(x1 + x2, w, q), shunt(b1, w, q)]
                .into_iter()
                .flatten()
                .collect();
            out.push(Network { label: describe(&stages, "Pi"), stages });
        }
    }
    out
}

/// Every T network (series, shunt, series) of loaded Q `q_net` (raised to the minimum when lower)
/// matching `z_load` to `z0`: two L sections through a virtual resistance above both ends.
#[must_use]
pub fn design_tee(z_load: C, z0: f64, mhz: f64, q_net: f64, q: Quality) -> Vec<Network> {
    let w = 2.0 * PI * mhz * 1e6;
    let qn = q_net.max(tee_min_q(z_load, z0));
    let r_v = (z0.min(z_load.re) * (1.0 + qn * qn)).max(z0).max(z_load.re);
    let mut out = Vec::new();
    for (b1, x1) in series_at_load(z_load, r_v) {
        for (x2, b2) in shunt_at_load(C::new(r_v, 0.0), z0) {
            let stages: Vec<Stage> = [series(x2, w, q), shunt(b2 + b1, w, q), series(x1, w, q)]
                .into_iter()
                .flatten()
                .collect();
            out.push(Network { label: describe(&stages, "T"), stages });
        }
    }
    out
}

/// `z` seen through `theta` radians of line of impedance `z0`.
fn through(z: C, z0: f64, theta: f64) -> C {
    let t = theta.tan();
    z0 * (z + J * z0 * t) / (z0 + J * z * t)
}

/// Every root of `f` on `[0, π)`, found by scanning and bisection.
fn roots(f: impl Fn(f64) -> f64) -> Vec<f64> {
    const N: usize = 3600;
    let mut out = Vec::new();
    #[allow(clippy::cast_precision_loss)]
    let at = |k: usize| PI * k as f64 / N as f64;
    let mut prev = (at(0), f(at(0)));
    for k in 1..=N {
        let x = at(k);
        let y = f(x);
        if prev.1.is_finite() && y.is_finite() && prev.1.signum() != y.signum() {
            let (mut lo, mut hi, mut flo) = (prev.0, x, prev.1);
            for _ in 0..200 {
                let mid = 0.5 * (lo + hi);
                let fm = f(mid);
                if fm.signum() == flo.signum() {
                    lo = mid;
                    flo = fm;
                } else {
                    hi = mid;
                }
            }
            // A sign change across a pole is not a root: keep only true zeros.
            let root = 0.5 * (lo + hi);
            let scale = prev.1.abs().max(y.abs()).max(1e-12);
            if f(root).abs() <= 1e-6 * scale {
                out.push(root);
            }
        }
        prev = (x, y);
    }
    out
}

/// A stub length (radians, in `[0, π)`) whose stub presents `want` — a susceptance for a parallel
/// stub, a reactance for a series one — normalised by the stub line's impedance.
fn stub_theta(want: f64, z0s: f64, short: bool, series: bool) -> f64 {
    let a = match (series, short) {
        // Parallel open: Y = j·tan βl / Z0s.  Parallel short: Y = −j·cot βl / Z0s.
        (false, false) => (want * z0s).atan(),
        (false, true) => (-1.0 / (want * z0s)).atan(),
        // Series open: Z = −j·Z0s·cot βl.  Series short: Z = j·Z0s·tan βl.
        (true, false) => (-z0s / want).atan(),
        (true, true) => (want / z0s).atan(),
    };
    a.rem_euclid(PI)
}

/// Single-stub matches of `z_load` to `z0` along `line` (the stub at the end nearer the source,
/// cut from `stub_line`): every solution, two per half wave of line.
#[must_use]
pub fn design_stub(z_load: C, z0: f64, mhz: f64, line: Line, stub_line: Line, short: bool, series: bool) -> Vec<Network> {
    let mut out = Vec::new();
    let residual = |th: f64| {
        let z = through(z_load, line.z0, th);
        if series { z.re - z0 } else { (ONE / z).re - 1.0 / z0 }
    };
    for th in roots(residual) {
        let z = through(z_load, line.z0, th);
        let want = if series { -z.im } else { -(ONE / z).im };
        let ts = stub_theta(want, stub_line.z0, short, series);
        let stages = vec![
            Stage::Stub { line: stub_line, length_m: stub_line.length(ts, mhz), short, series },
            Stage::Line { line, length_m: line.length(th, mhz) },
        ];
        out.push(Network { label: describe(&stages, "Stub"), stages });
    }
    out
}

/// Series-section matches: a length of `section` line set into `line` some distance from the
/// load, both chosen so the source sees `z0`.
#[must_use]
pub fn design_series_section(z_load: C, z0: f64, mhz: f64, line: Line, section: Line) -> Vec<Network> {
    let gamma = |z: C| (z - section.z0) / (z + section.z0);
    let target = gamma(C::new(z0, 0.0));
    let mut out = Vec::new();
    for th1 in roots(|th| gamma(through(z_load, line.z0, th)).norm() - target.norm()) {
        let g = gamma(through(z_load, line.z0, th1));
        // The section rotates Γ by e^{−2jθ2}; turn it onto the target's angle.
        let th2 = (0.5 * (g.arg() - target.arg())).rem_euclid(PI);
        let stages = vec![
            Stage::Line { line: section, length_m: section.length(th2, mhz) },
            Stage::Line { line, length_m: line.length(th1, mhz) },
        ];
        out.push(Network { label: "Series section".into(), stages });
    }
    out
}

/// Errors applying a network to a deck.
#[derive(Debug, thiserror::Error)]
pub enum MatchError {
    /// The deck does not parse.
    #[error("{0}")]
    Deck(String),
    /// No voltage source drives the feed.
    #[error("no EX 0 voltage source on tag {tag} segment {segment}")]
    NoSource {
        /// Wire tag.
        tag: i32,
        /// Segment within the tag.
        segment: usize,
    },
    /// Already matched there.
    #[error("that feed already has a matching network; remove it first")]
    Matched,
    /// No network on a dummy wire with that tag.
    #[error("no matching network on tag {0}")]
    NotFound(i32),
    /// No admittances (a network without a series path).
    #[error("the network has no series path")]
    Degenerate,
}

/// An admittance for the `NT` card, at six significant digits in the shorter of fixed or
/// exponent form, so that six of them fit the engine's 80 columns. The reference engines read
/// the card; nec-rs recomputes the network from its description at full precision.
fn admittance(v: f64) -> String {
    if v == 0.0 {
        return "0".into();
    }
    let r: f64 = format!("{v:.5e}").parse().unwrap_or(v);
    let (fixed, exp) = (format!("{r}"), format!("{r:e}"));
    if exp.len() < fixed.len() { exp } else { fixed }
}

fn value(f: Option<f64>) -> i64 {
    #[allow(clippy::cast_possible_truncation)]
    f.map_or(0, |v| v.round() as i64)
}

/// Write `network` into `text` at the source on segment `segment` (within the tag) of wire
/// `tag`, with the `NT` admittances at `mhz`. Returns the new deck text.
///
/// # Errors
/// The deck does not parse, no `EX 0` drives that segment, or it is already matched.
pub fn apply(text: &str, tag: i32, segment: usize, network: &Network, mhz: f64) -> Result<String, MatchError> {
    let all = cards(text, &[]).map_err(|e| MatchError::Deck(e.to_string()))?;
    let ex = all
        .iter()
        .find(|c| {
            c.mnemonic == "EX"
                && value(c.values.first().copied().flatten()) == 0
                && value(c.values.get(1).copied().flatten()) == i64::from(tag)
                && value(c.values.get(2).copied().flatten()) == i64::try_from(segment).unwrap_or(-1)
        })
        .ok_or(MatchError::NoSource { tag, segment })?;
    if all.iter().any(|c| c.mnemonic == "NT" && c.comment.as_deref().is_some_and(|m| m.trim_start().starts_with(MARKER))
        && value(c.values.get(2).copied().flatten()) == i64::from(tag)
        && value(c.values.get(3).copied().flatten()) == i64::try_from(segment).unwrap_or(-1))
    {
        return Err(MatchError::Matched);
    }
    let used: Vec<i64> = all.iter().filter(|c| c.mnemonic == "GW").map(|c| value(c.values.first().copied().flatten())).collect();
    let dummy = (1..)
        .map(|n| i64::from(MATCH_TAG_BASE) + n)
        .find(|t| !used.contains(t))
        .unwrap_or(i64::from(MATCH_TAG_BASE) + 1);
    let (y11, y12, y22) = network.y_params(mhz).ok_or(MatchError::Degenerate)?;
    let json = serde_json::to_string(network).map_err(|e| MatchError::Deck(e.to_string()))?;
    let f = format_value;

    // The NT card, after the source; its port 2 fields are the source's own, verbatim, so
    // removing the network restores them exactly.
    let mut ex_fields = ex.fields.clone();
    let nt = vec![
        dummy.to_string(),
        "1".into(),
        ex.fields[1].clone(),
        ex.fields[2].clone(),
        admittance(y11.re),
        admittance(y11.im),
        admittance(y12.re),
        admittance(y12.im),
        admittance(y22.re),
        admittance(y22.im),
    ];
    let mut out = insert_card(text, ex.line + 1, "NT", &nt);
    // Give the NT its comment: rewrite the line just inserted.
    let lines: Vec<&str> = out.lines().collect();
    let nt_text = format!("{}  ' {MARKER} {json}", lines[ex.line]);
    out = replace_line(&out, ex.line + 1, &nt_text);

    ex_fields[1] = dummy.to_string();
    ex_fields[2] = "1".into();
    out = replace_card(&out, ex.line, "EX", &ex_fields).ok_or_else(|| MatchError::Deck("source line".into()))?;

    // The dummy wire, 9.8 km up, as short as an EX 6 dummy wire (0.00051 λ, radius 1/20 of it).
    let c = 0.00051 * CVEL_MHZ_M / mhz;
    #[allow(clippy::cast_precision_loss)]
    let z = dummy as f64;
    let gw = vec![
        dummy.to_string(),
        "1".into(),
        f(-c),
        "0".into(),
        f(z),
        f(c),
        "0".into(),
        f(z),
        f(c / 20.0),
    ];
    let at = geometry_end(&out).unwrap_or(1);
    Ok(insert_card(&out, at, "GW", &gw))
}

/// Replace line `line` (from 1) of `text` verbatim, keeping its line endings.
fn replace_line(text: &str, line: usize, new: &str) -> String {
    let eol = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    if let Some(l) = lines.get_mut(line - 1) {
        *l = new.to_string();
    }
    let mut s = lines.join(eol);
    s.push_str(eol);
    s
}

/// The matching networks written into `text`: `(dummy tag, feed tag field, feed segment field,
/// network)`.
#[must_use]
pub fn find(text: &str) -> Vec<(i32, String, String, Network)> {
    let Ok(all) = cards(text, &["NT"]) else { return Vec::new() };
    all.iter()
        .filter_map(|c| {
            let json = c.comment.as_deref()?.trim_start().strip_prefix(MARKER)?;
            let net: Network = serde_json::from_str(json.trim()).ok()?;
            #[allow(clippy::cast_possible_truncation)]
            let tag = c.values.first().copied().flatten()? as i32;
            Some((tag, c.fields.get(2)?.clone(), c.fields.get(3)?.clone(), net))
        })
        .collect()
}

/// Remove the matching network on dummy wire `dummy_tag`: the source returns to the antenna's
/// feed segment and the `NT` card and the dummy wire go.
///
/// # Errors
/// No such network, or the deck does not parse.
pub fn remove(text: &str, dummy_tag: i32) -> Result<String, MatchError> {
    let all = cards(text, &[]).map_err(|e| MatchError::Deck(e.to_string()))?;
    let is_tag = |c: &nec_cards::edit::CardLine, i: usize| value(c.values.get(i).copied().flatten()) == i64::from(dummy_tag);
    let nt = all
        .iter()
        .find(|c| c.mnemonic == "NT" && is_tag(c, 0) && c.comment.as_deref().is_some_and(|m| m.trim_start().starts_with(MARKER)))
        .ok_or(MatchError::NotFound(dummy_tag))?;
    let gw = all.iter().find(|c| c.mnemonic == "GW" && is_tag(c, 0)).ok_or(MatchError::NotFound(dummy_tag))?;
    let mut out = text.to_string();
    for ex in all.iter().filter(|c| c.mnemonic == "EX" && is_tag(c, 1)) {
        let mut fields = ex.fields.clone();
        fields[1] = nt.fields[2].clone();
        fields[2] = nt.fields[3].clone();
        out = replace_card(&out, ex.line, "EX", &fields).ok_or(MatchError::NotFound(dummy_tag))?;
    }
    // Delete the later line first so the earlier keeps its number.
    let (first, second) = if nt.line > gw.line { (nt.line, gw.line) } else { (gw.line, nt.line) };
    out = delete_line(&out, first).ok_or(MatchError::NotFound(dummy_tag))?;
    delete_line(&out, second).ok_or(MatchError::NotFound(dummy_tag))
}

#[cfg(test)]
mod tests {
    use super::*;

    const MHZ: f64 = 14.15;
    const LOSSLESS: Quality = Quality { q_l: 0.0, q_c: 0.0 };
    const RG58: Line = Line { z0: 50.0, vf: 0.66 };
    // Twin lead: a series section can match an SWR up to (Z1/Z0)², here 36.
    const TWIN: Line = Line { z0: 300.0, vf: 0.8 };

    fn loads() -> Vec<C> {
        vec![C::new(12.0, -35.0), C::new(200.0, 80.0), C::new(25.0, 0.0), C::new(72.0, 42.0), C::new(500.0, -300.0)]
    }

    fn matched(nets: &[Network], z: C) {
        assert!(!nets.is_empty(), "no solution for {z}");
        for n in nets {
            let zin = n.input_impedance(z, MHZ);
            assert!((zin - C::new(50.0, 0.0)).norm() < 1e-6, "{}: {zin} for {z}", n.label);
        }
    }

    #[test]
    fn every_lumped_design_matches_when_lossless() {
        for z in loads() {
            matched(&design_l(z, 50.0, MHZ, LOSSLESS), z);
            matched(&design_pi(z, 50.0, MHZ, 0.0, LOSSLESS), z);
            matched(&design_pi(z, 50.0, MHZ, 5.0, LOSSLESS), z);
            matched(&design_tee(z, 50.0, MHZ, 0.0, LOSSLESS), z);
            matched(&design_tee(z, 50.0, MHZ, 5.0, LOSSLESS), z);
        }
    }

    #[test]
    fn every_line_design_matches() {
        for z in loads() {
            for short in [false, true] {
                for series in [false, true] {
                    matched(&design_stub(z, 50.0, MHZ, RG58, RG58, short, series), z);
                }
            }
            matched(&design_series_section(z, 50.0, MHZ, RG58, TWIN), z);
        }
    }

    #[test]
    fn a_textbook_l_network() {
        // 12.5 Ω to 50 Ω at 14.15 MHz: |X| = √(12.5·37.5) = 21.65 Ω in series, |B| = 0.01732 S.
        let nets = design_l(C::new(12.5, 0.0), 50.0, MHZ, LOSSLESS);
        let low = nets.iter().find(|n| n.label.contains("low-pass")).expect("a low-pass L");
        let w = 2.0 * PI * MHZ * 1e6;
        let (mut l, mut c) = (0.0, 0.0);
        for s in &low.stages {
            match s {
                Stage::Series { part: Component::Inductor { henry, .. } } => l = *henry,
                Stage::Shunt { part: Component::Capacitor { farad, .. } } => c = *farad,
                _ => {}
            }
        }
        assert!((w * l - (12.5_f64 * 37.5).sqrt()).abs() < 1e-9);
        assert!((w * c - (37.5_f64 / 12.5).sqrt() / 50.0).abs() < 1e-12);
    }

    #[test]
    fn y_params_of_a_line_are_the_tl_cards() {
        let n = Network { label: String::new(), stages: vec![Stage::Line { line: RG58, length_m: 3.0 }] };
        let (y11, y12, y22) = n.y_params(MHZ).expect("y");
        let bl = RG58.beta_l(3.0, MHZ);
        assert!((y11 - C::new(0.0, -1.0 / (50.0 * bl.tan()))).norm() < 1e-12);
        assert!((y12 - C::new(0.0, 1.0 / (50.0 * bl.sin()))).norm() < 1e-12);
        assert!((y22 - y11).norm() < 1e-12);
    }

    #[test]
    fn losses_cost_power_and_detune_slightly() {
        let z = C::new(12.0, -35.0);
        let lossy = design_l(z, 50.0, MHZ, Quality::default());
        for n in &lossy {
            let frac = n.loss_fraction(z, MHZ);
            assert!(frac > 0.0 && frac < 0.2, "{}: {frac}", n.label);
            let swr = {
                let g = ((n.input_impedance(z, MHZ) - 50.0) / (n.input_impedance(z, MHZ) + 50.0)).norm();
                (1.0 + g) / (1.0 - g)
            };
            assert!(swr < 1.3, "{}: SWR {swr}", n.label);
        }
    }

    const DECK: &str = "CE\nGW 1 11 0 -5.3 10 0 5.3 10 .001\nGE 0\nEX 0 1 6 0 1 0  ' feed\nFR 0 1 0 0 14.15 0\nEN\n";

    #[test]
    fn applying_and_removing_restores_the_deck() {
        let net = design_l(C::new(60.0, 30.0), 50.0, MHZ, Quality::default()).remove(0);
        let out = apply(DECK, 1, 6, &net, MHZ).expect("applies");
        // The engine input, comments stripped, fits 80 columns.
        let mut deck = nec_cards::parse_deck(&out).expect("parses");
        deck.resolve().expect("resolves");
        let input = nec_cards::to_engine_input(&deck).expect("emits");
        assert!(input.lines().all(|l| l.len() <= 80), "{input}");
        assert!(out.contains("GW 9801 1"), "{out}");
        assert!(out.contains("EX 0 9801 1 0 1 0  ' feed"), "{out}");
        let found = find(&out);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0, 9801);
        assert_eq!((found[0].1.as_str(), found[0].2.as_str()), ("1", "6"));
        assert_eq!(found[0].3, net);
        assert!(matches!(apply(&out, 1, 6, &net, MHZ), Err(MatchError::NoSource { .. }) | Err(MatchError::Matched)));
        assert_eq!(remove(&out, 9801).expect("removes"), DECK);
    }

    /// Solve `text` at `mhz` and return the first source's impedance.
    fn z_in(text: &str, mhz: f64) -> C {
        let m = crate::Model::new(text).expect("builds");
        let f = nec_core::Frequency::from_mhz(mhz).expect("f");
        let prog = m.program().expect("program");
        m.solve(f, prog.options(f)).expect("solves").inputs[0].impedance
    }

    #[test]
    fn a_matched_dipole_presents_z0_and_tracks_frequency() {
        let z_ant = z_in(DECK, MHZ);
        for net in design_l(z_ant, 50.0, MHZ, LOSSLESS).into_iter().chain(design_stub(z_ant, 50.0, MHZ, RG58, RG58, true, false)) {
            let matched = apply(DECK, 1, 6, &net, MHZ).expect("applies");
            // At the design frequency the source sees Z0 (the dummy wire adds a whisker).
            let z = z_in(&matched, MHZ);
            assert!((z - C::new(50.0, 0.0)).norm() < 0.05, "{}: {z}", net.label);
            // Off it, the network is recomputed there, so the source sees the network
            // transforming the antenna's own impedance at that frequency.
            let f2 = 14.35;
            let want = net.input_impedance(z_in(DECK, f2), f2);
            let got = z_in(&matched, f2);
            assert!((got - want).norm() < 0.05 * want.norm(), "{}: {got} vs {want}", net.label);
        }
    }
}
