//! Nelder–Mead simplex search in the unit cube.
//!
//! Nelder & Mead, "A simplex method for function minimization", Computer Journal 7 (1965),
//! with the dimension-adaptive coefficients of Gao & Han, "Implementing the Nelder-Mead
//! simplex algorithm with adaptive parameters", Comput. Optim. Appl. 51 (2012), which keep the
//! method effective beyond a handful of variables. Trial points are clamped into the cube.
//! On convergence the simplex is rebuilt once around the best point, the usual guard against
//! a simplex that collapsed onto a face of the box or stalled.

use super::{Report, Run, as_f64};
use rayon::prelude::*;

/// Simplex vertices closer than this (a fraction of each range) have converged.
const X_TOL: f64 = 1e-6;
/// Costs closer than this (relative) have converged.
const F_TOL: f64 = 1e-9;

pub(super) fn run<G>(
    g: &G,
    u0: &[f64],
    step: f64,
    budget: usize,
    evals0: usize,
    report: &mut Report<'_>,
) -> Run
where
    G: Fn(&[f64]) -> f64 + Sync,
{
    let n = u0.len();
    let start: Vec<f64> = u0.iter().map(|u| u.clamp(0.0, 1.0)).collect();
    if n == 0 {
        let c = g(&start);
        return (start, c, evals0 + 1, true, false);
    }
    let nf = as_f64(n.max(2));
    let (alpha, gamma, rho, sigma) = (1.0, 1.0 + 2.0 / nf, 0.75 - 0.5 / nf, 1.0 - 1.0 / nf);
    let clamp = |v: Vec<f64>| -> Vec<f64> { v.into_iter().map(|u| u.clamp(0.0, 1.0)).collect() };
    let simplex = |centre: &[f64], step: f64| {
        let mut pts = vec![centre.to_vec()];
        for i in 0..n {
            let mut p = centre.to_vec();
            p[i] += if p[i] + step <= 1.0 { step } else { -step };
            pts.push(p);
        }
        pts
    };

    let mut pts = simplex(&start, step);
    let mut fs: Vec<f64> = pts.par_iter().map(|p| g(p)).collect();
    let mut evals = n + 1;
    let mut restarted = false;
    let mut iteration = 0;
    loop {
        let mut order: Vec<usize> = (0..=n).collect();
        order.sort_by(|&a, &b| fs[a].total_cmp(&fs[b]));
        pts = order.iter().map(|&i| pts[i].clone()).collect();
        fs = order.iter().map(|&i| fs[i]).collect();
        iteration += 1;
        if report(evals0 + evals, iteration, &pts[0], fs[0]).is_break() {
            return (pts[0].clone(), fs[0], evals0 + evals, false, true);
        }

        let f_spread = fs[n] - fs[0];
        let x_spread = pts[1..]
            .iter()
            .flat_map(|p| p.iter().zip(&pts[0]).map(|(a, b)| (a - b).abs()))
            .fold(0.0, f64::max);
        if f_spread <= F_TOL * (1.0 + fs[0].abs()) && (x_spread <= X_TOL || f_spread == 0.0) {
            if restarted || evals + n + 1 > budget {
                return (pts[0].clone(), fs[0], evals0 + evals, true, false);
            }
            restarted = true;
            let best = (pts[0].clone(), fs[0]);
            pts = simplex(&best.0, step * 0.5);
            fs = std::iter::once(best.1)
                .chain(pts[1..].par_iter().map(|p| g(p)).collect::<Vec<_>>())
                .collect();
            evals += n;
            continue;
        }
        if evals >= budget {
            return (pts[0].clone(), fs[0], evals0 + evals, false, false);
        }

        let mut c = vec![0.0; n];
        for p in &pts[..n] {
            for (ci, pi) in c.iter_mut().zip(p) {
                *ci += pi / as_f64(n);
            }
        }
        let along = |from: &[f64], to: &[f64], t: f64| -> Vec<f64> {
            clamp(from.iter().zip(to).map(|(a, b)| a + t * (b - a)).collect())
        };
        let xr = along(&c, &pts[n], -alpha);
        let fr = g(&xr);
        evals += 1;
        if fr < fs[0] {
            let xe = along(&c, &xr, gamma);
            let fe = g(&xe);
            evals += 1;
            (pts[n], fs[n]) = if fe < fr { (xe, fe) } else { (xr, fr) };
        } else if fr < fs[n - 1] {
            (pts[n], fs[n]) = (xr, fr);
        } else {
            let outside = fr < fs[n];
            let xc = if outside {
                along(&c, &xr, rho)
            } else {
                along(&c, &pts[n], rho)
            };
            let fc = g(&xc);
            evals += 1;
            if (outside && fc <= fr) || (!outside && fc < fs[n]) {
                (pts[n], fs[n]) = (xc, fc);
            } else {
                let best = pts[0].clone();
                for p in &mut pts[1..] {
                    *p = along(&best, p, sigma);
                }
                let shrunk: Vec<f64> = pts[1..].par_iter().map(|p| g(p)).collect();
                fs[1..].copy_from_slice(&shrunk);
                evals += n;
            }
        }
    }
}
