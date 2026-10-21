//! A plain-text report of one solution, laid out like NEC's own output file: the tables a
//! NEC user knows, with the same headings and the same `1.23456E+01` numbers, so it can be read
//! alongside — or diffed against — an engine `.out` file.

use crate::Model;
use nec_cards::{Item, parse_deck};
use nec_solve::{GroundSpec, PatternRequest, Sense, Solution};
use num_complex::Complex64;
use std::fmt::Write as _;

/// A number as NEC prints it, `±d.ddddddE±dd` with `digits` after the point.
fn sci(v: f64, digits: usize) -> String {
    if !v.is_finite() {
        return format!("{v:>w$}", w = digits + 7);
    }
    let s = format!("{v:.digits$E}");
    let (mantissa, exponent) = s.split_once('E').unwrap_or((&s, "0"));
    let e: i32 = exponent.parse().unwrap_or(0);
    format!("{mantissa}E{}{:02}", if e < 0 { '-' } else { '+' }, e.abs())
}

/// Column of `width` holding a NEC number.
fn col(v: f64, width: usize) -> String {
    format!("{:>width$}", sci(v, 5))
}

/// A ground in words: "Real ground (Sommerfeld), εr 13, σ 0.005 S/m".
#[must_use]
pub fn ground_label(g: &GroundSpec) -> String {
    match g {
        GroundSpec::FreeSpace => "Free space".into(),
        GroundSpec::Perfect => "Perfect ground".into(),
        GroundSpec::Reflection {
            epsr,
            sigma,
            screen,
        } => format!(
            "Real ground (reflection coefficient), εr {epsr}, σ {sigma} S/m{}",
            screen.map_or(String::new(), |s| format!(", {} radials", s.count))
        ),
        GroundSpec::Sommerfeld { epsr, sigma } => {
            format!("Real ground (Sommerfeld), εr {epsr}, σ {sigma} S/m")
        }
        GroundSpec::PerfectNearField { epsr, sigma } => {
            format!("MiniNEC ground, εr {epsr}, σ {sigma} S/m")
        }
    }
}

fn heading(out: &mut String, title: &str) {
    let _ = writeln!(out, "\n\n{:^100}\n", format!("- - - {title} - - -"));
}

impl Model {
    /// The text report for `sol`, a solution of this model's deck `text`.
    #[must_use]
    pub fn report(&self, text: &str, sol: &Solution) -> String {
        let mut out = String::new();
        let f = sol.frequency;
        let wlam = f.wavelength_m();

        let _ = writeln!(out, "{:^100}", "NEC STUDIO — NEC-2 SOLUTION REPORT");
        heading(&mut out, "COMMENTS");
        if let Ok(deck) = parse_deck(text) {
            for it in &deck.items {
                if let Item::CommentCard { text, .. } = it {
                    let t = text.trim();
                    if !t.is_empty() {
                        let _ = writeln!(out, "{:20}{t}", "");
                    }
                }
            }
        }

        heading(&mut out, "STRUCTURE");
        let tags: std::collections::BTreeSet<i32> =
            self.geom.segments.iter().map(|s| s.tag).collect();
        let _ = writeln!(
            out,
            "{:20}{} SEGMENTS, {} WIRE TAGS, {} PATCHES",
            "",
            self.geom.len(),
            tags.len(),
            self.geom.patches.len()
        );
        if let Ok(prog) = self.program() {
            let _ = writeln!(out, "{:20}GROUND: {}", "", ground_label(&prog.ground));
        }

        heading(&mut out, "FREQUENCY");
        let _ = writeln!(
            out,
            "{:32}FREQUENCY={} MHZ\n{:32}WAVELENGTH={} METERS",
            "",
            sci(f.mhz(), 4),
            "",
            sci(wlam, 4)
        );

        heading(&mut out, "ANTENNA INPUT PARAMETERS");
        let _ = writeln!(
            out,
            "   TAG   SEG.    VOLTAGE (VOLTS)         CURRENT (AMPS)         IMPEDANCE (OHMS)        ADMITTANCE (MHOS)      POWER\n   NO.   NO.    REAL        IMAG.       REAL        IMAG.       REAL        IMAG.       REAL        IMAG.     (WATTS)"
        );
        for i in self.source_inputs(sol) {
            let _ = writeln!(
                out,
                " {:5} {:5}{}{}{}{}{}{}{}{}{}",
                i.tag,
                i.segment,
                col(i.voltage.re, 12),
                col(i.voltage.im, 12),
                col(i.current.re, 12),
                col(i.current.im, 12),
                col(i.impedance.re, 12),
                col(i.impedance.im, 12),
                col(i.admittance.re, 12),
                col(i.admittance.im, 12),
                col(i.power, 12)
            );
        }
        if self.lowered
            && self
                .geom
                .segments
                .iter()
                .any(|s| s.tag > nec_cards::lower::CURRENT_SOURCE_TAG_BASE)
        {
            let _ = writeln!(
                out,
                "\n   Current sources (4nec2 EX 6) are reported at the segment they drive: the voltage across it\n   and the impedance the structure presents there."
            );
        }

        heading(&mut out, "CURRENTS AND LOCATION");
        let _ = writeln!(
            out,
            "                            DISTANCES IN WAVELENGTHS\n\n  SEG.  TAG    COORD. OF SEG. CENTER     SEG.            - - - CURRENT (AMPS) - - -\n  NO.   NO.     X         Y         Z      LENGTH     REAL        IMAG.       MAG.        PHASE"
        );
        for (s, i) in self.geom.segments.iter().zip(&sol.currents) {
            let c = s.center();
            let _ = writeln!(
                out,
                " {:5} {:4} {:9.4} {:9.4} {:9.4} {:9.5}{}{}{}{:>9.3}",
                s.index,
                s.tag,
                c.x / wlam,
                c.y / wlam,
                c.z / wlam,
                s.length() / wlam,
                col(i.re, 12),
                col(i.im, 12),
                col(i.norm(), 12),
                i.arg().to_degrees()
            );
        }

        heading(&mut out, "CHARGE DENSITIES");
        let _ = writeln!(
            out,
            "  SEG.  TAG    - - - CHARGE DENSITY (COULOMBS/METER) - - -\n  NO.   NO.     REAL        IMAG.       MAG.        PHASE"
        );
        for (s, q) in self.geom.segments.iter().zip(sol.charge_densities()) {
            let _ = writeln!(
                out,
                " {:5} {:4}{}{}{}{:>9.3}",
                s.index,
                s.tag,
                col(q.re, 12),
                col(q.im, 12),
                col(q.norm(), 12),
                q.arg().to_degrees()
            );
        }

        if !sol.patch_currents.is_empty() {
            heading(&mut out, "SURFACE PATCH CURRENTS");
            let _ = writeln!(
                out,
                " PATCH     - - - CURRENT DENSITY (AMPS/METER) - - -\n  NO.        EX (MAG, PHASE)         EY (MAG, PHASE)         EZ (MAG, PHASE)"
            );
            for (k, j) in sol.patch_currents.iter().enumerate() {
                let _ = write!(out, " {:5}", k + 1);
                for c in j {
                    let _ = write!(out, "{}{:>10.3}  ", col(c.norm(), 13), c.arg().to_degrees());
                }
                out.push('\n');
            }
        }

        heading(&mut out, "POWER BUDGET");
        let p = sol.power;
        for (label, v) in [
            ("INPUT POWER", p.input),
            ("RADIATED POWER", p.radiated()),
            ("STRUCTURE LOSS", p.structure_loss),
            ("NETWORK LOSS", p.network_loss),
        ] {
            let _ = writeln!(out, "{:40}{label:<15}= {} WATTS", "", sci(v, 4));
        }
        let _ = writeln!(
            out,
            "{:40}{:<15}= {:.2} PERCENT",
            "",
            "EFFICIENCY",
            p.efficiency_percent()
        );

        if let Ok(prog) = self.program() {
            for req in prog.patterns.iter().filter(|r| r.mode != 1) {
                self.pattern_table(&mut out, sol, req);
            }
        }
        out
    }

    fn pattern_table(&self, out: &mut String, sol: &Solution, req: &PatternRequest) {
        let Ok(prog) = self.program() else { return };
        let pat = prog.pattern(&self.geom, sol, req);
        heading(out, "RADIATION PATTERNS");
        let (a, b) = if req.vertical_horizontal {
            ("VERT.", "HOR.")
        } else {
            ("MAJOR", "MINOR")
        };
        let _ = writeln!(
            out,
            " - - ANGLES - -     - {} GAINS -   - - - POLARIZATION - - -   - - - E(THETA) - - -    - - - E(PHI) - - -\n  THETA    PHI      {a:<6} {b:<6} TOTAL    AXIAL     TILT  SENSE   MAGNITUDE    PHASE    MAGNITUDE    PHASE\n DEGREES  DEGREES   DB     DB     DB      RATIO     DEG.             VOLTS/M   DEGREES    VOLTS/M   DEGREES",
            if req.directive { "DIRECTIVE" } else { "POWER" }
        );
        for pt in &pat.points {
            let (g1, g2) = if req.vertical_horizontal {
                (pt.vertical, pt.horizontal)
            } else {
                (pt.major, pt.minor)
            };
            let sense = match pt.sense {
                Sense::None => "",
                Sense::Linear => "LINEAR",
                Sense::Right => "RIGHT",
                Sense::Left => "LEFT",
            };
            let e = |c: Complex64| format!("{}{:>9.2}", col(c.norm(), 12), c.arg().to_degrees());
            let _ = writeln!(
                out,
                "{:8.2} {:8.2} {:7.2}{:7.2}{:7.2}{:10.5}{:9.2}  {sense:<6}{}{}",
                pt.theta,
                pt.phi,
                g1,
                g2,
                pt.total,
                pt.axial_ratio,
                pt.tilt,
                e(pt.e_theta),
                e(pt.e_phi)
            );
        }
        if let Some((avg, solid)) = pat.average {
            let _ = writeln!(
                out,
                "\n{:20}AVERAGE POWER GAIN= {}       SOLID ANGLE USED IN AVERAGING=({:.4})*PI STERADIANS.",
                "",
                sci(avg, 4),
                solid
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testdata::DIPOLE;

    #[test]
    fn numbers_are_written_as_nec_writes_them() {
        assert_eq!(sci(72.08, 5), "7.20800E+01");
        assert_eq!(sci(-0.000_123_4, 4), "-1.2340E-04");
        assert_eq!(sci(0.0, 5), "0.00000E+00");
    }

    #[test]
    fn the_report_has_every_table() {
        let deck = DIPOLE.replace(
            "FR 0 1 0 0 300 0\n",
            "FR 0 1 0 0 300 0\nRP 0 3 1 1000 0 0 45 0\n",
        );
        let m = Model::new(&deck).expect("builds");
        let prog = m.program().expect("program");
        let f = prog.frequencies[0];
        let sol = m.solve(f, prog.options(f)).expect("solves");
        let r = m.report(&deck, &sol);
        for t in [
            "ANTENNA INPUT PARAMETERS",
            "CURRENTS AND LOCATION",
            "CHARGE DENSITIES",
            "POWER BUDGET",
            "RADIATION PATTERNS",
        ] {
            assert!(r.contains(t), "{t}");
        }
        // Nine segment rows, and the source row's impedance as NEC prints it.
        let currents = &r[r.find("CURRENTS AND LOCATION").expect("table")
            ..r.find("CHARGE DENSITIES").expect("table")];
        let rows = currents
            .lines()
            .filter(|l| l.trim_start().starts_with(|c: char| c.is_ascii_digit()))
            .count();
        assert_eq!(rows, 9);
        let z = sol.inputs[0].impedance;
        assert!(r.contains(&sci(z.re, 5)), "{}", sci(z.re, 5));
    }
}
