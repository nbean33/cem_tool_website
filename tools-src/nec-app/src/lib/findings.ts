// What each model check means, in words a beginner can act on: a title, what to do, why it
// matters, and, where the fix is mechanical, a button that makes it. The measured detail stays
// in the check's own message. Typed over every kind the linter emits (FindingKind in types.ts,
// kept in step with crates/nec-geom/src/lint by a test there), so a new check cannot go
// unexplained.

import type { FindingKind } from "./types";

/** A fix the app can make by itself. */
export type FixAction =
  /** Turn on auto-segmentation. */
  | "autoseg"
  /** Use the extended thin-wire kernel (an EK card). */
  | "extended-kernel"
  /** Stop wires near the ground bonding to it (GE -1). */
  | "ground-no-bond"
  /** Use Sommerfeld ground instead of the reflection-coefficient method (GN 2). */
  | "sommerfeld"
  /** Go to the height knob. */
  | "raise";

export interface FindingHelp {
  /** A short plain title. */
  title: string;
  /** What to do about it. */
  fix: string;
  /** Why it matters. */
  why: string;
  action?: FixAction;
  /** The button's label for the action. */
  actionLabel?: string;
}

export const FINDINGS: Record<FindingKind, FindingHelp> = {
  "segment-too-long": {
    title: "Wire cut into too few pieces for this frequency",
    fix: "Give the wire more segments (Shape tab), or let auto-segmentation do it for every frequency.",
    why: "NEC needs segments shorter than about a tenth of a wavelength; longer ones give wrong currents and impedance.",
    action: "autoseg",
    actionLabel: "Turn on auto-segmentation",
  },
  "segment-too-short": {
    title: "Segments extremely short for this frequency",
    fix: "Use fewer segments on this wire (Shape tab), or check the wire's length and units.",
    why: "Below a thousandth of a wavelength NEC's arithmetic loses accuracy.",
  },
  "radius-too-large": {
    title: "Wire very thick for this frequency",
    fix: "Use a smaller radius (the wire knob or the Shape tab). Check the radius is in metres, not a diameter.",
    why: "NEC models wires as thin compared with the wavelength; very thick ones are outside that.",
  },
  "thin-wire-ratio-unusable": {
    title: "Segments too short for the wire's thickness",
    fix: "Use fewer, longer segments on this wire, or a thinner wire. The extended kernel helps down to a ratio of 0.5.",
    why: "When a segment is shorter than about twice its radius, NEC's thin-wire model breaks down.",
    action: "extended-kernel",
    actionLabel: "Use the extended kernel",
  },
  "thin-wire-ratio-low": {
    title: "Segments short for the wire's thickness",
    fix: "Use fewer segments here or a thinner wire, or switch on the extended kernel.",
    why: "Segments under about 8 times their radius lose accuracy with NEC's normal kernel.",
    action: "extended-kernel",
    actionLabel: "Use the extended kernel",
  },
  "zero-radius": {
    title: "Wire with no thickness",
    fix: "Give the wire a radius (Shape tab).",
    why: "NEC cannot compute a wire of zero radius.",
  },
  "unintended-ground-bond": {
    title: "Wire lying on the ground and connected to it",
    fix: "If the wire should not touch the ground, raise it; or tell NEC not to connect wires to the ground.",
    why: "With GE 1 every segment end at ground level is bonded to the ground, which the NEC manual calls a disaster for a horizontal wire.",
    action: "ground-no-bond",
    actionLabel: "Don't connect to ground (GE −1)",
  },
  "radius-jump": {
    title: "Big jump in wire thickness where wires meet",
    fix: "Make the joining wires closer in thickness, or turn on stepped-radius correction (Settings) for telescoping tubing.",
    why: "NEC handles a sudden change of radius at a junction poorly; resonance shifts.",
  },
  "length-jump": {
    title: "Big jump in segment length where wires meet",
    fix: "Adjust the segment counts so that segments either side of the junction are similar in length.",
    why: "Neighbouring segments very different in length make the currents at the junction inaccurate.",
  },
  "junction-length-radius": {
    title: "Short segment beside a thick wire at a junction",
    fix: "Use fewer segments on the short side, or thinner wire.",
    why: "Where wires meet, a segment shorter than a few times the other wire's radius sits partly inside it.",
  },
  "patch-too-large": {
    title: "Surface patch too large",
    fix: "Divide the surface into more, smaller patches.",
    why: "NEC's patch model needs patches no larger than about 0.04 square wavelengths.",
  },
  "open-end-feed": {
    title: "Source or load at an open wire end",
    fix: "Move it one or more segments in from the end (Feed & ground tab).",
    why: "The current at a free wire end is zero, so a source or load there does nothing useful.",
  },
  "coincident-segments": {
    title: "Two segments on top of each other",
    fix: "Delete the duplicate wire (Shape tab).",
    why: "Identical segments make NEC's equations unsolvable or meaningless.",
  },
  "crossing-wires": {
    title: "Wires cross without being joined",
    fix: "Join them at a common end point, or move one so they no longer touch.",
    why: "NEC does not connect wires that cross in the middle; the result is not what was built.",
  },
  "overlaid-wires": {
    title: "Wires lying along each other",
    fix: "Delete or move one of them (Shape tab).",
    why: "Wires on top of each other make NEC's results meaningless.",
  },
  "intersecting-wires": {
    title: "Wires closer than their thickness",
    fix: "Space them further apart, or use thinner wire.",
    why: "Two wires closer than the sum of their radii overlap, which NEC cannot represent.",
  },
  "sharp-junction": {
    title: "Wires meet at a sharp angle",
    fix: "Use shorter segments near the junction, or open up the angle.",
    why: "At a sharp angle part of one segment lies inside the other wire, reducing accuracy.",
  },
  "unequal-parallel-segmentation": {
    title: "Parallel wires cut differently",
    fix: "Give closely spaced parallel wires the same segment length, so their segments line up.",
    why: "Close parallel wires with mismatched segments give inaccurate results (4nec2 warns about this too).",
  },
  "below-ground": {
    title: "Part of the antenna is underground",
    fix: "Raise the antenna, or shorten whatever goes below ground level.",
    why: "NEC-2 cannot model wires buried in the ground.",
    action: "raise",
    actionLabel: "Go to the height",
  },
  "low-over-reflection-ground": {
    title: "Low over the ground for the fast ground model",
    fix: "Use the Sommerfeld ground model, which is accurate close to the ground.",
    why: "The quick reflection-coefficient ground is accurate only for wires at least about 0.2 wavelength up (0.1 for horizontal ones).",
    action: "sommerfeld",
    actionLabel: "Use Sommerfeld ground",
  },
  "too-close-to-ground": {
    title: "Wire almost touching the ground",
    fix: "Raise it, or connect it to the ground deliberately.",
    why: "A wire within a few radii of the ground interacts with its reflection in a way NEC models poorly.",
    action: "raise",
    actionLabel: "Go to the height",
  },
};

export const SEVERITY_WORDS: Record<"error" | "warning" | "info", string> = {
  error: "Problem",
  warning: "Worth a look",
  info: "Note",
};
