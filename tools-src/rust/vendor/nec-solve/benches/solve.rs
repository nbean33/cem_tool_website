//! Matrix fill, LU factorization and a complete solve at the corpus's median, p90 and p99 sizes
//! (85, 321 and 2100 segments). Run with `cargo bench -p nec-solve`.
//!
//! The structures are arrays of parallel half-wave dipoles, so every segment interacts with
//! every other at a realistic range of distances.

#![allow(missing_docs)] // criterion_group! generates an undocumented public function.

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use nec_cards::parse_deck;
use nec_geom::Geometry;
use nec_kernel::{Ground, ImageGround, pieces_by_segment, scale, scale_patches};
use nec_solve::fill::{Structure, fill};
use nec_solve::linalg::Factors;
use nec_solve::{Program, read_program, solve};
use num_complex::Complex64;
use std::hint::black_box;

/// `dipoles` dipoles of `segments` segments each, in a row 0.3 λ apart at 300 MHz.
fn array(dipoles: usize, segments: usize) -> (Geometry, Program) {
    let mut src = String::from("CE\n");
    for k in 0..dipoles {
        #[allow(clippy::cast_precision_loss)]
        let x = 0.3 * k as f64;
        src.push_str(&format!("GW {} {segments} {x} -.24 0 {x} .24 0 .001\n", k + 1));
    }
    src.push_str(&format!("GE 0\nEX 0 1 {} 0 1\nFR 0 1 0 0 299.8\nEN\n", segments / 2 + 1));
    let mut d = parse_deck(&src).expect("parses");
    d.resolve().expect("resolves");
    let g = Geometry::build(&d).expect("builds");
    let p = read_program(&d, &g).expect("program");
    (g, p)
}

fn benches(c: &mut Criterion) {
    let sizes = [(5, 17), (15, 21), (70, 30)]; // 85, 315 (p90 ~ 321), 2100 segments
    let mut group = c.benchmark_group("nec-solve");
    group.sample_size(10);
    for (dipoles, segments) in sizes {
        let (g, p) = array(dipoles, segments);
        let n = g.len();
        let f = p.frequencies[0];
        let segs = scale(&g, f);
        let patches = scale_patches(&g, f);
        let by_segment = pieces_by_segment(&segs, &g.connectivity);
        let zarray = vec![Complex64::new(0.0, 0.0); n];
        let st = Structure {
            segs: &segs,
            patches: &patches,
            by_segment: &by_segment,
            connectivity: &g.connectivity,
            rkh: nec_kernel::consts::DEFAULT_RKH,
            ground: &Ground::FreeSpace,
            image: &ImageGround::None,
            extended: None,
            zarray: &zarray,
        };
        group.bench_with_input(BenchmarkId::new("fill", n), &st, |b, st| b.iter(|| black_box(fill(st))));
        let z = fill(&st);
        group.bench_with_input(BenchmarkId::new("lu", n), &z, |b, z| b.iter(|| black_box(Factors::new(z))));
        group.bench_with_input(BenchmarkId::new("solve", n), &(), |b, ()| {
            b.iter(|| black_box(solve(&g, f, &p.sources, p.options(f)).expect("solves")));
        });
    }
    group.finish();
}

criterion_group!(nec_solve_benches, benches);
criterion_main!(nec_solve_benches);
