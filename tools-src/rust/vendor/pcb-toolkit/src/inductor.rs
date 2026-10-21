//! Planar spiral inductor calculator (Mohan/Wheeler modified).
//!
//! Reference: Mohan, Hershenson, Boyd, Lee — "Simple Accurate Expressions
//! for Planar Spiral Inductances", IEEE JSSC, October 1999.
//!
//! Supports square, hexagonal, octagonal, and circular geometries.

use serde::{Deserialize, Serialize};

use crate::CalcError;
use crate::all_finite;

// µ₀ and the mils→metres factor come from `crate::constants`. Both previously existed here
// as private duplicates (µ₀ in a second spelling, the length factor as 25.4e-6).
use crate::constants::{MIL_TO_M as MILS_TO_METERS, MU_0};

/// Spiral geometry shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpiralShape {
    Square,
    Hexagonal,
    Octagonal,
    Circle,
}

impl SpiralShape {
    /// Mohan coefficients (K1, K2) for each geometry.
    fn coefficients(self) -> (f64, f64) {
        match self {
            Self::Square => (2.34, 2.75),
            Self::Hexagonal => (2.33, 3.82),
            Self::Octagonal => (2.25, 3.55),
            Self::Circle => (2.23, 3.45),
        }
    }
}

/// Result of a planar spiral inductor calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InductorResult {
    /// Inner diameter of the spiral in mils.
    pub din_mils: f64,
    /// Fill factor ρ = (dout − din) / (dout + din).
    pub rho: f64,
    /// Average diameter d_avg = (dout + din) / 2, in mils.
    pub d_avg_mils: f64,
    /// Calculated inductance in nanohenries.
    pub inductance_nh: f64,
}

/// Calculate planar spiral inductor using the modified Wheeler / Mohan formula.
///
/// # Arguments
/// - `n_turns` — number of turns (must be ≥ 1)
/// - `width_mils` — trace width in mils (must be > 0)
/// - `spacing_mils` — inter-turn spacing in mils (must be > 0)
/// - `dout_mils` — outer diameter in mils (must be > 0)
/// - `shape` — spiral geometry
///
/// The inner diameter is derived as:
/// `din = dout − 2×n×(w+s) + 2×s`
///
/// # Errors
/// Returns [`CalcError::OutOfRange`] or [`CalcError::NegativeDimension`] if inputs are invalid.
pub fn planar_spiral(
    n_turns: u32,
    width_mils: f64,
    spacing_mils: f64,
    dout_mils: f64,
    shape: SpiralShape,
) -> Result<InductorResult, CalcError> {
    all_finite(&[
        ("width_mils", width_mils),
        ("spacing_mils", spacing_mils),
        ("dout_mils", dout_mils),
    ])?;
    if n_turns == 0 {
        return Err(CalcError::OutOfRange {
            name: "n_turns",
            value: n_turns as f64,
            expected: ">= 1",
        });
    }
    if width_mils <= 0.0 {
        return Err(CalcError::NegativeDimension {
            name: "width_mils",
            value: width_mils,
        });
    }
    if spacing_mils <= 0.0 {
        return Err(CalcError::NegativeDimension {
            name: "spacing_mils",
            value: spacing_mils,
        });
    }
    if dout_mils <= 0.0 {
        return Err(CalcError::NegativeDimension {
            name: "dout_mils",
            value: dout_mils,
        });
    }

    let n = n_turns as f64;
    let din_mils = dout_mils - 2.0 * n * (width_mils + spacing_mils) + 2.0 * spacing_mils;

    if din_mils <= 0.0 {
        return Err(CalcError::OutOfRange {
            name: "din_mils (derived)",
            value: din_mils,
            expected: "> 0 — reduce n_turns, width, or spacing, or increase dout",
        });
    }

    let rho = (dout_mils - din_mils) / (dout_mils + din_mils);
    let d_avg_mils = (dout_mils + din_mils) / 2.0;
    let d_avg_m = d_avg_mils * MILS_TO_METERS;

    let (k1, k2) = shape.coefficients();
    let inductance_h = k1 * MU_0 * n * n * d_avg_m / (1.0 + k2 * rho);
    let inductance_nh = inductance_h * 1e9;

    Ok(InductorResult {
        din_mils,
        rho,
        d_avg_mils,
        inductance_nh,
    })
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    // Saturn PDF page 30: n=5, w=10 mils, s=10 mils, dout=350 mils, Square
    // din=170, ρ=0.3462, L=248.5936 nH
    #[test]
    fn saturn_page30_square() {
        let result = planar_spiral(5, 10.0, 10.0, 350.0, SpiralShape::Square).unwrap();
        assert_relative_eq!(result.din_mils, 170.0, epsilon = 1e-10);
        assert_relative_eq!(result.rho, 0.34615, epsilon = 1e-4);
        assert_relative_eq!(result.inductance_nh, 248.59, epsilon = 0.2);
    }

    #[test]
    fn derived_din_matches_spec() {
        // din = 350 - 2×5×(10+10) + 2×10 = 350 - 200 + 20 = 170
        let result = planar_spiral(5, 10.0, 10.0, 350.0, SpiralShape::Square).unwrap();
        assert_relative_eq!(result.din_mils, 170.0, epsilon = 1e-10);
    }

    #[test]
    fn error_on_zero_turns() {
        assert!(planar_spiral(0, 10.0, 10.0, 350.0, SpiralShape::Square).is_err());
    }

    #[test]
    fn error_on_din_negative() {
        // Very large n will make din negative
        assert!(planar_spiral(50, 10.0, 10.0, 350.0, SpiralShape::Square).is_err());
    }

    #[test]
    fn hexagonal_shape_accepted() {
        assert!(planar_spiral(3, 10.0, 10.0, 200.0, SpiralShape::Hexagonal).is_ok());
    }
}

impl std::str::FromStr for SpiralShape {
    type Err = CalcError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "square" => Ok(Self::Square),
            "hexagonal" | "hex" => Ok(Self::Hexagonal),
            "octagonal" | "oct" => Ok(Self::Octagonal),
            "circle" | "circular" => Ok(Self::Circle),
            _ => Err(CalcError::UnknownVariant {
                kind: "spiral shape",
                value: s.to_string(),
                expected: "square, hexagonal, octagonal, circle",
            }),
        }
    }
}

#[cfg(test)]
mod parse_tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn shape_parsing() {
        assert_eq!(
            SpiralShape::from_str("square").unwrap(),
            SpiralShape::Square
        );
        assert_eq!(
            SpiralShape::from_str("CIRCULAR").unwrap(),
            SpiralShape::Circle
        );
        assert_eq!(
            SpiralShape::from_str("hex").unwrap(),
            SpiralShape::Hexagonal
        );
        assert!(SpiralShape::from_str("triangle").is_err());
    }
}

// ---------------------------------------------------------------------------------------
// Single-turn loop inductance
// ---------------------------------------------------------------------------------------

/// Shape of a single-turn loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoopShape {
    /// A circular loop, described by its diameter.
    Circular,
    /// A rectangular loop, described by its two side lengths.
    Rectangular,
}

impl LoopShape {
    /// Every variant, for a UI dropdown.
    pub fn all() -> &'static [LoopShape] {
        &[LoopShape::Circular, LoopShape::Rectangular]
    }
}

impl std::str::FromStr for LoopShape {
    type Err = CalcError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "circular" | "circle" | "round" => Ok(Self::Circular),
            "rectangular" | "rectangle" | "rect" | "square" => Ok(Self::Rectangular),
            _ => Err(CalcError::UnknownVariant {
                kind: "loop shape",
                value: s.to_string(),
                expected: "circular, rectangular",
            }),
        }
    }
}

/// Result of a single-turn loop inductance calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoopInductanceResult {
    /// Loop inductance (nH).
    pub inductance_nh: f64,
    /// Area enclosed by the loop (sq mils) — the quantity to minimise.
    pub area_sq_mils: f64,
    /// Perimeter of the loop (mils).
    pub perimeter_mils: f64,
}

/// Inductance of a single circular loop of round wire.
///
/// `L = 1.56e-8 · D · (ln(8D/d) − 2)` henries, with `D` the loop diameter and `d` the wire
/// diameter, both in inches. Source: the CIRCULAR worksheet shipped with Johnson & Graham,
/// *High-Speed Digital Design*.
///
/// This is a different animal from [`planar_spiral`]: that is a multi-turn spiral you draw
/// deliberately, this is the one turn you get by accident — a ground loop, the return path of
/// a scope probe, the area enclosed between a bypass capacitor and the pin it serves. The
/// useful property is how weakly it depends on wire diameter and how strongly on loop size:
/// the logarithm barely moves while the `D` out front is linear.
///
/// # Errors
/// Returns [`CalcError::NegativeDimension`] for non-positive dimensions, and
/// [`CalcError::UnphysicalResult`] if the wire is so thick for the loop that the bracket goes
/// non-positive — at which point it is not a loop.
pub fn circular_loop(
    loop_diameter_mils: f64,
    wire_diameter_mils: f64,
) -> Result<LoopInductanceResult, CalcError> {
    all_finite(&[
        ("loop_diameter_mils", loop_diameter_mils),
        ("wire_diameter_mils", wire_diameter_mils),
    ])?;
    if loop_diameter_mils <= 0.0 {
        return Err(CalcError::NegativeDimension {
            name: "loop_diameter_mils",
            value: loop_diameter_mils,
        });
    }
    if wire_diameter_mils <= 0.0 {
        return Err(CalcError::NegativeDimension {
            name: "wire_diameter_mils",
            value: wire_diameter_mils,
        });
    }

    let d_in = loop_diameter_mils / 1000.0;
    let w_in = wire_diameter_mils / 1000.0;
    let bracket = (8.0 * d_in / w_in).ln() - 2.0;
    if bracket <= 0.0 {
        return Err(CalcError::UnphysicalResult {
            name: "inductance_nh",
            value: bracket,
            reason: "wire diameter is too large for the loop — ln(8D/d) must exceed 2",
        });
    }

    let radius = loop_diameter_mils / 2.0;
    Ok(LoopInductanceResult {
        inductance_nh: 1.56e-8 * d_in * bracket * 1e9,
        area_sq_mils: std::f64::consts::PI * radius * radius,
        perimeter_mils: std::f64::consts::PI * loop_diameter_mils,
    })
}

/// Inductance of a single rectangular loop of round wire.
///
/// ```text
///   D = sqrt(x² + b²)
///   L = 10.16e-9 · [ x·ln(2bx/(d·(x + D))) + b·ln(2bx/(d·(b + D)))
///                    − 2(x + b − D) + (x + b)/2 ]
/// ```
///
/// henries, with `x` and `b` the side lengths and `d` the wire diameter, all in inches.
/// Source: the RECTANGL worksheet shipped with Johnson & Graham.
///
/// A long thin rectangle has considerably more inductance than a square enclosing the same
/// area, which is the practical lesson — a return path that detours *is* a long thin
/// rectangle.
///
/// # Errors
/// Returns [`CalcError::NegativeDimension`] for non-positive dimensions, and
/// [`CalcError::UnphysicalResult`] if the result is not a positive inductance, which happens
/// once the wire diameter becomes comparable to a side length.
pub fn rectangular_loop(
    length_mils: f64,
    breadth_mils: f64,
    wire_diameter_mils: f64,
) -> Result<LoopInductanceResult, CalcError> {
    all_finite(&[
        ("length_mils", length_mils),
        ("breadth_mils", breadth_mils),
        ("wire_diameter_mils", wire_diameter_mils),
    ])?;
    for (name, value) in [
        ("length_mils", length_mils),
        ("breadth_mils", breadth_mils),
        ("wire_diameter_mils", wire_diameter_mils),
    ] {
        if value <= 0.0 {
            return Err(CalcError::NegativeDimension { name, value });
        }
    }

    let x = length_mils / 1000.0;
    let b = breadth_mils / 1000.0;
    let d = wire_diameter_mils / 1000.0;
    let diag = (x * x + b * b).sqrt();

    let l_h = 10.16e-9
        * (x * (2.0 * b * x / (d * (x + diag))).ln() + b * (2.0 * b * x / (d * (b + diag))).ln()
            - 2.0 * (x + b - diag)
            + (x + b) / 2.0);

    if !l_h.is_finite() || l_h <= 0.0 {
        return Err(CalcError::UnphysicalResult {
            name: "inductance_nh",
            value: l_h * 1e9,
            reason: "not a positive inductance — wire diameter is too large for the loop",
        });
    }

    Ok(LoopInductanceResult {
        inductance_nh: l_h * 1e9,
        area_sq_mils: length_mils * breadth_mils,
        perimeter_mils: 2.0 * (length_mils + breadth_mils),
    })
}

#[cfg(test)]
mod loop_tests {
    use approx::assert_relative_eq;

    use super::*;

    /// The worked examples stored in the CIRCULAR worksheet: a 1.3 in loop of 10 mil wire is
    /// 100.32 nH, and the same loop in 100 mil wire is 53.63 nH.
    ///
    /// The pair is the useful part. Ten times the wire diameter removes only 47% of the
    /// inductance, because the dependence is logarithmic — you do not fix a loop with fatter
    /// wire, you fix it by making the loop smaller.
    #[test]
    fn hsdd_circular_worksheet_vectors() {
        let thin = circular_loop(1300.0, 10.0).unwrap();
        assert_relative_eq!(
            thin.inductance_nh,
            100.324_673_120_506_29,
            max_relative = 1e-9
        );
        let fat = circular_loop(1300.0, 100.0).unwrap();
        assert_relative_eq!(
            fat.inductance_nh,
            53.628_247_434_587_03,
            max_relative = 1e-9
        );
    }

    /// The rectangular form, checked term by term against the published expression at the
    /// geometry the RECTANGL worksheet stores (4 in × 1 in of 1 in wire). The geometry is
    /// absurd as a loop, but it is what the sheet holds and it exercises every term.
    #[test]
    fn hsdd_rectangl_worksheet_vector() {
        let r = rectangular_loop(4000.0, 1000.0, 1000.0).unwrap();
        let (x, b, d) = (4.0f64, 1.0f64, 1.0f64);
        let diag = (x * x + b * b).sqrt();
        let want = 10.16
            * (x * (2.0 * b * x / (d * (x + diag))).ln()
                + b * (2.0 * b * x / (d * (b + diag))).ln()
                - 2.0 * (x + b - diag)
                + (x + b) / 2.0);
        assert_relative_eq!(r.inductance_nh, want, max_relative = 1e-12);
    }

    /// Area and perimeter must be the plain formulas: the whole point of reporting them is to
    /// let two loops enclosing the same area be compared.
    #[test]
    fn area_and_perimeter_are_reported_correctly() {
        let c = circular_loop(1000.0, 10.0).unwrap();
        assert_relative_eq!(
            c.area_sq_mils,
            std::f64::consts::PI * 500.0 * 500.0,
            max_relative = 1e-12
        );
        assert_relative_eq!(
            c.perimeter_mils,
            std::f64::consts::PI * 1000.0,
            max_relative = 1e-12
        );

        let r = rectangular_loop(2000.0, 500.0, 10.0).unwrap();
        assert_relative_eq!(r.area_sq_mils, 1_000_000.0, max_relative = 1e-12);
        assert_relative_eq!(r.perimeter_mils, 5000.0, max_relative = 1e-12);
    }

    /// Equal area, different shape: the long thin loop must lose.
    #[test]
    fn a_long_thin_loop_is_worse_than_a_square_of_equal_area() {
        let square = rectangular_loop(1000.0, 1000.0, 10.0).unwrap();
        let thin = rectangular_loop(5000.0, 200.0, 10.0).unwrap();
        assert_relative_eq!(square.area_sq_mils, thin.area_sq_mils, max_relative = 1e-12);
        assert!(
            thin.inductance_nh > square.inductance_nh,
            "thin {} should exceed square {}",
            thin.inductance_nh,
            square.inductance_nh
        );
    }

    #[test]
    fn inductance_rises_with_size_and_falls_with_wire_diameter() {
        assert!(
            circular_loop(2000.0, 10.0).unwrap().inductance_nh
                > circular_loop(1000.0, 10.0).unwrap().inductance_nh
        );
        assert!(
            circular_loop(1000.0, 5.0).unwrap().inductance_nh
                > circular_loop(1000.0, 50.0).unwrap().inductance_nh
        );
        assert!(
            rectangular_loop(2000.0, 2000.0, 10.0)
                .unwrap()
                .inductance_nh
                > rectangular_loop(1000.0, 1000.0, 10.0)
                    .unwrap()
                    .inductance_nh
        );
        assert!(
            rectangular_loop(1000.0, 1000.0, 5.0).unwrap().inductance_nh
                > rectangular_loop(1000.0, 1000.0, 50.0)
                    .unwrap()
                    .inductance_nh
        );
    }

    #[test]
    fn rectangular_loop_is_symmetric_in_its_sides() {
        let a = rectangular_loop(3000.0, 700.0, 10.0).unwrap();
        let b = rectangular_loop(700.0, 3000.0, 10.0).unwrap();
        assert_relative_eq!(a.inductance_nh, b.inductance_nh, max_relative = 1e-12);
    }

    #[test]
    fn loop_shape_parsing() {
        use std::str::FromStr;
        assert_eq!(LoopShape::from_str("circle").unwrap(), LoopShape::Circular);
        assert_eq!(
            LoopShape::from_str("RECTANGLE").unwrap(),
            LoopShape::Rectangular
        );
        assert!(LoopShape::from_str("trapezoid").is_err());
        assert_eq!(LoopShape::all().len(), 2);
    }

    #[test]
    fn degenerate_loops_are_rejected() {
        // ln(8D/d) must exceed 2, so D/d must exceed e²/8 ≈ 0.924.
        assert!(circular_loop(100.0, 200.0).is_err());
        assert!(circular_loop(0.0, 10.0).is_err());
        assert!(circular_loop(1000.0, -1.0).is_err());
        assert!(rectangular_loop(0.0, 100.0, 10.0).is_err());
        assert!(rectangular_loop(100.0, 100.0, 0.0).is_err());
        // Wire far thicker than the loop.
        assert!(rectangular_loop(100.0, 100.0, 5000.0).is_err());
    }
}
