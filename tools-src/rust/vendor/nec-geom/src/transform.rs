//! Structure transforms: `GM` (move and copy), `GX` (reflect), `GR` (cylindrical repeat).
//!
//! All three act on whatever structure exists at that point in the deck, and all three affect
//! segment numbering, which `EX` and `LD` cards depend on. The rules below are quoted from the
//! NEC-2 manual and confirmed against the reference engine by `cargo xtask geometry`.

use crate::patch::{Patch, PatchError};
use crate::segment::Segment;
use nec_core::Point3;

/// A rotation followed by a translation, as `GM` defines it.
///
/// > "The order used is first rotation about X-axis, then rotation about the Y-axis, then
/// > rotation about the Z-axis and, finally, translation by (XS, YS, ZS). All operations refer
/// > to the fixed coordinate system axes."
///
/// Angles are right-handed: "A positive angle causes a right-hand rotation."
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    /// Row-major rotation matrix, the product `Rz · Ry · Rx`.
    m: [[f64; 3]; 3],
    /// Translation applied after rotating.
    t: Point3,
}

impl Transform {
    /// Build from rotation angles in degrees and a translation.
    #[must_use]
    pub fn from_gm(rox_deg: f64, roy_deg: f64, roz_deg: f64, t: Point3) -> Self {
        let (sx, cx) = rox_deg.to_radians().sin_cos();
        let (sy, cy) = roy_deg.to_radians().sin_cos();
        let (sz, cz) = roz_deg.to_radians().sin_cos();
        // Rz · Ry · Rx, written out so each entry can be checked by hand.
        let m = [
            [cy * cz, sx * sy * cz - cx * sz, cx * sy * cz + sx * sz],
            [cy * sz, sx * sy * sz + cx * cz, cx * sy * sz - sx * cz],
            [-sy, sx * cy, cx * cy],
        ];
        Self { m, t }
    }

    /// Apply to a point.
    #[must_use]
    pub fn apply(&self, p: Point3) -> Point3 {
        let m = &self.m;
        Point3::new(
            m[0][0] * p.x + m[0][1] * p.y + m[0][2] * p.z + self.t.x,
            m[1][0] * p.x + m[1][1] * p.y + m[1][2] * p.z + self.t.y,
            m[2][0] * p.x + m[2][1] * p.y + m[2][2] * p.z + self.t.z,
        )
    }

    /// Apply the rotation alone, to a direction.
    #[must_use]
    pub fn rotate(&self, v: Point3) -> Point3 {
        let m = &self.m;
        Point3::new(
            m[0][0] * v.x + m[0][1] * v.y + m[0][2] * v.z,
            m[1][0] * v.x + m[1][1] * v.y + m[1][2] * v.z,
            m[2][0] * v.x + m[2][1] * v.y + m[2][2] * v.z,
        )
    }

    fn apply_patch(&self, p: &Patch) -> Patch {
        Patch {
            center: self.apply(p.center),
            t1: self.rotate(p.t1),
            t2: self.rotate(p.t2),
            outline: p.outline.iter().map(|&q| self.apply(q)).collect(),
            ..p.clone()
        }
    }

    fn apply_segment(&self, s: &Segment) -> Segment {
        Segment {
            a: self.apply(s.a),
            b: self.apply(s.b),
            ..s.clone()
        }
    }
}

/// Increment a tag, except that tag 0 is never incremented.
///
/// > "Tags of segments which are generated from segments having no tags (tag equal to zero) are
/// > not incremented."
fn bump(tag: i32, by: i32) -> i32 {
    if tag == 0 { 0 } else { tag + by }
}

/// Renumber segments to match their position, after anything that appends or reorders.
fn renumber(segs: &mut [Segment]) {
    for (i, s) in segs.iter_mut().enumerate() {
        s.index = i + 1;
    }
}

fn renumber_patches(patches: &mut [Patch]) {
    for (i, p) in patches.iter_mut().enumerate() {
        p.index = i + 1;
    }
}

/// `GM` on surface patches. Unlike wire segments, `MOVE` applies to every patch whatever
/// `ITS` says, and patches have no tags to increment.
pub fn gm_patches(patches: &mut Vec<Patch>, nrpt: usize, xf: &Transform, first: usize) {
    if nrpt == 0 {
        for p in &mut patches[first..] {
            *p = xf.apply_patch(p);
        }
        return;
    }
    let mut previous: Vec<Patch> = patches[first..].to_vec();
    for _ in 0..nrpt {
        let next: Vec<Patch> = previous.iter().map(|p| xf.apply_patch(p)).collect();
        patches.extend(next.iter().cloned());
        previous = next;
    }
    renumber_patches(patches);
}

/// `GX` on surface patches (`REFLC`): each image has its centre and tangents mirrored, which
/// turns the tangent pair left-handed, so its `SALP` changes sign. A patch centred in the plane
/// of reflection would coincide with its own image, which NEC refuses.
pub fn gx_patches(patches: &mut Vec<Patch>, flags: i32, line: usize, first: usize) -> Result<(), PatchError> {
    let along_x = (flags / 100) % 10 != 0;
    let along_y = (flags / 10) % 10 != 0;
    let along_z = flags % 10 != 0;
    for (enabled, axis) in [(along_z, 2u8), (along_y, 1), (along_x, 0)] {
        if !enabled {
            continue;
        }
        let flip = |p: Point3| match axis {
            0 => Point3::new(-p.x, p.y, p.z),
            1 => Point3::new(p.x, -p.y, p.z),
            _ => Point3::new(p.x, p.y, -p.z),
        };
        let coord = |p: Point3| match axis {
            0 => p.x,
            1 => p.y,
            _ => p.z,
        };
        let mut images = Vec::with_capacity(patches.len());
        for p in &patches[first..] {
            if coord(p.center).abs() <= 1e-10 {
                return Err(PatchError::Data {
                    line,
                    reason: "a patch lies in the plane of symmetry",
                });
            }
            images.push(Patch {
                center: flip(p.center),
                t1: flip(p.t1),
                t2: flip(p.t2),
                salp: -p.salp,
                outline: p.outline.iter().map(|&q| flip(q)).collect(),
                ..p.clone()
            });
        }
        patches.extend(images);
    }
    renumber_patches(patches);
    Ok(())
}

/// `GR` on surface patches: the same as the equivalent `GM`.
pub fn gr_patches(patches: &mut Vec<Patch>, total: usize, first: usize) {
    if total <= 1 {
        return;
    }
    #[allow(clippy::cast_precision_loss)]
    let step = 360.0 / total as f64;
    gm_patches(patches, total - 1, &Transform::from_gm(0.0, 0.0, step, Point3::ORIGIN), first);
}

/// `GM ITGI NRPT ROX ROY ROZ XS YS ZS ITS`.
///
/// > "If NRPT is zero, the structure is moved by the specified rotation and translation leaving
/// > nothing in the original location. If NRPT is greater than zero, the original structure
/// > remains fixed and NRPT new structures are formed, each shifted from the previous one by
/// > the requested transformation."
///
/// `ITS` restricts the card to "the sequence of segments" starting at the first segment carrying
/// tag `ITS`; zero or blank means the whole structure. With `NRPT = 0` the tags of the moved
/// segments are themselves incremented by `ITGI`.
///
/// Segments before `first` — a numerical Green's function's structure, after `GF` — never move.
pub fn gm(segs: &mut Vec<Segment>, itgi: i32, nrpt: usize, xf: &Transform, its: i32, first: usize) {
    let start = if its == 0 {
        first
    } else {
        match segs.iter().position(|s| s.tag == its) {
            Some(i) => i.max(first),
            None => return, // no such tag: nothing to move
        }
    };

    if nrpt == 0 {
        for s in &mut segs[start..] {
            *s = xf.apply_segment(s);
            s.tag = bump(s.tag, itgi);
        }
        return;
    }

    let mut previous: Vec<Segment> = segs[start..].to_vec();
    for _ in 0..nrpt {
        let next: Vec<Segment> = previous
            .iter()
            .map(|s| {
                let mut c = xf.apply_segment(s);
                c.tag = bump(s.tag, itgi);
                c
            })
            .collect();
        segs.extend(next.iter().cloned());
        previous = next;
    }
    renumber(segs);
}

/// `GX ITX ×yz` — reflect the structure in the coordinate planes.
///
/// The second field is three digits, one per axis, in X-Y-Z order: `110` reflects along X and Y.
///
/// > "When combinations of reflections are requested, the reflections are done in reverse
/// > alphabetical order. ... the structure is first reflected along the Z-axis; the structure
/// > and its image are then reflected along the Y-axis; and, finally, these four structures are
/// > reflected along the X-axis"
///
/// and the tag increment "is doubled after each reflection." Segments before `first` (a
/// numerical Green's function's) are neither reflected nor copied.
pub fn gx(segs: &mut Vec<Segment>, itx: i32, flags: i32, first: usize) {
    let along_x = (flags / 100) % 10 != 0;
    let along_y = (flags / 10) % 10 != 0;
    let along_z = flags % 10 != 0;

    let mut inc = itx;
    for (enabled, axis) in [(along_z, 2u8), (along_y, 1), (along_x, 0)] {
        if !enabled {
            continue;
        }
        let flip = |p: Point3| match axis {
            0 => Point3::new(-p.x, p.y, p.z),
            1 => Point3::new(p.x, -p.y, p.z),
            _ => Point3::new(p.x, p.y, -p.z),
        };
        let images: Vec<Segment> = segs[first..]
            .iter()
            .map(|s| Segment {
                a: flip(s.a),
                b: flip(s.b),
                tag: bump(s.tag, inc),
                ..s.clone()
            })
            .collect();
        segs.extend(images);
        inc *= 2;
    }
    renumber(segs);
}

/// `GR ITGI NTOTAL` — repeat the structure around the Z axis.
///
/// > "The GR card produces the same effect on the structure as a GM card if I2 on the GR card is
/// > equal to (NRPT+1) on the GM card and if ROZ on the GM card is equal to 360/(NRPT+1)
/// > degrees."
pub fn gr(segs: &mut Vec<Segment>, itgi: i32, total: usize, first: usize) {
    if total <= 1 {
        return;
    }
    #[allow(clippy::cast_precision_loss)]
    let step = 360.0 / total as f64;
    let xf = Transform::from_gm(0.0, 0.0, step, Point3::ORIGIN);
    gm(segs, itgi, total - 1, &xf, 0, first);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Point3, b: Point3) -> bool {
        a.distance(b) < 1e-12
    }

    fn seg(tag: i32, a: Point3, b: Point3) -> Segment {
        Segment {
            index: 1,
            tag,
            a,
            b,
            radius: 1e-3,
        }
    }

    #[test]
    fn rotations_are_right_handed() {
        // +90 about Z takes +X to +Y; about X takes +Y to +Z; about Y takes +Z to +X.
        let z = Transform::from_gm(0.0, 0.0, 90.0, Point3::ORIGIN);
        assert!(close(z.apply(Point3::new(1.0, 0.0, 0.0)), Point3::new(0.0, 1.0, 0.0)));
        let x = Transform::from_gm(90.0, 0.0, 0.0, Point3::ORIGIN);
        assert!(close(x.apply(Point3::new(0.0, 1.0, 0.0)), Point3::new(0.0, 0.0, 1.0)));
        let y = Transform::from_gm(0.0, 90.0, 0.0, Point3::ORIGIN);
        assert!(close(y.apply(Point3::new(0.0, 0.0, 1.0)), Point3::new(1.0, 0.0, 0.0)));
    }

    #[test]
    fn rotation_order_is_x_then_y_then_z() {
        // X then Y applied to +Y: Rx(90) sends it to +Z, then Ry(90) sends +Z to +X.
        // The other order would leave it at +Z.
        let xy = Transform::from_gm(90.0, 90.0, 0.0, Point3::ORIGIN);
        assert!(close(xy.apply(Point3::new(0.0, 1.0, 0.0)), Point3::new(1.0, 0.0, 0.0)));
    }

    #[test]
    fn translation_comes_after_rotation() {
        let xf = Transform::from_gm(0.0, 0.0, 90.0, Point3::new(10.0, 0.0, 0.0));
        // Rotate (1,0,0) to (0,1,0), then translate: (10,1,0). Translate-first would give (0,11,0).
        assert!(close(xf.apply(Point3::new(1.0, 0.0, 0.0)), Point3::new(10.0, 1.0, 0.0)));
    }

    #[test]
    fn gm_with_nrpt_zero_moves_in_place_and_bumps_tags() {
        let mut s = vec![seg(1, Point3::ORIGIN, Point3::new(1.0, 0.0, 0.0))];
        let xf = Transform::from_gm(0.0, 0.0, 0.0, Point3::new(0.0, 0.0, 5.0));
        gm(&mut s, 100, 0, &xf, 0, 0);
        assert_eq!(s.len(), 1);
        assert!(close(s[0].a, Point3::new(0.0, 0.0, 5.0)));
        assert_eq!(s[0].tag, 101);
    }

    #[test]
    fn gm_copies_are_each_shifted_from_the_previous_one() {
        let mut s = vec![seg(1, Point3::ORIGIN, Point3::new(1.0, 0.0, 0.0))];
        let xf = Transform::from_gm(0.0, 0.0, 0.0, Point3::new(0.0, 2.0, 0.0));
        gm(&mut s, 1, 3, &xf, 0, 0);
        assert_eq!(s.len(), 4);
        let ys: Vec<f64> = s.iter().map(|x| x.a.y).collect();
        assert_eq!(ys, vec![0.0, 2.0, 4.0, 6.0]);
        assert_eq!(s.iter().map(|x| x.tag).collect::<Vec<_>>(), vec![1, 2, 3, 4]);
        assert_eq!(s.iter().map(|x| x.index).collect::<Vec<_>>(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn untagged_segments_keep_tag_zero() {
        let mut s = vec![seg(0, Point3::ORIGIN, Point3::new(1.0, 0.0, 0.0))];
        let xf = Transform::from_gm(0.0, 0.0, 0.0, Point3::new(0.0, 1.0, 0.0));
        gm(&mut s, 50, 2, &xf, 0, 0);
        assert!(s.iter().all(|x| x.tag == 0));
    }

    #[test]
    fn gm_its_moves_only_from_the_first_matching_tag_onward() {
        let mut s = vec![
            seg(1, Point3::ORIGIN, Point3::new(1.0, 0.0, 0.0)),
            seg(2, Point3::new(5.0, 0.0, 0.0), Point3::new(6.0, 0.0, 0.0)),
        ];
        let xf = Transform::from_gm(0.0, 0.0, 0.0, Point3::new(0.0, 0.0, 1.0));
        gm(&mut s, 0, 0, &xf, 2, 0);
        assert!(close(s[0].a, Point3::ORIGIN), "tag 1 precedes ITS and must not move");
        assert!(close(s[1].a, Point3::new(5.0, 0.0, 1.0)));
    }

    #[test]
    fn gx_reflects_z_then_y_then_x_and_doubles_the_tag_increment() {
        // The manual's own example: tags 1..100, I2 = 011, increment 100 -> the Z image gets
        // 101..200 and the Y reflection of both gets 201..400.
        let mut s = vec![seg(1, Point3::new(1.0, 2.0, 3.0), Point3::new(1.0, 2.0, 4.0))];
        gx(&mut s, 100, 11, 0);
        assert_eq!(s.len(), 4);
        assert_eq!(s.iter().map(|x| x.tag).collect::<Vec<_>>(), vec![1, 101, 201, 301]);
        // Z image first, then the Y images of the original and of the Z image.
        assert!(close(s[1].a, Point3::new(1.0, 2.0, -3.0)));
        assert!(close(s[2].a, Point3::new(1.0, -2.0, 3.0)));
        assert!(close(s[3].a, Point3::new(1.0, -2.0, -3.0)));
    }

    #[test]
    fn gx_100_reflects_along_x_only() {
        let mut s = vec![seg(1, Point3::new(1.0, 2.0, 3.0), Point3::new(2.0, 2.0, 3.0))];
        gx(&mut s, 1, 100, 0);
        assert_eq!(s.len(), 2);
        assert!(close(s[1].a, Point3::new(-1.0, 2.0, 3.0)));
    }

    #[test]
    fn gr_matches_the_equivalent_gm() {
        let base = vec![seg(1, Point3::new(1.0, 0.0, 0.0), Point3::new(2.0, 0.0, 0.0))];
        let mut via_gr = base.clone();
        gr(&mut via_gr, 10, 4, 0);
        let mut via_gm = base;
        gm(&mut via_gm, 10, 3, &Transform::from_gm(0.0, 0.0, 90.0, Point3::ORIGIN), 0, 0);
        assert_eq!(via_gr.len(), 4);
        for (a, b) in via_gr.iter().zip(&via_gm) {
            assert!(close(a.a, b.a) && close(a.b, b.b));
            assert_eq!(a.tag, b.tag);
        }
        // The fourth copy sits at 270 degrees.
        assert!(close(via_gr[3].a, Point3::new(0.0, -1.0, 0.0)));
    }
}
