//! Parsing NEC's `.out` report.
//!
//! The engine prints what it computed, which makes it usable as an oracle. This module reads
//! the tables that matter for validation, starting with `SEGMENTATION DATA` — enough to check
//! geometry before any electromagnetics exists.
//!
//! The format is fixed-width Fortran output, and **whitespace splitting is not safe**: when a
//! negative value fills its field, adjacent columns run together with no separator. Real row
//! from a corpus model, where the ALPHA and BETA columns collide:
//!
//! ```text
//!    188  14.81818  -0.18182   0.01000   0.51426    0.00000-135.00000   0.00103  -199  188  189     18
//! ```
//!
//! So rows are read with a numeric scanner that starts a new token at a sign character,
//! while keeping signs that belong to an exponent attached. An earlier whitespace-splitting
//! version silently truncated that model's table at row 187 of 264.

/// One row of the engine's `SEGMENTATION DATA` table.
///
/// ```text
///   SEG.   COORDINATES OF SEG. CENTER     SEG.     ORIENTATION ANGLES    WIRE    CONNECTION DATA   TAG
///   NO.       X         Y         Z       LENGTH     ALPHA     BETA      RADIUS    I-   I    I+    NO.
///      1   0.00000  -0.21493   0.00000   0.05373    0.00000  90.00000   0.00010     0    1    2      1
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentRow {
    /// Segment number.
    pub index: usize,
    /// Centre X.
    pub x: f64,
    /// Centre Y.
    pub y: f64,
    /// Centre Z.
    pub z: f64,
    /// Segment length.
    pub length: f64,
    /// Elevation angle, degrees.
    pub alpha: f64,
    /// Azimuth angle, degrees.
    pub beta: f64,
    /// Wire radius.
    pub radius: f64,
    /// Connected segment at end 1, 0 if free (NEC's `I-`).
    pub i_minus: i64,
    /// Connected segment at end 2, 0 if free (NEC's `I+`).
    pub i_plus: i64,
    /// Wire tag.
    pub tag: i32,
}

/// One row of the engine's `SURFACE PATCH DATA` table, in metres,
/// `FORMAT (1X,I4,3F10.5,1X,3F8.4,F10.5,1X,3F8.4,1X,3F8.4)`.
///
/// ```text
///  PATCH     COORD. OF PATCH CENTER       UNIT NORMAL VECTOR      PATCH            COMPONENTS OF UNIT TANGENT VECTORS
///   NO.      X         Y         Z         X       Y       Z       AREA       X1      Y1      Z1       X2      Y2      Z2
///     1   0.10000   0.05000   0.05000   1.0000  0.0000  0.0000   0.01000   0.0000  1.0000  0.0000   0.0000  0.0000  1.0000
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PatchRow {
    /// 1-based patch number.
    pub index: usize,
    /// Centre.
    pub center: [f64; 3],
    /// Outward unit normal.
    pub normal: [f64; 3],
    /// Area.
    pub area: f64,
    /// First unit tangent.
    pub t1: [f64; 3],
    /// Second unit tangent.
    pub t2: [f64; 3],
}

fn parse_patch_row(line: &str) -> Option<PatchRow> {
    let index: usize = line.get(0..5)?.trim().parse().ok()?;
    let num = |lo: usize, hi: usize| -> Option<f64> { real(line.get(lo..hi.min(line.len()))?.trim()) };
    let three = |lo: usize, w: usize| -> Option<[f64; 3]> { Some([num(lo, lo + w)?, num(lo + w, lo + 2 * w)?, num(lo + 2 * w, lo + 3 * w)?]) };
    Some(PatchRow {
        index,
        center: three(5, 10)?,
        normal: three(36, 8)?,
        area: num(60, 70)?,
        t1: three(71, 8)?,
        t2: three(96, 8)?,
    })
}

/// One row of the engine's "ISOLATION DATA" (maximum coupling, `CP`),
/// `FORMAT (2(1X,I4,1X,I4,1X,I5,2X),F9.3,2X,1P,2(2X,E12.5,1X,E12.5))`: the two segments, the
/// maximum coupling in dB, the load impedance on the second and the input impedance at the
/// first. A pair whose coupling factor falls outside `[0, 1]` is printed as an error, with no
/// figures (`None`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CouplingRow {
    /// First segment number.
    pub first: usize,
    /// Second segment number.
    pub second: usize,
    /// The printed figures.
    pub figures: Option<CouplingFigures>,
}

/// The figures of one isolation row.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CouplingFigures {
    /// Maximum coupling, dB.
    pub coupling_db: f64,
    /// Matched load on the second segment, ohms (re, im).
    pub load: (f64, f64),
    /// Input impedance at the first segment with that load, ohms (re, im).
    pub input: (f64, f64),
}

fn parse_coupling_row(line: &str) -> Option<CouplingRow> {
    let t: Vec<&str> = line.split_whitespace().collect();
    let first: usize = t.get(2)?.parse().ok()?;
    let second: usize = t.get(5)?.parse().ok()?;
    t.get(..2)?.iter().chain(t.get(3..5)?).try_for_each(|v| v.parse::<i64>().ok().map(drop))?;
    if line.contains("ERROR") {
        return Some(CouplingRow { first, second, figures: None });
    }
    let v: Vec<f64> = t.get(6..11)?.iter().map(|x| real(x)).collect::<Option<_>>()?;
    Some(CouplingRow {
        first,
        second,
        figures: Some(CouplingFigures {
            coupling_db: v[0],
            load: (v[1], v[2]),
            input: (v[3], v[4]),
        }),
    })
}

/// What a parsed `.out` file told us.
#[derive(Debug, Clone, Default)]
pub struct OutReport {
    /// Rows of the segmentation table.
    pub segments: Vec<SegmentRow>,
    /// Rows of the surface patch table.
    pub patches: Vec<PatchRow>,
    /// Rows of the isolation (maximum coupling) table.
    pub coupling: Vec<CouplingRow>,
    /// Every "CHARGE DENSITIES" table (`PQ`), in the order printed: one per solution, in the
    /// currents table's row format with coulombs per metre for amperes.
    pub charges: Vec<CurrentTable>,
    /// The unit the coordinates are printed in, as stated by the `COORDINATES IN ...` line.
    pub coordinate_unit: Option<String>,
    /// Frequencies, in MHz, in the order reported.
    pub frequencies_mhz: Vec<f64>,
    /// Wavelengths, in metres, in the order reported. Useful for confirming the engine's
    /// frequency-to-wavelength constant.
    pub wavelengths_m: Vec<f64>,
    /// Rows of every "ANTENNA INPUT PARAMETERS" table, each tagged with its frequency.
    pub inputs: Vec<InputRow>,
    /// Rows of every "STRUCTURE EXCITATION DATA AT NETWORK CONNECTION POINTS" table, in the
    /// input table's format.
    pub network_ports: Vec<InputRow>,
    /// Every "SURFACE PATCH CURRENTS" table, in the order printed: one per solution, for
    /// structures with patches.
    pub patch_currents: Vec<PatchCurrentTable>,
    /// Every "CURRENTS AND LOCATION" table, in the order printed: one per solution (per
    /// frequency, and per incidence angle for a plane-wave excitation).
    pub currents: Vec<CurrentTable>,
    /// Every "POWER BUDGET", one per voltage-source solution.
    pub power_budgets: Vec<PowerBudgetRow>,
    /// Every "RADIATION PATTERNS" table.
    pub patterns: Vec<PatternTable>,
    /// Every "RADIATED FIELDS NEAR GROUND" table (`RP` mode 1).
    pub ground_fields: Vec<GroundFieldTable>,
    /// Every "NEAR ELECTRIC FIELDS" or "NEAR MAGNETIC FIELDS" table.
    pub near_fields: Vec<NearFieldTable>,
}

/// One near-field table.
#[derive(Debug, Clone, Default)]
pub struct NearFieldTable {
    /// `NH` rather than `NE`.
    pub magnetic: bool,
    /// Rows: point (metres) and `(magnitude, phase°)` of the x, y, z components.
    pub rows: Vec<NearFieldRow>,
}

/// One near-field row, `FORMAT (2X,3(2X,F9.4),1X,3(3X,1P,E11.4,2X,0P,F7.2))`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NearFieldRow {
    /// Point, metres.
    pub point: [f64; 3],
    /// x, y, z components as (magnitude, phase in degrees).
    pub field: [(f64, f64); 3],
}

fn parse_near_field_row(line: &str) -> Option<NearFieldRow> {
    let num = |lo: usize, hi: usize| -> Option<f64> { real(line.get(lo..hi.min(line.len()))?.trim()) };
    // A coordinate too wide for F9.4 (below -999.9999 m, say) is printed as asterisks; the row
    // still holds the field, and the point is known from the request, so it reads as NaN.
    let coord = |lo: usize| -> Option<f64> {
        let t = line.get(lo..lo + 9)?.trim();
        if !t.is_empty() && t.chars().all(|c| c == '*') {
            Some(f64::NAN)
        } else {
            t.parse().ok()
        }
    };
    let point = [coord(4)?, coord(15)?, coord(26)?];
    let pair = |k: usize| -> Option<(f64, f64)> {
        let lo = 36 + 23 * k;
        Some((num(lo + 3, lo + 14)?, num(lo + 16, lo + 23)?))
    };
    Some(NearFieldRow {
        point,
        field: [pair(0)?, pair(1)?, pair(2)?],
    })
}

/// One "RADIATED FIELDS NEAR GROUND" table.
#[derive(Debug, Clone, Default)]
pub struct GroundFieldTable {
    /// 0-based frequency index.
    pub frequency_index: usize,
    /// Rows, in the order printed.
    pub rows: Vec<GroundFieldRow>,
}

/// One row, `FORMAT (3X,F9.2,2X,F7.2,2X,F9.2,1X,3(3X,1P,E11.4,2X,0P,F7.2))`: range, azimuth,
/// height, then magnitude and phase of `E_θ`, `E_φ` and `E_r`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GroundFieldRow {
    /// Horizontal range, metres.
    pub rho: f64,
    /// Azimuth, degrees.
    pub phi: f64,
    /// Height, metres.
    pub z: f64,
    /// `E_θ` magnitude and phase (degrees).
    pub e_theta: (f64, f64),
    /// `E_φ` magnitude and phase.
    pub e_phi: (f64, f64),
    /// `E_r` magnitude and phase.
    pub e_radial: (f64, f64),
}

fn parse_ground_field_row(line: &str) -> Option<GroundFieldRow> {
    let num = |lo: usize, hi: usize| -> Option<f64> { real(line.get(lo..hi.min(line.len()))?.trim()) };
    let rho: f64 = line.get(3..12)?.trim().parse().ok()?;
    let phi: f64 = line.get(14..21)?.trim().parse().ok()?;
    let z: f64 = line.get(23..32)?.trim().parse().ok()?;
    let pair = |k: usize| -> Option<(f64, f64)> {
        let lo = 33 + 23 * k;
        Some((num(lo + 3, lo + 14)?, num(lo + 16, lo + 23)?))
    };
    Some(GroundFieldRow {
        rho,
        phi,
        z,
        e_theta: pair(0)?,
        e_phi: pair(1)?,
        e_radial: pair(2)?,
    })
}

/// The engine's "POWER BUDGET", watts, printed to five significant figures.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PowerBudgetRow {
    /// 0-based frequency index.
    pub frequency_index: usize,
    /// `INPUT POWER`.
    pub input: f64,
    /// `RADIATED POWER`.
    pub radiated: f64,
    /// `STRUCTURE LOSS`.
    pub structure_loss: f64,
    /// `NETWORK LOSS`.
    pub network_loss: f64,
    /// `EFFICIENCY`, percent.
    pub efficiency: f64,
}

/// One "RADIATION PATTERNS" table.
#[derive(Debug, Clone, Default)]
pub struct PatternTable {
    /// 0-based frequency index.
    pub frequency_index: usize,
    /// Rows, in the order printed.
    pub rows: Vec<PatternRow>,
    /// The "AVERAGE POWER GAIN" that follows, if averaging was requested.
    pub average_gain: Option<f64>,
}

/// One row of a radiation pattern, `FORMAT (1X,F7.2,F9.2,3X,3F8.2,F11.5,F9.2,2X,A6,
/// 2(1P,E15.5,0P,F9.2))`.
///
/// ```text
///   THETA     PHI       VERT.   HOR.    TOTAL       AXIAL     TILT  SENSE   MAGNITUDE    PHASE  ...
///   90.00     0.00     -999.99    2.15    2.15    0.00000    90.00 LINEAR   0.00000E+00     0.00 ...
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct PatternRow {
    /// θ, degrees.
    pub theta: f64,
    /// φ, degrees.
    pub phi: f64,
    /// First gain column (major-axis or vertical), dB.
    pub gain1: f64,
    /// Second gain column (minor-axis or horizontal), dB.
    pub gain2: f64,
    /// Total gain, dB.
    pub total: f64,
    /// Axial ratio.
    pub axial_ratio: f64,
    /// Tilt, degrees.
    pub tilt: f64,
    /// `LINEAR`, `RIGHT`, `LEFT`, or blank.
    pub sense: String,
    /// `E_θ` magnitude and phase (degrees).
    pub e_theta: (f64, f64),
    /// `E_φ` magnitude and phase (degrees).
    pub e_phi: (f64, f64),
}

fn parse_pattern_row(line: &str) -> Option<PatternRow> {
    let num = |lo: usize, hi: usize| -> Option<f64> { real(line.get(lo..hi.min(line.len()))?.trim()) };
    let theta: f64 = line.get(1..8)?.trim().parse().ok()?;
    let phi: f64 = line.get(8..17)?.trim().parse().ok()?;
    Some(PatternRow {
        theta,
        phi,
        gain1: num(20, 28)?,
        gain2: num(28, 36)?,
        total: num(36, 44)?,
        axial_ratio: num(44, 55)?,
        tilt: num(55, 64)?,
        sense: line.get(66..72)?.trim().to_string(),
        e_theta: (num(72, 87)?, num(87, 96)?),
        e_phi: (num(96, 111)?, num(111, 120)?),
    })
}

/// One "SURFACE PATCH CURRENTS" table.
#[derive(Debug, Clone, Default)]
pub struct PatchCurrentTable {
    /// 0-based index of the frequency it belongs to.
    pub frequency_index: usize,
    /// Rows, in patch order.
    pub rows: Vec<PatchCurrentRow>,
}

/// One patch of a patch-currents table: the current density's components along the patch's two
/// tangents, as magnitude and phase, which NEC prints to five significant figures (the
/// Cartesian components beside them get only three).
///
/// ```text
///     1
///    0.100  0.050  0.050 1.2095E-03  111.88 8.4058E-03 -115.75  0.00E+00  0.00E+00 -4.51E-04 ...
/// ```
///
/// Each patch takes two lines: `FORMAT (1X,I4)` with its number, then
/// `FORMAT (1X,3F7.3,2(1P,E11.4,0P,F8.2),1P,6E10.2)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PatchCurrentRow {
    /// 1-based patch number.
    pub patch: usize,
    /// Along tangent 1: `(magnitude A/m, phase degrees)`.
    pub t1: (f64, f64),
    /// Along tangent 2: `(magnitude A/m, phase degrees)`.
    pub t2: (f64, f64),
}

fn parse_patch_current_values(line: &str) -> Option<((f64, f64), (f64, f64))> {
    for (lo, hi) in [(1, 8), (8, 15), (15, 22)] {
        line.get(lo..hi)?.trim().parse::<f64>().ok()?;
    }
    let num = |lo: usize, hi: usize| -> Option<f64> { real(line.get(lo..hi)?.trim()) };
    Some(((num(22, 33)?, num(33, 41)?), (num(41, 52)?, num(52, 60)?)))
}

/// One "CURRENTS AND LOCATION" table.
#[derive(Debug, Clone, Default)]
pub struct CurrentTable {
    /// 0-based index of the frequency it belongs to.
    pub frequency_index: usize,
    /// Rows, in segment order.
    pub rows: Vec<CurrentRow>,
}

/// One row of a currents table, `FORMAT (1X,2I5,3F9.4,F9.5,1X,1P,3E12.4,0P,F9.3)`.
///
/// ```text
///   SEG.  TAG    COORD. OF SEG. CENTER     SEG.            - - - CURRENT (AMPS) - - -
///   NO.   NO.     X        Y        Z      LENGTH     REAL        IMAG.       MAG.        PHASE
///      1    1   0.0000  -0.2123   0.5604  0.04245   3.6621E-03  5.8224E-05  3.6626E-03    0.911
/// ```
///
/// Currents carry only five significant figures.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CurrentRow {
    /// 1-based segment number.
    pub segment: usize,
    /// Wire tag.
    pub tag: i32,
    /// Current at the segment centre, `(re, im)` amperes; NaN where the field overflowed.
    pub current: (f64, f64),
}

fn parse_current_row(line: &str) -> Option<CurrentRow> {
    let segment: usize = line.get(1..6)?.trim().parse().ok()?;
    let tag: i32 = line.get(6..11)?.trim().parse().ok()?;
    // The coordinates and length must be numbers, which keeps other tables' rows out.
    for (lo, hi) in [(11, 20), (20, 29), (29, 38), (38, 47)] {
        line.get(lo..hi)?.trim().parse::<f64>().ok()?;
    }
    let re = real(line.get(48..60)?.trim())?;
    let im = real(line.get(60..72)?.trim())?;
    Some(CurrentRow {
        segment,
        tag,
        current: (re, im),
    })
}

/// One row of the engine's "ANTENNA INPUT PARAMETERS" table.
///
/// ```text
///    TAG   SEG.    VOLTAGE (VOLTS)         CURRENT (AMPS)         IMPEDANCE (OHMS)        ADMITTANCE (MHOS)      POWER
///      1     5 1.00000E+00 0.00000E+00 1.38737E-02 3.23330E-07 7.20788E+01-1.67981E-03 1.38737E-02 3.23330E-07 6.93685E-03
/// ```
///
/// Printed with `FORMAT (2(1X,I5),1P,9E12.5)`. Values already collide in the simplest model —
/// `7.20788E+01-1.67981E-03` above, from `models\Example1.nec` — so the row is read by column.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InputRow {
    /// The frequency this row belongs to, MHz (the most recent `FREQUENCY=` line).
    ///
    /// Printed to only five significant figures (`2.9979E+02` for 299.7925 MHz), so it is too
    /// coarse to match on; use [`InputRow::frequency_index`] instead.
    pub frequency_mhz: f64,
    /// 0-based position of this row's frequency in the run: the *k*-th `FREQUENCY=` block is the
    /// *k*-th frequency of the sweep.
    pub frequency_index: usize,
    /// Wire tag of the source segment.
    pub tag: i32,
    /// Source segment number.
    pub segment: usize,
    /// Whether NEC marked the row with `*`: a current-slope-discontinuity source (`EX 5`).
    pub slope_source: bool,
    /// Voltage, `(re, im)` volts.
    pub voltage: (f64, f64),
    /// Current, `(re, im)` amperes.
    pub current: (f64, f64),
    /// Impedance, `(re, im)` ohms.
    pub impedance: (f64, f64),
    /// Admittance, `(re, im)` siemens.
    pub admittance: (f64, f64),
    /// Input power, watts.
    pub power: f64,
}

/// Read one input-parameters row by column.
fn parse_input_row(line: &str, frequency_mhz: f64, frequency_index: usize) -> Option<InputRow> {
    let tag: i32 = line.get(0..6)?.trim().parse().ok()?;
    // An EX 5 (current-slope-discontinuity) source is printed with FORMAT (1X,I5,2H *,I4,...):
    // a `*` before the segment number.
    let seg_field = line.get(6..12)?.trim();
    let slope_source = seg_field.starts_with('*');
    let segment: usize = seg_field.trim_start_matches('*').trim().parse().ok()?;
    let mut v = [0.0_f64; 9];
    for (k, slot) in v.iter_mut().enumerate() {
        let lo = 12 + 12 * k;
        *slot = real(line.get(lo..(lo + 12).min(line.len()))?.trim())?;
    }
    Some(InputRow {
        frequency_mhz,
        frequency_index,
        tag,
        segment,
        slope_source,
        voltage: (v[0], v[1]),
        current: (v[2], v[3]),
        impedance: (v[4], v[5]),
        admittance: (v[6], v[7]),
        power: v[8],
    })
}

/// Parse the parts of a `.out` report this project validates against.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn parse_out(text: &str) -> OutReport {
    let mut report = OutReport::default();
    let mut in_table = false;
    let mut in_patches = false;
    let mut in_coupling = false;
    let mut in_charges = false;
    let mut in_inputs = false;
    let mut in_ports = false;
    let mut in_currents = false;
    let mut in_patch_currents = false;
    let mut pending_patch: Option<usize> = None;
    let mut in_pattern = false;
    let mut in_ground_fields = false;
    let mut in_near = false;
    let mut budget: Option<PowerBudgetRow> = None;
    let mut frequency = f64::NAN;

    for line in text.lines() {
        let upper = line.to_ascii_uppercase();

        // The power budget: five labelled lines.
        if upper.contains("POWER BUDGET") {
            budget = Some(PowerBudgetRow {
                frequency_index: report.frequencies_mhz.len().saturating_sub(1),
                ..PowerBudgetRow::default()
            });
            continue;
        }
        if let Some(b) = budget.as_mut() {
            let value = |label: &str| labelled_value(&upper, label);
            if let Some(v) = value("INPUT POWER   =") {
                b.input = v;
            } else if let Some(v) = value("RADIATED POWER=") {
                b.radiated = v;
            } else if let Some(v) = value("STRUCTURE LOSS=") {
                b.structure_loss = v;
            } else if let Some(v) = value("NETWORK LOSS  =") {
                b.network_loss = v;
            } else if let Some(v) = value("EFFICIENCY    =") {
                b.efficiency = v;
                report.power_budgets.push(*b);
                budget = None;
            }
            continue;
        }

        if upper.contains("NEAR ELECTRIC FIELDS") || upper.contains("NEAR MAGNETIC FIELDS") {
            in_near = true;
            report.near_fields.push(NearFieldTable {
                magnetic: upper.contains("MAGNETIC"),
                rows: Vec::new(),
            });
            continue;
        }
        if in_near {
            if let Some(row) = parse_near_field_row(line) {
                if let Some(t) = report.near_fields.last_mut() {
                    t.rows.push(row);
                }
                continue;
            }
            let started = report.near_fields.last().is_some_and(|t| !t.rows.is_empty());
            if !started {
                continue;
            }
            in_near = false;
        }

        if upper.contains("RADIATED FIELDS NEAR GROUND") {
            in_ground_fields = true;
            report.ground_fields.push(GroundFieldTable {
                frequency_index: report.frequencies_mhz.len().saturating_sub(1),
                rows: Vec::new(),
            });
            continue;
        }
        if in_ground_fields {
            if let Some(row) = parse_ground_field_row(line) {
                if let Some(t) = report.ground_fields.last_mut() {
                    t.rows.push(row);
                }
                continue;
            }
            let started = report.ground_fields.last().is_some_and(|t| !t.rows.is_empty());
            if !started {
                continue;
            }
            in_ground_fields = false;
        }

        if upper.contains("RADIATION PATTERNS") {
            in_pattern = true;
            report.patterns.push(PatternTable {
                frequency_index: report.frequencies_mhz.len().saturating_sub(1),
                ..PatternTable::default()
            });
            continue;
        }
        if let Some(v) = labelled_value(&upper, "AVERAGE POWER GAIN=") {
            if let Some(t) = report.patterns.last_mut() {
                t.average_gain = Some(v);
            }
            in_pattern = false;
            continue;
        }
        if in_pattern {
            if let Some(row) = parse_pattern_row(line) {
                if let Some(t) = report.patterns.last_mut() {
                    t.rows.push(row);
                }
                continue;
            }
            let started = report.patterns.last().is_some_and(|t| !t.rows.is_empty());
            if !started {
                continue;
            }
            in_pattern = false;
        }

        if upper.contains("SURFACE PATCH CURRENTS") {
            in_patch_currents = true;
            pending_patch = None;
            report.patch_currents.push(PatchCurrentTable {
                frequency_index: report.frequencies_mhz.len().saturating_sub(1),
                rows: Vec::new(),
            });
            continue;
        }
        if in_patch_currents {
            let trimmed = line.trim();
            if let Some(patch) = pending_patch.take()
                && let Some((t1, t2)) = parse_patch_current_values(line)
            {
                if let Some(t) = report.patch_currents.last_mut() {
                    t.rows.push(PatchCurrentRow { patch, t1, t2 });
                }
                continue;
            }
            if let Ok(patch) = trimmed.parse::<usize>() {
                pending_patch = Some(patch);
                continue;
            }
            // Between the title and the rows: blank lines and four header lines.
            let started = report.patch_currents.last().is_some_and(|t| !t.rows.is_empty());
            if !started
                && (trimmed.is_empty()
                    || upper.contains("WAVELENGTHS")
                    || upper.contains("AMPS/METER")
                    || upper.contains("COMPONENTS")
                    || upper.contains("PATCH CENTER")
                    || upper.contains("MAG."))
            {
                continue;
            }
            in_patch_currents = false;
        }

        if upper.contains("CURRENTS AND LOCATION") {
            in_currents = true;
            report.currents.push(CurrentTable {
                frequency_index: report.frequencies_mhz.len().saturating_sub(1),
                rows: Vec::new(),
            });
            continue;
        }
        if in_currents {
            if let Some(row) = parse_current_row(line) {
                if let Some(t) = report.currents.last_mut() {
                    t.rows.push(row);
                }
                continue;
            }
            // Between the title and the rows: blank lines, "DISTANCES IN WAVELENGTHS", and the
            // two header lines. Once rows have started, anything else ends the table.
            let started = report.currents.last().is_some_and(|t| !t.rows.is_empty());
            if !started
                && (line.trim().is_empty()
                    || upper.contains("DISTANCES")
                    || upper.contains("SEG.")
                    || upper.contains("NO."))
            {
                continue;
            }
            in_currents = false;
        }

        if upper.contains("NETWORK CONNECTION POINTS") {
            in_ports = true;
            continue;
        }
        if in_ports {
            if let Some(row) = parse_input_row(line, frequency, report.frequencies_mhz.len().saturating_sub(1)) {
                report.network_ports.push(row);
                continue;
            }
            if line.trim().is_empty() || upper.contains("TAG") || upper.contains("NO.") {
                continue;
            }
            in_ports = false;
        }

        if upper.contains("ANTENNA INPUT PARAMETERS") {
            in_inputs = true;
            continue;
        }
        if in_inputs {
            if let Some(row) = parse_input_row(line, frequency, report.frequencies_mhz.len().saturating_sub(1)) {
                report.inputs.push(row);
                continue;
            }
            // Blank lines and the two header lines sit between the title and the rows.
            if line.trim().is_empty() || upper.contains("TAG") || upper.contains("NO.") {
                continue;
            }
            in_inputs = false;
        }

        if upper.contains("COORDINATES IN") {
            if let Some(rest) = upper.split("COORDINATES IN").nth(1) {
                report.coordinate_unit = Some(rest.trim().to_string());
            }
            continue;
        }
        if let Some(v) = labelled_value(&upper, "FREQUENCY=") {
            report.frequencies_mhz.push(v);
            frequency = v;
            continue;
        }
        if let Some(v) = labelled_value(&upper, "WAVELENGTH=") {
            report.wavelengths_m.push(v);
            continue;
        }

        if upper.contains("CHARGE DENSITIES") {
            in_charges = true;
            report.charges.push(CurrentTable {
                frequency_index: report.frequencies_mhz.len().saturating_sub(1),
                rows: Vec::new(),
            });
            continue;
        }
        if in_charges {
            if let Some(row) = parse_current_row(line) {
                if let Some(t) = report.charges.last_mut() {
                    t.rows.push(row);
                }
                continue;
            }
            let started = report.charges.last().is_some_and(|t| !t.rows.is_empty());
            if !started {
                continue;
            }
            in_charges = false;
        }

        if upper.contains("ISOLATION DATA") {
            in_coupling = true;
            continue;
        }
        if in_coupling {
            if let Some(row) = parse_coupling_row(line) {
                report.coupling.push(row);
                continue;
            }
            if report.coupling.is_empty() {
                continue;
            }
            in_coupling = false;
        }

        if upper.contains("SURFACE PATCH DATA") {
            in_patches = true;
            continue;
        }
        if in_patches {
            if let Some(row) = parse_patch_row(line) {
                report.patches.push(row);
                continue;
            }
            let started = !report.patches.is_empty();
            if !started && (line.trim().is_empty() || upper.contains("COORD") || upper.contains("NO.")) {
                continue;
            }
            in_patches = false;
        }

        // The table's own header line is the only place "I-" and "I+" appear together.
        if upper.contains("I-") && upper.contains("I+") {
            in_table = true;
            continue;
        }

        if in_table {
            match parse_segment_row(line) {
                Some(row) => report.segments.push(row),
                // A blank line inside the table is padding; anything else ends it.
                None if line.trim().is_empty() => {}
                None => in_table = false,
            }
        }
    }

    report
}

/// Pull the number following a label such as `FREQUENCY=`.
fn labelled_value(upper_line: &str, label: &str) -> Option<f64> {
    let rest = upper_line.split(label).nth(1)?;
    let tok = rest.split_whitespace().next()?;
    tok.parse::<f64>().ok()
}

/// Extract successive numeric literals from a fixed-width Fortran line.
///
/// A sign starts a new token, which is what recovers columns that have run together. A sign
/// immediately following an exponent marker stays part of that number, so `1.0E-05` survives.
fn scan_numbers(line: &str) -> Vec<&str> {
    let b = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;

    while i < b.len() {
        if !(b[i].is_ascii_digit() || b[i] == b'+' || b[i] == b'-' || b[i] == b'.') {
            i += 1;
            continue;
        }
        let start = i;
        if b[i] == b'+' || b[i] == b'-' {
            i += 1;
        }
        let mantissa_start = i;
        while i < b.len() && (b[i].is_ascii_digit() || b[i] == b'.') {
            i += 1;
        }
        if i == mantissa_start {
            // A lone sign or stray '.'; not a number.
            i = start + 1;
            continue;
        }
        // Optional exponent, whose sign belongs to this number.
        if i < b.len() && matches!(b[i], b'e' | b'E' | b'd' | b'D') {
            let mut j = i + 1;
            if j < b.len() && (b[j] == b'+' || b[j] == b'-') {
                j += 1;
            }
            if j < b.len() && b[j].is_ascii_digit() {
                i = j;
                while i < b.len() && b[i].is_ascii_digit() {
                    i += 1;
                }
            }
        }
        out.push(&line[start..i]);
    }
    out
}

/// Column ranges of a segmentation row, from the Fortran format
/// `(1X,I5,4F10.5,1X,3F10.5,1X,3I5,2X,I5)`.
///
/// Fixed-width reading is the only reliable way in once values fill their fields. Two kinds of
/// collision occur in the corpus:
///
/// ```text
///    188  14.81818 ...    0.00000-135.00000   0.00103  -199  188  189     18
///    4631032.51628   0.00000 133.22372  51.54445  -45.00000 ...
/// ```
///
/// The first has a sign to split on; the second — segment 463 glued to X = 1032.51628 — has none,
/// and truncated `LFvertical\cutler_single .nec`'s table at row 462 of 2142 until rows were read
/// by column.
const COLS: [(usize, usize); 12] = [
    (0, 6),   // segment number (1X,I5)
    (6, 16),  // centre x
    (16, 26), // centre y
    (26, 36), // centre z
    (36, 46), // length
    (47, 57), // alpha (after 1X)
    (57, 67), // beta
    (67, 77), // radius
    (77, 83), // I- (1X,I5)
    (83, 88), // I
    (88, 93), // I+
    (93, 100), // tag (2X,I5)
];

/// Read a row by column. Returns `None` if any field is missing or unparseable.
fn fixed_width_fields(line: &str) -> Option<Vec<&str>> {
    let mut out = Vec::with_capacity(COLS.len());
    for (lo, hi) in COLS {
        // The tag field may be short if trailing blanks were trimmed.
        let s = line.get(lo..hi.min(line.len()))?.trim();
        if s.is_empty() {
            return None;
        }
        out.push(s);
    }
    Some(out)
}

/// A segmentation row is twelve numbers, the first and last integral.
///
/// Read by column first; if that fails (a different engine build with another layout), fall back
/// to the sign-aware scanner. Either path must agree that field 9 repeats the segment number.
fn parse_segment_row(line: &str) -> Option<SegmentRow> {
    fixed_width_fields(line)
        .and_then(|f| row_from_fields(&f))
        .or_else(|| {
            let f = scan_numbers(line);
            if f.len() == 12 { row_from_fields(&f) } else { None }
        })
}

/// Parse a real field, reading Fortran's overflow marker as NaN.
///
/// When a value cannot fit its field width Fortran prints asterisks instead: X = -1032.51628
/// needs eleven characters and `F10.5` allows ten, so `LFvertical\cutler_single .nec` prints
/// `**********`. The value is genuinely lost from the report. It is read as NaN, meaning "not
/// available" — comparisons skip NaN fields — so that the rest of the row, and the rest of the
/// table, are still checked.
fn real(s: &str) -> Option<f64> {
    if !s.is_empty() && s.bytes().all(|b| b == b'*') {
        return Some(f64::NAN);
    }
    // The engine's C runtime (MinGW/MSVCRT) prints NaN and infinity as `1.#QNBE+00`,
    // `-1.#IND`, `1.#INF`, `1.#R` and the like.
    if s.contains('#') {
        return Some(f64::NAN);
    }
    s.parse().ok()
}

fn row_from_fields(f: &[&str]) -> Option<SegmentRow> {
    Some(SegmentRow {
        index: f[0].parse().ok()?,
        x: real(f[1])?,
        y: real(f[2])?,
        z: real(f[3])?,
        length: real(f[4])?,
        alpha: real(f[5])?,
        beta: real(f[6])?,
        radius: real(f[7])?,
        i_minus: f[8].parse().ok()?,
        // f[9] repeats the segment number; checked rather than stored.
        i_plus: {
            let repeat: usize = f[9].parse().ok()?;
            let idx: usize = f[0].parse().ok()?;
            if repeat != idx {
                return None;
            }
            f[10].parse().ok()?
        },
        tag: f[11].parse().ok()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_surface_patch_data() {
        let text = "\
                                            - - - SURFACE PATCH DATA - - -

                                                 COORDINATES IN METERS

 PATCH     COORD. OF PATCH CENTER       UNIT NORMAL VECTOR      PATCH            COMPONENTS OF UNIT TANGENT VECTORS
  NO.      X         Y         Z         X       Y       Z       AREA       X1      Y1      Z1       X2      Y2      Z2
    1   0.10000   0.05000   0.05000   1.0000  0.0000  0.0000   0.01000   0.0000  1.0000  0.0000   0.0000  0.0000  1.0000
    2   0.05000   0.10000   0.05000   0.0000  1.0000  0.0000   0.01000  -1.0000  0.0000  0.0000   0.0000  0.0000  1.0000


";
        let r = parse_out(text);
        assert_eq!(r.patches.len(), 2);
        assert_eq!(r.patches[1].center, [0.05, 0.1, 0.05]);
        assert_eq!(r.patches[1].normal, [0.0, 1.0, 0.0]);
        assert_eq!(r.patches[1].t1, [-1.0, 0.0, 0.0]);
        assert_eq!(r.patches[1].area, 0.01);
        assert_eq!(r.patches[0].t2, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn reads_surface_patch_currents() {
        // From the engine's output for models/SPatch/Nec-Ex4.nec.
        let text = "                                         - - - - SURFACE PATCH CURRENTS - - - -

                                                  DISTANCE IN WAVELENGTHS
                                                  CURRENT IN AMPS/METER

                            - - SURFACE COMPONENTS - -                   - - - RECTANGULAR COMPONENTS - - -
      PATCH CENTER      TANGENT VECTOR 1   TANGENT VECTOR 2           X                   Y                   Z
     X      Y      Z     MAG.       PHASE   MAG.       PHASE    REAL      IMAG.     REAL      IMAG.     REAL      IMAG. 
    1
   0.100  0.050  0.050 1.2095E-03  111.88 8.4058E-03 -115.75  0.00E+00  0.00E+00 -4.51E-04  1.12E-03 -3.65E-03 -7.57E-03
    2
   0.050  0.100  0.050 1.2018E-03  -68.27 8.2434E-03 -116.57 -4.45E-04  1.12E-03 -2.27E-15  5.70E-15 -3.69E-03 -7.37E-03



                                          - - - ANTENNA INPUT PARAMETERS - - -
";
        let r = parse_out(text);
        assert_eq!(r.patch_currents.len(), 1);
        let rows = &r.patch_currents[0].rows;
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1].patch, 2);
        assert_eq!(rows[0].t1, (1.2095e-3, 111.88));
        assert_eq!(rows[1].t2, (8.2434e-3, -116.57));
    }

    /// Verbatim excerpt from a real engine run on `models/Example1.nec`.
    const EXAMPLE1_OUT: &str = "\
                                 - - - - SEGMENTATION DATA - - - -

                                        COORDINATES IN METERS

                         I+ AND I- INDICATE THE SEGMENTS BEFORE AND AFTER I


  SEG.   COORDINATES OF SEG. CENTER     SEG.     ORIENTATION ANGLES    WIRE    CONNECTION DATA   TAG
  NO.       X         Y         Z       LENGTH     ALPHA     BETA      RADIUS    I-   I    I+    NO.
     1   0.00000  -0.21493   0.00000   0.05373    0.00000  90.00000   0.00010     0    1    2      1
     2   0.00000  -0.16120   0.00000   0.05373    0.00000  90.00000   0.00010     1    2    3      1
     9   0.00000   0.21493   0.00000   0.05373    0.00000  90.00000   0.00010     8    9    0      1

                                 - - - - - - FREQUENCY - - - - - -
                                    FREQUENCY= 3.0000E+02 MHZ
                                    WAVELENGTH= 9.9933E-01 METERS
";

    #[test]
    fn reads_the_segmentation_table() {
        let r = parse_out(EXAMPLE1_OUT);
        assert_eq!(r.segments.len(), 3);

        let s1 = &r.segments[0];
        assert_eq!(s1.index, 1);
        assert!((s1.y + 0.21493).abs() < 1e-9);
        assert!((s1.length - 0.05373).abs() < 1e-9);
        assert!((s1.beta - 90.0).abs() < 1e-9);
        assert!((s1.radius - 0.0001).abs() < 1e-12);
        assert_eq!(s1.i_minus, 0, "segment 1 is free at end 1");
        assert_eq!(s1.i_plus, 2);
        assert_eq!(s1.tag, 1);

        let last = r.segments.last().expect("rows");
        assert_eq!(last.index, 9);
        assert_eq!(last.i_minus, 8);
        assert_eq!(last.i_plus, 0, "segment 9 is free at end 2");
    }

    #[test]
    fn reads_input_parameters_including_a_glued_impedance() {
        // Verbatim from the engine's run on models/Example1.nec; the impedance's real and
        // imaginary parts touch: 7.20788E+01-1.67981E-03.
        let out = "\
                                    FREQUENCY= 3.0000E+02 MHZ
                                          - - - ANTENNA INPUT PARAMETERS - - -

   TAG   SEG.    VOLTAGE (VOLTS)         CURRENT (AMPS)         IMPEDANCE (OHMS)        ADMITTANCE (MHOS)      POWER
   NO.   NO.    REAL        IMAG.       REAL        IMAG.       REAL        IMAG.       REAL        IMAG.     (WATTS)
     1     5 1.00000E+00 0.00000E+00 1.38737E-02 3.23330E-07 7.20788E+01-1.67981E-03 1.38737E-02 3.23330E-07 6.93685E-03

                           - - - CURRENTS AND LOCATION - - -
";
        let r = parse_out(out);
        assert_eq!(r.inputs.len(), 1);
        let row = r.inputs[0];
        assert!((row.frequency_mhz - 300.0).abs() < 1e-9);
        assert_eq!(row.frequency_index, 0);
        assert_eq!((row.tag, row.segment), (1, 5));
        assert!((row.impedance.0 - 72.0788).abs() < 1e-9);
        assert!((row.impedance.1 + 1.679_81e-3).abs() < 1e-12);
        assert!((row.current.0 - 1.387_37e-2).abs() < 1e-12);
        assert!((row.power - 6.936_85e-3).abs() < 1e-12);
    }

    #[test]
    fn reads_a_current_slope_source_row_marked_with_a_star() {
        // EX 5 rows use FORMAT (1X,I5,2H *,I4,...). Taken from the reference engine's output.
        let line = "     1 *   6 1.00000E+00 0.00000E+00 2.01371E-02 1.68040E-03 4.93161E+01-4.11533E+00 2.01371E-02 1.68040E-03 1.00686E-02";
        let r = parse_input_row(line, 300.0, 0).expect("row");
        assert_eq!((r.tag, r.segment), (1, 6));
        assert!(r.slope_source);
        assert_eq!(r.impedance, (49.3161, -4.11533));
    }

    #[test]
    fn reads_the_currents_table() {
        // From the reference engine's output.
        let text = [
            "                             - - - CURRENTS AND LOCATION - - -",
            "",
            "                                 DISTANCES IN WAVELENGTHS",
            "",
            "  SEG.  TAG    COORD. OF SEG. CENTER     SEG.            - - - CURRENT (AMPS) - - -",
            "  NO.   NO.     X        Y        Z      LENGTH     REAL        IMAG.       MAG.        PHASE",
            "     1    1   0.0000  -0.2123   0.5604  0.04245   3.6621E-03  5.8224E-05  3.6626E-03    0.911",
            "     2    1   0.0000  -0.1698   0.5604  0.04245   9.3802E-03  2.3719E-04  9.3832E-03    1.449",
            "",
            "                    - - - POWER BUDGET - - -",
        ]
        .join("
");
        let r = parse_out(&text);
        assert_eq!(r.currents.len(), 1);
        let rows = &r.currents[0].rows;
        assert_eq!(rows.len(), 2);
        assert_eq!((rows[1].segment, rows[1].tag), (2, 1));
        assert_eq!(rows[1].current, (9.3802e-3, 2.3719e-4));
    }

    #[test]
    fn reads_a_pattern_row_and_the_power_budget() {
        // Rows and budget lines as the reference engine prints them.
        let text = [
            "                                                - - - RADIATION PATTERNS - - -",
            "",
            "  - - ANGLES - -       - POWER GAINS -       - - - POLARIZATION - - -    - - - E(THETA) - - -    - - - E(PHI) - - -",
            "  THETA     PHI       VERT.   HOR.    TOTAL       AXIAL     TILT   SENSE     MAGNITUDE    PHASE      MAGNITUDE    PHASE",
            " DEGREES  DEGREES       DB       DB       DB        RATIO     DEG.             VOLTS/M    DEGREES      VOLTS/M    DEGREES",
            "   90.00     0.00    -999.99    2.12    2.12    0.00000   -90.00  LINEAR    0.00000E+00     0.00    8.23169E-01   -91.80",
            "  100.00    45.00     -17.06   -1.86   -1.73    0.00000   -80.15  LINEAR    9.04367E-02    88.33    5.20804E-01   -91.67",
            "",
            "",
            "                                        - - - POWER BUDGET - - -",
            "",
            "                                           INPUT POWER   = 1.0069E-02 WATTS",
            "                                           RADIATED POWER= 1.0069E-02 WATTS",
            "                                           STRUCTURE LOSS= 0.0000E+00 WATTS",
            "                                           NETWORK LOSS  = 0.0000E+00 WATTS",
            "                                           EFFICIENCY    = 100.00 PERCENT",
        ]
        .join("\n");
        let r = parse_out(&text);
        assert_eq!(r.patterns.len(), 1);
        let rows = &r.patterns[0].rows;
        assert_eq!(rows.len(), 2);
        assert_eq!((rows[0].theta, rows[0].phi), (90.0, 0.0));
        assert_eq!((rows[0].gain1, rows[0].gain2, rows[0].total), (-999.99, 2.12, 2.12));
        assert_eq!(rows[0].sense, "LINEAR");
        assert_eq!(rows[0].e_phi, (0.823_169, -91.8));
        assert_eq!((rows[1].theta, rows[1].tilt, rows[1].e_theta), (100.0, -80.15, (0.090_436_7, 88.33)));
        assert_eq!(r.power_budgets.len(), 1);
        assert!((r.power_budgets[0].input - 1.0069e-2).abs() < 1e-15);
        assert!((r.power_budgets[0].efficiency - 100.0).abs() < 1e-12);
    }

    #[test]
    fn reads_units_and_frequency() {
        let r = parse_out(EXAMPLE1_OUT);
        assert_eq!(r.coordinate_unit.as_deref(), Some("METERS"));
        assert_eq!(r.frequencies_mhz, vec![300.0]);
        assert_eq!(r.wavelengths_m.len(), 1);

        // This is the measurement that pins NEC's constant to 299.8 rather than physical c.
        let lambda = r.wavelengths_m[0];
        assert!((lambda - 299.8 / 300.0).abs() < 1e-4, "lambda = {lambda}");
    }

    #[test]
    fn the_table_ends_at_the_next_section() {
        let r = parse_out(EXAMPLE1_OUT);
        // The FREQUENCY banner must not be mistaken for more rows.
        assert_eq!(r.segments.len(), 3);
    }

    #[test]
    fn reads_a_row_whose_columns_have_run_together() {
        // Verbatim from Aperiodic\2ELK9AY.NEC's output: ALPHA and BETA collide because -135
        // fills its field, and the I- column is negative. Whitespace splitting yields 11
        // tokens here and used to truncate the table at row 187 of 264.
        let line = "   188  14.81818  -0.18182   0.01000   0.51426    0.00000-135.00000   \
                    0.00103  -199  188  189     18";
        let row = parse_segment_row(line).expect("row must parse");
        assert_eq!(row.index, 188);
        assert!((row.alpha - 0.0).abs() < 1e-12);
        assert!((row.beta + 135.0).abs() < 1e-9, "beta = {}", row.beta);
        assert_eq!(row.i_minus, -199, "NEC signs connectivity to encode orientation");
        assert_eq!(row.i_plus, 189);
        assert_eq!(row.tag, 18);
    }

    #[test]
    fn reads_a_row_whose_index_is_glued_to_a_positive_coordinate() {
        // Verbatim from LFvertical\cutler_single .nec: segment 463 and X = 1032.51628 have no
        // separator and no sign between them.
        let line = "   4631032.51628   0.00000 133.22372  51.54445  -45.00000   0.00000   \
                    0.07500   462  463  464   1002";
        let row = parse_segment_row(line).expect("row must parse");
        assert_eq!(row.index, 463);
        assert!((row.x - 1032.51628).abs() < 1e-9, "x = {}", row.x);
        assert!((row.z - 133.22372).abs() < 1e-9);
        assert_eq!(row.i_minus, 462);
        assert_eq!(row.i_plus, 464);
        assert_eq!(row.tag, 1002);
    }

    #[test]
    fn an_overflowed_field_is_unavailable_but_the_row_survives() {
        // Verbatim from LFvertical\cutler_single .nec: X = -1032.51628 overflowed F10.5.
        let line = "   658**********   0.00000 133.22372  51.54445  -45.00000-180.00000   \
                    0.07500   657  658  659   1050";
        let row = parse_segment_row(line).expect("row must still parse");
        assert_eq!(row.index, 658);
        assert!(row.x.is_nan(), "overflowed field reads as NaN, i.e. not available");
        assert!((row.z - 133.22372).abs() < 1e-9);
        assert!((row.beta + 180.0).abs() < 1e-9);
        assert_eq!((row.i_minus, row.i_plus, row.tag), (657, 659, 1050));
    }

    #[test]
    fn exponent_signs_stay_attached() {
        assert_eq!(scan_numbers("1.0E-05 -2.5e+3"), vec!["1.0E-05", "-2.5e+3"]);
        // But a sign that is not part of an exponent starts a new number.
        assert_eq!(scan_numbers("0.00000-135.00000"), vec!["0.00000", "-135.00000"]);
    }

    #[test]
    fn rejects_rows_whose_repeated_index_disagrees() {
        // Guards against mis-splitting a row: field 9 repeats the segment number.
        let bad = "  SEG. I-   I    I+  NO.\n     1   0 0 0 0 0 0 0   0    7    2      1\n";
        assert!(parse_out(bad).segments.is_empty());
    }

    #[test]
    fn an_empty_report_is_not_an_error() {
        let r = parse_out("nothing to see here\n");
        assert!(r.segments.is_empty());
        assert!(r.frequencies_mhz.is_empty());
    }
}
