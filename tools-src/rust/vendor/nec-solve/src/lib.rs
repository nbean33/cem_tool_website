//! Filling and solving the NEC-2 wire interaction matrix.
//!
//! Scope so far: thin-wire structures in free space or over ground (perfect, reflection
//! coefficient with optional radial screen, or Sommerfeld), with loads (`LD`), transmission lines
//! and networks (`TL`, `NT`), driven by voltage sources (`EX` 0 and 5) or an incident field
//! (`EX` 1–4), producing segment currents, input impedance and the power budget, and from them
//! radiation patterns (`RP`; [`farfield`]) and near fields (`NE`, `NH`; [`nearfield`]). See
//! [`program::read_program`] for what a deck may contain. Decks using 4nec2's extension cards must be lowered first with
//! `nec_cards::lower_4nec2`.
//!
//! ```
//! use nec_cards::parse_deck;
//! use nec_geom::Geometry;
//! use nec_solve::{read_program, solve};
//!
//! // models/Example1.nec: a half-wave dipole in free space at 300 MHz.
//! let mut deck = parse_deck(
//!     "CE\nGW 1 9 0 -.2418 0 0 .2418 0 .0001\nGE 0\nEX 0 1 5 0 1 0\nFR 0 1 0 0 300 0\nEN\n",
//! )?;
//! deck.resolve()?;
//! let geom = Geometry::build(&deck)?;
//! let prog = read_program(&deck, &geom)?;
//!
//! let sol = solve(&geom, prog.frequencies[0], &prog.sources, Default::default())?;
//! let z = sol.inputs[0].impedance;
//! assert!((z.re - 72.0788).abs() < 0.01);   // a half-wave dipole's ~72 ohms
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod coupling;
pub mod excitation;
pub mod farfield;
pub mod fill;
pub mod linalg;
pub mod nearfield;
pub mod network;
pub mod ngf;
pub mod program;
pub mod solve;

pub use coupling::{Coupling, CouplingRow, MaximumCoupling, coupling};
pub use excitation::{IncidentField, IncidentGround, Polarization};
pub use farfield::{FarFieldGround, GainBasis, Pattern, PatternPoint, PatternRequest, Sense, pattern};
pub use nearfield::NearFieldRequest;
pub use network::{Network, NetworkKind};
pub use ngf::{GreensFunction, NgfError};
pub use program::{GroundSpec, IncidentSpec, Program, ProgramError, ScreenSpec, SegmentLoad, read_program, read_program_on};
pub use solve::{InputParameters, PowerBudget, Solution, SolveError, SolveOptions, SourceKind, VoltageSource, solve};

#[cfg(test)]
mod tests {
    use super::*;
    use nec_cards::parse_deck;
    use nec_geom::Geometry;

    fn run(src: &str) -> Solution {
        let mut deck = parse_deck(src).expect("parses");
        deck.resolve().expect("resolves");
        let geom = Geometry::build(&deck).expect("builds");
        let prog = read_program(&deck, &geom).expect("program");
        solve(
            &geom,
            prog.frequencies[0],
            &prog.sources,
            prog.options(prog.frequencies[0]),
        )
        .expect("solves")
    }

    #[test]
    fn example1_input_impedance_matches_the_reference_engine() {
        // models/Example1.nec. The reference engine (nec2dxs3k0.exe) prints
        //   IMPEDANCE (OHMS)  7.20788E+01 -1.67981E-03
        let sol = run("CM Example 1\nCE\nGW 1 9 0 -.2418 0 0 .2418 0 .0001\nGE 0\n\
                       EX 0 1 5 0 1 0\nFR 0 1 0 0 300 0\nEN\n");
        let z = sol.inputs[0].impedance;
        let engine = num_complex::Complex64::new(72.0788, -1.679_81e-3);
        // Compared relative to |Z|, not component by component. The reactance of a resonant
        // dipole is a near-zero residual, so it amplifies differences that are negligible
        // overall: ours is -1.73e-3 against the engine's -1.68e-3, a 7.6e-7 relative difference
        // in Z. That is the size of INTX's own integration error — tightening its tolerance from
        // 1e-4 moves our value by about the same amount — so it is noise, not a porting fault.
        let rel = (z - engine).norm() / engine.norm();
        assert!(rel < 1e-5, "Z = {z}, engine {engine}, relative difference {rel:.2e}");
        assert!((z.re - 72.0788).abs() < 5e-4, "R = {} (engine 72.0788)", z.re);
    }

    #[test]
    fn coincident_wires_match_the_reference_engine() {
        // A dipole with a parasitic wire entered twice. The coincident copies trip an
        // inconsistency inside NEC-2's fill (see nec_kernel::pieces_by_segment), which this
        // reproduces; summing the basis function's tails instead fails this comparison. The
        // reference engine (nec2dxs) prints
        //   IMPEDANCE (OHMS)  7.31875E+01  8.65091E+00
        let sol = run("CE\nGW 1 9 0 -0.24 0 0 0.24 0 0.001\n\
                       GW 2 3 0.2 -0.1 0 0.2 0.1 0 0.001\n\
                       GW 3 3 0.2 -0.1 0 0.2 0.1 0 0.001\nGE 0\n\
                       EX 0 1 5 0 1 0\nFR 0 1 0 0 299.7925 0\nEN\n");
        let z = sol.inputs[0].impedance;
        let engine = num_complex::Complex64::new(73.1875, 8.650_91);
        let rel = (z - engine).norm() / engine.norm();
        assert!(rel < 1e-5, "Z = {z}, engine {engine}, relative difference {rel:.2e}");
    }

    #[test]
    fn every_ground_type_matches_the_reference_engine() {
        // Impedances printed by the reference engine (nec2dxs) for each deck. Every ground
        // option takes a different path through EFLD: the image as is (perfect), Fresnel-weighted
        // (reflection coefficient, with and without a radial screen), and the Sommerfeld
        // correction from the interpolated grid (near) and Norton's formulas (far).
        let vdip = "CE\nGW 1 11 0 0 0.3 0 0 0.8 0.001\nGE 0\n";
        let cases: [(String, f64, f64); 7] = [
            (format!("{vdip}GN 1\nEX 0 1 6 0 1 0\nFR 0 1 0 0 320 0\nEN\n"), 100.971, 114.209),
            // GN 3 is undocumented; NEC fills the matrix exactly as for perfect ground.
            (format!("{vdip}GN 3 0 0 0 13 0.005\nEX 0 1 6 0 1 0\nFR 0 1 0 0 320 0\nEN\n"), 100.971, 114.209),
            (format!("{vdip}GN 0 0 0 0 13 0.005\nEX 0 1 6 0 1 0\nFR 0 1 0 0 320 0\nEN\n"), 101.892, 112.821),
            (
                "CE\nGW 1 11 -0.25 0 0.1 0.25 0 0.1 0.001\nGE 0\nGN 0 8 0 0 4 0.001 0.5 0.002\n\
                 EX 0 1 6 0 1 0\nFR 0 1 0 0 320 0\nEN\n"
                    .to_string(),
                48.8179,
                155.537,
            ),
            (format!("{vdip}GN 2 0 0 0 13 0.005\nEX 0 1 6 0 1 0\nFR 0 1 0 0 320 0\nEN\n"), 101.777, 113.564),
            (
                "CE\nGW 1 21 -1.5 0 0.4 1.5 0 0.4 0.002\nGE 0\nGN 2 0 0 0 13 0.005\n\
                 EX 0 1 11 0 1 0\nFR 0 1 0 0 320 0\nEN\n"
                    .to_string(),
                377.186,
                -475.516,
            ),
            (
                "CE\nGW 1 10 0 0 0 0 0 0.25 0.001\nGE 1\nGN 2 0 0 0 13 0.005\n\
                 EX 0 1 1 0 1 0\nFR 0 1 0 0 300 0\nEN\n"
                    .to_string(),
                54.2827,
                -102.211,
            ),
        ];
        for (deck, re, im) in cases {
            let z = run(&deck).inputs[0].impedance;
            let engine = num_complex::Complex64::new(re, im);
            let rel = (z - engine).norm() / engine.norm();
            assert!(rel < 1e-5, "Z = {z}, engine {engine} ({rel:.2e}) for\n{deck}");
        }
    }

    /// Two parallel dipoles, the first fed at its centre (segment 6); `cards` go after `GE`.
    fn pair(cards: &str, mhz: f64) -> String {
        format!(
            "CE\nGW 1 11 0 -0.25 0.6 0 0.25 0.6 0.001\nGW 2 11 0.15 -0.25 0.6 0.15 0.25 0.6 0.001\n\
             GE 0\n{cards}\nFR 0 1 0 0 {mhz} 0\nEN\n"
        )
    }

    #[test]
    fn every_load_type_matches_the_reference_engine() {
        // Impedances printed by the reference engine for each deck.
        let cases: [(&str, f64, f64, f64); 8] = [
            ("LD 0 1 6 6 10 5e-8 2e-12\nEX 0 1 6 0 1 0", 320.0, 100.474, 4.563_18),
            ("LD 1 2 6 6 1000 3e-8 1e-11\nEX 0 1 6 0 1 0", 320.0, 72.1375, 100.661),
            ("LD 2 2 0 0 20 1e-7 0\nEX 0 1 6 0 1 0", 280.0, 34.6622, 10.9453),
            ("LD 3 2 0 0 1e5 1e-6 1e-11\nEX 0 1 6 0 1 0", 320.0, 105.31, 119.32),
            ("LD 4 1 3 5 25 -40\nEX 0 1 6 0 1 0", 300.0, 92.094, -4.295_51),
            ("LD 5 0 0 0 5.8e7\nEX 0 1 6 0 1 0", 280.0, 19.7316, -12.3657),
            // A second LD after another card replaces the first set of loads...
            ("LD 4 1 6 6 1000 0\nEX 0 1 6 0 1 0\nLD 4 2 6 6 20 30", 320.0, 93.857, 143.954),
            // ...but LD -1 directly after an LD does not clear it: 1000 ohms remain.
            ("LD 4 1 6 6 1000 0\nLD -1\nEX 0 1 6 0 1 0", 300.0, 1051.66, 85.9234),
        ];
        for (cards, mhz, re, im) in cases {
            let deck = pair(cards, mhz);
            let z = run(&deck).inputs[0].impedance;
            let engine = num_complex::Complex64::new(re, im);
            let rel = (z - engine).norm() / engine.norm();
            assert!(rel < 1e-5, "Z = {z}, engine {engine} ({rel:.2e}) for\n{deck}");
        }
    }

    /// Two dipoles plus a short dummy wire (tag 9) far off to the side; `cards` go after `GE`.
    fn trio(cards: &str, mhz: f64) -> String {
        format!(
            "CE\nGW 1 11 0 -0.25 0.6 0 0.25 0.6 0.001\nGW 2 11 0.25 -0.24 0.6 0.25 0.24 0.6 0.001\n\
             GW 9 1 2 0 0.6 2 0.02 0.6 0.001\nGE 0\n{cards}\nFR 0 1 0 0 {mhz} 0\nEN\n"
        )
    }

    #[test]
    fn networks_and_slope_sources_match_the_reference_engine() {
        // Impedance at the first source, as the reference engine printed it.
        let cases: [(&str, f64, f64, f64); 6] = [
            // A source on the dummy wire, fed to the dipole through 1.3 m of 75-ohm line.
            ("EX 0 9 1 0 1 0\nTL 9 1 1 6 75 1.3", 300.0, 26.6142, -11.7279),
            ("EX 0 9 1 0 1 0\nTL 9 1 1 6 -75 1.3", 300.0, 26.6117, -11.7253),
            // One source feeding both dipoles through different lengths.
            ("EX 0 9 1 0 1 0\nTL 9 1 1 6 50 0.7\nTL 9 1 2 6 50 0.95", 300.0, 51.0726, 9.753_21),
            ("EX 0 9 1 0 1 0\nNT 9 1 1 6 0.01 -0.02 0 0.015 0.002 -0.01", 280.0, 29.2244, 27.4074),
            // EX 5, the current-slope-discontinuity source, alone and with a line attached.
            ("EX 5 1 6 0 1 0", 280.0, 49.3161, -4.115_33),
            ("EX 5 1 6 0 1 0\nTL 1 3 9 1 50 0.3", 320.0, 156.194, 171.224),
        ];
        for (cards, mhz, re, im) in cases {
            let deck = trio(cards, mhz);
            let z = run(&deck).inputs[0].impedance;
            let engine = num_complex::Complex64::new(re, im);
            let rel = (z - engine).norm() / engine.norm();
            assert!(rel < 1e-5, "Z = {z}, engine {engine} ({rel:.2e}) for\n{deck}");
        }
    }

    #[test]
    fn the_extended_thin_wire_kernel_matches_the_reference_engine() {
        // Fat wires under EK, each exercising a different end treatment, against the engine's
        // printed input impedance.
        let cases: [(&str, f64, f64, f64); 4] = [
            // A dipole of length/radius 4.5: smooth continuations and two free ends.
            ("GW 1 11 0 -0.25 0.6 0 0.25 0.6 0.01\nGE 0\nEK\nEX 0 1 6 0 1 0", 300.0, 102.688, 46.6211),
            // A radius step: the thin side's centre lies inside the fat neighbour's tube.
            (
                "GW 1 6 0 -0.25 0.6 0 0 0.6 0.008\nGW 2 6 0 0 0.6 0 0.25 0.6 0.004\nGE 0\nEK\n\
                 EX 0 1 5 0 1 0",
                320.0,
                140.019,
                88.6707,
            ),
            // A three-wire junction falls back to the thin-wire end term.
            (
                "GW 1 6 0 -0.25 0.6 0 0 0.6 0.006\nGW 2 6 0 0 0.6 0 0.25 0.6 0.006\n\
                 GW 3 5 0 0 0.6 0 0 0.85 0.006\nGE 0\nEK\nEX 0 3 2 0 1 0",
                300.0,
                30.741,
                10.4995,
            ),
            // A vertical monopole bonded to perfect ground.
            ("GW 1 10 0 0 0 0 0 0.25 0.008\nGE 1\nGN 1\nEK\nEX 0 1 1 0 1 0", 280.0, 39.4625, 7.037_42),
        ];
        for (cards, mhz, re, im) in cases {
            let deck = format!("CE\n{cards}\nFR 0 1 0 0 {mhz} 0\nEN\n");
            let z = run(&deck).inputs[0].impedance;
            let engine = num_complex::Complex64::new(re, im);
            let rel = (z - engine).norm() / engine.norm();
            assert!(rel < 1e-5, "Z = {z}, engine {engine} ({rel:.2e}) for\n{deck}");
        }
    }

    /// Solve a deck at its first frequency and compute its first RP card's pattern.
    fn first_pattern(src: &str) -> (Solution, Pattern) {
        let mut deck = parse_deck(src).expect("parses");
        deck.resolve().expect("resolves");
        let geom = Geometry::build(&deck).expect("builds");
        let prog = read_program(&deck, &geom).expect("program");
        let f = prog.frequencies[0];
        let sol = solve(&geom, f, &prog.sources, prog.options(f)).expect("solves");
        let pat = prog.pattern(&geom, &sol, &prog.patterns[0]);
        (sol, pat)
    }

    fn polar(mag: f64, deg: f64) -> num_complex::Complex64 {
        num_complex::Complex64::from_polar(mag, deg.to_radians())
    }

    #[test]
    fn free_space_pattern_matches_the_reference_engine() {
        // models/Example1.nec with RP 0 3 2 1001 90 0 10 45: vertical/horizontal power gain,
        // averaged. The engine prints, at theta 90 phi 0, horizontal 2.12 dB and
        // E(phi) 8.23169E-01 at -91.80 degrees; at theta 100 phi 45, -17.06 / -1.86 / -1.73 dB;
        // and AVERAGE POWER GAIN= 1.15609E+00.
        let (_, pat) = first_pattern(
            "CE\nGW 1 9 0 -.2418 0 0 .2418 0 .0001\nGE 0\nEX 0 1 5 0 1 0\nFR 0 1 0 0 300 0\n\
             RP 0 3 2 1001 90 0 10 45 0 0\nEN\n",
        );
        let p = &pat.points[0];
        assert!((p.horizontal - 2.12).abs() < 0.006, "{p:?}");
        assert!((p.e_phi - polar(0.823_169, -91.80)).norm() < 1e-4, "{}", p.e_phi);
        let p = &pat.points[4];
        assert!((p.theta, p.phi) == (100.0, 45.0));
        for (ours, engine) in [(p.vertical, -17.06), (p.horizontal, -1.86), (p.total, -1.73)] {
            assert!((ours - engine).abs() < 0.006, "{ours} vs {engine}");
        }
        let (avg, _) = pat.average.expect("averaged");
        assert!((avg - 1.156_09).abs() < 1e-5, "{avg}");
    }

    #[test]
    fn cliff_screen_and_ground_wave_patterns_match_the_reference_engine() {
        // A horizontal dipole 0.3 m up and a vertical wire, at 300 MHz, over reflection-
        // coefficient ground, with each far-field ground treatment. Engine values at theta 20
        // phi 0 (or, for the ground wave, 20 m out at 1 m height).
        let structure = "CE\nGW 1 11 0 -0.25 0.3 0 0.25 0.3 0.001\nGW 2 9 0.3 0 0.05 0.3 0 0.45 0.001\nGE 0\n";
        let tail = "EX 0 1 6 0 1 0\nFR 0 1 0 0 300 0\n";
        // RP 2: linear cliff 1.5 m out, 0.3 m down, to sea water.
        let (_, pat) = first_pattern(&format!(
            "{structure}GN 0 0 0 0 13 0.005\nGD 0 0 0 0 80 5 1.5 -0.3\n{tail}RP 2 10 4 1000 0 0 10 90 0 0\nEN\n"
        ));
        let p = &pat.points[2];
        assert!((p.horizontal - 5.28).abs() < 0.006, "{p:?}");
        assert!((p.e_phi - polar(0.878_732, -30.36)).norm() < 1e-4, "{}", p.e_phi);
        // RP 4: twelve 1.5 m radials.
        let (_, pat) = first_pattern(&format!(
            "{structure}GN 0 12 0 0 13 0.005 1.5 0.001\n{tail}RP 4 10 4 1000 0 0 10 90 0 0\nEN\n"
        ));
        let p = &pat.points[2];
        assert!((p.horizontal - 5.71).abs() < 0.006, "{p:?}");
        assert!((p.e_phi - polar(0.884_183, -36.93)).norm() < 1e-4, "{}", p.e_phi);
        // RP 1: fields near Sommerfeld ground, with the surface wave.
        let (_, pat) = first_pattern(&format!(
            "{structure}GN 2 0 0 0 13 0.005\n{tail}RP 1 4 3 0 0 0 0.5 60 20 0\nEN\n"
        ));
        let p = &pat.points[2];
        assert!((p.theta, p.phi) == (1.0, 0.0), "{p:?}");
        assert!((p.e_phi - polar(5.3402e-3, -61.10)).norm() < 1e-4 * 5.34e-3, "{}", p.e_phi);
    }

    #[test]
    fn near_fields_match_the_reference_engine() {
        // models/Example1.nec with NE and NH on a 2x2x2 grid. The engine prints, at
        // (0.05, -0.3, -0.1) m: EX 1.0022E+00 at 79.66, EY 2.7068E+00 at -141.72,
        // EZ 2.0045E+00 at -100.34; and at (0.05, 0, -0.1) m: HX 1.7601E-02 at 170.22,
        // HY 0, HZ 8.8006E-03 at 170.22.
        let src = "CE\nGW 1 9 0 -.2418 0 0 .2418 0 .0001\nGE 0\nEX 0 1 5 0 1 0\nFR 0 1 0 0 300 0\n\
                   NE 0 2 2 2 0.05 -0.3 -0.1 0.2 0.3 0.2\nNH 0 2 2 2 0.05 -0.3 -0.1 0.2 0.3 0.2\nEN\n";
        let mut deck = parse_deck(src).expect("parses");
        deck.resolve().expect("resolves");
        let geom = Geometry::build(&deck).expect("builds");
        let prog = read_program(&deck, &geom).expect("program");
        let f = prog.frequencies[0];
        let sol = solve(&geom, f, &prog.sources, prog.options(f)).expect("solves");
        let close = |ours: num_complex::Complex64, mag: f64, deg: f64| {
            (ours - polar(mag, deg)).norm() <= 2e-4 * mag.max(1e-12)
        };
        let e = prog.near_field(&geom, &sol, &prog.near_fields[0]);
        let (p, [ex, ey, ez]) = e[0];
        // NEC builds the grid by repeated addition, so 0.05 arrives as 0.04999...
        assert!((p[0] - 0.05).abs() < 1e-12 && (p[1] + 0.3).abs() < 1e-12 && (p[2] + 0.1).abs() < 1e-12);
        assert!(close(ex, 1.0022, 79.66) && close(ey, 2.7068, -141.72) && close(ez, 2.0045, -100.34), "{:?}", e[0]);
        let h = prog.near_field(&geom, &sol, &prog.near_fields[1]);
        let (p, [hx, hy, hz]) = h[2];
        assert!((p[0] - 0.05).abs() < 1e-12 && p[1].abs() < 1e-12 && (p[2] + 0.1).abs() < 1e-12, "{p:?}");
        assert!(close(hx, 1.7601e-2, 170.22) && hy.norm() < 1e-9 && close(hz, 8.8006e-3, 170.22), "{:?}", h[2]);
    }

    #[test]
    fn a_plane_wave_induces_the_reference_engines_currents() {
        // EX 1: linearly polarized plane wave from theta 30, phi 10, eta 20 degrees, on three
        // wires. The engine prints segment 15's current as -1.2021E-03 -8.4196E-04 A.
        let src = "CE\nGW 1 11 0 -0.25 0.6 0 0.25 0.6 0.001\nGW 2 11 0.25 -0.24 0.6 0.25 0.24 0.6 0.001\n\
                   GW 3 9 0.1 0.1 0.3 0.15 0.12 0.75 0.001\nGE 0\nEX 1 1 1 0 30 10 20 0 0 0\n\
                   FR 0 1 0 0 310 0\nEN\n";
        let mut deck = parse_deck(src).expect("parses");
        deck.resolve().expect("resolves");
        let geom = Geometry::build(&deck).expect("builds");
        let prog = read_program(&deck, &geom).expect("program");
        let fields = prog.incident_fields();
        assert_eq!(fields.len(), 1);
        let mut options = prog.options(prog.frequencies[0]);
        options.incident = Some(fields[0]);
        let sol = solve(&geom, prog.frequencies[0], &prog.sources, options).expect("solves");
        assert!(sol.inputs.is_empty(), "an incident field has no input impedance");
        let engine = num_complex::Complex64::new(-1.2021e-3, -8.4196e-4);
        let i = sol.currents[14];
        assert!((i - engine).norm() < 1e-4 * engine.norm(), "{i} vs {engine}");
    }

    #[test]
    fn a_source_written_as_zero_volts_drives_one_volt() {
        // NEC replaces |V| < 1e-20 with 1 V. Fed equally, the two parallel dipoles see the same
        // impedance; the engine prints 151.463 + 29.5577j for both, with 1 V on each.
        let sol = run(&pair("EX 0 1 6 0 1 0\nEX 0 2 6 0 0 0", 300.0));
        assert_eq!(sol.inputs[1].voltage, num_complex::Complex64::new(1.0, 0.0));
        let engine = num_complex::Complex64::new(151.463, 29.5577);
        for input in &sol.inputs {
            let rel = (input.impedance - engine).norm() / engine.norm();
            assert!(rel < 1e-5, "segment {}: {} vs {engine}", input.segment, input.impedance);
        }
    }

    #[test]
    fn current_distribution_is_symmetric_on_a_centre_fed_dipole() {
        let sol = run("CE\nGW 1 9 0 -.2418 0 0 .2418 0 .0001\nGE 0\n\
                       EX 0 1 5 0 1 0\nFR 0 1 0 0 300 0\nEN\n");
        let c = &sol.currents;
        // Not exact: INTX's adaptive steps are not mirror-symmetric, so mirror-image segments
        // integrate over slightly different sample points. Observed asymmetry is ~4e-9.
        for k in 0..4 {
            assert!(
                (c[k] - c[8 - k]).norm() < 1e-7 * c[4].norm(),
                "segments {} and {} differ",
                k + 1,
                9 - k
            );
        }
        // And peaks at the feed.
        assert!(c.iter().all(|x| x.norm() <= c[4].norm() + 1e-15));
    }

    #[test]
    fn unsupported_cards_are_refused_not_ignored() {
        let mut deck = parse_deck(
            "CE\nGW 1 9 0 -.2418 0 0 .2418 0 .0001\nGE 0\nGN 2 4 0 0 13 .005 .5 .001\n\
             EX 0 1 5 0 1 0\nFR 0 1 0 0 300 0\nEN\n",
        )
        .expect("parses");
        deck.resolve().expect("resolves");
        let geom = Geometry::build(&deck).expect("builds");
        assert!(matches!(
            read_program(&deck, &geom),
            Err(ProgramError::Unsupported { .. })
        ));
    }
}
