//! Minimal 3-D point/vector type.
//!
//! Deliberately small: geometry construction lives in `nec-geom`. This is only the shared
//! coordinate type so crates can exchange points without depending on a linear-algebra crate.

/// A point or vector in Cartesian space. Units are whatever the caller is working in
/// (metres before scaling, wavelengths after).
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct Point3 {
    /// X coordinate.
    pub x: f64,
    /// Y coordinate.
    pub y: f64,
    /// Z coordinate. NEC treats +Z as up; ground, when present, lies at Z = 0.
    pub z: f64,
}

impl Point3 {
    /// Construct a point.
    #[must_use]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// The origin.
    pub const ORIGIN: Self = Self::new(0.0, 0.0, 0.0);

    /// Vector from `self` to `other`.
    #[must_use]
    pub fn to(self, other: Self) -> Self {
        Self::new(other.x - self.x, other.y - self.y, other.z - self.z)
    }

    /// Euclidean length.
    #[must_use]
    pub fn norm(self) -> f64 {
        self.dot(self).sqrt()
    }

    /// Dot product.
    #[must_use]
    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Cross product.
    #[must_use]
    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    /// Scale by a factor.
    #[must_use]
    pub fn scaled(self, k: f64) -> Self {
        Self::new(self.x * k, self.y * k, self.z * k)
    }

    /// Midpoint between two points.
    #[must_use]
    pub fn midpoint(self, other: Self) -> Self {
        Self::new(
            0.5 * (self.x + other.x),
            0.5 * (self.y + other.y),
            0.5 * (self.z + other.z),
        )
    }

    /// Unit vector in the same direction, or `None` if the length is zero.
    #[must_use]
    pub fn normalized(self) -> Option<Self> {
        let n = self.norm();
        if n > 0.0 { Some(self.scaled(1.0 / n)) } else { None }
    }

    /// Distance to another point.
    #[must_use]
    pub fn distance(self, other: Self) -> f64 {
        self.to(other).norm()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn norm_and_distance_agree() {
        let a = Point3::new(1.0, 2.0, 2.0);
        assert!((a.norm() - 3.0).abs() < 1e-15);
        assert!((Point3::ORIGIN.distance(a) - 3.0).abs() < 1e-15);
    }

    #[test]
    fn cross_is_perpendicular_to_both() {
        let a = Point3::new(1.0, 0.5, -2.0);
        let b = Point3::new(-3.0, 2.0, 0.25);
        let c = a.cross(b);
        assert!(c.dot(a).abs() < 1e-14);
        assert!(c.dot(b).abs() < 1e-14);
    }

    #[test]
    fn zero_vector_has_no_direction() {
        assert!(Point3::ORIGIN.normalized().is_none());
    }

    #[test]
    fn midpoint_of_example1_dipole_is_the_origin() {
        let a = Point3::new(0.0, -0.2418, 0.0);
        let b = Point3::new(0.0, 0.2418, 0.0);
        assert_eq!(a.midpoint(b), Point3::ORIGIN);
    }
}
