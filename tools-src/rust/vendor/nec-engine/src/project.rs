//! NEC Studio's own project file: a NEC deck plus what a deck cannot say.
//!
//! The model itself stays a NEC deck, stored verbatim — its cards, `SY` variables and comments
//! — so importing a `.nec` file and exporting it again gives back what was read (with `\n`
//! line endings), and every deck the engine accepts is a valid project. Around it the project keeps the working
//! settings that a deck has no cards for: the frequency being looked at, the reference
//! impedance, the sweep, the optimizer's variables and goals, and the display units.
//!
//! The file is TOML:
//!
//! ```toml
//! format = 1
//! name = "3-element Yagi"
//! deck = """
//! CE
//! SY h=15
//! GW 1 21 -3.2 -5.31 h -3.2 5.31 h 0.006
//! ...
//! EN
//! """
//!
//! [settings]
//! frequency_mhz = 14.15
//! z0 = 50.0
//! units = "metres"
//!
//! [settings.sweep]
//! start_mhz = 14.0
//! stop_mhz = 14.35
//! steps = 41
//! pattern = true
//! ```

use crate::optim::Method;
use crate::{Direction, Goal, Parameter};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// The project format this build writes; it reads this and every earlier one.
pub const FORMAT: u32 = 1;

/// A model and its working settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    /// The file format's version.
    pub format: u32,
    /// A short name; the file name when imported.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// Free-form notes.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub notes: String,
    /// The NEC deck, verbatim.
    pub deck: String,
    /// Everything else.
    #[serde(default)]
    pub settings: Settings,
}

/// The settings a deck has no cards for.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// The frequency being looked at, MHz; the deck's first when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_mhz: Option<f64>,
    /// Reference impedance for SWR, ohms.
    pub z0: f64,
    /// Length units for display and editing. The deck itself is always in metres (after any
    /// `GS` scaling); these only change what the editor shows.
    pub units: Units,
    /// The frequency sweep.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sweep: Option<SweepSettings>,
    /// The optimizer's set-up.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimizer: Option<OptimizerSettings>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            frequency_mhz: None,
            z0: 50.0,
            units: Units::Metres,
            sweep: None,
            optimizer: None,
        }
    }
}

/// Length units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Units {
    /// Metres, NEC's own.
    #[default]
    Metres,
    /// Feet.
    Feet,
    /// Inches.
    Inches,
}

impl Units {
    /// Metres in one of this unit.
    #[must_use]
    pub fn metres(self) -> f64 {
        match self {
            Self::Metres => 1.0,
            Self::Feet => 0.3048,
            Self::Inches => 0.0254,
        }
    }
}

/// A frequency sweep.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SweepSettings {
    /// First frequency, MHz.
    pub start_mhz: f64,
    /// Last frequency, MHz.
    pub stop_mhz: f64,
    /// Number of frequencies.
    pub steps: usize,
    /// Whether each point also computes gain and front-to-back.
    #[serde(default)]
    pub pattern: bool,
}

/// The optimizer's set-up, without the reference impedance (that is [`Settings::z0`]).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptimizerSettings {
    /// The variables to tune and their ranges.
    #[serde(default)]
    pub parameters: Vec<Parameter>,
    /// Frequencies at which the goals are judged, MHz; empty takes the sweep's band.
    #[serde(default)]
    pub frequencies_mhz: Vec<f64>,
    /// What to achieve.
    #[serde(default)]
    pub goals: Vec<Goal>,
    /// Forward, for gain and front-to-back goals.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<Direction>,
    /// Which minimizer.
    pub method: Method,
    /// Evaluation budget.
    pub max_evaluations: usize,
}

/// Why a project file could not be read or written.
#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    /// Not valid TOML, or not a project.
    #[error("not a NEC Studio project: {0}")]
    Parse(#[from] toml::de::Error),
    /// Written by a newer build.
    #[error("project format {0} is newer than this program reads ({FORMAT})")]
    TooNew(u32),
    /// Could not be serialized (a non-finite number, for example).
    #[error("could not write the project: {0}")]
    Write(#[from] toml::ser::Error),
}

impl Project {
    /// A project around a NEC deck, with default settings; `name` is usually the file's stem.
    /// Line endings become `\n`, so the deck reads as plain text in the file; nothing else
    /// changes.
    #[must_use]
    pub fn from_deck(deck: &str, name: &str) -> Self {
        Self {
            format: FORMAT,
            name: name.to_string(),
            notes: String::new(),
            deck: deck.replace("\r\n", "\n"),
            settings: Settings::default(),
        }
    }

    /// Reads a project file's text.
    ///
    /// # Errors
    /// The text is not a project, or a newer format than this build reads.
    pub fn from_toml(text: &str) -> Result<Self, ProjectError> {
        let p: Self = toml::from_str(text)?;
        if p.format > FORMAT {
            return Err(ProjectError::TooNew(p.format));
        }
        Ok(p)
    }

    /// The project as TOML text.
    ///
    /// # Errors
    /// A setting cannot be represented (TOML has no NaN-free guarantee for every float).
    pub fn to_toml(&self) -> Result<String, ProjectError> {
        // Written in three pieces so the deck can be a literal string — tabs and quotes as
        // typed, which a basic string would escape — whenever its text allows one.
        #[derive(Serialize)]
        struct Head<'a> {
            format: u32,
            #[serde(skip_serializing_if = "str::is_empty")]
            name: &'a str,
            #[serde(skip_serializing_if = "str::is_empty")]
            notes: &'a str,
        }
        #[derive(Serialize)]
        struct Deck<'a> {
            deck: &'a str,
        }
        #[derive(Serialize)]
        struct Tail<'a> {
            settings: &'a Settings,
        }
        let mut out = toml::to_string_pretty(&Head {
            format: self.format,
            name: &self.name,
            notes: &self.notes,
        })?;
        let literal = !self.deck.contains("'''")
            && !self.deck.ends_with('\'')
            && self
                .deck
                .chars()
                .all(|c| c == '\t' || c == '\n' || !c.is_control());
        if literal {
            out.push_str("deck = '''\n");
            out.push_str(&self.deck);
            out.push_str("'''\n");
        } else {
            out.push_str(&toml::to_string_pretty(&Deck { deck: &self.deck })?);
        }
        out.push('\n');
        out.push_str(&toml::to_string_pretty(&Tail {
            settings: &self.settings,
        })?);
        Ok(out)
    }

    /// The deck to export as a `.nec` file — exactly the deck the project holds.
    #[must_use]
    pub fn to_nec(&self) -> &str {
        &self.deck
    }

    /// Whether a path names a project file rather than a NEC deck: `.toml`, conventionally
    /// `name.nec.toml`.
    #[must_use]
    pub fn is_project_path(path: &Path) -> bool {
        path.extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("toml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GoalKind;

    const DECK: &str = "CM a dipole, with 'quotes' and \"\"\" in a comment\nCE\nSY len=.2418\n\
                        GW 1 9 0 -len 0 0 len 0 .0001\nGE 0\nEX 0 1 5 0 1 0\nFR 0 1 0 0 300 0\nEN\n";

    #[test]
    fn a_deck_survives_the_round_trip_exactly() {
        let mut p = Project::from_deck(DECK, "dipole");
        p.settings.frequency_mhz = Some(300.0);
        p.settings.units = Units::Feet;
        p.settings.sweep = Some(SweepSettings {
            start_mhz: 280.0,
            stop_mhz: 320.0,
            steps: 41,
            pattern: true,
        });
        p.settings.optimizer = Some(OptimizerSettings {
            parameters: vec![Parameter {
                name: "len".into(),
                min: 0.2,
                max: 0.3,
            }],
            frequencies_mhz: vec![295.0, 305.0],
            goals: vec![
                Goal {
                    kind: GoalKind::Swr,
                    target: Some(1.5),
                    weight: 1.0,
                },
                Goal {
                    kind: GoalKind::Gain,
                    target: None,
                    weight: 0.5,
                },
            ],
            direction: Some(Direction {
                azimuth: 90.0,
                elevation: None,
            }),
            method: Method::NelderMead,
            max_evaluations: 600,
        });
        let text = p.to_toml().expect("writes");
        let back = Project::from_toml(&text).expect("reads");
        assert_eq!(back, p);
        assert_eq!(back.to_nec(), DECK);
    }

    #[test]
    fn a_deck_no_literal_string_can_hold_still_round_trips() {
        for deck in ["CM '''\nEN\n", "CE\nEN '", "CM bell \u{7}\nEN\n"] {
            let p = Project::from_deck(deck, "");
            let back = Project::from_toml(&p.to_toml().expect("writes")).expect("reads");
            assert_eq!(back.deck, deck);
        }
    }

    #[test]
    fn a_minimal_file_takes_the_defaults() {
        let p = Project::from_toml("format = 1\ndeck = \"CE\\nEN\\n\"\n").expect("reads");
        assert_eq!(p.settings, Settings::default());
        assert!(p.name.is_empty());
    }

    #[test]
    fn a_newer_format_is_refused() {
        assert!(matches!(
            Project::from_toml("format = 99\ndeck = \"\"\n"),
            Err(ProjectError::TooNew(99))
        ));
    }

    #[test]
    fn project_paths_are_toml() {
        assert!(Project::is_project_path(Path::new("yagi.nec.toml")));
        assert!(!Project::is_project_path(Path::new("yagi.nec")));
    }
}
