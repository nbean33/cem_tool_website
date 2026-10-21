//! AWG wire gauge property lookup — Saturn PCB Toolkit Mode 8.
//!
//! Contains a 44-entry table covering AWG 4/0 (largest) through AWG 40 (smallest).
//! Diameter data is in inches; resistance data is in ohms per 1000 ft.
//! Area is computed in circular mils (diameter_mils²).

use serde::{Deserialize, Serialize};

use crate::CalcError;
use crate::all_finite;

/// AWG wire gauge sizes from 4/0 (largest) to 40 (smallest).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Awg {
    Awg4_0,
    Awg3_0,
    Awg2_0,
    Awg1_0,
    Awg1,
    Awg2,
    Awg3,
    Awg4,
    Awg5,
    Awg6,
    Awg7,
    Awg8,
    Awg9,
    Awg10,
    Awg11,
    Awg12,
    Awg13,
    Awg14,
    Awg15,
    Awg16,
    Awg17,
    Awg18,
    Awg19,
    Awg20,
    Awg21,
    Awg22,
    Awg23,
    Awg24,
    Awg25,
    Awg26,
    Awg27,
    Awg28,
    Awg29,
    Awg30,
    Awg31,
    Awg32,
    Awg33,
    Awg34,
    Awg35,
    Awg36,
    Awg37,
    Awg38,
    Awg39,
    Awg40,
}

/// Properties computed for a single AWG wire gauge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WireGaugeResult {
    /// Human-readable AWG label: "4/0", "3/0", ..., "40".
    pub awg_label: &'static str,
    /// Diameter in inches.
    pub diameter_in: f64,
    /// Diameter in mils (diameter_in × 1000).
    pub diameter_mils: f64,
    /// DC resistance in ohms per 1000 ft.
    pub resistance_ohm_per_kft: f64,
    /// Cross-sectional area in circular mils (diameter_mils²).
    pub area_circular_mils: f64,
    /// Current rating in amperes at 700 circular mils per ampere.
    ///
    /// This field was called `area_saturn` and documented as "Saturn display area:
    /// diameter_mils² / 700.0" — a quantity with no units and no stated meaning, which was
    /// enough to flag it in `VALIDATION.md` as unexplained public API. It is not an area.
    /// 700 circular mils per ampere is the long-standing conservative figure for **power
    /// transmission** wiring, and dividing the circular-mil area by it reproduces the
    /// standard table to the digit it is published at: AWG 10 → 15 A, 12 → 9.3 A,
    /// 14 → 5.9 A, 18 → 2.3 A, 22 → 0.92 A.
    ///
    /// This is the conservative column. Chassis wiring is conventionally rated far higher
    /// (AWG 12 at ~41 A) because it allows a much larger temperature rise; do not read this
    /// number as a limit for short runs in free air.
    pub ampacity_a: f64,
}

/// Circular mils of copper per ampere, the basis for [`WireGaugeResult::ampacity_a`].
///
/// The conservative power-transmission figure. Named rather than inlined so the one place it
/// is divided by reads as a rule and not as a fudge.
const CIRCULAR_MILS_PER_AMPERE: f64 = 700.0;

/// Static table entry: (label, diameter_in, resistance_ohm_per_kft).
struct Entry {
    label: &'static str,
    diameter_in: f64,
    resistance_ohm_per_kft: f64,
}

static TABLE: [Entry; 44] = [
    Entry {
        label: "4/0",
        diameter_in: 0.4600,
        resistance_ohm_per_kft: 0.050,
    },
    Entry {
        label: "3/0",
        diameter_in: 0.4096,
        resistance_ohm_per_kft: 0.060,
    },
    Entry {
        label: "2/0",
        diameter_in: 0.3648,
        resistance_ohm_per_kft: 0.080,
    },
    Entry {
        label: "1/0",
        diameter_in: 0.3249,
        resistance_ohm_per_kft: 0.100,
    },
    Entry {
        label: "1",
        diameter_in: 0.2893,
        resistance_ohm_per_kft: 0.120,
    },
    Entry {
        label: "2",
        diameter_in: 0.2576,
        resistance_ohm_per_kft: 0.160,
    },
    Entry {
        label: "3",
        diameter_in: 0.2294,
        resistance_ohm_per_kft: 0.200,
    },
    Entry {
        label: "4",
        diameter_in: 0.2043,
        resistance_ohm_per_kft: 0.250,
    },
    Entry {
        label: "5",
        diameter_in: 0.1819,
        resistance_ohm_per_kft: 0.310,
    },
    Entry {
        label: "6",
        diameter_in: 0.1620,
        resistance_ohm_per_kft: 0.400,
    },
    Entry {
        label: "7",
        diameter_in: 0.1443,
        resistance_ohm_per_kft: 0.500,
    },
    Entry {
        label: "8",
        diameter_in: 0.1285,
        resistance_ohm_per_kft: 0.630,
    },
    Entry {
        label: "9",
        diameter_in: 0.1144,
        resistance_ohm_per_kft: 0.790,
    },
    Entry {
        label: "10",
        diameter_in: 0.1019,
        resistance_ohm_per_kft: 1.000,
    },
    Entry {
        label: "11",
        diameter_in: 0.0907,
        resistance_ohm_per_kft: 1.260,
    },
    Entry {
        label: "12",
        diameter_in: 0.0808,
        resistance_ohm_per_kft: 1.590,
    },
    Entry {
        label: "13",
        diameter_in: 0.0720,
        resistance_ohm_per_kft: 2.000,
    },
    Entry {
        label: "14",
        diameter_in: 0.0641,
        resistance_ohm_per_kft: 2.530,
    },
    Entry {
        label: "15",
        diameter_in: 0.0571,
        resistance_ohm_per_kft: 3.190,
    },
    Entry {
        label: "16",
        diameter_in: 0.0508,
        resistance_ohm_per_kft: 4.020,
    },
    Entry {
        label: "17",
        diameter_in: 0.0453,
        resistance_ohm_per_kft: 5.060,
    },
    Entry {
        label: "18",
        diameter_in: 0.0403,
        resistance_ohm_per_kft: 6.390,
    },
    Entry {
        label: "19",
        diameter_in: 0.0359,
        resistance_ohm_per_kft: 8.050,
    },
    Entry {
        label: "20",
        diameter_in: 0.0320,
        resistance_ohm_per_kft: 10.150,
    },
    Entry {
        label: "21",
        diameter_in: 0.0285,
        resistance_ohm_per_kft: 12.800,
    },
    Entry {
        label: "22",
        diameter_in: 0.0254,
        resistance_ohm_per_kft: 16.140,
    },
    Entry {
        label: "23",
        diameter_in: 0.0226,
        resistance_ohm_per_kft: 20.360,
    },
    Entry {
        label: "24",
        diameter_in: 0.0201,
        resistance_ohm_per_kft: 25.670,
    },
    Entry {
        label: "25",
        diameter_in: 0.0179,
        resistance_ohm_per_kft: 32.370,
    },
    Entry {
        label: "26",
        diameter_in: 0.0159,
        resistance_ohm_per_kft: 40.810,
    },
    Entry {
        label: "27",
        diameter_in: 0.0142,
        resistance_ohm_per_kft: 51.470,
    },
    Entry {
        label: "28",
        diameter_in: 0.0126,
        resistance_ohm_per_kft: 64.900,
    },
    Entry {
        label: "29",
        diameter_in: 0.0113,
        resistance_ohm_per_kft: 81.830,
    },
    Entry {
        label: "30",
        diameter_in: 0.0100,
        resistance_ohm_per_kft: 103.200,
    },
    Entry {
        label: "31",
        diameter_in: 0.0089,
        resistance_ohm_per_kft: 130.100,
    },
    Entry {
        label: "32",
        diameter_in: 0.0080,
        resistance_ohm_per_kft: 164.100,
    },
    Entry {
        label: "33",
        diameter_in: 0.0071,
        resistance_ohm_per_kft: 206.900,
    },
    Entry {
        label: "34",
        diameter_in: 0.0063,
        resistance_ohm_per_kft: 260.900,
    },
    Entry {
        label: "35",
        diameter_in: 0.0056,
        resistance_ohm_per_kft: 329.000,
    },
    Entry {
        label: "36",
        diameter_in: 0.0050,
        resistance_ohm_per_kft: 414.800,
    },
    Entry {
        label: "37",
        diameter_in: 0.0045,
        resistance_ohm_per_kft: 523.100,
    },
    Entry {
        label: "38",
        diameter_in: 0.0040,
        resistance_ohm_per_kft: 659.600,
    },
    Entry {
        label: "39",
        diameter_in: 0.0035,
        resistance_ohm_per_kft: 831.800,
    },
    Entry {
        label: "40",
        diameter_in: 0.0031,
        resistance_ohm_per_kft: 1049.000,
    },
];

static ALL_AWG: [Awg; 44] = [
    Awg::Awg4_0,
    Awg::Awg3_0,
    Awg::Awg2_0,
    Awg::Awg1_0,
    Awg::Awg1,
    Awg::Awg2,
    Awg::Awg3,
    Awg::Awg4,
    Awg::Awg5,
    Awg::Awg6,
    Awg::Awg7,
    Awg::Awg8,
    Awg::Awg9,
    Awg::Awg10,
    Awg::Awg11,
    Awg::Awg12,
    Awg::Awg13,
    Awg::Awg14,
    Awg::Awg15,
    Awg::Awg16,
    Awg::Awg17,
    Awg::Awg18,
    Awg::Awg19,
    Awg::Awg20,
    Awg::Awg21,
    Awg::Awg22,
    Awg::Awg23,
    Awg::Awg24,
    Awg::Awg25,
    Awg::Awg26,
    Awg::Awg27,
    Awg::Awg28,
    Awg::Awg29,
    Awg::Awg30,
    Awg::Awg31,
    Awg::Awg32,
    Awg::Awg33,
    Awg::Awg34,
    Awg::Awg35,
    Awg::Awg36,
    Awg::Awg37,
    Awg::Awg38,
    Awg::Awg39,
    Awg::Awg40,
];

impl Awg {
    /// Returns the table index for this gauge (0 = 4/0, 43 = 40).
    fn index(self) -> usize {
        self as usize
    }

    /// Returns all 44 AWG variants in order from largest (4/0) to smallest (40).
    pub fn all() -> &'static [Awg] {
        &ALL_AWG
    }

    /// Returns the `Awg` variant for the given ComboBox-style index, or `None` if out of range.
    pub fn from_index(idx: usize) -> Option<Awg> {
        ALL_AWG.get(idx).copied()
    }
}

/// Look up wire gauge properties for the given AWG size.
///
/// All data originates from the Saturn PCB Toolkit Mode 8 binary table.
/// This function is infallible because all inputs are compile-time enum variants.
pub fn lookup(awg: Awg) -> WireGaugeResult {
    let entry = &TABLE[awg.index()];
    let diameter_mils = entry.diameter_in * 1000.0;
    let area_circular_mils = diameter_mils * diameter_mils;
    WireGaugeResult {
        awg_label: entry.label,
        diameter_in: entry.diameter_in,
        diameter_mils,
        resistance_ohm_per_kft: entry.resistance_ohm_per_kft,
        area_circular_mils,
        ampacity_a: area_circular_mils / CIRCULAR_MILS_PER_AMPERE,
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    // AWG 4/0: diameter=0.4600 in, resistance=0.050 ohm/kft
    // diameter_mils = 460.0, area_circular_mils = 211600.0
    // ampacity = 211600 / 700 = 302.285714...
    #[test]
    fn awg_4_0_properties() {
        let r = lookup(Awg::Awg4_0);
        assert_eq!(r.awg_label, "4/0");
        assert_relative_eq!(r.diameter_in, 0.4600, epsilon = 1e-10);
        assert_relative_eq!(r.resistance_ohm_per_kft, 0.050, epsilon = 1e-10);
        assert_relative_eq!(r.diameter_mils, 460.0, epsilon = 1e-10);
        assert_relative_eq!(r.area_circular_mils, 211_600.0, epsilon = 1e-6);
        assert_relative_eq!(r.ampacity_a, 302.285_714_285, epsilon = 1e-6);
    }

    /// `ampacity_a` must reproduce the published power-transmission current table.
    ///
    /// This is the check that identifies the quantity. The field used to be called
    /// `area_saturn` and carried no units, no basis and no test beyond one echo of its own
    /// arithmetic — which is why `VALIDATION.md` listed it as unexplained public API. The
    /// table is published to two significant figures, so each row is asserted to within half
    /// of its own last printed digit rather than a blanket relative tolerance: AWG 10 is
    /// 14.83 A before rounding to the published 15, which a 0.5% tolerance would reject for
    /// the wrong reason. Agreeing with every row across four decades of area is not something
    /// the 700 figure could do by coincidence.
    #[test]
    fn ampacity_matches_the_published_power_transmission_table() {
        // (gauge, published amps, half of the last published digit)
        for (awg, published_a, half_digit) in [
            (Awg::Awg10, 15.0, 0.5),
            (Awg::Awg12, 9.3, 0.05),
            (Awg::Awg14, 5.9, 0.05),
            (Awg::Awg16, 3.7, 0.05),
            (Awg::Awg18, 2.3, 0.05),
            (Awg::Awg22, 0.92, 0.005),
        ] {
            let r = lookup(awg);
            assert!(
                (r.ampacity_a - published_a).abs() <= half_digit,
                "AWG {}: {} is not {published_a} +/- {half_digit}",
                r.awg_label,
                r.ampacity_a
            );
        }
    }

    // AWG 22: diameter=0.0254 in, resistance=16.140 ohm/kft
    #[test]
    fn awg_22_properties() {
        let r = lookup(Awg::Awg22);
        assert_eq!(r.awg_label, "22");
        assert_relative_eq!(r.diameter_in, 0.0254, epsilon = 1e-10);
        assert_relative_eq!(r.resistance_ohm_per_kft, 16.140, epsilon = 1e-10);
        assert_relative_eq!(r.diameter_mils, 25.4, epsilon = 1e-8);
        assert_relative_eq!(r.area_circular_mils, 645.16, epsilon = 1e-6);
    }

    // AWG 40: diameter=0.0031 in, resistance=1049.000 ohm/kft
    #[test]
    fn awg_40_properties() {
        let r = lookup(Awg::Awg40);
        assert_eq!(r.awg_label, "40");
        assert_relative_eq!(r.diameter_in, 0.0031, epsilon = 1e-10);
        assert_relative_eq!(r.resistance_ohm_per_kft, 1049.000, epsilon = 1e-10);
        assert_relative_eq!(r.diameter_mils, 3.1, epsilon = 1e-10);
        assert_relative_eq!(r.area_circular_mils, 9.61, epsilon = 1e-8);
    }

    #[test]
    fn all_has_44_entries() {
        assert_eq!(Awg::all().len(), 44);
    }

    #[test]
    fn from_index_boundary() {
        assert_eq!(Awg::from_index(0), Some(Awg::Awg4_0));
        assert_eq!(Awg::from_index(43), Some(Awg::Awg40));
        assert_eq!(Awg::from_index(44), None);
    }
}

impl std::str::FromStr for Awg {
    type Err = CalcError;

    /// Table-driven: matches the gauge's own label, so it cannot drift from the table.
    ///
    /// Accepts an optional `awg` prefix and the all-zeros spellings of the 0-gauges
    /// (`0000` = `4/0`). The CLI previously carried a 53-arm hand-written match and the Tauri
    /// bridge a separate label lookup; the two disagreed about the zero aliases.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lowered = s.trim().to_ascii_lowercase();
        let k = lowered.strip_prefix("awg").unwrap_or(&lowered).trim();
        let k = match k {
            "0000" => "4/0",
            "000" => "3/0",
            "00" => "2/0",
            "0" => "1/0",
            other => other,
        };
        Self::all()
            .iter()
            .copied()
            .find(|a| lookup(*a).awg_label.eq_ignore_ascii_case(k))
            .ok_or_else(|| CalcError::UnknownVariant {
                kind: "AWG gauge",
                value: s.to_string(),
                expected: "4/0 through 40",
            })
    }
}

#[cfg(test)]
mod parse_tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn every_gauge_round_trips_through_its_label() {
        for a in Awg::all() {
            let label = lookup(*a).awg_label;
            assert_eq!(Awg::from_str(label).unwrap(), *a, "label {label}");
        }
    }

    #[test]
    fn accepts_prefix_and_zero_aliases() {
        assert_eq!(Awg::from_str("awg12").unwrap(), Awg::Awg12);
        assert_eq!(Awg::from_str("AWG 12").unwrap(), Awg::Awg12);
        assert_eq!(Awg::from_str("0000").unwrap(), Awg::Awg4_0);
        assert_eq!(Awg::from_str("4/0").unwrap(), Awg::Awg4_0);
        assert_eq!(Awg::from_str("0").unwrap(), Awg::Awg1_0);
        assert!(Awg::from_str("41").is_err());
    }
}

// ---------------------------------------------------------------------------------------
// Reverse lookup: diameter → gauge
// ---------------------------------------------------------------------------------------

/// Result of a reverse AWG lookup.
///
/// `Serialize` only, unlike the other result types in this crate: `awg_label` is a
/// `&'static str` and a derived `Deserialize` cannot promise that lifetime. Results are
/// written out, never read back, so nothing needs it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReverseAwgResult {
    /// The measured diameter that was looked up (mils).
    pub diameter_mils: f64,
    /// Nearest standard gauge, by absolute difference in diameter. Pass it to [`lookup`]
    /// for the full record; the few fields worth showing are flattened out below, because
    /// the desktop app's result renderer reads top-level keys only.
    pub nearest: Awg,
    /// Label of the nearest gauge: "4/0", "3/0", …, "40".
    pub awg_label: &'static str,
    /// Tabulated diameter of that gauge (mils).
    pub standard_diameter_mils: f64,
    /// DC resistance of that gauge (Ω per 1000 ft).
    pub resistance_ohm_per_kft: f64,
    /// Current rating of that gauge at 700 circular mils per ampere (A).
    pub ampacity_a: f64,
    /// Difference between the measured diameter and the nearest gauge's, as a percentage.
    /// Positive means the measured wire is thicker than the standard it matched.
    pub difference_percent: f64,
    /// The continuous gauge number implied by the defining law, `AWG = −10 − 20·log₁₀(d_in)`.
    ///
    /// Non-integer, and the honest answer when a wire is between gauges. The table this
    /// crate carries holds the *standard* diameters, which depart from the ideal law by up
    /// to 2% (AWG 10 is 101.9 mil where the law gives 100.0), so this figure and
    /// [`Self::nearest`] can disagree by a fraction of a gauge. That is a property of the
    /// AWG standard, not of this calculation.
    pub awg_continuous: f64,
}

/// Nearest standard AWG gauge for a measured wire diameter.
///
/// The counterpart to [`lookup`], which only went forwards. Useful for the thing you
/// actually do at a bench — put a caliper on an unlabelled wire and ask what it is, then
/// read its current rating and resistance off the result.
///
/// Matching is by nearest tabulated diameter rather than by rounding
/// [`ReverseAwgResult::awg_continuous`], so the answer is always a gauge this crate can
/// describe, and `difference_percent` tells you how good the match is. Anything much beyond
/// a couple of percent means the wire is between gauges, stranded (whose overall diameter
/// exceeds the equivalent solid gauge), or not AWG at all.
///
/// # Errors
/// Returns [`CalcError::NegativeDimension`] if the diameter is not positive, and
/// [`CalcError::OutOfRange`] if it falls outside the 4/0–40 table by more than a factor of
/// two — at that point "nearest" would be meaningless.
pub fn from_diameter(diameter_mils: f64) -> Result<ReverseAwgResult, CalcError> {
    all_finite(&[("diameter_mils", diameter_mils)])?;
    if diameter_mils <= 0.0 {
        return Err(CalcError::NegativeDimension {
            name: "diameter_mils",
            value: diameter_mils,
        });
    }

    let largest = lookup(Awg::all()[0]).diameter_mils;
    let smallest = lookup(*Awg::all().last().expect("table is non-empty")).diameter_mils;
    if diameter_mils > largest * 2.0 || diameter_mils < smallest / 2.0 {
        return Err(CalcError::OutOfRange {
            name: "diameter_mils",
            value: diameter_mils,
            expected: "within a factor of two of the 4/0 (460 mil) to 40 (3.1 mil) table",
        });
    }

    let nearest = *Awg::all()
        .iter()
        .min_by(|a, b| {
            let da = (lookup(**a).diameter_mils - diameter_mils).abs();
            let db = (lookup(**b).diameter_mils - diameter_mils).abs();
            da.total_cmp(&db)
        })
        .expect("table is non-empty");

    let props = lookup(nearest);
    Ok(ReverseAwgResult {
        diameter_mils,
        nearest,
        awg_label: props.awg_label,
        standard_diameter_mils: props.diameter_mils,
        resistance_ohm_per_kft: props.resistance_ohm_per_kft,
        ampacity_a: props.ampacity_a,
        difference_percent: (diameter_mils - props.diameter_mils) / props.diameter_mils * 100.0,
        awg_continuous: -10.0 - 20.0 * (diameter_mils / 1000.0).log10(),
    })
}

#[cfg(test)]
mod reverse_tests {
    use approx::assert_relative_eq;

    use super::*;

    /// Every tabulated gauge must round-trip exactly: look up its diameter, and the reverse
    /// lookup must name the same gauge with a zero error.
    ///
    /// This is the strong form of the test — it is 44 assertions, and it is the one that
    /// would catch a nearest-neighbour search that is off by one anywhere in the table.
    #[test]
    fn every_gauge_round_trips() {
        for awg in Awg::all() {
            let d = lookup(*awg).diameter_mils;
            let back = from_diameter(d).unwrap();
            assert_eq!(back.nearest, *awg, "{} mil matched the wrong gauge", d);
            assert_relative_eq!(back.difference_percent, 0.0, epsilon = 1e-9);
        }
    }

    /// 10 mil is AWG 30 both ways — the one gauge where the standard diameter and the
    /// defining law agree exactly, since `10^(-(30+10)/20) = 0.010` in.
    #[test]
    fn ten_mils_is_awg_30() {
        let r = from_diameter(10.0).unwrap();
        assert_eq!(r.awg_label, "30");
        assert_relative_eq!(r.awg_continuous, 30.0, max_relative = 1e-12);
        assert_relative_eq!(r.difference_percent, 0.0, epsilon = 1e-9);
    }

    /// The continuous law and the standard table are not the same thing, and the result
    /// reports both rather than pretending otherwise. AWG 10 is tabulated at 101.9 mil where
    /// the law would put it at 100.0, so the continuous figure comes out just under 10.
    #[test]
    fn the_continuous_law_and_the_table_disagree_slightly() {
        let r = from_diameter(101.9).unwrap();
        assert_eq!(r.awg_label, "10");
        assert_relative_eq!(r.difference_percent, 0.0, epsilon = 1e-9);
        assert!(
            r.awg_continuous > 9.8 && r.awg_continuous < 10.0,
            "continuous gauge was {}",
            r.awg_continuous
        );
    }

    /// A wire between gauges must report which way it leans.
    #[test]
    fn an_off_size_wire_reports_its_error() {
        // 27 mil sits between AWG 21 (28.5) and 22 (25.4), nearer 21.
        let r = from_diameter(27.0).unwrap();
        assert_eq!(r.awg_label, "21");
        assert!(
            r.difference_percent < 0.0,
            "27 mil is thinner than AWG 21, got {}",
            r.difference_percent
        );
        assert!(r.difference_percent.abs() < 10.0);
    }

    /// The continuous law must be monotonically decreasing in diameter — a thicker wire is
    /// always a lower gauge number. Easy to get backwards, since the sign is inverted.
    #[test]
    fn thicker_wire_means_a_lower_gauge_number() {
        let thin = from_diameter(5.0).unwrap();
        let thick = from_diameter(200.0).unwrap();
        assert!(thin.awg_continuous > thick.awg_continuous);
        assert!(
            lookup(thin.nearest).diameter_mils < lookup(thick.nearest).diameter_mils,
            "nearest gauges are ordered backwards"
        );
    }

    /// The reverse lookup carries the gauge's electrical properties, so a caliper reading
    /// gives you resistance and current rating in one call — and they must be the same
    /// numbers [`lookup`] would return for that gauge.
    #[test]
    fn the_result_carries_the_gauge_properties() {
        let r = from_diameter(25.4).unwrap();
        assert_eq!(r.awg_label, "22");
        assert_relative_eq!(r.resistance_ohm_per_kft, 16.140, max_relative = 1e-9);
        assert_relative_eq!(r.ampacity_a, 0.92, max_relative = 5e-3);

        let forward = lookup(r.nearest);
        assert_eq!(r.awg_label, forward.awg_label);
        assert_relative_eq!(
            r.standard_diameter_mils,
            forward.diameter_mils,
            max_relative = 1e-12
        );
        assert_relative_eq!(
            r.resistance_ohm_per_kft,
            forward.resistance_ohm_per_kft,
            max_relative = 1e-12
        );
        assert_relative_eq!(r.ampacity_a, forward.ampacity_a, max_relative = 1e-12);
    }

    #[test]
    fn out_of_range_diameters_are_rejected() {
        assert!(from_diameter(0.0).is_err());
        assert!(from_diameter(-5.0).is_err());
        assert!(from_diameter(5000.0).is_err());
        assert!(from_diameter(0.1).is_err());
        // Just inside the factor-of-two band is still accepted.
        assert!(from_diameter(900.0).is_ok());
        assert!(from_diameter(2.0).is_ok());
    }
}
