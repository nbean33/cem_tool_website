//! Segments: the discrete elements the method of moments actually solves on.

use nec_core::Point3;

/// One straight wire segment.
///
/// NEC numbers segments from 1 in the order the geometry cards create them, and that numbering
/// is part of the user interface: `EX` and `LD` cards address segments by number. The index is
/// therefore stored rather than inferred from position in a vector.
#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    /// 1-based global segment number, as NEC numbers them.
    pub index: usize,
    /// Wire tag from the originating `GW` card. Tags need not be unique or ordered.
    pub tag: i32,
    /// End 1.
    pub a: Point3,
    /// End 2.
    pub b: Point3,
    /// Wire radius, in the same length unit as the coordinates.
    pub radius: f64,
}

impl Segment {
    /// Centre of the segment, which is where NEC places the match point.
    #[must_use]
    pub fn center(&self) -> Point3 {
        self.a.midpoint(self.b)
    }

    /// Segment length.
    #[must_use]
    pub fn length(&self) -> f64 {
        self.a.distance(self.b)
    }

    /// Unit vector from end 1 to end 2, or `None` for a degenerate segment.
    #[must_use]
    pub fn direction(&self) -> Option<Point3> {
        self.a.to(self.b).normalized()
    }

    /// Orientation angles in degrees, as NEC prints them in its `SEGMENTATION DATA` table:
    /// `alpha` is elevation above the XY plane, `beta` is azimuth from the +X axis.
    ///
    /// A wire along +Y gives `(0, 90)`, which is what the engine reports for the
    /// `Example1.nec` dipole. Returns `None` for a degenerate segment.
    #[must_use]
    pub fn orientation_deg(&self) -> Option<(f64, f64)> {
        let d = self.direction()?;
        let alpha = d.z.clamp(-1.0, 1.0).asin().to_degrees();
        let beta = d.y.atan2(d.x).to_degrees();
        Some((alpha, beta))
    }

    /// Ratio of length to radius.
    ///
    /// NEC's thin-wire kernel assumes this is large; small values are the classic source of
    /// quietly wrong answers, so it is surfaced for the model linter rather than buried.
    #[must_use]
    pub fn length_over_radius(&self) -> f64 {
        if self.radius > 0.0 {
            self.length() / self.radius
        } else {
            f64::INFINITY
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seg(a: Point3, b: Point3) -> Segment {
        Segment {
            index: 1,
            tag: 1,
            a,
            b,
            radius: 1e-4,
        }
    }

    #[test]
    fn example1_first_segment_matches_the_engine_table() {
        // The engine's SEGMENTATION DATA for models/Example1.nec, segment 1:
        //   centre (0, -0.21493, 0), length 0.05373, alpha 0, beta 90, radius 0.0001
        let total = 0.2418 * 2.0;
        let seg_len = total / 9.0;
        let s = seg(
            Point3::new(0.0, -0.2418, 0.0),
            Point3::new(0.0, -0.2418 + seg_len, 0.0),
        );
        let c = s.center();
        assert!((c.y + 0.214_93).abs() < 5e-6, "centre y = {}", c.y);
        assert!((s.length() - 0.053_73).abs() < 5e-6, "length = {}", s.length());

        let (alpha, beta) = s.orientation_deg().expect("non-degenerate");
        assert!(alpha.abs() < 1e-9, "alpha = {alpha}");
        assert!((beta - 90.0).abs() < 1e-9, "beta = {beta}");
    }

    #[test]
    fn vertical_segment_has_ninety_degree_elevation() {
        let s = seg(Point3::ORIGIN, Point3::new(0.0, 0.0, 5.0));
        let (alpha, _) = s.orientation_deg().expect("non-degenerate");
        assert!((alpha - 90.0).abs() < 1e-9, "alpha = {alpha}");
    }

    #[test]
    fn degenerate_segment_has_no_direction() {
        let s = seg(Point3::ORIGIN, Point3::ORIGIN);
        assert!(s.direction().is_none());
        assert!(s.orientation_deg().is_none());
    }

    #[test]
    fn zero_radius_gives_infinite_ratio_rather_than_nan() {
        let mut s = seg(Point3::ORIGIN, Point3::new(1.0, 0.0, 0.0));
        s.radius = 0.0;
        assert!(s.length_over_radius().is_infinite());
    }
}
