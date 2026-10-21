//! Radiation patterns (`RP`; NEC's `RDPAT`, `FFLD`, `GFLD`).
//!
//! The far field of a wire structure is a sum over segments of each segment's current, integrated
//! along it in closed form (the current is `A + B·sin ks + C·cos ks`), times the phase of the
//! segment's position. Over ground, the image of the structure adds in, weighted by Fresnel
//! reflection coefficients — or, for the cliff and radial-screen options, coefficients that vary
//! with where each segment's reflection meets the ground. `RP` mode 1 instead evaluates the field
//! at a finite distance including Norton's ground (surface) wave.
//!
//! From the fields `RDPAT` derives gains (power or directive; major/minor axis or
//! vertical/horizontal; total), the polarization ellipse (axial ratio, tilt, sense), optional
//! normalized gain, and the average power gain over the angles requested.

use nec_kernel::cmath::{cdiv, csqrt, rdiv, single};
use nec_kernel::{ImageGround, RadialScreen, SurfacePatch, WireSeg};
use num_complex::Complex64;

type C = Complex64;

use nec_kernel::consts::{PI, TP};
/// Degrees to radians and back, as `RDPAT` writes them (`TA`, `TD`): NEC's truncated literals.
#[allow(clippy::approx_constant)]
const TA: f64 = 1.745_329_252e-2;
#[allow(clippy::approx_constant)]
const TD: f64 = 57.295_779_51;
/// `DATA CONSX/0.,-29.97922085D+0/` in `FFLD`: `−jη/(4π)`.
const CONST: C = C::new(0.0, -29.979_220_85);

/// What an `RP` card asks for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PatternRequest {
    /// `I1`: 0 normal; 1 fields near the ground with the surface wave; 2/3 linear/circular cliff;
    /// 4 radial-wire screen; 5/6 screen plus linear/circular cliff.
    pub mode: i32,
    /// Number of θ values (or, in mode 1, of heights).
    pub n_theta: usize,
    /// Number of φ values.
    pub n_phi: usize,
    /// First θ, degrees (mode 1: first height z, metres).
    pub theta0: f64,
    /// First φ, degrees.
    pub phi0: f64,
    /// θ step (mode 1: height step).
    pub dtheta: f64,
    /// φ step.
    pub dphi: f64,
    /// Range in metres; zero for the far field without the `exp(−jkR)/R` factor.
    pub range_m: f64,
    /// Gain normalization factor, dB; zero to normalize to the maximum.
    pub gain_norm_db: f64,
    /// `XNDA` X: vertical/horizontal rather than major/minor-axis gains.
    pub vertical_horizontal: bool,
    /// `XNDA` N: which gain to normalize (1 major, 2 minor, 3 vertical, 4 horizontal, 5 total), or 0.
    pub normalize: u8,
    /// `XNDA` D: directive rather than power gain.
    pub directive: bool,
    /// `XNDA` A: 0 none, 1 average gain, 2 average gain without printing the pattern.
    pub average: u8,
}

impl PatternRequest {
    /// Decode an `RP` card's fields as NEC's main program does, including its constraints:
    /// averaging needs at least two angles in each direction and is not available in mode 1.
    #[must_use]
    pub fn from_fields(mode: i32, n_theta: i32, n_phi: i32, xnda: i32, f: [f64; 6]) -> Self {
        let n_theta = usize::try_from(n_theta.max(1)).unwrap_or(1);
        let n_phi = usize::try_from(n_phi.max(1)).unwrap_or(1);
        let ipd = xnda / 10;
        let mut iavp = xnda - ipd * 10;
        let inor = ipd / 10;
        let ipd = ipd - inor * 10;
        let iax = inor / 10;
        let inor = inor - iax * 10;
        if n_theta < 2 || n_phi < 2 || mode == 1 {
            iavp = 0;
        }
        Self {
            mode,
            n_theta,
            n_phi,
            theta0: f[0],
            phi0: f[1],
            dtheta: f[2],
            dphi: f[3],
            range_m: f[4],
            gain_norm_db: f[5],
            vertical_horizontal: iax != 0,
            normalize: u8::try_from(inor.clamp(0, 5)).unwrap_or(0),
            directive: ipd != 0,
            average: u8::try_from(iavp.clamp(0, 2)).unwrap_or(0),
        }
    }
}

/// The ground as the far-field routines see it, at one frequency.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FarFieldGround {
    /// Any ground at all (`KSYMP = 2`).
    pub present: bool,
    /// Perfect ground (`IPERF = 1`): both reflection coefficients −1.
    pub perfect: bool,
    /// `ZRATI = 1/√ε_c` of the ground under the antenna.
    pub zrati: C,
    /// A second medium (`GD`, or `GN` fields 3–6): edge distance and height in wavelengths, and
    /// its `ZRATI`.
    pub second: Option<(f64, f64, C)>,
    /// A radial-wire screen.
    pub screen: Option<RadialScreen>,
}

impl FarFieldGround {
    /// The ground as the image-based routines see it (see [`ImageGround`]).
    #[must_use]
    pub fn image(&self) -> ImageGround {
        if !self.present {
            ImageGround::None
        } else if self.perfect {
            ImageGround::Perfect
        } else {
            ImageGround::Finite {
                zrati: self.zrati,
                screen: self.screen,
            }
        }
    }

    /// `ZRATI2 = √(1/(ε_r2 − jσ₂λ·59.96))` and the cliff in wavelengths, from `GD`'s values.
    #[must_use]
    pub fn second_medium(epsr2: f64, sig2: f64, clt_m: f64, cht_m: f64, wlam: f64) -> (f64, f64, C) {
        let epsc2 = C::new(epsr2, -sig2 * wlam * single(59.96));
        (clt_m / wlam, cht_m / wlam, csqrt(rdiv(1.0, epsc2)))
    }
}

fn phase(arg: f64) -> C {
    C::new(arg.cos(), arg.sin())
}

/// The segment integrals of `FFLD`/`GFLD`: the far-field weights of the constant, sine and cosine
/// current shapes on a segment of half electrical length `el` seen at direction cosine `omega`.
fn integrals(omega: f64, el: f64) -> (f64, f64, f64) {
    let sill = omega * el;
    let top = el + sill;
    let bot = el - sill;
    let a = if omega.abs() < 1e-7 { (2.0 - omega * omega * el * el / 3.0) * el } else { 2.0 * sill.sin() / omega };
    let too = if top.abs() < 1e-7 { 1.0 - top * top / 6.0 } else { top.sin() / top };
    let boo = if bot.abs() < 1e-7 { 1.0 - bot * bot / 6.0 } else { bot.sin() / bot };
    (a, el * (boo - too), el * (boo + too))
}

/// `a·A + b·B·(−j) + c·C`, as `FFLD` forms `RR + j·RI` from `AIR`..`CII`.
fn moment(abc: &[C; 3], a: f64, b: f64, c: f64) -> C {
    let [ca, cb, cc] = *abc;
    C::new(a * ca.re + b * cb.im + c * cc.re, a * ca.im - b * cb.re + c * cc.im)
}

/// Fresnel reflection coefficients `(RRV, RRH)` for direction cosine `roz` (`cos θ`) and
/// `thz = −sin θ`, ground relative impedance `zrati`.
fn fresnel(zrati: C, roz: f64, thz: f64) -> (C, C) {
    let zrsin = csqrt(C::new(1.0, 0.0) - zrati * zrati * thz * thz);
    let rrv = -cdiv(roz - zrati * zrsin, roz + zrati * zrsin);
    let rrh = cdiv(zrati * roz - zrsin, zrati * roz + zrsin);
    (rrv, rrh)
}

/// Surface-patch currents for the far field: patches in kernel units and their Cartesian current
/// densities, A/m.
#[derive(Debug, Clone, Copy, Default)]
pub struct Surface<'a> {
    /// The patches.
    pub patches: &'a [SurfacePatch],
    /// Current density on each, `(x, y, z)`.
    pub currents: &'a [[C; 3]],
}

/// `FFLDS`: the far field `(x, y, z)` of the patch currents in direction `ro`, without the
/// factor `exp(jkr)/(r/λ)`.
fn fflds(ro: [f64; 3], surf: &Surface<'_>) -> [C; 3] {
    const TPI: f64 = 6.283_185_308;
    let cons = C::new(0.0, 188.365);
    let mut e = [C::new(0.0, 0.0); 3];
    for (pt, j) in surf.patches.iter().zip(surf.currents) {
        let arg = TPI * (ro[0] * pt.center[0] + ro[1] * pt.center[1] + ro[2] * pt.center[2]);
        let ct = C::new(arg.cos() * pt.area, arg.sin() * pt.area);
        for k in 0..3 {
            e[k] += j[k] * ct;
        }
    }
    let ct = e[0] * ro[0] + e[1] * ro[1] + e[2] * ro[2];
    [cons * (ct * ro[0] - e[0]), cons * (ct * ro[1] - e[1]), cons * (ct * ro[2] - e[2])]
}

/// `FFLD`: far-zone `(E_θ, E_φ)` in direction (θ, φ), radians, without the factor
/// `exp(jkr)/(r/λ)`.
#[must_use]
#[allow(clippy::too_many_lines, clippy::many_single_char_names, clippy::similar_names, clippy::too_many_arguments)]
pub fn ffld(
    thet: f64,
    phi: f64,
    segs: &[WireSeg],
    abc: &[[C; 3]],
    surf: &Surface<'_>,
    mode: i32,
    g: &FarFieldGround,
) -> (C, C) {
    let zero = C::new(0.0, 0.0);
    let phx = -phi.sin();
    let phy = phi.cos();
    let mut roz = thet.cos();
    let thx = roz * phy;
    let thy = -roz * phx;
    let thz = -thet.sin();
    let rox = -thz * phy;
    let roy = thz * phx;
    let rozs = roz;
    let (mut cix, mut ciy, mut ciz) = (zero, zero, zero);
    let (mut ccx, mut ccy, mut ccz) = (zero, zero, zero);
    let (mut rrv, mut rrh) = (zero, zero);
    let (mut rrv1, mut rrh1, mut rrv2, mut rrh2) = (zero, zero, zero, zero);
    let (mut tthet, mut darg) = (0.0, 0.0);
    let passes = if g.present { 2 } else { 1 };

    for k in 1..=passes {
        if k == 2 {
            (rrv, rrh) = if g.perfect {
                (C::new(-1.0, 0.0), C::new(-1.0, 0.0))
            } else {
                fresnel(g.zrati, roz, thz)
            };
            if mode > 1 {
                rrv1 = rrv;
                rrh1 = rrh;
                tthet = thet.tan();
                if mode != 4 {
                    let (_, ch, zrati2) = g.second.unwrap_or((0.0, 0.0, g.zrati));
                    (rrv2, rrh2) = fresnel(zrati2, roz, thz);
                    darg = -TP * 2.0 * ch * roz;
                }
            }
            roz = -roz;
            (ccx, ccy, ccz) = (cix, ciy, ciz);
        }
        (cix, ciy, ciz) = (zero, zero, zero);

        for (s, abc_i) in segs.iter().zip(abc) {
            let [cab, sab, salp] = s.dir;
            let [x, y, z] = s.center;
            let omega = -(rox * cab + roy * sab + roz * salp);
            let (a, b, c) = integrals(omega, PI * s.length);
            let rri = moment(abc_i, a, b, c);
            let mut arg = TP * (x * rox + y * roy + z * roz);
            if !(k == 2 && mode >= 2) {
                let exa = phase(arg) * rri;
                cix += exa * cab;
                ciy += exa * sab;
                ciz += exa * salp;
                continue;
            }
            // The image of each segment in cliff and ground-screen problems, with the
            // reflection coefficients of where its specular point falls.
            let (cl, _, _) = g.second.unwrap_or((0.0, 0.0, g.zrati));
            let dr = z * tthet;
            let mut d = dr * phy + x;
            let mut screened = false;
            if mode != 2 {
                d = (d * d + (y - dr * phx).powi(2)).sqrt();
                if mode != 3 {
                    let sc = g.screen.unwrap_or(RadialScreen {
                        t1: zero,
                        t2: 1.0,
                        length: 0.0,
                    });
                    if sc.length - d >= 0.0 {
                        // Radial-wire screen reflection coefficients.
                        let d = d + sc.t2;
                        let zscrn = sc.t1 * d * (d / sc.t2).ln();
                        let zscrn = cdiv(zscrn * g.zrati, nec_kernel::consts::ETA * g.zrati + zscrn);
                        let zrsin = csqrt(C::new(1.0, 0.0) - zscrn * zscrn * thz * thz);
                        rrv = cdiv(roz + zscrn * zrsin, -roz + zscrn * zrsin);
                        rrh = cdiv(zscrn * roz + zrsin, zscrn * roz - zrsin);
                        screened = true;
                    } else if mode == 5 {
                        d = dr * phy + x;
                    }
                }
            }
            if !screened {
                // Beyond the cliff edge the second medium reflects, from the cliff's height.
                if mode != 4 && cl - d <= 0.0 {
                    rrv = rrv2;
                    rrh = rrh2;
                    arg += darg;
                } else {
                    rrv = rrv1;
                    rrh = rrh1;
                }
            }
            let exa = phase(arg) * rri;
            let (tix, tiy, tiz) = (exa * cab, exa * sab, exa * salp);
            let cdp = (tix * phx + tiy * phy) * (rrh - rrv);
            cix += tix * rrv + cdp * phx;
            ciy += tiy * rrv + cdp * phy;
            ciz -= tiz * rrv;
        }

        if k == 2 {
            if mode >= 2 {
                cix += ccx;
                ciy += ccy;
                ciz += ccz;
            } else {
                // The image of the whole structure over an infinite plane.
                let cdp = (cix * phx + ciy * phy) * (rrh - rrv);
                cix = ccx + cix * rrv + cdp * phx;
                ciy = ccy + ciy * rrv + cdp * phy;
                ciz = ccz - ciz * rrv;
            }
        }
    }
    if surf.patches.is_empty() {
        return (
            (cix * thx + ciy * thy + ciz * thz) * CONST,
            (cix * phx + ciy * phy) * CONST,
        );
    }
    // The patches, with their own image: negated over perfect ground, Fresnel-weighted
    // otherwise, whatever the far-field ground mode.
    let mut e = fflds([rox, roy, rozs], surf);
    if g.present {
        let mut gi = fflds([rox, roy, -rozs], surf);
        if g.perfect {
            for v in &mut gi {
                *v = -*v;
            }
        } else {
            let rrv = csqrt(C::new(1.0, 0.0) - g.zrati * g.zrati * thz * thz);
            let rrh = g.zrati * rozs;
            let rrh = cdiv(rrh - rrv, rrh + rrv);
            let rrv = g.zrati * rrv;
            let rrv = -cdiv(rozs - rrv, rozs + rrv);
            let eth = (gi[0] * phx + gi[1] * phy) * (rrh - rrv);
            gi = [gi[0] * rrv + eth * phx, gi[1] * rrv + eth * phy, gi[2] * rrv];
        }
        e = [e[0] + gi[0], e[1] + gi[1], e[2] - gi[2]];
    }
    let (ex, ey, ez) = (e[0] + cix * CONST, e[1] + ciy * CONST, e[2] + ciz * CONST);
    (ex * thx + ey * thy + ez * thz, ex * phx + ey * phy)
}

/// `GFLD`: the field at a finite distance — horizontal range `rho` and height `rz` in
/// wavelengths, azimuth `phi` radians — including the ground wave, as `(E_θ, E_φ, E_r)`.
#[must_use]
///
/// Patches contribute to the space wave only, as in NEC.
#[allow(clippy::many_single_char_names, clippy::similar_names, clippy::too_many_arguments)]
pub fn gfld(
    rho: f64,
    phi: f64,
    rz: f64,
    segs: &[WireSeg],
    abc: &[[C; 3]],
    surf: &Surface<'_>,
    g: &FarFieldGround,
) -> (C, C, C) {
    let zero = C::new(0.0, 0.0);
    let r = (rho * rho + rz * rz).sqrt();
    let ux = if g.perfect { C::new(1.0, 0.0) } else { g.zrati };
    if !g.present || ux.norm() > 0.5 || r > 1e5 {
        // The space wave only.
        let thet = if rz < 1e-20 { PI * 0.5 } else { (rho / rz).atan() };
        let (eth, eph) = ffld(thet, phi, segs, abc, surf, 0, g);
        let exa = phase(-TP * r) / r;
        return (eth * exa, eph * exa, zero);
    }
    let u = ux;
    let u2 = u * u;
    let phx = -phi.sin();
    let phy = phi.cos();
    let rx = rho * phy;
    let ry = -rho * phx;
    let (mut cix, mut ciy, mut ciz) = (zero, zero, zero);
    for (s, abc_i) in segs.iter().zip(abc) {
        let [dx, dy, dz] = s.dir;
        let [x, y, z] = s.center;
        let rix = rx - x;
        let riy = ry - y;
        let rhp = (rix * rix + riy * riy).sqrt();
        let (rhx, rhy) = if rhp < 1e-6 { (1.0, 0.0) } else { (rix / rhp, riy / rhp) };
        let mut calp = 1.0 - dz * dz;
        let (cph, sph) = if calp < 1e-6 {
            // NEC leaves CALP unsquared here.
            (rhx, rhy)
        } else {
            calp = calp.sqrt();
            let (cbet, sbet) = (dx / calp, dy / calp);
            (rhx * cbet + rhy * sbet, rhy * cbet - rhx * sbet)
        };
        let el = PI * s.length;
        let mut terms = [(zero, 0.0, 0.0); 2];
        for (k, rfl) in [1.0, -1.0].into_iter().enumerate() {
            let riz = rz - z * rfl;
            let rxyz = (rix * rix + riy * riy + riz * riz).sqrt();
            let (rnx, rny, rnz) = (rix / rxyz, riy / rxyz, riz / rxyz);
            let omega = -(rnx * dx + rny * dy + rnz * dz * rfl);
            let (a, b, c) = integrals(omega, el);
            let arg = TP * (x * rnx + y * rny + z * rnz * rfl);
            terms[k] = (phase(arg) * moment(abc_i, a, b, c) / TP, rxyz, riz);
        }
        let (xx1, r1, zmh) = terms[0];
        let (xx2, r2, zph) = terms[1];
        let (erv, ezv, erh, ezh, eph) = nec_kernel::sommerfeld::gwave(u, u2, xx1, xx2, r1, r2, zmh, zph);
        let erh = erh * cph * calp + erv * dz;
        let eph = eph * sph * calp;
        let ezh = ezh * cph * calp + ezv * dz;
        cix += erh * rhx - eph * rhy;
        ciy += erh * rhy + eph * rhx;
        ciz += ezh;
    }
    let exa = phase(-TP * r);
    cix *= exa;
    ciy *= exa;
    ciz *= exa;
    let (rnx, rny, rnz) = (rx / r, ry / r, rz / r);
    let thx = rnz * phy;
    let thy = -rnz * phx;
    let thz = -rho / r;
    (
        cix * thx + ciy * thy + ciz * thz,
        cix * phx + ciy * phy,
        cix * rnx + ciy * rny + ciz * rnz,
    )
}

/// Polarization sense of the field ellipse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sense {
    /// No field.
    None,
    /// Axial ratio below 1e-5.
    Linear,
    /// Right-hand elliptic.
    Right,
    /// Left-hand elliptic.
    Left,
}

/// One direction of a radiation pattern, as `RDPAT` prints it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PatternPoint {
    /// θ, degrees (mode 1: height, metres).
    pub theta: f64,
    /// φ, degrees.
    pub phi: f64,
    /// Major-axis gain, dB.
    pub major: f64,
    /// Minor-axis gain, dB.
    pub minor: f64,
    /// Vertical (θ-polarized) gain, dB.
    pub vertical: f64,
    /// Horizontal (φ-polarized) gain, dB.
    pub horizontal: f64,
    /// Total gain, dB.
    pub total: f64,
    /// Axial ratio of the polarization ellipse.
    pub axial_ratio: f64,
    /// Tilt of the ellipse, degrees.
    pub tilt: f64,
    /// Sense of rotation.
    pub sense: Sense,
    /// `E_θ` as printed: volts per metre times range (or at `range_m` if given).
    pub e_theta: C,
    /// `E_φ`, likewise.
    pub e_phi: C,
    /// `E_r` (mode 1 only).
    pub e_radial: C,
}

/// A computed pattern.
#[derive(Debug, Clone, PartialEq)]
pub struct Pattern {
    /// The directions, in NEC's order: θ fastest, then φ. Directions below a ground plane are
    /// omitted, as NEC omits them.
    pub points: Vec<PatternPoint>,
    /// Average power gain and the solid angle it was taken over (in units of π steradians).
    pub average: Option<(f64, f64)>,
    /// The normalization factor used, dB, when normalized gains were requested.
    pub normalization_db: Option<f64>,
}

/// How field intensity becomes gain: `RDPAT`'s `GCON` and `GCOP`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GainBasis {
    /// Multiplies |E|² for the printed gains.
    pub gcon: f64,
    /// Multiplies |E|² for the average power gain.
    pub gcop: f64,
}

impl GainBasis {
    /// For voltage sources: power gain relative to input power (`directive` divides by radiated
    /// power instead).
    #[must_use]
    pub fn from_power(wlam: f64, input: f64, radiated: f64, directive: bool) -> Self {
        let gcop = wlam * wlam * 2.0 * PI / (single(376.73) * input);
        Self {
            gcon: if directive { gcop * input / radiated } else { gcop },
            gcop,
        }
    }

    /// For an incident plane wave of axial ratio `ratio`: bistatic scattering cross-section,
    /// `4π/(1 + r²)` per unit |E|².
    #[must_use]
    pub fn for_plane_wave(ratio: f64) -> Self {
        let g = 4.0 * PI / (1.0 + ratio * ratio);
        Self { gcon: g, gcop: g }
    }

    /// The input power `RDPAT` assigns an elementary current source of moment `moment` A·m.
    #[must_use]
    pub fn current_element_power(moment: f64, wlam: f64) -> f64 {
        single(394.51) * moment * moment * wlam * wlam
    }
}

fn db10(x: f64) -> f64 {
    if x < 1e-20 { single(-999.99) } else { 10.0 * x.log10() }
}

fn cang(z: C) -> f64 {
    if z.im == 0.0 && z.re == 0.0 { 0.0 } else { z.im.atan2(z.re) * TD }
}

/// `RDPAT`.
#[must_use]
#[allow(clippy::too_many_lines, clippy::similar_names, clippy::too_many_arguments)]
pub fn pattern(
    req: &PatternRequest,
    segs: &[WireSeg],
    abc: &[[C; 3]],
    surf: &Surface<'_>,
    wlam: f64,
    g: &FarFieldGround,
    gain: GainBasis,
) -> Pattern {
    let mut points = Vec::new();
    let mut gains_for_norm: Vec<f64> = Vec::new();
    let mut gmax = single(-1e10);
    let mut pint = 0.0;
    let tmp1 = req.dphi * TA;
    let tmp2 = 0.5 * req.dtheta * TA;
    let (exrm, exra) = if req.range_m < 1e-20 {
        (1.0, 0.0)
    } else {
        let exra = req.range_m / wlam;
        (1.0 / req.range_m, -360.0 * (exra - exra.trunc()))
    };

    let mut phi = req.phi0 - req.dphi;
    for kph in 1..=req.n_phi {
        phi += req.dphi;
        let pha = phi * TA;
        let mut thet = req.theta0 - req.dtheta;
        for kth in 1..=req.n_theta {
            thet += req.dtheta;
            if g.present && thet > single(90.01) && req.mode != 1 {
                continue;
            }
            let tha = thet * TA;
            let (eth, eph, erd) = if req.mode == 1 {
                gfld(req.range_m / wlam, pha, thet / wlam, segs, abc, surf, g)
            } else {
                let (a, b) = ffld(tha, pha, segs, abc, surf, req.mode, g);
                (a, b, C::new(0.0, 0.0))
            };
            let ethm2 = (eth * eth.conj()).re;
            let ethm = ethm2.sqrt();
            let etha = cang(eth);
            let ephm2 = (eph * eph.conj()).re;
            let ephm = ephm2.sqrt();
            let epha = cang(eph);
            if req.mode == 1 {
                points.push(PatternPoint {
                    theta: thet,
                    phi,
                    major: 0.0,
                    minor: 0.0,
                    vertical: 0.0,
                    horizontal: 0.0,
                    total: 0.0,
                    axial_ratio: 0.0,
                    tilt: 0.0,
                    sense: Sense::None,
                    e_theta: eth,
                    e_phi: eph,
                    e_radial: erd,
                });
                continue;
            }

            // The polarization ellipse.
            let (tilt, emajr2, eminr2, axrat, sense);
            if ethm2 <= 1e-20 && ephm2 <= 1e-20 {
                (tilt, emajr2, eminr2, axrat, sense) = (0.0, 0.0, 0.0, 0.0, Sense::None);
            } else {
                let mut dfaz = epha - etha;
                let dfaz2 = if epha < 0.0 { dfaz + 360.0 } else { dfaz - 360.0 };
                if dfaz.abs() > dfaz2.abs() {
                    dfaz = dfaz2;
                }
                let cdfaz = (dfaz * TA).cos();
                let mut t1 = ethm2 - ephm2;
                let mut t2 = 2.0 * ephm * ethm * cdfaz;
                let tilta = if t2 == 0.0 && t1 == 0.0 { 0.0 } else { 0.5 * t2.atan2(t1) };
                let stilta = tilta.sin();
                t1 = t1 * stilta * stilta;
                t2 = t2 * stilta * tilta.cos();
                let emaj = -t1 + t2 + ethm2;
                let emin = (t1 - t2 + ephm2).max(0.0);
                let ar = (emin / emaj).sqrt();
                sense = if ar <= 1e-5 {
                    Sense::Linear
                } else if dfaz <= 0.0 {
                    Sense::Right
                } else {
                    Sense::Left
                };
                (tilt, emajr2, eminr2, axrat) = (tilta * TD, emaj, emin, ar);
            }
            let gnmj = db10(gain.gcon * emajr2);
            let gnmn = db10(gain.gcon * eminr2);
            let gnv = db10(gain.gcon * ethm2);
            let gnh = db10(gain.gcon * ephm2);
            let gtot = db10(gain.gcon * (ethm2 + ephm2));
            if req.normalize >= 1 {
                let v = [gnmj, gnmn, gnv, gnh, gtot][usize::from(req.normalize) - 1];
                gains_for_norm.push(v);
                if v > gmax {
                    gmax = v;
                }
            }
            if req.average != 0 {
                let mut t3 = tha - tmp2;
                let mut t4 = tha + tmp2;
                if kth == 1 {
                    t3 = tha;
                }
                if kth == req.n_theta {
                    t4 = tha;
                }
                let mut da = (tmp1 * (t3.cos() - t4.cos())).abs();
                if kph == 1 || kph == req.n_phi {
                    da *= 0.5;
                }
                pint += gain.gcop * (ethm2 + ephm2) * da;
                if req.average == 2 {
                    continue;
                }
            }
            let scale = wlam * exrm;
            let rot = phase(exra * TA);
            points.push(PatternPoint {
                theta: thet,
                phi,
                major: gnmj,
                minor: gnmn,
                vertical: gnv,
                horizontal: gnh,
                total: gtot,
                axial_ratio: axrat,
                tilt,
                sense,
                e_theta: eth * scale * rot,
                e_phi: eph * scale * rot,
                e_radial: C::new(0.0, 0.0),
            });
        }
    }

    let average = (req.average != 0).then(|| {
        let t3 = req.theta0 * TA;
        #[allow(clippy::cast_precision_loss)]
        let t4 = t3 + req.dtheta * TA * (req.n_theta - 1) as f64;
        #[allow(clippy::cast_precision_loss)]
        let solid = (req.dphi * TA * (req.n_phi - 1) as f64 * (t3.cos() - t4.cos())).abs();
        (pint / solid, solid / PI)
    });
    let normalization_db = (req.normalize >= 1).then(|| {
        if req.gain_norm_db.abs() > 1e-20 { req.gain_norm_db } else { gmax }
    });
    Pattern {
        points,
        average,
        normalization_db,
    }
}
