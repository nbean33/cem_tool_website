//! Physical invariants the solver must respect whatever the structure: reciprocity, power
//! conservation, and indifference to where the structure sits and how it is turned.
//!
//! These are independent of NEC's formulation, so they catch errors the engine comparison
//! cannot — an error NEC shares, or one in a path the corpus never exercises. Each runs over a
//! handful of structures and parameters rather than one hand-picked case.

use nec_cards::parse_deck;
use nec_core::Frequency;
use nec_geom::Geometry;
use nec_solve::{PatternRequest, Program, Solution, SourceKind, VoltageSource, read_program, solve};
use num_complex::Complex64;

type C = Complex64;

fn model(src: &str) -> (Geometry, Program) {
    let mut d = parse_deck(src).expect("parses");
    d.resolve().expect("resolves");
    let g = Geometry::build(&d).expect("builds");
    let p = read_program(&d, &g).expect("program");
    (g, p)
}

fn drive(g: &Geometry, p: &Program, seg: usize) -> Solution {
    let f = p.frequencies[0];
    let src = VoltageSource {
        segment: seg,
        voltage: C::new(1.0, 0.0),
        kind: SourceKind::AppliedField,
    };
    solve(g, f, &[src], p.options(f)).expect("solves")
}

/// The deck with every wire's segments tripled (a centre segment stays a centre segment), and
/// the one centre-segment load moved with it.
fn refine(src: &str) -> String {
    src.lines()
        .map(|l| {
            let f: Vec<&str> = l.split_whitespace().collect();
            match f.first() {
                Some(&"GW") => {
                    let n: usize = f[2].parse().expect("segments");
                    let mut g = f.clone();
                    let t = (3 * n).to_string();
                    g[2] = &t;
                    g.join(" ")
                }
                Some(&"LD") => l.replace(" 6 6 ", " 17 17 "),
                _ => l.to_string(),
            }
        })
        .collect::<Vec<_>>()
        .join("
")
        + "
"
}

/// The transfer admittance between two ports is the same measured either way:
/// `I₂/V₁ = I₁/V₂`. NEC tests the field only at segment centres (point matching, not Galerkin),
/// so its matrix is not exactly symmetric: the agreement is close rather than exact, and gets
/// closer as the segmentation is refined — which a genuine error would not.
#[test]
fn transfer_admittances_are_reciprocal() {
    let cases = [
        // Two parallel dipoles, unequal lengths and radii, in free space.
        "CE\nGW 1 11 0 -.25 0 0 .25 0 .001\nGW 2 9 .3 -.2 .1 .3 .2 .1 .003\nGE 0\nEX 0 1 6 0 1\nFR 0 1 0 0 299.8\nEN\n",
        // Skewed wires over perfect ground.
        "CE\nGW 1 11 0 -.25 .5 0 .25 .5 .001\nGW 2 9 .3 -.2 .3 .5 .2 .6 .002\nGE 0\nGN 1\nEX 0 1 6 0 1\nFR 0 1 0 0 299.8\nEN\n",
        // A loaded Yagi-like pair over reflection-coefficient ground.
        "CE\nGW 1 11 0 -.24 .4 0 .24 .4 .002\nGW 2 11 .2 -.25 .4 .2 .25 .4 .002\nGE 0\nGN 0 0 0 0 13 .005\n\
         LD 4 2 6 6 50 20\nEX 0 1 6 0 1\nFR 0 1 0 0 299.8\nEN\n",
    ];
    let asymmetry = |src: &str| {
        let (g, p) = model(src);
        // The centre segment of each wire.
        let a = g.by_tag(1).nth(g.by_tag(1).count() / 2).expect("wire 1").index - 1;
        let b = g.by_tag(2).nth(g.by_tag(2).count() / 2).expect("wire 2").index - 1;
        let y21 = drive(&g, &p, a).currents[b];
        let y12 = drive(&g, &p, b).currents[a];
        (y21 - y12).norm() / y21.norm()
    };
    for src in cases {
        let coarse = asymmetry(src);
        assert!(coarse < 5e-3, "{coarse:.1e}\n{src}");
        // The asymmetry is the point-matching error, so it falls as the segments shrink.
        let fine = asymmetry(&refine(src));
        assert!(fine < coarse / 2.0, "asymmetry {coarse:.1e} -> {fine:.1e} with segments thirded\n{src}");
    }
}

/// With the average gain integrated over the whole sphere (or the half above a perfect ground),
/// the power radiated into the far field is the input power less the losses: NEC's average power
/// gain equals the efficiency.
#[test]
fn radiated_power_balances_the_input() {
    let cases = [
        // A lossless dipole, then one with a resistive load, in free space.
        ("CE\nGW 1 21 0 -.24 0 0 .24 0 .001\nGE 0\nEX 0 1 11 0 1\nFR 0 1 0 0 299.8\nEN\n", false),
        ("CE\nGW 1 21 0 -.24 0 0 .24 0 .001\nGE 0\nLD 0 1 5 5 30 0 0\nEX 0 1 11 0 1\nFR 0 1 0 0 299.8\nEN\n", false),
        // A copper-wire loop, and a monopole over perfect ground (half sphere).
        ("CE\nGA 1 24 .16 0 360 .001\nGE 0\nLD 5 1 0 0 5.8e7\nEX 0 1 1 0 1\nFR 0 1 0 0 299.8\nEN\n", false),
        ("CE\nGW 1 11 0 0 0 0 0 .24 .001\nGE 1\nGN 1\nEX 0 1 1 0 1\nFR 0 1 0 0 299.8\nEN\n", true),
    ];
    for (src, half) in cases {
        let (g, p) = model(src);
        let f = p.frequencies[0];
        let sol = solve(&g, f, &p.sources, p.options(f)).expect("solves");
        let n_theta = if half { 46 } else { 91 };
        let req = PatternRequest::from_fields(0, n_theta, 91, 1, [0.0, 0.0, 2.0, 4.0, 0.0, 0.0]);
        let avg = p.pattern(&g, &sol, &req).average.expect("averaged").0;
        let eff = sol.power.efficiency_percent() / 100.0;
        // Over a perfect ground NEC averages over the upper half sphere only, so the average
        // power gain there is twice the efficiency.
        let expected = if half { 2.0 * eff } else { eff };
        assert!((avg / expected - 1.0).abs() < 5e-3, "average gain {avg} vs {expected}\n{src}");
    }
}

/// Moving a structure in free space changes nothing; neither does turning it, beyond the
/// integration tolerance (the kernel integrates along each segment adaptively, and turning a
/// structure changes where the integration lands to within NEC's 1e-4 convergence test).
#[test]
fn moving_or_turning_the_structure_changes_nothing() {
    let base = "GW 1 15 0 -.24 0 0 .24 0 .001\nGW 2 9 .15 -.2 .05 .15 .2 .05 .0015\nGW 3 5 0 .24 0 .15 .2 .05 .001\n";
    let deck = |gm: &str| format!("CE\n{base}{gm}GE 0\nEX 0 1 8 0 1\nFR 0 1 0 0 299.8\nEN\n");
    let z = |gm: &str| {
        let (g, p) = model(&deck(gm));
        let f: Frequency = p.frequencies[0];
        solve(&g, f, &p.sources, p.options(f)).expect("solves").inputs[0].impedance
    };
    let reference = z("");
    for (gm, tol) in [
        ("GM 0 0 0 0 0 3.7 -12.5 40.2\n", 1e-9),
        ("GM 0 0 0 0 0 1000 1000 1000\n", 1e-7),
        ("GM 0 0 0 0 90 0 0 0\n", 2e-5),
        ("GM 0 0 33 -71 128 0.4 0.2 -0.9\n", 2e-5),
    ] {
        let moved = z(gm);
        let d = (moved - reference).norm() / reference.norm();
        assert!(d < tol, "{gm}: {moved} vs {reference}: {d:.1e}");
    }
}
