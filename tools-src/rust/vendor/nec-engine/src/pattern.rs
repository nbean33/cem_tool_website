//! Reading a radiation pattern as 4nec2's pattern window does: circularly polarized gains, and
//! for each plane cut its beamwidth, lobes, front-to-back and front-to-rear.
//!
//! * **Circular components.** A far field `E_θ θ̂ + E_φ φ̂` splits into right- and left-hand
//!   circular parts. With NEC's `e^{jωt}` and `θ̂ × φ̂ = r̂`, the right-hand (IEEE) unit vector is
//!   `(θ̂ − jφ̂)/√2`, so `E_R = (E_θ + jE_φ)/√2` and `E_L = (E_θ − jE_φ)/√2`, and each gets the
//!   share `|E_R|² / (|E_θ|² + |E_φ|²)` of the total gain. Where NEC calls a field's sense
//!   `RIGHT`, the right-hand gain is the larger (tested below on a helix).
//! * **Back directions.** Over ground the back of a direction is the same elevation at the
//!   opposite azimuth. In free space 4nec2 mirrors the elevation too — the back of (θ, φ) is
//!   (180° − θ, φ + 180°), the exactly opposite direction — and so does nec-rs.
//! * **Front-to-rear** is, in 4nec2's help, "the difference between the forward-gain and the
//!   maximum gain-value for the backward half of the radiation pattern which is centered around
//!   the backward-direction". In a cut that is the half of the cut within 90° of the back angle.
//!   Over the whole pattern it is, in free space, the hemisphere around the back direction and,
//!   over ground, the half of the sky within 90° of azimuth of it.

use nec_geom::Geometry;
use nec_solve::{GroundSpec, PatternPoint, PatternRequest, Program, Solution};
use serde::Serialize;

/// NEC's gain for a direction with no field, dB.
pub const NO_FIELD_DB: f64 = -999.99;

/// The half-power drop, dB.
const HALF_POWER_DB: f64 = 3.0;

/// Right- and left-hand circularly polarized gains of a pattern point, dB.
#[must_use]
pub fn circular(p: &PatternPoint) -> (f64, f64) {
    let (et, ep) = (p.e_theta, p.e_phi);
    let total = et.norm_sqr() + ep.norm_sqr();
    if total <= 0.0 || p.total <= NO_FIELD_DB {
        return (NO_FIELD_DB, NO_FIELD_DB);
    }
    let j = num_complex::Complex64::i();
    let share = |e: num_complex::Complex64| {
        let s = e.norm_sqr() / 2.0 / total;
        if s < 1e-20 { NO_FIELD_DB } else { p.total + 10.0 * s.log10() }
    };
    (share(et + j * ep), share(et - j * ep))
}

/// The back of direction (θ, φ), degrees: the opposite azimuth at the same elevation over
/// ground, the opposite direction in free space.
#[must_use]
pub fn back_direction(theta: f64, phi: f64, ground: bool) -> (f64, f64) {
    let theta = if ground { theta } else { 180.0 - theta };
    (theta, (phi + 180.0).rem_euclid(360.0))
}

fn unit(theta: f64, phi: f64) -> [f64; 3] {
    let (t, p) = (theta.to_radians(), phi.to_radians());
    [t.sin() * p.cos(), t.sin() * p.sin(), t.cos()]
}

/// Front-to-rear over a set of directions: `forward` minus the strongest gain among `points` in
/// the rear half around the back of (θ, φ) (see the module notes). `None` when no point is there.
#[must_use]
pub fn front_to_rear(forward: f64, theta: f64, phi: f64, ground: bool, points: &[PatternPoint]) -> Option<f64> {
    let (bt, bp) = back_direction(theta, phi, ground);
    let b = unit(bt, bp);
    points
        .iter()
        .filter(|p| {
            if ground {
                (p.phi - bp).to_radians().cos() >= -1e-9
            } else {
                let u = unit(p.theta, p.phi);
                u[0] * b[0] + u[1] * b[1] + u[2] * b[2] >= -1e-9
            }
        })
        .map(|p| p.total)
        .fold(None, |m: Option<f64>, g| Some(m.map_or(g, |m| m.max(g))))
        .map(|rear| forward - rear)
}

/// How a cut's angles relate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CutShape {
    /// A full circle (an azimuth cut, or an elevation cut in free space); the back of angle `a`
    /// is `a + 180°`.
    Circle,
    /// A half circle from horizon to horizon over ground (elevation 0° to 180° through the
    /// zenith); the back of `a` is `180° − a`.
    OverGround,
}

/// One lobe of a cut: a local maximum.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Lobe {
    /// Angle, degrees.
    pub angle: f64,
    /// Gain, dB.
    pub gain: f64,
}

/// What 4nec2's pattern "info" reports for one cut.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CutStats {
    /// The strongest gain, dB, and its angle.
    pub peak: f64,
    /// See `peak`.
    pub peak_angle: f64,
    /// Angle between the half-power (−3 dB) points either side of the peak, degrees; `None`
    /// when the gain does not fall that far on one side (over ground, before the horizon).
    pub beamwidth: Option<f64>,
    /// The half-power angles below and above the peak, where found.
    pub half_power: [Option<f64>; 2],
    /// Every lobe, strongest first.
    pub lobes: Vec<Lobe>,
    /// The strongest lobe after the main one, relative to it, dB (negative).
    pub side_lobe: Option<f64>,
    /// The peak against the back of its angle, dB.
    pub front_to_back: Option<f64>,
    /// The peak against the strongest gain within 90° of the back angle, dB.
    pub front_to_rear: Option<f64>,
}

/// Wrap an angle difference into (−180°, 180°].
fn wrap180(d: f64) -> f64 {
    let r = (d + 180.0).rem_euclid(360.0) - 180.0;
    if r <= -180.0 { r + 360.0 } else { r }
}

/// Statistics of the cut `gains` at `angles` (degrees, increasing).
#[must_use]
pub fn cut_stats(angles: &[f64], gains: &[f64], shape: CutShape) -> Option<CutStats> {
    let circle = shape == CutShape::Circle;
    let mut n = angles.len().min(gains.len());
    // A circle's last angle may repeat its first.
    if circle && n > 1 && (angles[n - 1] - angles[0] - 360.0).abs() < 1e-6 {
        n -= 1;
    }
    let (a, g) = (&angles[..n], &gains[..n]);
    let (ip, &peak) = g.iter().enumerate().max_by(|x, y| x.1.total_cmp(y.1))?;
    if peak <= NO_FIELD_DB {
        return None;
    }
    let step = |i: usize, dir: isize| -> Option<usize> {
        let j = i.cast_signed() + dir;
        if circle {
            Some(j.rem_euclid(n.cast_signed()).cast_unsigned())
        } else if j < 0 || j >= n.cast_signed() {
            None
        } else {
            Some(j.cast_unsigned())
        }
    };
    let delta = |i: usize, j: usize| if circle { wrap180(a[j] - a[i]) } else { a[j] - a[i] };

    // Walk out from the peak to where the gain drops 3 dB, interpolating linearly in dB.
    let threshold = peak - HALF_POWER_DB;
    let half_power_side = |dir: isize| -> Option<f64> {
        let (mut i, mut offset) = (ip, 0.0);
        for _ in 0..n {
            let j = step(i, dir)?;
            if j == ip {
                return None;
            }
            let d = delta(i, j);
            if g[j] < threshold {
                let t = (g[i] - threshold) / (g[i] - g[j]);
                return Some(offset + t * d);
            }
            offset += d;
            i = j;
        }
        None
    };
    let (lo, hi) = (half_power_side(-1), half_power_side(1));
    let beamwidth = lo.zip(hi).map(|(l, h)| h - l);

    // Local maxima.
    let mut lobes: Vec<Lobe> = (0..n)
        .filter(|&i| g[i] > NO_FIELD_DB)
        .filter(|&i| {
            let before = step(i, -1).map_or(f64::NEG_INFINITY, |j| g[j]);
            let after = step(i, 1).map_or(f64::NEG_INFINITY, |j| g[j]);
            g[i] > before && g[i] >= after
        })
        .map(|i| Lobe { angle: a[i], gain: g[i] })
        .collect();
    lobes.sort_by(|x, y| y.gain.total_cmp(&x.gain));
    let side_lobe = lobes.get(1).map(|l| l.gain - peak);

    // Back and rear.
    let peak_angle = a[ip];
    let back = match shape {
        CutShape::Circle => peak_angle + 180.0,
        CutShape::OverGround => 180.0 - peak_angle,
    };
    let distance = |x: f64| if circle { wrap180(x - back).abs() } else { (x - back).abs() };
    // The gain at the back angle, interpolated between the samples either side of it.
    let front_to_back = (0..n)
        .filter_map(|i| Some((i, step(i, 1)?)))
        .find_map(|(i, j)| {
            let (di, dj) = (delta(i, j), if circle { wrap180(back - a[i]) } else { back - a[i] });
            (di.abs() > 1e-12 && (0.0..=di).contains(&dj)).then(|| g[i] + dj / di * (g[j] - g[i]))
        })
        .map(|gb| peak - gb);
    let rear = (0..n).filter(|&i| distance(a[i]) <= 90.0 + 1e-9).map(|i| g[i]).fold(None, |m: Option<f64>, x| {
        Some(m.map_or(x, |m| m.max(x)))
    });
    Some(CutStats {
        peak,
        peak_angle,
        beamwidth,
        half_power: [lo.map(|l| peak_angle + l), hi.map(|h| peak_angle + h)],
        lobes,
        side_lobe,
        front_to_back,
        front_to_rear: rear.map(|r| peak - r),
    })
}

/// The statistics of each component of a cut.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ComponentStats {
    /// Total gain.
    pub total: Option<CutStats>,
    /// Vertical (θ) polarization.
    pub vertical: Option<CutStats>,
    /// Horizontal (φ) polarization.
    pub horizontal: Option<CutStats>,
    /// Right-hand circular.
    pub rhcp: Option<CutStats>,
    /// Left-hand circular.
    pub lhcp: Option<CutStats>,
}

/// A plane cut through the pattern, with every gain component, dB.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Cut {
    /// Azimuth (an azimuth cut) or elevation (an elevation cut, running over the zenith to the
    /// back), degrees.
    pub angle: Vec<f64>,
    /// Total gain.
    pub gain: Vec<f64>,
    /// Vertical (θ) and horizontal (φ) components.
    pub vertical: Vec<f64>,
    /// See `vertical`.
    pub horizontal: Vec<f64>,
    /// Right- and left-hand circular components.
    pub rhcp: Vec<f64>,
    /// See `rhcp`.
    pub lhcp: Vec<f64>,
    /// The elevation an azimuth cut is taken at, or the azimuth of an elevation cut, degrees.
    pub at: f64,
    /// Beamwidth, lobes and ratios of each component.
    pub stats: ComponentStats,
}

impl Cut {
    fn new(points: &[(f64, &PatternPoint)], at: f64, shape: CutShape) -> Self {
        let circular: Vec<(f64, f64)> = points.iter().map(|(_, p)| circular(p)).collect();
        let angle: Vec<f64> = points.iter().map(|e| e.0).collect();
        let gain: Vec<f64> = points.iter().map(|e| e.1.total).collect();
        let vertical: Vec<f64> = points.iter().map(|e| e.1.vertical).collect();
        let horizontal: Vec<f64> = points.iter().map(|e| e.1.horizontal).collect();
        let rhcp: Vec<f64> = circular.iter().map(|c| c.0).collect();
        let lhcp: Vec<f64> = circular.iter().map(|c| c.1).collect();
        let stats = ComponentStats {
            total: cut_stats(&angle, &gain, shape),
            vertical: cut_stats(&angle, &vertical, shape),
            horizontal: cut_stats(&angle, &horizontal, shape),
            rhcp: cut_stats(&angle, &rhcp, shape),
            lhcp: cut_stats(&angle, &lhcp, shape),
        };
        Self { angle, gain, vertical, horizontal, rhcp, lhcp, at, stats }
    }
}

/// `RP 0 nθ nφ XNDA θ0 φ0 dθ dφ` as a request.
pub(crate) fn request(n_theta: i32, n_phi: i32, xnda: i32, theta0: f64, phi0: f64, dtheta: f64, dphi: f64) -> PatternRequest {
    PatternRequest::from_fields(0, n_theta, n_phi, xnda, [theta0, phi0, dtheta, dphi, 0.0, 0.0])
}

/// 1° cuts through direction (θ, φ): the azimuth cut at its elevation (at the horizon, or 30°
/// up over ground, when θ is within 2.5° of the zenith), and the elevation cut in the vertical
/// plane at azimuth φ, front and back.
#[must_use]
pub fn cuts(prog: &Program, geom: &Geometry, sol: &Solution, theta: f64, phi: f64) -> (Cut, Cut) {
    let ground = prog.ground != GroundSpec::FreeSpace;
    let cut_theta = if theta < 2.5 {
        if ground { 60.0 } else { 90.0 }
    } else {
        theta
    };
    let az = prog.pattern(geom, sol, &request(1, 361, 0, cut_theta, 0.0, 0.0, 1.0));
    let az_points: Vec<(f64, &PatternPoint)> = az.points.iter().map(|p| (p.phi, p)).collect();
    let azimuth = Cut::new(&az_points, 90.0 - cut_theta, CutShape::Circle);

    let n = if ground { 91 } else { 181 };
    let front = prog.pattern(geom, sol, &request(n, 1, 0, 0.0, phi, 1.0, 0.0));
    let back = prog.pattern(geom, sol, &request(n, 1, 0, 0.0, phi + 180.0, 1.0, 0.0));
    let mut el: Vec<(f64, &PatternPoint)> = front.points.iter().map(|p| (90.0 - p.theta, p)).collect();
    el.extend(back.points.iter().map(|p| (90.0 + p.theta, p)));
    el.sort_by(|a, b| a.0.total_cmp(&b.0));
    // The zenith appears from both sides over ground; the nadir too in free space.
    el.dedup_by(|a, b| (a.0 - b.0).abs() < 1e-9);
    let elevation = Cut::new(&el, phi.rem_euclid(360.0), if ground { CutShape::OverGround } else { CutShape::Circle });
    (azimuth, elevation)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cosine_lobe_has_the_textbook_beamwidth() {
        // 10·log10(cos²θ): half power at ±45°, a null at ±90°, a back lobe as strong as the front.
        let angles: Vec<f64> = (0..=360).map(f64::from).collect();
        let gains: Vec<f64> = angles
            .iter()
            .map(|a: &f64| {
                let c = a.to_radians().cos().powi(2);
                if c < 1e-20 { NO_FIELD_DB } else { 10.0 * c.log10() }
            })
            .collect();
        let s = cut_stats(&angles, &gains, CutShape::Circle).expect("stats");
        // Linear interpolation in dB between 1° samples is good to about 0.15°.
        assert!((s.beamwidth.expect("bw") - 90.0).abs() < 0.2, "{s:?}");
        assert_eq!(s.lobes.len(), 2);
        assert!(s.side_lobe.expect("back lobe").abs() < 1e-9);
        assert!(s.front_to_back.expect("f/b").abs() < 1e-9);
        assert!(s.front_to_rear.expect("f/r").abs() < 1e-9);
    }

    #[test]
    fn a_cardioid_has_its_front_to_back_and_front_to_rear() {
        // (1 + cos a)/2 squared, plus a −30 dB floor: F/B just over 30 dB; the rear half's best is at ±90°
        // from the back, where (1 + cos 90°)/2 = ½, so F/R ≈ 6 dB.
        let angles: Vec<f64> = (0..360).map(f64::from).collect();
        let gains: Vec<f64> = angles
            .iter()
            .map(|a: &f64| {
                let f = ((1.0 + a.to_radians().cos()) / 2.0).powi(2) + 1e-3;
                10.0 * f.log10()
            })
            .collect();
        let s = cut_stats(&angles, &gains, CutShape::Circle).expect("stats");
        assert!((s.front_to_back.expect("f/b") - 10.0 * (1.001_f64 / 1e-3).log10()).abs() < 1e-9, "{s:?}");
        let expect = 10.0 * (1.0 + 1e-3_f64).log10() - 10.0 * (0.25_f64 + 1e-3).log10();
        assert!((s.front_to_rear.expect("f/r") - expect).abs() < 1e-9, "{s:?}");
        assert_eq!(s.lobes.len(), 1);
        assert!(s.side_lobe.is_none());
        // Half power where ((1 + cos a)/2)² = ½: a = 65.53°.
        assert!((s.beamwidth.expect("bw") - 2.0 * 65.53).abs() < 0.2, "{s:?}");
    }

    #[test]
    fn over_ground_a_low_beam_may_have_no_lower_half_power_point() {
        // Gain rising toward the horizon on the front side: no −3 dB point below the peak.
        let angles: Vec<f64> = (0..=180).map(f64::from).collect();
        let gains: Vec<f64> = angles.iter().map(|a| -a / 10.0).collect();
        let s = cut_stats(&angles, &gains, CutShape::OverGround).expect("stats");
        assert_eq!(s.peak_angle, 0.0);
        assert!(s.half_power[0].is_none() && s.beamwidth.is_none());
        assert!((s.half_power[1].expect("upper") - 30.0).abs() < 1e-9);
        // The back of 0° elevation is 180°: 18 dB down.
        assert!((s.front_to_back.expect("f/b") - 18.0).abs() < 1e-9);
    }

    #[test]
    fn circular_components_follow_nec_s_sense() {
        // NEC-2 manual: a right-handed helix for positive length. Five turns, axial mode, fed
        // against a ground plane would do; in free space the axial beam is circular too.
        let deck = "CE\nGH 1 100 0.0725 0.362 0.0538 0.0538 0.0538 0.0538 0.001\nGW 2 1 0 0 -0.05 0.0538 0 0 0.001\nGE 0\nEX 0 2 1 0 1 0\nFR 0 1 0 0 1000 0\nEN\n";
        let m = crate::Model::new(deck).expect("builds");
        let prog = m.program().expect("program");
        let f = prog.frequencies[0];
        let sol = m.solve(f, prog.options(f)).expect("solves");
        let pat = prog.pattern(&m.geom, &sol, &request(37, 1, 0, 0.0, 0.0, 5.0, 0.0));
        let mut checked = 0;
        for p in &pat.points {
            let (r, l) = circular(p);
            match p.sense {
                nec_solve::Sense::Right => assert!(r > l, "{p:?}"),
                nec_solve::Sense::Left => assert!(l > r, "{p:?}"),
                _ => continue,
            }
            checked += 1;
            // The two shares make up the total.
            let sum = 10.0 * (10f64.powf(r / 10.0) + 10f64.powf(l / 10.0)).log10();
            assert!((sum - p.total).abs() < 1e-9);
        }
        assert!(checked > 20);
        // Along the axis of a right-handed helix the wave is right-hand circular.
        let axis = &pat.points[0];
        assert_eq!(axis.sense, nec_solve::Sense::Right, "{axis:?}");
    }

    #[test]
    fn back_directions_mirror_the_elevation_in_free_space_only() {
        assert_eq!(back_direction(60.0, 10.0, true), (60.0, 190.0));
        assert_eq!(back_direction(60.0, 10.0, false), (120.0, 190.0));
    }
}
