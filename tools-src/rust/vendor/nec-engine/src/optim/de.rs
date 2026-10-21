//! Differential evolution in the unit cube.
//!
//! Storn & Price, "Differential Evolution – A Simple and Efficient Heuristic for Global
//! Optimization over Continuous Spaces", J. Global Optimization 11 (1997): the `DE/rand/1/bin`
//! scheme, with the mutation factor dithered per generation over `[0.5, 1)` (which the authors
//! later recommended over a fixed one) and a crossover rate of 0.7. The first population is a
//! Latin hypercube containing the starting point, so the user's design is never lost and the
//! rest of the box is sampled evenly. Each generation's trial vectors are evaluated in
//! parallel.

use super::{Report, Rng, Run, as_f64};
use rayon::prelude::*;

const CROSSOVER: f64 = 0.7;
/// The population has converged when its costs' standard deviation is below this fraction of
/// their mean (plus a tiny absolute floor).
const TOL: f64 = 1e-4;

pub(super) fn run<G>(
    g: &G,
    u0: &[f64],
    budget: usize,
    rng: &mut Rng,
    report: &mut Report<'_>,
) -> Run
where
    G: Fn(&[f64]) -> f64 + Sync,
{
    let n = u0.len();
    let np = (15 * n).clamp(15, 60);

    // Latin hypercube: each variable's range cut into `np` strata, one sample per stratum.
    let mut pop = vec![vec![0.0; n]; np];
    for d in 0..n {
        let mut strata: Vec<usize> = (0..np).collect();
        for i in (1..np).rev() {
            strata.swap(i, rng.below(i + 1));
        }
        for (p, s) in pop.iter_mut().zip(strata) {
            p[d] = (as_f64(s) + rng.uniform()) / as_f64(np);
        }
    }
    pop[0] = u0.iter().map(|u| u.clamp(0.0, 1.0)).collect();
    let mut fs: Vec<f64> = pop.par_iter().map(|p| g(p)).collect();
    let mut evals = np;

    let mut generation = 0;
    loop {
        let best = (0..np)
            .min_by(|&a, &b| fs[a].total_cmp(&fs[b]))
            .unwrap_or(0);
        generation += 1;
        if report(evals, generation, &pop[best], fs[best]).is_break() {
            return (pop[best].clone(), fs[best], evals, false, true);
        }
        let mean = fs.iter().sum::<f64>() / as_f64(np);
        let std = (fs.iter().map(|f| (f - mean).powi(2)).sum::<f64>() / as_f64(np)).sqrt();
        if std <= 1e-12 + TOL * mean.abs() {
            return (pop[best].clone(), fs[best], evals, true, false);
        }
        if evals + np > budget {
            return (pop[best].clone(), fs[best], evals, false, false);
        }

        let f = 0.5 + 0.5 * rng.uniform();
        let trials: Vec<Vec<f64>> = (0..np)
            .map(|i| {
                let mut pick = |not: &[usize]| loop {
                    let r = rng.below(np);
                    if !not.contains(&r) {
                        return r;
                    }
                };
                let r1 = pick(&[i]);
                let r2 = pick(&[i, r1]);
                let r3 = pick(&[i, r1, r2]);
                let forced = rng.below(n);
                (0..n)
                    .map(|d| {
                        if d == forced || rng.uniform() < CROSSOVER {
                            let v = pop[r1][d] + f * (pop[r2][d] - pop[r3][d]);
                            // Outside the box, resample the coordinate uniformly.
                            if (0.0..=1.0).contains(&v) {
                                v
                            } else {
                                rng.uniform()
                            }
                        } else {
                            pop[i][d]
                        }
                    })
                    .collect()
            })
            .collect();
        let ft: Vec<f64> = trials.par_iter().map(|t| g(t)).collect();
        evals += np;
        for (i, (t, c)) in trials.into_iter().zip(ft).enumerate() {
            if c <= fs[i] {
                pop[i] = t;
                fs[i] = c;
            }
        }
    }
}
