//! Conductor current capacity calculator.
//!
//! Supports:
//! - IPC-2221A (legacy empirical formula)
//! - IPC-2152 (with modifier charts — table data needed)
//!
//! Also computes: DC resistance, voltage drop, power dissipation,
//! skin depth, current density.
//!
//! # Known limitations
//!
//! - The IPC-2152 modifier charts are simplified piecewise approximations. Saturn's lookup
//!   tables were only partially extracted, so the area, temperature-rise and
//!   board-thickness modifiers are fits rather than the tables themselves.
//! - **DC only.** `resistance_dc` is temperature-corrected (via
//!   [`crate::resistance::temperature_factor`]) but has no skin-effect or proximity term, so
//!   at any frequency a digital edge cares about the real resistance is higher. `skin_depth`
//!   is reported so the gap is visible rather than implied.
//! - There is no solve-for-width mode. Given a target current this will not tell you the
//!   width; that is an inverse solver, and `STATUS.md` records the decision to leave inverse
//!   solvers out of scope.

use serde::{Deserialize, Serialize};

use crate::CalcError;
use crate::all_finite;
use crate::copper::EtchFactor;

/// IPC-2221A constant for external (surface) layers.
const K_EXTERNAL: f64 = 0.048;

/// IPC-2221A constant for internal layers.
const K_INTERNAL: f64 = 0.024;

/// Copper resistivity in ohm-mil units (6.787e-4 Ω·mil).
///
/// Copper resistivity at 20 °C in Ω·mil, for `R = ρL/A` with `L` in mils and `A` in mil².
///
/// Re-exported from [`crate::resistance`], which derives it from the SI constant rather than
/// typing `6.787e-4` a second time. Saturn carries the same figure.
use crate::resistance::COPPER_RESISTIVITY_OHM_MIL as RESISTIVITY_OHM_MIL;

// Copper resistivity (Ω·m) and µ₀ come from `crate::constants`, which is their single home.
// Both previously existed here as private duplicates.
use crate::constants::{COPPER_RESISTIVITY_OHM_M, MU_0};

/// Inputs for conductor current capacity calculation.
pub struct CurrentInput {
    /// Trace width (mils).
    pub width: f64,
    /// Copper thickness (mils).
    pub thickness: f64,
    /// Trace length (mils). Used for resistance and voltage drop.
    pub length: f64,
    /// Allowed temperature rise above ambient (°C).
    pub temperature_rise: f64,
    /// Ambient temperature (°C). Not currently used in IPC-2221A, reserved.
    pub ambient_temp: f64,
    /// Frequency (Hz). Used for skin depth calculation. 0 = DC only.
    pub frequency: f64,
    /// Etch factor affecting cross-section geometry.
    pub etch_factor: EtchFactor,
    /// Whether the trace is on an internal layer (halves the current capacity).
    pub is_internal: bool,
}

/// Result of a conductor current capacity calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CurrentResult {
    /// Maximum current capacity (A) per IPC-2221A.
    pub current_capacity: f64,
    /// Conductor cross-sectional area (sq mils).
    pub cross_section: f64,
    /// DC resistance (Ohms).
    pub resistance_dc: f64,
    /// Voltage drop at maximum current (V).
    pub voltage_drop: f64,
    /// Power dissipation at maximum current (W).
    pub power_dissipation: f64,
    /// Current density at maximum current (A/mil²).
    pub current_density: f64,
    /// Skin depth at the given frequency (mils). 0 if frequency is 0.
    pub skin_depth_mils: f64,
}

/// Calculate conductor current capacity and related electrical properties.
///
/// Uses the IPC-2221A empirical formula:
///   I = k × ΔT^0.44 × A^0.725
/// where k = 0.048 (external) or 0.024 (internal), ΔT in °C, A in mil².
pub fn calculate(input: &CurrentInput) -> Result<CurrentResult, CalcError> {
    let CurrentInput {
        width,
        thickness,
        length,
        temperature_rise,
        ambient_temp,
        etch_factor,
        is_internal,
        frequency,
    } = *input;

    all_finite(&[
        ("width", width),
        ("thickness", thickness),
        ("length", length),
        ("temperature_rise", temperature_rise),
        ("ambient_temp", ambient_temp),
        ("frequency", frequency),
    ])?;

    if width <= 0.0 {
        return Err(CalcError::NegativeDimension {
            name: "width",
            value: width,
        });
    }
    if thickness <= 0.0 {
        return Err(CalcError::NegativeDimension {
            name: "thickness",
            value: thickness,
        });
    }
    if length <= 0.0 {
        return Err(CalcError::NegativeDimension {
            name: "length",
            value: length,
        });
    }
    if temperature_rise <= 0.0 {
        return Err(CalcError::OutOfRange {
            name: "temperature_rise",
            value: temperature_rise,
            expected: "> 0",
        });
    }
    if frequency < 0.0 {
        return Err(CalcError::OutOfRange {
            name: "frequency",
            value: frequency,
            expected: ">= 0",
        });
    }

    // Cross-sectional area (sq mils), accounting for etch factor
    let cross_section = etch_factor.cross_section_sq_mils(width, thickness);

    // IPC-2221A current capacity
    let k = if is_internal { K_INTERNAL } else { K_EXTERNAL };
    let current_capacity = k * temperature_rise.powf(0.44) * cross_section.powf(0.725);

    // DC resistance: R = ρ × L / A  (in ohm-mil units), corrected to the ambient
    // temperature. `ambient_temp` was documented as "reserved, not currently used" — this is
    // what it is for. Copper rises 0.39%/°C, so the default 25 °C ambient is already 2% above
    // the 20 °C figure the resistivity constant is quoted at.
    let resistance_dc = RESISTIVITY_OHM_MIL * length / cross_section
        * crate::resistance::temperature_factor(ambient_temp)?;

    // Voltage drop and power at max current
    let voltage_drop = current_capacity * resistance_dc;
    let power_dissipation = current_capacity * voltage_drop;

    // Current density (A/mil²)
    let current_density = current_capacity / cross_section;

    // Skin depth: δ = √(ρ / (π × f × µ₀))
    let skin_depth_mils = if frequency > 0.0 {
        let delta_m = (COPPER_RESISTIVITY_OHM_M / (std::f64::consts::PI * frequency * MU_0)).sqrt();
        // Convert m to mils: 1 m = 1 / 2.54e-5 mils = 39370.079 mils
        delta_m / crate::constants::MIL_TO_M
    } else {
        0.0
    };

    Ok(CurrentResult {
        current_capacity,
        cross_section,
        resistance_dc,
        voltage_drop,
        power_dissipation,
        current_density,
        skin_depth_mils,
    })
}

/// Inputs for IPC-2152 conductor current capacity calculation.
pub struct Ipc2152Input {
    /// Trace width (mils).
    pub width: f64,
    /// Copper thickness (mils).
    pub thickness: f64,
    /// Trace length (mils).
    pub length: f64,
    /// Allowed temperature rise above ambient (°C).
    pub temperature_rise: f64,
    /// Ambient temperature (°C).
    pub ambient_temp: f64,
    /// Frequency (Hz). 0 = DC only.
    pub frequency: f64,
    /// Etch factor.
    pub etch_factor: EtchFactor,
    /// Whether the trace is on an internal layer.
    pub is_internal: bool,
    /// Board thickness (mils).
    pub board_thickness_mils: f64,
    /// Whether the board has an adjacent copper plane.
    pub has_copper_plane: bool,
    /// Material thermal conductivity modifier (default 1.0).
    pub material_modifier: f64,
    /// User-supplied modifier (default 1.0).
    pub user_modifier: f64,
}

/// Result of an IPC-2152 calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ipc2152Result {
    /// Final current capacity with all modifiers (A).
    pub current_capacity: f64,
    /// Conductor cross-sectional area (sq mils).
    pub cross_section: f64,
    /// DC resistance (Ohms).
    pub resistance_dc: f64,
    /// Voltage drop at calculated current (V).
    pub voltage_drop: f64,
    /// Power dissipation (W).
    pub power_dissipation: f64,
    /// Current density (A/mil²).
    pub current_density: f64,
    /// Skin depth (mils). 0 if frequency is 0.
    pub skin_depth_mils: f64,
    /// Area modifier applied.
    pub m_area: f64,
    /// Temperature rise modifier applied.
    pub m_temp: f64,
    /// Board thickness modifier applied.
    pub m_board: f64,
}

fn m_temp_lookup(dt: f64) -> f64 {
    if dt <= 10.0 {
        0.40
    } else if dt <= 20.0 {
        0.48
    } else if dt <= 30.0 {
        0.58
    } else if dt <= 40.0 {
        0.67
    } else if dt <= 50.0 {
        0.75
    } else if dt <= 60.0 {
        0.85
    } else if dt <= 70.0 {
        0.95
    } else if dt <= 80.0 {
        1.00
    } else if dt <= 90.0 {
        1.10
    } else if dt <= 100.0 {
        1.20
    } else {
        1.30
    }
}

fn m_board_lookup(thickness_mils: f64, has_plane: bool) -> f64 {
    if has_plane {
        if thickness_mils <= 10.0 {
            1.63
        } else if thickness_mils <= 20.0 {
            1.59
        } else if thickness_mils <= 30.0 {
            1.56
        } else if thickness_mils <= 40.0 {
            1.52
        } else if thickness_mils <= 50.0 {
            1.49
        } else if thickness_mils <= 60.0 {
            1.46
        } else if thickness_mils <= 70.0 {
            1.43
        } else if thickness_mils <= 80.0 {
            1.41
        } else if thickness_mils <= 90.0 {
            1.37
        } else if thickness_mils <= 100.0 {
            1.34
        } else {
            1.24
        }
    } else {
        if thickness_mils <= 10.0 {
            1.59
        } else if thickness_mils <= 20.0 {
            1.55
        } else if thickness_mils <= 30.0 {
            1.52
        } else if thickness_mils <= 40.0 {
            1.48
        } else if thickness_mils <= 50.0 {
            1.45
        } else if thickness_mils <= 60.0 {
            1.42
        } else if thickness_mils <= 70.0 {
            1.39
        } else if thickness_mils <= 80.0 {
            1.37
        } else if thickness_mils <= 90.0 {
            1.33
        } else if thickness_mils <= 100.0 {
            1.30
        } else {
            1.20
        }
    }
}

fn m_area_lookup(area: f64) -> f64 {
    if area <= 20.0 {
        3.0364 * area.powf(-0.145)
    } else if area <= 60.0 {
        2.9143 * area.powf(-0.129)
    } else if area <= 100.0 {
        2.7877 * area.powf(-0.114)
    } else {
        2.801 * area.powf(-0.111)
    }
}

/// Calculate conductor current capacity per IPC-2152 with modifier tables.
///
/// Applies area, temperature rise, and board thickness modifiers to the
/// IPC-2221A base formula, then scales by material and user modifiers.
pub fn calculate_ipc2152(input: &Ipc2152Input) -> Result<Ipc2152Result, CalcError> {
    let Ipc2152Input {
        width,
        thickness,
        length,
        temperature_rise,
        ambient_temp,
        frequency,
        ref etch_factor,
        is_internal,
        board_thickness_mils,
        has_copper_plane,
        material_modifier,
        user_modifier,
    } = *input;

    all_finite(&[
        ("width", width),
        ("thickness", thickness),
        ("length", length),
        ("temperature_rise", temperature_rise),
        ("ambient_temp", ambient_temp),
        ("frequency", frequency),
        ("board_thickness_mils", board_thickness_mils),
        ("material_modifier", material_modifier),
        ("user_modifier", user_modifier),
    ])?;

    if width <= 0.0 {
        return Err(CalcError::NegativeDimension {
            name: "width",
            value: width,
        });
    }
    if thickness <= 0.0 {
        return Err(CalcError::NegativeDimension {
            name: "thickness",
            value: thickness,
        });
    }
    if length <= 0.0 {
        return Err(CalcError::NegativeDimension {
            name: "length",
            value: length,
        });
    }
    if temperature_rise <= 0.0 {
        return Err(CalcError::OutOfRange {
            name: "temperature_rise",
            value: temperature_rise,
            expected: "> 0",
        });
    }
    if frequency < 0.0 {
        return Err(CalcError::OutOfRange {
            name: "frequency",
            value: frequency,
            expected: ">= 0",
        });
    }
    if board_thickness_mils <= 0.0 {
        return Err(CalcError::NegativeDimension {
            name: "board_thickness_mils",
            value: board_thickness_mils,
        });
    }

    // Cross-sectional area (sq mils), accounting for etch factor
    let cross_section = etch_factor.cross_section_sq_mils(width, thickness);

    // IPC-2221A base current
    let k = if is_internal { K_INTERNAL } else { K_EXTERNAL };
    let i_base = k * temperature_rise.powf(0.44) * cross_section.powf(0.725);

    // IPC-2152 modifiers
    let m_area = m_area_lookup(cross_section);
    let m_temp = m_temp_lookup(temperature_rise);
    let m_board = m_board_lookup(board_thickness_mils, has_copper_plane);

    let current_capacity = i_base * m_area * m_temp * m_board * material_modifier * user_modifier;

    // Temperature-adjusted DC resistance.
    //
    // This previously read `rho_adj = (1 + 0.00393·(T − 20)) · rho_base_lookup(T)`, which
    // applied the temperature correction **twice**: `rho_base_lookup` is itself that same
    // linear law sampled every 20 °C (its 20 °C entry is 0.0006787, its 40 °C entry 0.000732
    // — a ratio of 1.0785 against the linear 1.0786, and likewise at every other step). The
    // product overestimated resistance by ~12% at 50 °C and ~25% at 80 °C, and introduced
    // step discontinuities at each table boundary. One linear model, continuous everywhere;
    // `rho_base_lookup` is deleted, since nothing else referenced it.
    let resistance_dc = RESISTIVITY_OHM_MIL * length / cross_section
        * crate::resistance::temperature_factor(ambient_temp)?;

    let voltage_drop = current_capacity * resistance_dc;
    let power_dissipation = current_capacity * voltage_drop;
    let current_density = current_capacity / cross_section;

    let skin_depth_mils = if frequency > 0.0 {
        let delta_m = (COPPER_RESISTIVITY_OHM_M / (std::f64::consts::PI * frequency * MU_0)).sqrt();
        delta_m / crate::constants::MIL_TO_M
    } else {
        0.0
    };

    Ok(Ipc2152Result {
        current_capacity,
        cross_section,
        resistance_dc,
        voltage_drop,
        power_dissipation,
        current_density,
        skin_depth_mils,
        m_area,
        m_temp,
        m_board,
    })
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    // Saturn test vector: 1MHz skin depth = 2.599 mil
    #[test]
    fn skin_depth_1mhz() {
        let result = calculate(&CurrentInput {
            width: 10.0,
            thickness: 1.4,
            length: 1000.0,
            temperature_rise: 10.0,
            ambient_temp: 25.0,
            frequency: 1_000_000.0,
            etch_factor: EtchFactor::None,
            is_internal: false,
        })
        .unwrap();

        assert_relative_eq!(result.skin_depth_mils, 2.599, max_relative = 0.005);
    }

    // IPC-2221A formula verification [docs/notes/17-test-vectors.md]:
    // External, dT=10°C, A=100 sq.mils → I = 0.048 × 10^0.44 × 100^0.725 = 3.73 A
    //
    // Note: Saturn PDF page 6/46 vectors use IPC-2152 mode (5.076A / 3.723A),
    // which requires table data not yet implemented.
    #[test]
    fn ipc2221a_external_a100() {
        // W=50, T=2.0, no etch → A = 100 sq mils
        let result = calculate(&CurrentInput {
            width: 50.0,
            thickness: 2.0,
            length: 1000.0,
            temperature_rise: 10.0,
            ambient_temp: 25.0,
            frequency: 0.0,
            etch_factor: EtchFactor::None,
            is_internal: false,
        })
        .unwrap();

        assert_relative_eq!(result.cross_section, 100.0, max_relative = 1e-10);
        assert_relative_eq!(result.current_capacity, 3.73, max_relative = 0.005);
    }

    // IPC-2221A internal: same area → I = 0.024 × 10^0.44 × 100^0.725 = 1.86 A
    #[test]
    fn ipc2221a_internal_a100() {
        let result = calculate(&CurrentInput {
            width: 50.0,
            thickness: 2.0,
            length: 1000.0,
            temperature_rise: 10.0,
            ambient_temp: 25.0,
            frequency: 0.0,
            etch_factor: EtchFactor::None,
            is_internal: true,
        })
        .unwrap();

        assert_relative_eq!(result.current_capacity, 1.86, max_relative = 0.005);
    }

    // Internal layers use k=0.024 → half the current of external
    #[test]
    fn internal_lower_than_external() {
        let ext = calculate(&CurrentInput {
            width: 10.0,
            thickness: 1.4,
            length: 1000.0,
            temperature_rise: 10.0,
            ambient_temp: 25.0,
            frequency: 0.0,
            etch_factor: EtchFactor::None,
            is_internal: false,
        })
        .unwrap();

        let int = calculate(&CurrentInput {
            width: 10.0,
            thickness: 1.4,
            length: 1000.0,
            temperature_rise: 10.0,
            ambient_temp: 25.0,
            frequency: 0.0,
            etch_factor: EtchFactor::None,
            is_internal: true,
        })
        .unwrap();

        // k_int / k_ext = 0.024 / 0.048 = 0.5
        assert_relative_eq!(
            int.current_capacity / ext.current_capacity,
            0.5,
            max_relative = 1e-10
        );
    }

    #[test]
    fn resistance_and_power() {
        let result = calculate(&CurrentInput {
            width: 10.0,
            thickness: 1.4,
            length: 1000.0,
            temperature_rise: 10.0,
            ambient_temp: 25.0,
            frequency: 0.0,
            etch_factor: EtchFactor::None,
            is_internal: false,
        })
        .unwrap();

        // R = 6.787e-4 × 1000 / 14 ≈ 0.04848 Ω at 20 °C, then corrected to the 25 °C
        // ambient this input asks for. The temperature factor used to be absent from this
        // path entirely, so the bare `ρL/A` was returned whatever the ambient; 25 °C is
        // 2.0% above the temperature the resistivity constant is quoted at.
        let expected_r = RESISTIVITY_OHM_MIL * 1000.0 / 14.0
            * crate::resistance::temperature_factor(25.0).unwrap();
        assert_relative_eq!(result.resistance_dc, expected_r, max_relative = 1e-10);

        // V = I × R
        assert_relative_eq!(
            result.voltage_drop,
            result.current_capacity * result.resistance_dc,
            max_relative = 1e-10
        );

        // P = I × V
        assert_relative_eq!(
            result.power_dissipation,
            result.current_capacity * result.voltage_drop,
            max_relative = 1e-10
        );
    }

    #[test]
    fn rejects_negative_width() {
        let result = calculate(&CurrentInput {
            width: -1.0,
            thickness: 1.4,
            length: 1000.0,
            temperature_rise: 10.0,
            ambient_temp: 25.0,
            frequency: 0.0,
            etch_factor: EtchFactor::None,
            is_internal: false,
        });
        assert!(result.is_err());
    }

    #[test]
    fn rejects_zero_temperature_rise() {
        let result = calculate(&CurrentInput {
            width: 10.0,
            thickness: 1.4,
            length: 1000.0,
            temperature_rise: 0.0,
            ambient_temp: 25.0,
            frequency: 0.0,
            etch_factor: EtchFactor::None,
            is_internal: false,
        });
        assert!(result.is_err());
    }

    #[test]
    fn m_temp_boundary_values() {
        assert_eq!(m_temp_lookup(10.0), 0.40);
        assert_eq!(m_temp_lookup(10.1), 0.48);
        assert_eq!(m_temp_lookup(80.0), 1.00);
        assert_eq!(m_temp_lookup(100.0), 1.20);
        assert_eq!(m_temp_lookup(101.0), 1.30);
    }

    #[test]
    fn m_board_no_plane() {
        assert_eq!(m_board_lookup(10.0, false), 1.59);
        assert_eq!(m_board_lookup(50.0, false), 1.45);
        assert_eq!(m_board_lookup(101.0, false), 1.20);
    }

    #[test]
    fn m_board_with_plane() {
        assert_eq!(m_board_lookup(10.0, true), 1.63);
        assert_eq!(m_board_lookup(50.0, true), 1.49);
        assert_eq!(m_board_lookup(101.0, true), 1.24);
    }

    #[test]
    fn m_area_segments() {
        let a20 = m_area_lookup(20.0);
        let a60 = m_area_lookup(60.0);
        let a100 = m_area_lookup(100.0);
        assert!(a20 > a60);
        assert!(a60 > a100);
        assert_relative_eq!(a100, 2.7877 * 100.0_f64.powf(-0.114), max_relative = 1e-6);
    }

    #[test]
    fn ipc2152_modifiers_applied() {
        let result = calculate_ipc2152(&Ipc2152Input {
            width: 50.0,
            thickness: 2.0,
            length: 1000.0,
            temperature_rise: 10.0,
            ambient_temp: 25.0,
            frequency: 0.0,
            etch_factor: EtchFactor::None,
            is_internal: false,
            board_thickness_mils: 62.0,
            has_copper_plane: false,
            material_modifier: 1.0,
            user_modifier: 1.0,
        })
        .unwrap();

        assert_relative_eq!(result.cross_section, 100.0, max_relative = 1e-10);
        let i_base = K_EXTERNAL * 10.0_f64.powf(0.44) * 100.0_f64.powf(0.725);
        assert!(
            result.current_capacity != i_base,
            "IPC-2152 should differ from IPC-2221A base"
        );
        assert!(result.m_area > 0.0);
        assert_relative_eq!(result.m_temp, 0.40, max_relative = 1e-10);
        assert_relative_eq!(result.m_board, 1.39, max_relative = 1e-10);
    }

    #[test]
    fn ipc2152_keeps_existing_tests_unaffected() {
        let result = calculate(&CurrentInput {
            width: 50.0,
            thickness: 2.0,
            length: 1000.0,
            temperature_rise: 10.0,
            ambient_temp: 25.0,
            frequency: 0.0,
            etch_factor: EtchFactor::None,
            is_internal: false,
        })
        .unwrap();
        assert_relative_eq!(result.current_capacity, 3.73, max_relative = 0.005);
    }
}
