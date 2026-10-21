//! Capacitive/inductive reactance and resonant frequency calculator.
//!
//! Xc = 1 / (2πfC)
//! Xl = 2πfL
//! f_res = 1 / (2π√(LC))

use serde::{Deserialize, Serialize};

use crate::CalcError;
use crate::all_finite;

/// Result of a reactance calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReactanceResult {
    /// Capacitive reactance in Ohms. None if capacitance was not provided.
    pub xc_ohms: Option<f64>,
    /// Inductive reactance in Ohms. None if inductance was not provided.
    pub xl_ohms: Option<f64>,
    /// Resonant frequency in Hz. None if both L and C were not provided.
    pub f_res_hz: Option<f64>,
}

/// Calculate capacitive reactance, inductive reactance, and resonant frequency.
///
/// # Arguments
/// - `freq_hz` — frequency in Hz (must be > 0)
/// - `capacitance_f` — capacitance in Farads (None to skip Xc and f_res)
/// - `inductance_h` — inductance in Henries (None to skip Xl and f_res)
///
/// # Errors
/// Returns [`CalcError::OutOfRange`] if `freq_hz` ≤ 0 or either value is ≤ 0.
pub fn reactance(
    freq_hz: f64,
    capacitance_f: Option<f64>,
    inductance_h: Option<f64>,
) -> Result<ReactanceResult, CalcError> {
    // `unwrap_or(0.0)` stands in for an absent optional: zero is finite, so a `None`
    // passes, while a `Some(NaN)` is still caught.
    all_finite(&[
        ("freq_hz", freq_hz),
        ("capacitance_f", capacitance_f.unwrap_or(0.0)),
        ("inductance_h", inductance_h.unwrap_or(0.0)),
    ])?;
    if freq_hz <= 0.0 {
        return Err(CalcError::OutOfRange {
            name: "freq_hz",
            value: freq_hz,
            expected: "> 0",
        });
    }

    if let Some(c) = capacitance_f {
        if c <= 0.0 {
            return Err(CalcError::OutOfRange {
                name: "capacitance_f",
                value: c,
                expected: "> 0",
            });
        }
    }

    if let Some(l) = inductance_h {
        if l <= 0.0 {
            return Err(CalcError::OutOfRange {
                name: "inductance_h",
                value: l,
                expected: "> 0",
            });
        }
    }

    let two_pi_f = 2.0 * std::f64::consts::PI * freq_hz;

    let xc_ohms = capacitance_f.map(|c| 1.0 / (two_pi_f * c));
    let xl_ohms = inductance_h.map(|l| two_pi_f * l);

    let f_res_hz = match (capacitance_f, inductance_h) {
        (Some(c), Some(l)) => Some(1.0 / (2.0 * std::f64::consts::PI * (l * c).sqrt())),
        _ => None,
    };

    Ok(ReactanceResult {
        xc_ohms,
        xl_ohms,
        f_res_hz,
    })
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    // Saturn PDF page 41: f=1 MHz, C=1 µF, L=1 mH
    // Xc=0.1592 Ω, Xl=6283.1800 Ω, f_res=5032.9255 Hz
    #[test]
    fn saturn_page41_vector() {
        let result = reactance(1e6, Some(1e-6), Some(1e-3)).unwrap();
        assert_relative_eq!(result.xc_ohms.unwrap(), 0.15915494, epsilon = 1e-4);
        assert_relative_eq!(result.xl_ohms.unwrap(), 6283.1853, epsilon = 1e-1);
        assert_relative_eq!(result.f_res_hz.unwrap(), 5032.9255, epsilon = 1e-2);
    }

    // C=10nF, f=1kHz → Xc = 15915 Ω
    #[test]
    fn xc_10nf_1khz() {
        let result = reactance(1e3, Some(10e-9), None).unwrap();
        assert_relative_eq!(result.xc_ohms.unwrap(), 15915.494, epsilon = 1e-1);
    }

    // L=100µH, f=1kHz → Xl = 0.6283 Ω
    #[test]
    fn xl_100uh_1khz() {
        let result = reactance(1e3, None, Some(100e-6)).unwrap();
        assert_relative_eq!(result.xl_ohms.unwrap(), 0.6283185, epsilon = 1e-4);
    }

    #[test]
    fn error_on_zero_freq() {
        assert!(reactance(0.0, Some(1e-6), None).is_err());
    }

    #[test]
    fn error_on_negative_capacitance() {
        assert!(reactance(1e6, Some(-1e-6), None).is_err());
    }
}

// ---------------------------------------------------------------------------------------
// Reactance to a rising edge
// ---------------------------------------------------------------------------------------

/// Result of an edge-domain reactance calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EdgeReactanceResult {
    /// The 10–90% rise time the calculation was made for (ns).
    pub rise_time_ns: f64,
    /// Impedance a capacitance presents to the edge (Ω). None if C was not provided.
    pub xc_ohms: Option<f64>,
    /// Impedance an inductance presents to the edge (Ω). None if L was not provided.
    pub xl_ohms: Option<f64>,
    /// Knee frequency of the edge (Hz), for comparison with the sine-domain figures.
    pub knee_frequency_hz: f64,
}

/// Reactance as a *rising edge* sees it, rather than as a sine wave sees it.
///
/// ```text
///   Xc = Tr / (π·C)      Xl = π·L / Tr
/// ```
///
/// Source: the CAPAC and MLINE worksheets shipped with Johnson & Graham, *High-Speed Digital
/// Design*. These are the forms that actually answer the question you have in front of a
/// schematic — "is this 100 pF load going to matter to a 5 ns edge?" — because a digital edge
/// is a step, not a tone, and [`reactance`] asks you to pick a single frequency for something
/// that has a spectrum.
///
/// Both forms are **exactly** the sine-domain reactance evaluated at the edge's knee
/// frequency `f = 0.5/Tr`, which is what makes them worth trusting rather than merely
/// convenient: substituting into `1/(2πfC)` gives `Tr/(πC)`, and into `2πfL` gives `πL/Tr`.
/// So these are not a separate approximation — they are [`reactance`] with the frequency
/// already chosen for you, by the only rule that has a defensible answer. The knee frequency
/// is reported alongside so you can see which frequency that was.
///
/// # Errors
/// Returns [`CalcError::OutOfRange`] if `rise_time_ns` ≤ 0 or either value is ≤ 0.
pub fn reactance_to_edge(
    rise_time_ns: f64,
    capacitance_f: Option<f64>,
    inductance_h: Option<f64>,
) -> Result<EdgeReactanceResult, CalcError> {
    // See `reactance` above for why an absent optional maps to 0.0 here.
    all_finite(&[
        ("rise_time_ns", rise_time_ns),
        ("capacitance_f", capacitance_f.unwrap_or(0.0)),
        ("inductance_h", inductance_h.unwrap_or(0.0)),
    ])?;
    if rise_time_ns <= 0.0 {
        return Err(CalcError::OutOfRange {
            name: "rise_time_ns",
            value: rise_time_ns,
            expected: "> 0",
        });
    }
    if let Some(c) = capacitance_f {
        if c <= 0.0 {
            return Err(CalcError::OutOfRange {
                name: "capacitance_f",
                value: c,
                expected: "> 0",
            });
        }
    }
    if let Some(l) = inductance_h {
        if l <= 0.0 {
            return Err(CalcError::OutOfRange {
                name: "inductance_h",
                value: l,
                expected: "> 0",
            });
        }
    }

    let tr_s = rise_time_ns * 1e-9;
    Ok(EdgeReactanceResult {
        rise_time_ns,
        xc_ohms: capacitance_f.map(|c| tr_s / (std::f64::consts::PI * c)),
        xl_ohms: inductance_h.map(|l| std::f64::consts::PI * l / tr_s),
        knee_frequency_hz: 0.5 / tr_s,
    })
}

#[cfg(test)]
mod edge_tests {
    use approx::assert_relative_eq;

    use super::*;

    /// The two figures the source worksheets quote: 100 pF to a 5 ns edge looks like
    /// 15.9 Ω, and 100 nH of lead inductance looks like 62.8 Ω.
    #[test]
    fn hsdd_capac_worksheet_vectors() {
        let r = reactance_to_edge(5.0, Some(100e-12), Some(100e-9)).unwrap();
        assert_relative_eq!(
            r.xc_ohms.unwrap(),
            15.915_494_309_189_533,
            max_relative = 1e-12
        );
        assert_relative_eq!(
            r.xl_ohms.unwrap(),
            62.831_853_071_795_86,
            max_relative = 1e-12
        );
    }

    /// Both edge forms are exactly the sine-domain reactance at the knee frequency.
    ///
    /// This is the test that establishes what [`reactance_to_edge`] *is*, and it is the one
    /// I had wrong when writing this module: the docs originally claimed the inductive pair
    /// differed by π²/2. They do not — `2π·(0.5/Tr)·L` is `πL/Tr` identically, the same way
    /// `1/(2π·(0.5/Tr)·C)` is `Tr/(πC)`. Asserting both directions at several rise times
    /// also pins the ns→s conversion in both functions at once.
    #[test]
    fn both_edge_forms_equal_the_sine_value_at_the_knee() {
        for (tr_ns, c, l) in [
            (5.0, 100e-12, 100e-9),
            (0.35, 10e-12, 5e-9),
            (1.0, 1e-9, 1e-6),
        ] {
            let edge = reactance_to_edge(tr_ns, Some(c), Some(l)).unwrap();
            let sine = reactance(edge.knee_frequency_hz, Some(c), Some(l)).unwrap();
            assert_relative_eq!(
                edge.xc_ohms.unwrap(),
                sine.xc_ohms.unwrap(),
                max_relative = 1e-12
            );
            assert_relative_eq!(
                edge.xl_ohms.unwrap(),
                sine.xl_ohms.unwrap(),
                max_relative = 1e-12
            );
        }
    }

    /// A faster edge makes a capacitor look like less and an inductor like more, linearly.
    #[test]
    fn halving_the_rise_time_halves_xc_and_doubles_xl() {
        let slow = reactance_to_edge(2.0, Some(100e-12), Some(100e-9)).unwrap();
        let fast = reactance_to_edge(1.0, Some(100e-12), Some(100e-9)).unwrap();
        assert_relative_eq!(
            fast.xc_ohms.unwrap() / slow.xc_ohms.unwrap(),
            0.5,
            max_relative = 1e-12
        );
        assert_relative_eq!(
            fast.xl_ohms.unwrap() / slow.xl_ohms.unwrap(),
            2.0,
            max_relative = 1e-12
        );
    }

    #[test]
    fn omitted_values_stay_none() {
        let r = reactance_to_edge(1.0, Some(1e-12), None).unwrap();
        assert!(r.xc_ohms.is_some() && r.xl_ohms.is_none());
    }

    #[test]
    fn invalid_inputs_are_rejected() {
        assert!(reactance_to_edge(0.0, Some(1e-12), None).is_err());
        assert!(reactance_to_edge(1.0, Some(-1e-12), None).is_err());
        assert!(reactance_to_edge(1.0, None, Some(0.0)).is_err());
    }
}
