//! Copper weight to thickness conversion tables.

use serde::{Deserialize, Serialize};

use crate::CalcError;

/// Standard copper weight options (oz/ft²).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CopperWeight {
    /// 0.25 oz/ft²
    Oz025,
    /// 0.5 oz/ft²
    Oz05,
    /// 1 oz/ft²
    Oz1,
    /// 1.5 oz/ft²
    Oz15,
    /// 2 oz/ft²
    Oz2,
    /// 2.5 oz/ft²
    Oz25,
    /// 3 oz/ft²
    Oz3,
    /// 4 oz/ft²
    Oz4,
    /// 5 oz/ft²
    Oz5,
}

impl CopperWeight {
    /// Copper thickness in mils.
    pub fn thickness_mils(self) -> f64 {
        match self {
            Self::Oz025 => 0.35,
            Self::Oz05 => 0.70,
            Self::Oz1 => 1.40,
            Self::Oz15 => 2.10,
            Self::Oz2 => 2.80,
            Self::Oz25 => 3.50,
            Self::Oz3 => 4.20,
            Self::Oz4 => 5.60,
            Self::Oz5 => 7.00,
        }
    }

    /// Copper thickness in mm.
    pub fn thickness_mm(self) -> f64 {
        match self {
            Self::Oz025 => 0.009,
            Self::Oz05 => 0.018,
            Self::Oz1 => 0.035,
            Self::Oz15 => 0.053,
            Self::Oz2 => 0.070,
            Self::Oz25 => 0.088,
            Self::Oz3 => 0.106,
            Self::Oz4 => 0.142,
            Self::Oz5 => 0.178,
        }
    }

    /// Parse from a string like "1oz", "0.5oz", "2.5oz".
    pub fn from_str_oz(s: &str) -> Result<Self, CalcError> {
        match s.trim().to_lowercase().trim_end_matches("oz").trim() {
            "0.25" => Ok(Self::Oz025),
            "0.5" => Ok(Self::Oz05),
            "1" => Ok(Self::Oz1),
            "1.5" => Ok(Self::Oz15),
            "2" => Ok(Self::Oz2),
            "2.5" => Ok(Self::Oz25),
            "3" => Ok(Self::Oz3),
            "4" => Ok(Self::Oz4),
            "5" => Ok(Self::Oz5),
            _ => Err(CalcError::UnknownVariant {
                kind: "copper weight",
                value: s.to_string(),
                expected: "0.25oz, 0.5oz, 1oz, 1.5oz, 2oz, 2.5oz, 3oz, 4oz, 5oz",
            }),
        }
    }
}

/// Plating thickness options.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PlatingThickness {
    Bare,
    Oz05,
    Oz1,
    Oz15,
    Oz2,
    Oz25,
    Oz3,
}

impl PlatingThickness {
    /// Plating thickness in mils.
    pub fn thickness_mils(self) -> f64 {
        match self {
            Self::Bare => 0.0,
            Self::Oz05 => 0.70,
            Self::Oz1 => 1.40,
            Self::Oz15 => 2.10,
            Self::Oz2 => 2.80,
            Self::Oz25 => 3.50,
            Self::Oz3 => 4.20,
        }
    }
}

/// Etch factor affecting conductor cross-section geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EtchFactor {
    /// Rectangular cross-section (no etch compensation).
    None,
    /// 1:1 etch — trapezoid with top = W - 2T.
    OneToOne,
    /// 2:1 etch — trapezoid with top = W - T.
    TwoToOne,
}

impl EtchFactor {
    /// Cross-sectional area in square mils given width W and thickness T (both in mils).
    pub fn cross_section_sq_mils(self, width: f64, thickness: f64) -> f64 {
        match self {
            Self::None => width * thickness,
            Self::OneToOne => (width + (width - 2.0 * thickness)) * thickness / 2.0,
            Self::TwoToOne => (width + (width - thickness)) * thickness / 2.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copper_weight_values() {
        assert!((CopperWeight::Oz1.thickness_mils() - 1.40).abs() < 1e-10);
        assert!((CopperWeight::Oz1.thickness_mm() - 0.035).abs() < 1e-10);
    }

    #[test]
    fn etch_factor_rectangular() {
        // 10 mil wide, 1.4 mil thick, no etch → 14 sq mils
        let area = EtchFactor::None.cross_section_sq_mils(10.0, 1.4);
        assert!((area - 14.0).abs() < 1e-10);
    }

    #[test]
    fn parse_copper_weight() {
        assert_eq!(CopperWeight::from_str_oz("1oz").unwrap(), CopperWeight::Oz1);
        assert_eq!(
            CopperWeight::from_str_oz("0.5oz").unwrap(),
            CopperWeight::Oz05
        );
        assert_eq!(
            CopperWeight::from_str_oz("2.5").unwrap(),
            CopperWeight::Oz25
        );
    }
}

// -- String parsing --------------------------------------------------------
//
// These live in the library so the CLI and the Tauri bridge parse identically. Each
// previously had a separate hand-written parser in each front end.

impl std::str::FromStr for CopperWeight {
    type Err = CalcError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_oz(s)
    }
}

impl std::str::FromStr for PlatingThickness {
    type Err = CalcError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().trim_end_matches("oz").trim() {
            "bare" | "0" | "none" => Ok(Self::Bare),
            "0.5" => Ok(Self::Oz05),
            "1" => Ok(Self::Oz1),
            "1.5" => Ok(Self::Oz15),
            "2" => Ok(Self::Oz2),
            "2.5" => Ok(Self::Oz25),
            "3" => Ok(Self::Oz3),
            _ => Err(CalcError::UnknownVariant {
                kind: "plating thickness",
                value: s.to_string(),
                expected: "bare, 0.5oz, 1oz, 1.5oz, 2oz, 2.5oz, 3oz",
            }),
        }
    }
}

impl std::str::FromStr for EtchFactor {
    type Err = CalcError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "none" | "0" => Ok(Self::None),
            "1:1" | "1to1" | "1" => Ok(Self::OneToOne),
            "2:1" | "2to1" | "2" => Ok(Self::TwoToOne),
            _ => Err(CalcError::UnknownVariant {
                kind: "etch factor",
                value: s.to_string(),
                expected: "none, 1:1, 2:1",
            }),
        }
    }
}

#[cfg(test)]
mod parse_tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn plating_accepts_the_forms_both_front_ends_used() {
        for (s, want) in [
            ("bare", PlatingThickness::Bare),
            ("BARE", PlatingThickness::Bare),
            ("0", PlatingThickness::Bare),
            ("0.5oz", PlatingThickness::Oz05),
            ("0.5", PlatingThickness::Oz05),
            (" 1oz ", PlatingThickness::Oz1),
            ("3oz", PlatingThickness::Oz3),
        ] {
            assert_eq!(
                PlatingThickness::from_str(s).unwrap(),
                want,
                "parsing {s:?}"
            );
        }
        assert!(PlatingThickness::from_str("4oz").is_err());
    }

    #[test]
    fn etch_accepts_colon_and_word_forms() {
        for (s, want) in [
            ("none", EtchFactor::None),
            ("1:1", EtchFactor::OneToOne),
            ("1to1", EtchFactor::OneToOne),
            ("2:1", EtchFactor::TwoToOne),
            ("2TO1", EtchFactor::TwoToOne),
        ] {
            assert_eq!(EtchFactor::from_str(s).unwrap(), want, "parsing {s:?}");
        }
        assert!(EtchFactor::from_str("3:1").is_err());
    }

    #[test]
    fn copper_weight_from_str_matches_from_str_oz() {
        assert_eq!(CopperWeight::from_str("1oz").unwrap(), CopperWeight::Oz1);
        assert_eq!(CopperWeight::from_str("2.5").unwrap(), CopperWeight::Oz25);
        assert!(CopperWeight::from_str("7oz").is_err());
    }

    /// The error must name the valid values, since the front ends no longer do.
    #[test]
    fn errors_list_the_alternatives() {
        let e = EtchFactor::from_str("nope").unwrap_err().to_string();
        assert!(
            e.contains("1:1") && e.contains("2:1"),
            "unhelpful message: {e}"
        );
    }
}
