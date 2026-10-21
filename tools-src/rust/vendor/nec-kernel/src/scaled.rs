//! Segments as the kernel sees them: centre, direction, length and radius in **wavelengths**.
//!
//! NEC never fills the interaction matrix in metres. The main program multiplies every
//! coordinate, length and radius by `FR = FMHZ / CVEL` (one over the wavelength) before calling
//! the fill, so `k = 2π` throughout the kernel. This mirrors NEC's `/DATA/` arrays after that
//! scaling: `X, Y, Z` (centre), `CAB, SAB, SALP` (direction cosines), `SI` (length), `BI` (radius).

use nec_core::Frequency;
use nec_geom::Geometry;

/// One wire segment in wavelength units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WireSeg {
    /// Centre, in wavelengths. NEC's `X, Y, Z`.
    pub center: [f64; 3],
    /// Unit direction from end 1 to end 2. NEC's `CAB, SAB, SALP`.
    pub dir: [f64; 3],
    /// Length in wavelengths. NEC's `SI`.
    pub length: f64,
    /// Radius in wavelengths. NEC's `BI`.
    pub radius: f64,
}

/// Convert a built geometry to kernel units at one frequency.
///
/// Follows `DATAGN`'s conversion: centre is the endpoint average, direction is the normalized
/// difference, and the vertical cosine is clamped to `[-1, 1]` before use.
#[must_use]
pub fn scale(geom: &Geometry, f: Frequency) -> Vec<WireSeg> {
    let inv_lambda = 1.0 / f.wavelength_m();
    geom.segments
        .iter()
        .map(|s| {
            let d = s.a.to(s.b);
            let len = d.norm();
            let (cab, sab, salp) = if len > 0.0 {
                (d.x / len, d.y / len, (d.z / len).clamp(-1.0, 1.0))
            } else {
                (0.0, 0.0, 0.0)
            };
            let c = s.center();
            WireSeg {
                center: [c.x * inv_lambda, c.y * inv_lambda, c.z * inv_lambda],
                dir: [cab, sab, salp],
                length: len * inv_lambda,
                radius: s.radius * inv_lambda,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use nec_cards::parse_deck;

    #[test]
    fn example1_scales_into_wavelengths() {
        // models/Example1.nec at 300 MHz: lambda = CVEL/300 m. Nine segments of 0.05373 m.
        let mut d = parse_deck("GW 1 9 0 -.2418 0 0 .2418 0 .0001\nGE 0\nEN\n").expect("parses");
        d.resolve().expect("resolves");
        let g = Geometry::build(&d).expect("builds");
        let f = Frequency::from_mhz(300.0).expect("valid");
        let w = scale(&g, f);

        let lambda = nec_core::CVEL_MHZ_M / 300.0;
        assert_eq!(w.len(), 9);
        assert!((w[0].length - (0.4836 / 9.0) / lambda).abs() < 1e-12);
        assert!((w[0].radius - 1e-4 / lambda).abs() < 1e-15);
        let d = w[0].dir;
        assert!(d[0].abs() < 1e-15 && (d[1] - 1.0).abs() < 1e-15 && d[2].abs() < 1e-15, "{d:?}");
        assert!(w[4].center.iter().all(|c| c.abs() < 1e-12), "segment 5 is centred");
    }
}
