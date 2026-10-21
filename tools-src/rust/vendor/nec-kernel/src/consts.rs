//! NEC-2's own numerical constants.
//!
//! These are deliberately **not** the correctly rounded mathematical values. NEC-2 hard-codes
//! truncated literals, and reproducing its numbers means using the same ones. Each is quoted from
//! the reference source (`reference/nec2dxs/nec2dxs.f`, public-domain NEC-2D) with its location.
//!
//! The differences from exact values are tiny — around 1e-10 relative for π, 4e-8 for η — and far
//! below the six significant figures the engine prints. They are kept anyway because matching the
//! oracle is cheaper to reason about than explaining a residual.

/// π as NEC writes it: `DATA PI/3.141592654D+0/` (`EFLD`, `TBF`, `SBF`, and others).
///
/// Clippy's `approx_constant` rightly flags this as an imprecise π. That is the point: it is
/// NEC's literal, kept so results reproduce the reference engine rather than drift from it.
#[allow(clippy::approx_constant)]
pub const PI: f64 = 3.141_592_654;

/// 2π as NEC writes it: `DATA TP/6.283185308D+0/`. Also the wavenumber, since NEC works in
/// wavelengths: `k = 2π / λ = 2π` when lengths are measured in λ.
pub const TP: f64 = 6.283_185_308;

/// Free-space impedance as `EFLD` declares it: `DATA ETA/376.73/`.
///
/// Note the literal has no `D` exponent, so Fortran stores it as a *single*-precision constant
/// before widening it to the `REAL*8` variable. The value NEC actually computes with is
/// therefore `376.73f32` widened, `376.7300109863281`, not `376.73`.
#[allow(clippy::cast_lossless)]
pub const ETA: f64 = 376.73_f32 as f64;

/// The thin-wire kernel's leading constant, `η / (8π²)`, applied as `j × CON`:
/// `DATA CONX/0.,4.771341189D+0/` in `EKSC`.
pub const CON: f64 = 4.771_341_189;

/// Euler's constant as `TBF`/`SBF` write it: `.577215664D+0`.
pub const EULER_GAMMA: f64 = 0.577_215_664;

/// Default distance, in wavelengths, beyond which a segment interaction uses the lumped
/// current-element approximation rather than the full kernel: `RKH=1.` in the main program,
/// changeable with a `KH` card. The engine reports it as "APPROXIMATE INTEGRATION EMPLOYED FOR
/// SEGMENTS MORE THAN 1.000 WAVELENGTHS APART".
pub const DEFAULT_RKH: f64 = 1.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eta_is_the_widened_single_precision_literal() {
        assert!((ETA - 376.730_010_986_328_1).abs() < 1e-12, "ETA = {ETA}");
        // And it is not the decimal literal itself.
        assert!((ETA - 376.73).abs() > 1e-6);
    }

    #[test]
    fn con_is_eta_over_eight_pi_squared() {
        // NEC's literal agrees with eta/(8 pi^2) to the precision it was written with.
        let derived = 376.73 / (8.0 * core::f64::consts::PI * core::f64::consts::PI);
        assert!((CON - derived).abs() < 1e-6, "CON {CON} vs {derived}");
    }

    #[test]
    fn truncated_pi_is_close_to_but_not_exactly_pi() {
        let d = (PI - core::f64::consts::PI).abs();
        assert!(d > 0.0 && d < 1e-9);
        assert!((TP - 2.0 * PI).abs() < 1e-9);
    }
}
