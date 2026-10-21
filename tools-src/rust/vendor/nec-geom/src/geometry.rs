//! Turning a resolved card deck into segments.

use crate::connect::{CONNECT_FRACTION, Connectivity, Junction, WireEnd, build_connectivity};
use crate::patch::{self, Patch, PatchError};
use crate::segment::Segment;
use crate::transform::{self, Transform};
use nec_cards::{Card, Deck, Item};
use nec_core::Point3;

/// What the `GE` card says about ground.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GroundFlag {
    /// `GE 0` — free space, no ground plane.
    #[default]
    FreeSpace,
    /// `GE 1` — ground plane present, and wire ends at Z = 0 are bonded to it.
    GroundedEnds,
    /// `GE -1` — ground plane present, but wire ends are not connected to it.
    Ground,
}

impl GroundFlag {
    /// Interpret the `GE` card's first field.
    #[must_use]
    pub fn from_code(code: i32) -> Self {
        match code {
            1 => Self::GroundedEnds,
            -1 => Self::Ground,
            // NEC treats anything else as free space; the quick reference lists only 0, 1, -1.
            _ => Self::FreeSpace,
        }
    }

    /// Whether a ground plane exists at all.
    #[must_use]
    pub fn has_ground(self) -> bool {
        !matches!(self, Self::FreeSpace)
    }
}

/// Errors while building geometry.
#[derive(Debug, thiserror::Error)]
pub enum GeomError {
    /// A `GW` card asked for a non-positive segment count.
    #[error("line {line}: GW tag {tag} requests {count} segments")]
    BadSegmentCount {
        /// Source line.
        line: usize,
        /// Wire tag.
        tag: i32,
        /// Requested count.
        count: i64,
    },

    /// A `GW` card described a wire of zero length.
    #[error("line {line}: GW tag {tag} has zero length")]
    ZeroLengthWire {
        /// Source line.
        line: usize,
        /// Wire tag.
        tag: i32,
    },

    /// A card was malformed.
    #[error(transparent)]
    Card(#[from] nec_cards::CardError),

    /// The model exceeds this implementation's segment ceiling.
    #[error("model has {actual} unknowns (one per segment, two per patch); this build supports {limit}")]
    TooManySegments {
        /// Segment count requested.
        actual: usize,
        /// The enforced limit.
        limit: usize,
    },

    /// A card in the geometry section that is not a NEC-2 geometry card at all.
    ///
    /// The case that prompted this: `Nec4\Catenary.nec` builds its wire with `CW`, a NEC-4
    /// catenary card. It used to be ignored silently, yielding an empty structure.
    #[error("line {line}: {mnemonic} is not a NEC-2 geometry card{hint}")]
    NotNec2Geometry {
        /// Source line.
        line: usize,
        /// The card mnemonic.
        mnemonic: String,
        /// Extra context when the card is recognizably from another NEC version.
        hint: &'static str,
    },

    /// A malformed geometry card.
    #[error("line {line}: {reason}")]
    Data {
        /// Source line.
        line: usize,
        /// What was wrong.
        reason: &'static str,
    },

    /// Malformed surface-patch input.
    #[error(transparent)]
    Patch(#[from] PatchError),

    /// A `GF` card, but no numerical Green's function to read.
    #[error("line {line}: GF needs a numerical Green's function, written by a WG card")]
    NeedsGreensFunction {
        /// Source line.
        line: usize,
    },

    /// A geometry card is recognized but not yet implemented.
    #[error("line {line}: {mnemonic} card is not implemented yet")]
    Unimplemented {
        /// Source line.
        line: usize,
        /// The card mnemonic.
        mnemonic: String,
    },
}

/// The largest model this build will attempt, in unknowns: one per wire segment and two per
/// surface patch.
///
/// At 8000 unknowns the complex interaction matrix is about 1 GB, which is the practical
/// ceiling for an in-core dense solve. The reference corpus is far below this: 99% of models
/// are under 2100 segments and the median is 85, so refusing larger models with a clear error
/// is better than building the out-of-core machinery NEC-2 needed in the 1980s.
pub const MAX_SEGMENTS: usize = 8000;

/// A built structure: wire segments and surface patches.
#[derive(Debug, Clone, Default)]
pub struct Geometry {
    /// Segments in NEC's numbering order, index 0 holding segment 1.
    pub segments: Vec<Segment>,
    /// Surface patches in NEC's numbering order, index 0 holding patch 1.
    pub patches: Vec<Patch>,
    /// What `GE` declared about ground.
    pub ground: GroundFlag,
    /// Per-segment endpoint connectivity.
    pub connectivity: Vec<Connectivity>,
    /// Segments that came from a numerical Green's function (`GF`): the first this many.
    pub base_segments: usize,
    /// Patches that came from a numerical Green's function: the first this many.
    pub base_patches: usize,
    /// Patches of the numerical Green's function that a new wire ends on (0-based). Each is
    /// replaced by four quarter patches appended after every other patch, as NEC's `SUBPH` does
    /// for a Green's function; the saved patch stays in place, but carries no current.
    pub superseded_patches: Vec<usize>,
    /// For each segment, the deck line of the card that made it: the `GW`, `GA` or `GH` card,
    /// or the `GM`, `GX` or `GR` card whose copy it is. Zero for a segment of a numerical
    /// Green's function. An editor uses this to find the card behind a picked segment.
    pub origins: Vec<usize>,
}

/// A patch shape waiting for its `SC` card.
enum PendingPatch {
    /// `SP` with `NS = 1, 2, 3`: corners 1 and 2 are known.
    Single { ns: i32, c1: Point3, c2: Point3 },
    /// `SM NX NY` with corners 1 and 2.
    Surface { nx: usize, ny: usize, c1: Point3, c2: Point3 },
}

impl Geometry {
    /// Build geometry from a deck that has already had `resolve()` called on it.
    ///
    /// Cards are processed in source order, because `GS` scaling and the `GM`/`GX`/`GR`
    /// transforms act on whatever structure exists at that point in the deck.
    pub fn build(deck: &Deck) -> Result<Self, GeomError> {
        Self::build_on(deck, None)
    }

    /// [`Geometry::build`] for a deck that may begin with `GF`: the structure of a numerical
    /// Green's function, `base`, comes first, and the deck's own cards add to it.
    ///
    /// As in NEC, `GS`, `GM`, `GX` and `GR` then act on the new segments and patches only, new
    /// segments are numbered after the base's, and a new wire may join a base wire but not end
    /// on a base patch.
    #[allow(clippy::too_many_lines)]
    pub fn build_on(deck: &Deck, base: Option<&Geometry>) -> Result<Self, GeomError> {
        let mut segs: Vec<Segment> = Vec::new();
        let mut patches: Vec<Patch> = Vec::new();
        let mut ground = GroundFlag::FreeSpace;
        let (mut first_seg, mut first_patch) = (0, 0);

        let cards: Vec<&Card> = deck
            .items
            .iter()
            .filter_map(|it| match it {
                Item::Card(c) => Some(c),
                _ => None,
            })
            .collect();
        // Corners 3 and 4 of the last rectangular or quadrilateral patch, which a following `SC`
        // card may continue from (NEC's `ISCT`).
        let mut strip: Option<(Point3, Point3)> = None;
        let mut origins: Vec<usize> = Vec::new();
        let mut last_line = 0;
        let mut k = 0;
        while k < cards.len() {
            let card = cards[k];
            k += 1;
            // Segments the previous card added are its own.
            origins.resize(segs.len(), last_line);
            last_line = card.line;
            let prior_strip = strip.take();
            match card.mnemonic.as_str() {
                "GW" => {
                    let radius = card.real_or(8, 0.0)?;
                    // A zero radius means a tapered wire, specified by the `GC` card that must
                    // follow.
                    let taper = if radius == 0.0 {
                        match cards.get(k) {
                            Some(gc) if gc.mnemonic == "GC" => {
                                k += 1;
                                Some(Taper::from_gc(gc, card.int(1)?)?)
                            }
                            _ => {
                                return Err(GeomError::Data {
                                    line: card.line,
                                    reason: "a GW card with zero radius must be followed by a GC card",
                                });
                            }
                        }
                    } else {
                        None
                    };
                    add_wire(card, &mut segs, taper)?;
                }
                "GA" => add_arc(card, &mut segs)?,
                "GH" => add_helix(card, &mut segs)?,
                "GS" => scale(card, &mut segs[first_seg..], &mut patches[first_patch..])?,
                "GM" => {
                    let xf = Transform::from_gm(
                        card.real_or(2, 0.0)?,
                        card.real_or(3, 0.0)?,
                        card.real_or(4, 0.0)?,
                        Point3::new(card.real_or(5, 0.0)?, card.real_or(6, 0.0)?, card.real_or(7, 0.0)?),
                    );
                    // ITS is read as a decimal and rounded: "This number is input as a decimal
                    // number but is rounded to an integer before use."
                    #[allow(clippy::cast_possible_truncation)]
                    let its = card.real_or(8, 0.0)?.round() as i32;
                    let nrpt = usize::try_from(card.int(1).unwrap_or(0)).unwrap_or(0);
                    transform::gm(&mut segs, card.int(0).unwrap_or(0), nrpt, &xf, its, first_seg);
                    transform::gm_patches(&mut patches, nrpt, &xf, first_patch);
                }
                "GX" => {
                    transform::gx(&mut segs, card.int(0).unwrap_or(0), card.int(1).unwrap_or(0), first_seg);
                    transform::gx_patches(&mut patches, card.int(1).unwrap_or(0), card.line, first_patch)?;
                }
                "GR" => {
                    let total = usize::try_from(card.int(1).unwrap_or(0)).unwrap_or(0);
                    transform::gr(&mut segs, card.int(0).unwrap_or(0), total, first_seg);
                    transform::gr_patches(&mut patches, total, first_patch);
                }
                "SP" => {
                    if card.int(0)? != 0 {
                        return Err(patch_error(card.line, "the first field of an SP card must be 0"));
                    }
                    let ns = card.int(1)?;
                    let c1 = point(card, 2)?;
                    if ns == 0 {
                        patches.push(patch::single(
                            patches.len() + 1,
                            patch::Shape::Arbitrary {
                                center: c1,
                                elevation_deg: card.real_or(5, 0.0)?,
                                azimuth_deg: card.real_or(6, 0.0)?,
                                area: card.real_or(7, 0.0)?,
                            },
                            card.line,
                        )?);
                    } else if (1..=3).contains(&ns) {
                        let pending = PendingPatch::Single { ns, c1, c2: point(card, 5)? };
                        strip = complete_patch(pending, cards.get(k).copied(), card.line, &mut patches)?;
                        k += 1;
                    } else {
                        return Err(patch_error(card.line, "SP patch shape must be 0 to 3"));
                    }
                }
                "SM" => {
                    let nx = usize::try_from(card.int(0)?).unwrap_or(0);
                    let ny = usize::try_from(card.int(1)?).unwrap_or(0);
                    if nx < 1 || ny < 1 {
                        return Err(patch_error(card.line, "SM needs at least one patch each way"));
                    }
                    let pending = PendingPatch::Surface { nx, ny, c1: point(card, 2)?, c2: point(card, 5)? };
                    complete_patch(pending, cards.get(k).copied(), card.line, &mut patches)?;
                    k += 1;
                }
                // An `SC` on its own continues a strip of rectangular or quadrilateral patches:
                // the new patch's first two corners are the last one's fourth and third.
                "SC" => {
                    let Some((c3_prev, c4_prev)) = prior_strip else {
                        return Err(patch_error(card.line, "SC card does not follow a patch it can continue"));
                    };
                    if card.int(0)? != 0 {
                        return Err(patch_error(card.line, "the first field of an SC card must be 0"));
                    }
                    let ns = card.int(1)? + 1;
                    let (c1, c2, c3) = (c4_prev, c3_prev, point(card, 2)?);
                    let shape = match ns {
                        2 => patch::Shape::Rectangular([c1, c2, c3]),
                        4 => patch::Shape::Quadrilateral([c1, c2, c3, point(card, 5)?]),
                        _ => return Err(patch_error(card.line, "a continuing SC card must give shape 1 or 3")),
                    };
                    let p = patch::single(patches.len() + 1, shape, card.line)?;
                    strip = Some(strip_corners(&p));
                    patches.push(p);
                }
                "GE" => {
                    ground = GroundFlag::from_code(card.int(0).unwrap_or(0));
                    // GE ends the geometry section; later cards are program control.
                    break;
                }
                // The numerical Green's function: the saved structure comes first.
                "GF" => {
                    if !segs.is_empty() || !patches.is_empty() {
                        return Err(GeomError::Data {
                            line: card.line,
                            reason: "GF must be the first geometry card",
                        });
                    }
                    let Some(base) = base else {
                        return Err(GeomError::NeedsGreensFunction { line: card.line });
                    };
                    segs.clone_from(&base.segments);
                    patches.clone_from(&base.patches);
                    (first_seg, first_patch) = (segs.len(), patches.len());
                    last_line = 0;
                }
                // A GC that does not follow a zero-radius GW.
                "GC" => {
                    return Err(GeomError::Data {
                        line: card.line,
                        reason: "a GC card must follow a GW card with zero radius",
                    });
                }
                // Anything else before GE is not NEC-2 geometry. Refuse it rather than skip it:
                // an ignored geometry card means a silently wrong structure.
                other => {
                    let hint = match other {
                        "CW" | "IS" | "VC" | "UM" => " (it is a NEC-4 card)",
                        _ => "",
                    };
                    return Err(GeomError::NotNec2Geometry {
                        line: card.line,
                        mnemonic: card.mnemonic.clone(),
                        hint,
                    });
                }
            }
        }

        origins.resize(segs.len(), last_line);

        let unknowns = segs.len() + 2 * patches.len();
        if unknowns > MAX_SEGMENTS {
            return Err(GeomError::TooManySegments {
                actual: unknowns,
                limit: MAX_SEGMENTS,
            });
        }

        let mut connectivity = build_connectivity(&segs, ground);
        connect_patches(&segs, &mut patches, &mut connectivity, first_patch);
        // A new wire ending on a base patch: the saved patch cannot be split in place, so four
        // quarters are appended and the saved one superseded (NEC's NPCON). Base patches in
        // order, each against the new segments' end 1 then end 2.
        let mut superseded_patches = Vec::new();
        let l1 = |p: Point3, q: Point3| (p.x - q.x).abs() + (p.y - q.y).abs() + (p.z - q.z).abs();
        for i in 0..first_patch {
            let c = patches[i].center;
            for s in &segs[first_seg..] {
                let slen = l1(s.a, s.b) * CONNECT_FRACTION;
                let end = if l1(s.a, c) <= slen {
                    WireEnd::A
                } else if l1(s.b, c) <= slen {
                    WireEnd::B
                } else {
                    continue;
                };
                let first = patches.len();
                let mut quarters = patch::quarters(&patches[i]);
                for (k, q) in quarters.iter_mut().enumerate() {
                    q.index = first + k + 1;
                }
                patches.extend(quarters);
                let slot = &mut connectivity[s.index - 1];
                match end {
                    WireEnd::A => slot.end1 = Junction::Patch(first + 1),
                    WireEnd::B => slot.end2 = Junction::Patch(first + 1),
                }
                superseded_patches.push(i);
                break;
            }
        }
        Ok(Self {
            segments: segs,
            patches,
            ground,
            connectivity,
            base_segments: first_seg,
            base_patches: first_patch,
            superseded_patches,
            origins,
        })
    }

    /// Number of segments.
    #[must_use]
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    /// Whether the structure has no segments and no patches.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty() && self.patches.is_empty()
    }

    /// The order of the linear system: one unknown per segment, two per patch.
    #[must_use]
    pub fn unknowns(&self) -> usize {
        self.segments.len() + 2 * self.patches.len()
    }

    /// Segment by 1-based NEC number.
    #[must_use]
    pub fn segment(&self, number: usize) -> Option<&Segment> {
        self.segments.get(number.checked_sub(1)?)
    }

    /// All segments carrying a given tag, in order.
    pub fn by_tag(&self, tag: i32) -> impl Iterator<Item = &Segment> {
        self.segments.iter().filter(move |s| s.tag == tag)
    }

    /// Resolve a (tag, position) pair as `EX` and `LD` cards address segments.
    ///
    /// With `tag == 0` the position is an absolute segment number; otherwise it is the
    /// n-th segment carrying that tag, counting from 1.
    #[must_use]
    pub fn resolve_segment(&self, tag: i32, position: usize) -> Option<&Segment> {
        if tag == 0 {
            self.segment(position)
        } else {
            self.by_tag(tag).nth(position.checked_sub(1)?)
        }
    }
}

fn patch_error(line: usize, reason: &'static str) -> GeomError {
    GeomError::Patch(PatchError::Data { line, reason })
}

fn point(card: &Card, first: usize) -> Result<Point3, GeomError> {
    Ok(Point3::new(card.real_or(first, 0.0)?, card.real_or(first + 1, 0.0)?, card.real_or(first + 2, 0.0)?))
}

/// The corners an `SC` continuation starts from: the patch's third and fourth.
fn strip_corners(p: &Patch) -> (Point3, Point3) {
    (p.outline[2], p.outline[3])
}

/// Finish an `SP` (shapes 1–3) or `SM` card with the `SC` card that must follow it. Returns the
/// corners a later `SC` may continue from, for rectangles and quadrilaterals.
fn complete_patch(
    pending: PendingPatch,
    next: Option<&Card>,
    line: usize,
    patches: &mut Vec<Patch>,
) -> Result<Option<(Point3, Point3)>, GeomError> {
    let Some(sc) = next.filter(|c| c.mnemonic == "SC") else {
        return Err(patch_error(line, "this patch card must be followed by an SC card"));
    };
    let c3 = point(sc, 2)?;
    match pending {
        PendingPatch::Single { ns, c1, c2 } => {
            let shape = match ns {
                1 => patch::Shape::Rectangular([c1, c2, c3]),
                2 => patch::Shape::Triangular([c1, c2, c3]),
                _ => patch::Shape::Quadrilateral([c1, c2, c3, point(sc, 5)?]),
            };
            let p = patch::single(patches.len() + 1, shape, line)?;
            let corners = (ns != 2).then(|| strip_corners(&p));
            patches.push(p);
            Ok(corners)
        }
        PendingPatch::Surface { nx, ny, c1, c2 } => {
            patches.extend(patch::surface(patches.len() + 1, nx, ny, c1, c2, c3));
            Ok(None)
        }
    }
}

/// Wire–surface connections (the patch half of `CONECT`): a segment end at a patch's centre
/// joins the wire to the surface, and the patch is split into four around the junction.
///
/// Patches are taken in order, each against every segment's end 1 then end 2; the first end
/// within `10⁻³` of its segment's L1 length (`|Δx| + |Δy| + |Δz|`, unlike the Euclidean length
/// used between wires) wins. Each patch takes at most one wire.
fn connect_patches(segs: &[Segment], patches: &mut Vec<Patch>, conn: &mut [Connectivity], first: usize) {
    let l1 = |p: Point3, q: Point3| (p.x - q.x).abs() + (p.y - q.y).abs() + (p.z - q.z).abs();
    let mut i = first;
    while i < patches.len() {
        let c = patches[i].center;
        for s in segs {
            let slen = l1(s.a, s.b) * CONNECT_FRACTION;
            let end = if l1(s.a, c) <= slen {
                Some(WireEnd::A)
            } else if l1(s.b, c) <= slen {
                Some(WireEnd::B)
            } else {
                None
            };
            if let Some(end) = end {
                let slot = &mut conn[s.index - 1];
                match end {
                    WireEnd::A => slot.end1 = Junction::Patch(i + 1),
                    WireEnd::B => slot.end2 = Junction::Patch(i + 1),
                }
                patch::subdivide(patches, i);
                break;
            }
        }
        i += 1;
    }
}

/// A tapered wire's progression, from the `GC` card that follows a zero-radius `GW`.
#[derive(Debug, Clone, Copy)]
struct Taper {
    /// Ratio of each segment's length to the previous one's (`RDEL`).
    rdel: f64,
    /// First segment's radius.
    rad1: f64,
    /// Ratio of each segment's radius to the previous one's (`RRAD`).
    rrad: f64,
}

impl Taper {
    /// `GC I1 I2 RDEL RAD1 RAD2`: `RRAD = (RAD2/RAD1)^(1/(NS−1))`.
    fn from_gc(gc: &Card, ns: i32) -> Result<Self, GeomError> {
        let rdel = gc.real_or(2, 1.0)?;
        let rad1 = gc.real_or(3, 0.0)?;
        let rad2 = gc.real_or(4, 0.0)?;
        if rad1 == 0.0 || rad2 == 0.0 {
            return Err(GeomError::Data { line: gc.line, reason: "a GC card needs both end radii" });
        }
        // `YS1=(ZS1/YS1)**(1./(NS-1.))`: the exponent is computed in single precision.
        #[allow(clippy::cast_precision_loss)]
        let exponent = if ns > 1 { f64::from(1.0_f32 / (ns as f32 - 1.0)) } else { 0.0 };
        Ok(Self {
            rdel,
            rad1,
            rrad: (rad2 / rad1).powf(exponent),
        })
    }
}

/// `GW tag ns x1 y1 z1 x2 y2 z2 radius` — divide the wire into `ns` segments: equal ones, or a
/// geometric progression of lengths and radii for a tapered wire (`WIRE`).
fn add_wire(card: &Card, segs: &mut Vec<Segment>, taper: Option<Taper>) -> Result<(), GeomError> {
    let tag = card.int(0)?;
    let count = i64::from(card.int(1)?);
    if count <= 0 {
        return Err(GeomError::BadSegmentCount {
            line: card.line,
            tag,
            count,
        });
    }
    let n = usize::try_from(count).map_err(|_| GeomError::BadSegmentCount {
        line: card.line,
        tag,
        count,
    })?;

    let p1 = Point3::new(card.real(2)?, card.real(3)?, card.real(4)?);
    let p2 = Point3::new(card.real(5)?, card.real(6)?, card.real(7)?);

    if p1.distance(p2) <= 0.0 {
        return Err(GeomError::ZeroLengthWire {
            line: card.line,
            tag,
        });
    }

    let d = p1.to(p2);
    let base = segs.len();
    match taper {
        Some(t) if (t.rdel - 1.0).abs() >= 1e-6 => {
            // Segment lengths in geometric progression; positions accumulate as `WIRE` does, and
            // the last end is pinned to the wire's end.
            let len = d.norm();
            let u = d.scaled(1.0 / len);
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            let mut delz = len * (1.0 - t.rdel) / (1.0 - t.rdel.powi(n as i32));
            let mut radius = t.rad1;
            let mut a = p1;
            for i in 0..n {
                let b = if i + 1 == n { p2 } else { Point3::new(a.x + u.x * delz, a.y + u.y * delz, a.z + u.z * delz) };
                segs.push(Segment {
                    index: base + i + 1,
                    tag,
                    a,
                    b,
                    radius,
                });
                delz *= t.rdel;
                radius *= t.rrad;
                a = b;
            }
        }
        _ => {
            let (mut radius, rrad) = taper.map_or((card.real_or(8, 0.0)?, 1.0), |t| (t.rad1, t.rrad));
            for i in 0..n {
                // Interpolate endpoints from the wire ends rather than accumulating a step, so
                // segment boundaries stay exactly coincident and connectivity matching is robust.
                #[allow(clippy::cast_precision_loss)]
                let t0 = i as f64 / n as f64;
                #[allow(clippy::cast_precision_loss)]
                let t1 = (i + 1) as f64 / n as f64;
                segs.push(Segment {
                    index: base + i + 1,
                    tag,
                    a: Point3::new(p1.x + d.x * t0, p1.y + d.y * t0, p1.z + d.z * t0),
                    b: Point3::new(p1.x + d.x * t1, p1.y + d.y * t1, p1.z + d.z * t1),
                    radius,
                });
                radius *= rrad;
            }
        }
    }
    Ok(())
}

/// The number of segments a `GA`/`GH` card asks for.
fn segment_count(card: &Card) -> Result<usize, GeomError> {
    let tag = card.int(0)?;
    let count = i64::from(card.int(1)?);
    usize::try_from(count).ok().filter(|&n| n > 0).ok_or(GeomError::BadSegmentCount {
        line: card.line,
        tag,
        count,
    })
}

/// `GA tag ns rada ang1 ang2 rad` — an arc of radius `rada` in the XZ plane, centred on the
/// origin, from `ang1` to `ang2` degrees measured from +X toward +Z (`ARC`).
fn add_arc(card: &Card, segs: &mut Vec<Segment>) -> Result<(), GeomError> {
    /// `DATA TA/.01745329252D+0/` in `ARC`.
    const TA: f64 = 0.017_453_292_52;
    let tag = card.int(0)?;
    let n = segment_count(card)?;
    let rada = card.real_or(2, 0.0)?;
    let (ang1, ang2) = (card.real_or(3, 0.0)?, card.real_or(4, 0.0)?);
    let radius = card.real_or(5, 0.0)?;
    if (ang2 - ang1).abs() >= 360.000_01 {
        return Err(GeomError::Data { line: card.line, reason: "arc angle exceeds 360 degrees" });
    }
    let mut ang = ang1 * TA;
    #[allow(clippy::cast_precision_loss)]
    let dang = (ang2 - ang1) * TA / n as f64;
    let base = segs.len();
    let mut a = Point3::new(rada * ang.cos(), 0.0, rada * ang.sin());
    for i in 0..n {
        ang += dang;
        let b = Point3::new(rada * ang.cos(), 0.0, rada * ang.sin());
        segs.push(Segment {
            index: base + i + 1,
            tag,
            a,
            b,
            radius,
        });
        a = b;
    }
    Ok(())
}

/// `GH tag ns s hl a1 b1 a2 b2 rad` — a helix along +Z with turn spacing `s` and total length
/// `hl`, radius `a1`/`b1` (x/y) at the start tapering to `a2`/`b2` at the end; a negative `hl`
/// makes it left-handed by swapping x and y (`HELIX`).
fn add_helix(card: &Card, segs: &mut Vec<Segment>) -> Result<(), GeomError> {
    /// `DATA PI/3.1415926D+0/` in `HELIX`: deliberately NEC's truncated value.
    #[allow(clippy::approx_constant)]
    const PI: f64 = 3.141_592_6;
    let tag = card.int(0)?;
    let n = segment_count(card)?;
    let s = card.real_or(2, 0.0)?;
    let hl = card.real_or(3, 0.0)?;
    let a1 = card.real_or(4, 0.0)?;
    let mut b1 = card.real_or(5, 0.0)?;
    let a2 = card.real_or(6, 0.0)?;
    let mut b2 = card.real_or(7, 0.0)?;
    let radius = card.real_or(8, 0.0)?;
    if s == 0.0 {
        return Err(GeomError::Data { line: card.line, reason: "GH turn spacing must not be zero" });
    }
    #[allow(clippy::cast_precision_loss)]
    let zinc = (hl / n as f64).abs();
    let at = |z: f64, b1: f64, b2: f64| -> Point3 {
        #[allow(clippy::float_cmp)]
        let (rx, ry) = if a2 == a1 {
            (a1, b1)
        } else {
            (a1 + (a2 - a1) * z / hl.abs(), b1 + (b2 - b1) * z / hl.abs())
        };
        let (x, y) = (rx * (2.0 * PI * z / s).cos(), ry * (2.0 * PI * z / s).sin());
        if hl > 0.0 { Point3::new(x, y, z) } else { Point3::new(y, x, z) }
    };
    #[allow(clippy::float_cmp)]
    if a2 == a1 {
        if b1 == 0.0 {
            b1 = a1;
        }
    } else if b2 == 0.0 {
        b2 = a2;
    }
    let base = segs.len();
    let mut z = 0.0;
    for i in 0..n {
        if i > 0 {
            z += zinc;
        }
        segs.push(Segment {
            index: base + i + 1,
            tag,
            a: at(z, b1, b2),
            b: at(z + zinc, b1, b2),
            radius,
        });
    }
    Ok(())
}

/// `GS i1 i2 scale` — multiply the structure's dimensions by `scale`.
///
/// Radii are scaled along with coordinates. The NEC-2 manual says so directly — "All structure
/// dimensions, including wire radius, are [scaled]" — and the reference engine agrees: the
/// geometry gate compares the engine's printed radius against ours on the 127 verified corpus
/// models that carry a `GS` card, and they all match. Patch areas scale by the square.
fn scale(card: &Card, segs: &mut [Segment], patches: &mut [Patch]) -> Result<(), GeomError> {
    let factor = card.real_or(2, 1.0)?;
    if factor == 0.0 || !factor.is_finite() {
        return Ok(());
    }
    for s in segs.iter_mut() {
        s.a = s.a.scaled(factor);
        s.b = s.b.scaled(factor);
        s.radius *= factor;
    }
    for p in patches.iter_mut() {
        p.center = p.center.scaled(factor);
        p.area *= factor * factor;
        for q in &mut p.outline {
            *q = q.scaled(factor);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nec_cards::parse_deck;

    fn build(src: &str) -> Geometry {
        let mut deck = parse_deck(src).expect("parses");
        deck.resolve().expect("resolves");
        Geometry::build(&deck).expect("builds")
    }

    #[test]
    fn example1_segmentation_matches_the_engine() {
        // models/Example1.nec. The engine's SEGMENTATION DATA reports nine segments of
        // length 0.05373 with centres stepping by that amount from -0.21493.
        let g = build("GW 1 9 0 -.2418 0 0 .2418 0 .0001\nGE 0\nEN\n");
        assert_eq!(g.len(), 9);
        assert_eq!(g.ground, GroundFlag::FreeSpace);

        for (i, s) in g.segments.iter().enumerate() {
            assert_eq!(s.index, i + 1);
            assert_eq!(s.tag, 1);
            assert!((s.length() - 0.053_73).abs() < 5e-6, "seg {} len", i + 1);
            assert!((s.radius - 1e-4).abs() < 1e-12);
        }

        let c1 = g.segment(1).expect("seg 1").center();
        assert!((c1.y + 0.214_93).abs() < 5e-6, "seg 1 centre y = {}", c1.y);
        let c9 = g.segment(9).expect("seg 9").center();
        assert!((c9.y - 0.214_93).abs() < 5e-6, "seg 9 centre y = {}", c9.y);

        // The midpoint segment straddles the feed point, which is why Example1 excites 5.
        let c5 = g.segment(5).expect("seg 5").center();
        assert!(c5.y.abs() < 1e-12, "seg 5 should be centred on the origin");
    }

    #[test]
    fn segment_boundaries_are_exactly_coincident() {
        // Interpolating from the wire ends (rather than accumulating a step) is what makes
        // endpoint matching exact, which connectivity depends on.
        let g = build("GW 1 7 0 0 0 1 2 3 1e-3\nGE 0\nEN\n");
        for w in g.segments.windows(2) {
            assert_eq!(w[0].b, w[1].a, "boundary between {} and {}", w[0].index, w[1].index);
        }
    }

    #[test]
    fn ground_flags() {
        assert_eq!(
            build("GW 1 1 0 0 0 0 0 1 1e-3\nGE 1\nEN\n").ground,
            GroundFlag::GroundedEnds
        );
        assert_eq!(
            build("GW 1 1 0 0 0 0 0 1 1e-3\nGE -1\nEN\n").ground,
            GroundFlag::Ground
        );
        assert!(GroundFlag::GroundedEnds.has_ground());
        assert!(!GroundFlag::FreeSpace.has_ground());
    }

    #[test]
    fn gs_scales_coordinates_and_radii_together() {
        let g = build("GW 1 2 0 0 0 0 0 1 1e-3\nGS 0 0 0.3048\nGE 0\nEN\n");
        let s = g.segment(1).expect("seg 1");
        assert!((s.length() - 0.5 * 0.3048).abs() < 1e-12);
        assert!(
            (s.radius - 1e-3 * 0.3048).abs() < 1e-15,
            "radius must scale too, got {}",
            s.radius
        );
    }

    #[test]
    fn segments_are_numbered_continuously_across_wires() {
        let g = build(
            "GW 1 3 0 0 0 1 0 0 1e-3\n\
             GW 7 2 1 0 0 1 1 0 1e-3\n\
             GE 0\nEN\n",
        );
        assert_eq!(g.len(), 5);
        assert_eq!(
            g.segments.iter().map(|s| s.index).collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5]
        );
        assert_eq!(g.by_tag(7).count(), 2);
        // Tag-relative addressing, as EX and LD cards use.
        assert_eq!(g.resolve_segment(7, 1).expect("tag 7 seg 1").index, 4);
        assert_eq!(g.resolve_segment(0, 4).expect("absolute 4").index, 4);
    }

    #[test]
    fn rejects_bad_wires() {
        let mut d = parse_deck("GW 1 0 0 0 0 1 0 0 1e-3\nGE 0\nEN\n").expect("parses");
        d.resolve().expect("resolves");
        assert!(matches!(
            Geometry::build(&d),
            Err(GeomError::BadSegmentCount { .. })
        ));

        let mut d = parse_deck("GW 1 3 1 2 3 1 2 3 1e-3\nGE 0\nEN\n").expect("parses");
        d.resolve().expect("resolves");
        assert!(matches!(
            Geometry::build(&d),
            Err(GeomError::ZeroLengthWire { .. })
        ));
    }

    #[test]
    fn arcs_helices_and_tapers_are_generated() {
        // A quarter circle of radius 2 from +X to +Z.
        let g = build("GA 1 4 2 0 90 1e-3
GE 0
EN
");
        assert_eq!(g.len(), 4);
        assert!((g.segments[0].a.x - 2.0).abs() < 1e-9);
        assert!((g.segments[3].b.z - 2.0).abs() < 1e-9 && g.segments[3].b.x.abs() < 1e-9);

        // Two turns of a helix 1 m long: it ends where it started, one metre up.
        let g = build("GH 1 16 .5 1 .1 .1 .1 .1 1e-3
GE 0
EN
");
        assert_eq!(g.len(), 16);
        let (a, b) = (g.segments[0].a, g.segments[15].b);
        assert!((a.x - 0.1).abs() < 1e-6 && (b.x - 0.1).abs() < 1e-6, "{a:?} {b:?}");
        assert!((b.z - 1.0).abs() < 1e-12);

        // A tapered wire: lengths double, radius goes 4 mm -> 1 mm geometrically.
        let g = build("GW 1 3 0 0 0 0 0 7 0
GC 0 0 2 .004 .001
GE 0
EN
");
        let lens: Vec<f64> = g.segments.iter().map(Segment::length).collect();
        assert!((lens[0] - 1.0).abs() < 1e-12 && (lens[1] - 2.0).abs() < 1e-12 && (lens[2] - 4.0).abs() < 1e-12);
        assert!((g.segments[0].radius - 0.004).abs() < 1e-15);
        assert!((g.segments[2].radius - 0.001).abs() < 1e-9);
    }

    #[test]
    fn a_zero_radius_wire_needs_its_gc_card() {
        let mut d = parse_deck("GW 1 3 0 0 0 0 0 7 0
GE 0
EN
").expect("parses");
        d.resolve().expect("resolves");
        assert!(matches!(Geometry::build(&d), Err(GeomError::Data { .. })));
    }

    #[test]
    fn nec_example4_patches_and_their_wire_connection() {
        // Example 4 of the NEC-2 manual: a T antenna on a box over ground. Two patches, the GX
        // that mirrors them, the box top, and a wire standing on the top's centre.
        let g = build(
            "SP 0 0 .1 .05 .05 0 0 .01
             SP 0 0 .05 .1 .05 0 90 .01
             GX 0 110
             SP 0 0 0 0 .1 90 0 .04
             GW 1 4 0 0 .1 0 0 .3 .001
             GE 1
EN
",
        );
        // Eight side patches from the reflections, then the top split into four for the wire.
        assert_eq!(g.patches.len(), 12);
        assert_eq!(g.connectivity[0].end1, Junction::Patch(9));
        let top = &g.patches[8..12];
        assert!(top.iter().all(|p| (p.area - 0.01).abs() < 1e-12 && (p.center.z - 0.1).abs() < 1e-12));
        assert!((top[0].center.x - 0.05).abs() < 1e-12 && (top[0].center.y - 0.05).abs() < 1e-12);
        // The reflected side patches face outward: X images face -X.
        let images_x: Vec<_> = g.patches[4..8].iter().map(|p| p.normal().x).collect();
        // (NEC's degrees-to-radians constant is not exact, so 90° leaves ~5e-12.)
        assert!(images_x.iter().all(|&nx| nx <= 1e-9), "{images_x:?}");
        assert!((g.patches[4].normal().x + 1.0).abs() < 1e-12);
    }

    #[test]
    fn sc_cards_continue_a_strip_of_patches() {
        let g = build(
            "SP 0 1 0 0 0 1 0 0
SC 0 0 1 1 0
SC 0 1 1 2 0
SC 0 1 1 3 0
GE 0
EN
",
        );
        assert_eq!(g.patches.len(), 3);
        let ys: Vec<f64> = g.patches.iter().map(|p| p.center.y).collect();
        assert!((ys[0] - 0.5).abs() < 1e-12 && (ys[1] - 1.5).abs() < 1e-12 && (ys[2] - 2.5).abs() < 1e-12, "{ys:?}");
    }

    #[test]
    fn gs_scales_patch_areas_by_the_square() {
        let g = build("SM 2 2 0 0 0 2 0 0
SC 0 0 2 2 0
GS 0 0 .5
GE 0
EN
");
        assert_eq!(g.patches.len(), 4);
        assert!(g.patches.iter().all(|p| (p.area - 0.25).abs() < 1e-12));
        assert!((g.patches[3].center.x - 0.75).abs() < 1e-12);
    }

    #[test]
    fn a_nec4_geometry_card_is_refused_not_ignored() {
        // Nec4\Catenary.nec: CW used to vanish silently, leaving an empty structure.
        let mut d =
            parse_deck("CW 1 39 -19.64 0 20 19.64 0 20 0.001 2 19.64 1\nGE 0\nEN\n").expect("parses");
        d.resolve().expect("resolves");
        let err = Geometry::build(&d).expect_err("must refuse");
        assert!(matches!(err, GeomError::NotNec2Geometry { .. }), "{err}");
        assert!(err.to_string().contains("NEC-4"), "{err}");
    }

    #[test]
    fn gm_copies_extend_segment_numbering() {
        // A wire copied twice, 1 m apart, with tag increment 1: the EX/LD addressing that
        // depends on segment numbers must see the copies as segments 3..6.
        let g = build("GW 1 2 0 0 0 1 0 0 1e-3\nGM 1 2 0 0 0 0 1 0\nGE 0\nEN\n");
        assert_eq!(g.len(), 6);
        assert_eq!(g.by_tag(3).count(), 2);
        assert_eq!(g.resolve_segment(2, 1).expect("tag 2").index, 3);
        assert!((g.segment(5).expect("seg 5").a.y - 2.0).abs() < 1e-12);
    }

    #[test]
    fn each_segment_knows_the_card_that_made_it() {
        // Lines: 1 CE, 2 GW (3 segs), 3 GW (2 segs), 4 GM copying both once (5 segs), 5 GE.
        let g = build("CE
GW 1 3 0 0 0 0 0 1 .001
GW 2 2 1 0 0 1 0 1 .001
GM 2 1 0 0 0 5 0 0
GE 0
EN
");
        assert_eq!(g.origins, [2, 2, 2, 3, 3, 4, 4, 4, 4, 4]);
    }
}
