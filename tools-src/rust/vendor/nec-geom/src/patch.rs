//! Surface patches (`SP`, `SM`, `SC`): NEC-2's model of closed conducting surfaces.
//!
//! A patch is a small flat piece of surface, described for the solver by nothing but its centre,
//! its area and two unit tangent vectors `t1`, `t2` — NEC's magnetic-field integral equation
//! (MFIE) samples the surface current at patch centres only, so a patch's shape never enters the
//! numerics. Each patch carries two unknowns, the surface current density along `t1` and along
//! `t2`. The outward normal is `t1 × t2`, flipped by [`Patch::salp`] on patches made by a
//! reflection (`GX`), which reverses the handedness of the tangents.
//!
//! The shape *is* kept here, as [`Patch::outline`], so a viewer can draw the surface.
//!
//! # Correspondence with the Fortran
//!
//! [`single`] and [`surface`] are NEC's `PATCH`; [`subdivide`] is its entry point `SUBPH`, which
//! splits a patch into four where a wire ends at its centre. The arithmetic follows the Fortran
//! operation for operation, so centres and areas agree with the engine's `SURFACE PATCH DATA`.

use nec_core::Point3;

/// `DATA TA/0.01745329252D+0/` in `DATAGN`: degrees to radians as NEC writes it.
const TA: f64 = 0.017_453_292_52;

/// One surface patch.
#[derive(Debug, Clone, PartialEq)]
pub struct Patch {
    /// 1-based patch number, as NEC numbers them.
    pub index: usize,
    /// Centre, where the MFIE is matched.
    pub center: Point3,
    /// First unit tangent.
    pub t1: Point3,
    /// Second unit tangent, `n × t1` for the patch as generated.
    pub t2: Point3,
    /// Area, in the square of the coordinates' unit.
    pub area: f64,
    /// NEC's `SALP` for a patch: `+1`, or `−1` on a reflected patch whose tangents now form a
    /// left-handed pair with the outward normal. The normal is `salp · (t1 × t2)`.
    pub salp: f64,
    /// Corners, for drawing only. An arbitrary-shape patch (`SP 0 0`) is drawn as a square of
    /// its area.
    pub outline: Vec<Point3>,
}

impl Patch {
    /// The outward unit normal, `salp · (t1 × t2)`.
    #[must_use]
    pub fn normal(&self) -> Point3 {
        cross(self.t1, self.t2).scaled(self.salp)
    }
}

/// Why patch input was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PatchError {
    /// `PATCH DATA ERROR`: an `SP`/`SM` that needed an `SC` card without one, or an `SC` that
    /// continues nothing.
    #[error("line {line}: {reason}")]
    Data {
        /// Source line.
        line: usize,
        /// What was wrong.
        reason: &'static str,
    },
    /// The four corners of a quadrilateral patch are not coplanar.
    #[error("line {line}: the corners of a quadrilateral patch do not lie in a plane")]
    NotPlanar {
        /// Source line.
        line: usize,
    },
}

fn cross(a: Point3, b: Point3) -> Point3 {
    Point3::new(
        a.y * b.z - a.z * b.y,
        a.z * b.x - a.x * b.z,
        a.x * b.y - a.y * b.x,
    )
}

fn sub(a: Point3, b: Point3) -> Point3 {
    Point3::new(a.x - b.x, a.y - b.y, a.z - b.z)
}

fn add(a: Point3, b: Point3) -> Point3 {
    Point3::new(a.x + b.x, a.y + b.y, a.z + b.z)
}

/// The shape of a single patch (`SP`'s `NS` field).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shape {
    /// `NS = 0`: centre, normal direction (elevation and azimuth, degrees) and area.
    Arbitrary {
        /// Centre.
        center: Point3,
        /// Elevation of the normal above the XY plane, degrees.
        elevation_deg: f64,
        /// Azimuth of the normal from +X, degrees.
        azimuth_deg: f64,
        /// Area.
        area: f64,
    },
    /// `NS = 1`: three corners of a rectangle; the fourth is `c1 + c3 − c2`.
    Rectangular([Point3; 3]),
    /// `NS = 2`: a triangle.
    Triangular([Point3; 3]),
    /// `NS = 3`: a planar quadrilateral.
    Quadrilateral([Point3; 4]),
}

/// `PATCH` for `NX = 0`: one new patch, numbered `index`.
pub fn single(index: usize, shape: Shape, line: usize) -> Result<Patch, PatchError> {
    match shape {
        Shape::Arbitrary {
            center,
            elevation_deg,
            azimuth_deg,
            area,
        } => {
            let (x2, y2) = (elevation_deg * TA, azimuth_deg * TA);
            let znv = x2.cos();
            let xnv = znv * y2.cos();
            let ynv = znv * y2.sin();
            let znv = x2.sin();
            let xa = (xnv * xnv + ynv * ynv).sqrt();
            let t1 = if xa < 1e-6 {
                Point3::new(1.0, 0.0, 0.0)
            } else {
                Point3::new(-ynv / xa, xnv / xa, 0.0)
            };
            let t2 = cross(Point3::new(xnv, ynv, znv), t1);
            let half = 0.5 * area.sqrt();
            let outline = [(1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)]
                .iter()
                .map(|&(a, b)| add(center, add(t1.scaled(a * half), t2.scaled(b * half))))
                .collect();
            Ok(Patch {
                index,
                center,
                t1,
                t2,
                area,
                salp: 1.0,
                outline,
            })
        }
        Shape::Rectangular([c1, c2, c3]) => {
            let (s1, s2) = (sub(c2, c1), sub(c3, c2));
            let (n, xa) = unit_normal(s1, s2);
            let center = Point3::new(
                c1.x + 0.5 * (s1.x + s2.x),
                c1.y + 0.5 * (s1.y + s2.y),
                c1.z + 0.5 * (s1.z + s2.z),
            );
            let c4 = Point3::new(c1.x + c3.x - c2.x, c1.y + c3.y - c2.y, c1.z + c3.z - c2.z);
            Ok(flat(index, center, s1, n, xa, vec![c1, c2, c3, c4]))
        }
        Shape::Triangular([c1, c2, c3]) => {
            let (s1, s2) = (sub(c2, c1), sub(c3, c2));
            let (n, xa) = unit_normal(s1, s2);
            let center = Point3::new(
                (c1.x + c2.x + c3.x) / 3.0,
                (c1.y + c2.y + c3.y) / 3.0,
                (c1.z + c2.z + c3.z) / 3.0,
            );
            Ok(flat(index, center, s1, n, 0.5 * xa, vec![c1, c2, c3]))
        }
        Shape::Quadrilateral([c1, c2, c3, c4]) => {
            let (s1, s2) = (sub(c2, c1), sub(c3, c2));
            let (n, xa) = unit_normal(s1, s2);
            // The second triangle, c1-c3-c4: the centre is the area-weighted mean of the two
            // triangles' centroids.
            let n2 = cross(sub(c3, c1), sub(c4, c1));
            let xst = (n2.x * n2.x + n2.y * n2.y + n2.z * n2.z).sqrt();
            let salpn = 1.0 / (3.0 * (xa + xst));
            let center = Point3::new(
                (xa * (c1.x + c2.x + c3.x) + xst * (c1.x + c3.x + c4.x)) * salpn,
                (xa * (c1.y + c2.y + c3.y) + xst * (c1.y + c3.y + c4.y)) * salpn,
                (xa * (c1.z + c2.z + c3.z) + xst * (c1.z + c3.z + c4.z)) * salpn,
            );
            if (n.x * n2.x + n.y * n2.y + n.z * n2.z) / xst <= 0.9998 {
                return Err(PatchError::NotPlanar { line });
            }
            Ok(flat(
                index,
                center,
                s1,
                n,
                0.5 * (xa + xst),
                vec![c1, c2, c3, c4],
            ))
        }
    }
}

/// The unit normal of `s1 × s2`, and that product's length.
fn unit_normal(s1: Point3, s2: Point3) -> (Point3, f64) {
    let v = cross(s1, s2);
    let xa = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt();
    (Point3::new(v.x / xa, v.y / xa, v.z / xa), xa)
}

/// A patch whose first tangent is along `s1`.
fn flat(
    index: usize,
    center: Point3,
    s1: Point3,
    n: Point3,
    area: f64,
    outline: Vec<Point3>,
) -> Patch {
    let xst = (s1.x * s1.x + s1.y * s1.y + s1.z * s1.z).sqrt();
    let t1 = Point3::new(s1.x / xst, s1.y / xst, s1.z / xst);
    Patch {
        index,
        center,
        t1,
        t2: cross(n, t1),
        area,
        salp: 1.0,
        outline,
    }
}

/// `PATCH` for `NX, NY > 0` (`SM`): the parallelogram `c1, c2, c3` divided into `nx × ny`
/// patches, `nx` along `c1→c2` and `ny` along `c2→c3`, numbered from `first` with the `c1→c2`
/// direction varying fastest.
#[must_use]
pub fn surface(
    first: usize,
    nx: usize,
    ny: usize,
    c1: Point3,
    c2: Point3,
    c3: Point3,
) -> Vec<Patch> {
    #[allow(clippy::cast_precision_loss)]
    let (fx, fy) = (nx as f64, ny as f64);
    let s1 = sub(c2, c1).scaled(1.0 / fx);
    let s2 = sub(c3, c2).scaled(1.0 / fy);
    let (n, xa) = unit_normal(s1, s2);
    let proto = flat(0, c1, s1, n, xa, Vec::new());
    // The first patch's centre, then each one stepped from a base point one cell back, as
    // `PATCH` does.
    let c0 = Point3::new(
        c1.x + 0.5 * (s1.x + s2.x),
        c1.y + 0.5 * (s1.y + s2.y),
        c1.z + 0.5 * (s1.z + s2.z),
    );
    let mut base = sub(sub(c0, s1), s2);
    let mut out = Vec::with_capacity(nx * ny);
    for iy in 0..ny {
        base = add(base, s2);
        for ix in 1..=nx {
            #[allow(clippy::cast_precision_loss)]
            let xst = ix as f64;
            let center = Point3::new(
                base.x + xst * s1.x,
                base.y + xst * s1.y,
                base.z + xst * s1.z,
            );
            #[allow(clippy::cast_precision_loss)]
            let corner = add(add(c1, s1.scaled((ix - 1) as f64)), s2.scaled(iy as f64));
            out.push(Patch {
                index: first + out.len(),
                center,
                outline: vec![
                    corner,
                    add(corner, s1),
                    add(add(corner, s1), s2),
                    add(corner, s2),
                ],
                ..proto.clone()
            });
        }
    }
    out
}

/// `SUBPH`: replace patch `at` (0-based) with four quarter patches, for a wire connecting at its
/// centre. The new patches take numbers `at+1 ..= at+4` and every later patch moves up by three.
///
/// The quarters are squares of a quarter of the area, centred `±¼√A` along each tangent, in
/// the order (+,+), (−,+), (−,−), (+,−) — whatever the original shape, as in NEC.
pub fn subdivide(patches: &mut Vec<Patch>, at: usize) {
    let quarters = quarters(&patches[at]);
    patches.splice(at..=at, quarters);
    for (i, q) in patches.iter_mut().enumerate() {
        q.index = i + 1;
    }
}

/// The four quarter patches `SUBPH` makes of `p`, in order, numbered as `p`.
#[must_use]
pub fn quarters(p: &Patch) -> Vec<Patch> {
    let xa = p.area * 0.25;
    let d = xa.sqrt() * 0.5;
    let mut quarters = Vec::with_capacity(4);
    let (mut xt, mut yt) = (d, d);
    for ix in 1..=4 {
        let center = Point3::new(
            p.center.x + xt * p.t1.x + yt * p.t2.x,
            p.center.y + xt * p.t1.y + yt * p.t2.y,
            p.center.z + xt * p.t1.z + yt * p.t2.z,
        );
        let outline = [(1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)]
            .iter()
            .map(|&(a, b)| add(center, add(p.t1.scaled(a * d), p.t2.scaled(b * d))))
            .collect();
        quarters.push(Patch {
            center,
            area: xa,
            outline,
            ..p.clone()
        });
        if ix == 2 {
            yt = -yt;
        }
        if ix == 1 || ix == 3 {
            xt = -xt;
        }
    }
    quarters
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Point3, b: Point3) -> bool {
        a.distance(b) < 1e-12
    }

    #[test]
    fn a_rectangle_has_its_centre_area_and_tangents() {
        let p = single(
            1,
            Shape::Rectangular([
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(2.0, 0.0, 0.0),
                Point3::new(2.0, 1.0, 0.0),
            ]),
            1,
        )
        .expect("valid");
        assert!(close(p.center, Point3::new(1.0, 0.5, 0.0)));
        assert!((p.area - 2.0).abs() < 1e-12);
        assert!(close(p.t1, Point3::new(1.0, 0.0, 0.0)));
        assert!(close(p.t2, Point3::new(0.0, 1.0, 0.0)));
        assert!(close(p.normal(), Point3::new(0.0, 0.0, 1.0)));
    }

    #[test]
    fn an_arbitrary_patch_faces_its_angles() {
        // Nec-Ex4: SP 0 0 .1 .05 .05 0 0 .01 — normal along +X.
        let p = single(
            1,
            Shape::Arbitrary {
                center: Point3::new(0.1, 0.05, 0.05),
                elevation_deg: 0.0,
                azimuth_deg: 0.0,
                area: 0.01,
            },
            1,
        )
        .expect("valid");
        assert!(close(p.normal(), Point3::new(1.0, 0.0, 0.0)));
        assert!(close(p.t1, Point3::new(0.0, 1.0, 0.0)));
        // Straight up: t1 falls back to +X.
        let up = single(
            1,
            Shape::Arbitrary {
                center: Point3::ORIGIN,
                elevation_deg: 90.0,
                azimuth_deg: 0.0,
                area: 1.0,
            },
            1,
        )
        .expect("valid");
        assert!(close(up.t1, Point3::new(1.0, 0.0, 0.0)));
    }

    #[test]
    fn a_surface_is_numbered_along_c1_c2_first() {
        let s = surface(
            5,
            3,
            2,
            Point3::ORIGIN,
            Point3::new(3.0, 0.0, 0.0),
            Point3::new(3.0, 2.0, 0.0),
        );
        assert_eq!(s.len(), 6);
        assert_eq!(s[0].index, 5);
        assert!(close(s[0].center, Point3::new(0.5, 0.5, 0.0)));
        assert!(close(s[1].center, Point3::new(1.5, 0.5, 0.0)));
        assert!(close(s[3].center, Point3::new(0.5, 1.5, 0.0)));
        assert!(s.iter().all(|p| (p.area - 1.0).abs() < 1e-12));
    }

    #[test]
    fn a_quadrilateral_must_be_planar() {
        let warped = Shape::Quadrilateral([
            Point3::ORIGIN,
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(0.0, 1.0, 0.5),
        ]);
        assert_eq!(single(1, warped, 7), Err(PatchError::NotPlanar { line: 7 }));
    }

    #[test]
    fn subdividing_inserts_four_quarters_in_place() {
        let mut ps = surface(
            1,
            3,
            1,
            Point3::ORIGIN,
            Point3::new(3.0, 0.0, 0.0),
            Point3::new(3.0, 1.0, 0.0),
        );
        subdivide(&mut ps, 1);
        assert_eq!(ps.len(), 6);
        assert_eq!(
            ps.iter().map(|p| p.index).collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5, 6]
        );
        // The quarters of the middle patch (centre (1.5, 0.5)), then the old third patch.
        assert!(close(ps[1].center, Point3::new(1.75, 0.75, 0.0)));
        assert!(close(ps[2].center, Point3::new(1.25, 0.75, 0.0)));
        assert!(close(ps[3].center, Point3::new(1.25, 0.25, 0.0)));
        assert!(close(ps[4].center, Point3::new(1.75, 0.25, 0.0)));
        assert!((ps[1].area - 0.25).abs() < 1e-12);
        assert!(close(ps[5].center, Point3::new(2.5, 0.5, 0.0)));
    }
}
