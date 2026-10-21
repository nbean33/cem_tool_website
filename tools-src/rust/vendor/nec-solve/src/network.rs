//! Non-radiating networks between segments: transmission lines (`TL`) and two-port admittance
//! networks (`NT`), solved as NEC's `NETWK` does.
//!
//! A network connects two segments ("ports") through a short-circuit admittance matrix
//! `[[Y11, Y12], [Y12, Y22]]`. It does not radiate and is not part of the interaction matrix;
//! instead, each port segment gets an unknown applied field (the network's voltage across that
//! segment), and a small system — one equation per port — makes the current the structure
//! carries at each port equal to the current the network draws there.
//!
//! The structure's response to a unit field at each port comes from the factored interaction
//! matrix, so the cost is one extra solve per port. A port that is also a voltage source is not
//! an unknown: its voltage is known, so it moves to the right-hand side, and the current the
//! network draws from it is added to the source current reported at the input.

use faer::Mat;
use faer::linalg::solvers::Solve;
use num_complex::Complex64;

type C = Complex64;

/// `TP` as `NETWK` writes it, `DATA TP/6.283185308D+0/`.
const TP: f64 = 6.283_185_308;

/// One network between two segments.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Network {
    /// 0-based segment index of port 1.
    pub port1: usize,
    /// 0-based segment index of port 2.
    pub port2: usize,
    /// What connects them.
    pub kind: NetworkKind,
}

/// What a network is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NetworkKind {
    /// `NT`: a two-port given by its short-circuit admittances, siemens.
    Admittance {
        /// `Y11`.
        y11: C,
        /// `Y12` (= `Y21`).
        y12: C,
        /// `Y22`.
        y22: C,
    },
    /// `TL`: a lossless transmission line, with optional shunt admittances at its ends.
    Line {
        /// Characteristic impedance, ohms.
        z0: f64,
        /// Length in metres; zero means the distance between the two segment centres.
        length_m: f64,
        /// A crossed line (negative `Z0` on the card): the conductors swap ends.
        crossed: bool,
        /// Shunt admittance across port 1, siemens.
        shunt1: C,
        /// Shunt admittance across port 2, siemens.
        shunt2: C,
    },
}

impl Network {
    /// `(Y11, Y12, Y22)` at wavelength `wlam` (metres). `distance_m` is the distance between
    /// the port segments' centres, used for a line of unspecified length.
    #[must_use]
    pub fn admittances(&self, wlam: f64, distance_m: f64) -> (C, C, C) {
        match self.kind {
            NetworkKind::Admittance { y11, y12, y22 } => (y11, y12, y22),
            NetworkKind::Line {
                z0,
                length_m,
                crossed,
                shunt1,
                shunt2,
            } => {
                // NEC fills an unspecified length in from the centres on the first frequency;
                // the distance in metres is the same at every frequency.
                let length = if length_m <= 0.0 { distance_m } else { length_m };
                let bl = TP * length / wlam;
                let y12i = 1.0 / (z0 * bl.sin());
                let y_end = -y12i * bl.cos();
                // A crossed line negates the transfer admittance (NTYP = 3).
                let y12 = C::new(0.0, y12i);
                let y12 = if crossed { -y12 } else { y12 };
                (
                    C::new(shunt1.re, y_end + shunt1.im),
                    y12,
                    C::new(shunt2.re, y_end + shunt2.im),
                )
            }
        }
    }
}

/// A network port: an unknown network voltage, or a voltage source's known one.
#[derive(Clone, Copy)]
enum Port {
    /// Index into the network equations (`NTEQA`).
    Eq(usize),
    /// Index into the source ports (`NTSCA`).
    Src(usize),
}

/// The solved network: the structure's excitation with the network's fields applied, and the
/// network current drawn from each voltage source.
pub(crate) struct NetworkSolution {
    /// The right-hand side to solve the structure with.
    pub rhs: Mat<C>,
    /// For each source segment that is also a network port, the network's current there
    /// (in the kernel's units, before multiplying by the wavelength).
    pub source_currents: Vec<(usize, C)>,
    /// Ports that are not sources: segment and the network field applied there (`RHNT`); the
    /// port voltage is this times the segment length in metres.
    pub eq_ports: Vec<(usize, C)>,
    /// Ports that are voltage sources: segment and source voltage (`VSRC`).
    pub src_ports: Vec<(usize, C)>,
}

/// `NETWK`.
///
/// `currents(rhs)` solves the structure for a right-hand side and returns segment-centre
/// currents in kernel units (`SOLGF` then `CABC`). `seg_len` is each segment's length in
/// wavelengths, `sources` the `(segment, voltage)` of each voltage source in deck order.
pub(crate) fn solve_networks(
    networks: &[Network],
    sources: &[(usize, C)],
    seg_len: &[f64],
    distance_m: impl Fn(usize, usize) -> f64,
    wlam: f64,
    excitation: &Mat<C>,
    currents: impl Fn(&Mat<C>) -> Vec<C>,
) -> NetworkSolution {
    let mut eq: Vec<usize> = Vec::new();
    let mut src: Vec<(usize, C)> = Vec::new();
    let mut port = |seg: usize| -> Port {
        // The first voltage source on the segment, as NETWK's search finds it.
        if let Some(&(_, v)) = sources.iter().find(|(s, _)| *s == seg) {
            let i = src.iter().position(|(s, _)| *s == seg).unwrap_or_else(|| {
                src.push((seg, v));
                src.len() - 1
            });
            Port::Src(i)
        } else {
            let i = eq.iter().position(|s| *s == seg).unwrap_or_else(|| {
                eq.push(seg);
                eq.len() - 1
            });
            Port::Eq(i)
        }
    };
    let ports: Vec<(Port, Port)> = networks.iter().map(|n| (port(n.port1), port(n.port2))).collect();

    let (neq, nsrc) = (eq.len(), src.len());
    // CMN, as NEC fills it: equation rows, with source-port columns kept apart.
    let mut cmn = vec![vec![C::new(0.0, 0.0); neq]; neq];
    let mut cmn_src = vec![vec![C::new(0.0, 0.0); nsrc]; neq];
    let mut rhnx_eq = vec![C::new(0.0, 0.0); neq];
    let mut rhnx_src = vec![C::new(0.0, 0.0); nsrc];

    for (net, &(p1, p2)) in networks.iter().zip(&ports) {
        let (y11, y12, y22) = net.admittances(wlam, distance_m(net.port1, net.port2));
        let mut add = |row: usize, col: Port, v: C| match col {
            Port::Eq(c) => cmn[row][c] += v,
            Port::Src(c) => cmn_src[row][c] += v,
        };
        let mut rhs = |p: Port, v: C| match p {
            Port::Eq(i) => rhnx_eq[i] += v,
            Port::Src(i) => rhnx_src[i] += v,
        };
        match p1 {
            Port::Eq(r1) => {
                let si = seg_len[net.port1];
                add(r1, Port::Eq(r1), -(y11 * si));
                add(r1, p2, -(y12 * si));
            }
            Port::Src(s1) => {
                let v = src[s1].1;
                rhs(p1, y11 * v / wlam);
                rhs(p2, y12 * v / wlam);
            }
        }
        match p2 {
            Port::Eq(r2) => {
                let si = seg_len[net.port2];
                add(r2, Port::Eq(r2), -(y22 * si));
                add(r2, p1, -(y12 * si));
            }
            Port::Src(s2) => {
                let v = src[s2].1;
                rhs(p1, y12 * v / wlam);
                rhs(p2, y22 * v / wlam);
            }
        }
    }

    // The structure's response at every port to a unit field at each port.
    let n = excitation.nrows();
    for (i, &seg) in eq.iter().enumerate() {
        let mut unit: Mat<C> = Mat::zeros(n, 1);
        unit[(seg, 0)] = C::new(1.0, 0.0);
        let c = currents(&unit);
        for (j, &sj) in eq.iter().enumerate() {
            cmn[i][j] += c[sj];
        }
    }

    // FACTR un-transposes CMN before factoring, so the system solved is CMNᵀ·x = b, where b is
    // the network's own source terms plus the structure's current at each port.
    let mut x = vec![C::new(0.0, 0.0); neq];
    if neq > 0 {
        let c0 = currents(excitation);
        let a = Mat::from_fn(neq, neq, |i, j| cmn[j][i]);
        let b = Mat::from_fn(neq, 1, |i, _| rhnx_eq[i] + c0[eq[i]]);
        let sol = a.partial_piv_lu().solve(&b);
        for (i, xi) in x.iter_mut().enumerate() {
            *xi = sol[(i, 0)];
        }
    }

    // Apply the network fields to the structure.
    let mut rhs = excitation.clone();
    for (i, &seg) in eq.iter().enumerate() {
        rhs[(seg, 0)] -= x[i];
    }

    // The current each source port delivers into the network.
    let source_currents = src
        .iter()
        .enumerate()
        .map(|(t, &(seg, _))| {
            let drawn = (0..neq).fold(rhnx_src[t], |acc, j| acc - cmn_src[j][t] * x[j]);
            (seg, drawn)
        })
        .collect();

    NetworkSolution {
        rhs,
        source_currents,
        eq_ports: eq.iter().copied().zip(x.iter().copied()).collect(),
        src_ports: src,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_quarter_wave_line_has_the_textbook_admittances() {
        // A lossless line a quarter wavelength long: Y11 = Y22 = −j·cot(π/2)/Z0 = 0 and
        // Y12 = j/(Z0·sin(π/2)) = j/Z0.
        let net = Network {
            port1: 0,
            port2: 1,
            kind: NetworkKind::Line {
                z0: 50.0,
                length_m: 2.5,
                crossed: false,
                shunt1: C::new(0.0, 0.0),
                shunt2: C::new(0.0, 0.0),
            },
        };
        let (y11, y12, y22) = net.admittances(10.0, 0.0);
        assert!(y11.norm() < 1e-9 && y22.norm() < 1e-9, "{y11} {y22}");
        assert!((y12 - C::new(0.0, 0.02)).norm() < 1e-9, "{y12}");
    }
}
