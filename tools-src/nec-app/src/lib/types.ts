// Mirrors of the Rust view structs in src-tauri/src/engine.rs.

export interface SegmentView {
  a: [number, number, number];
  b: [number, number, number];
  radius: number;
  tag: number;
  /** Deck line of the card that made the segment. */
  origin: number;
  /** A dummy wire carrying a 4nec2 current source, not part of the antenna. */
  auxiliary: boolean;
}

/** Every kind of model check the linter (crates/nec-geom/src/lint) emits. A test there keeps
 * this list in step with the Rust code. */
export type FindingKind =
  | "below-ground"
  | "coincident-segments"
  | "crossing-wires"
  | "intersecting-wires"
  | "junction-length-radius"
  | "length-jump"
  | "low-over-reflection-ground"
  | "open-end-feed"
  | "overlaid-wires"
  | "patch-too-large"
  | "radius-jump"
  | "radius-too-large"
  | "segment-too-long"
  | "segment-too-short"
  | "sharp-junction"
  | "thin-wire-ratio-low"
  | "thin-wire-ratio-unusable"
  | "too-close-to-ground"
  | "unequal-parallel-segmentation"
  | "unintended-ground-bond"
  | "zero-radius";

export interface Finding {
  severity: "info" | "warning" | "error";
  kind: FindingKind;
  segment: number | null;
  message: string;
}

export interface Variable {
  name: string;
  value: number;
  expression: string;
  tunable: boolean;
  /** The SY line's comment: a label, perhaps ending in a unit ("Overall length, m"). */
  comment: string | null;
}

export interface PatchView {
  outline: [number, number, number][];
  center: [number, number, number];
  normal: [number, number, number];
}

export interface NgfView {
  writes: string | null;
  reads: string | null;
  base_segments: number;
  locked_mhz: number | null;
  runs: number;
}

export interface Analysis {
  segments: SegmentView[];
  patches: PatchView[];
  ngf: NgfView;
  wires: number;
  ground: string;
  has_ground: boolean;
  frequencies_mhz: number[];
  sources: number[];
  findings: Finding[];
  variables: Variable[];
  unsolvable: string | null;
  lowered: boolean;
  /** The frequency auto-segmentation was done for, when it is on. */
  segmented_for: number | null;
  /** Auto-segmentation's changes, one per wire card refined. */
  autoseg_log: { line: number; tag: number; from: number; to: number }[];
  /** The stepped-radius correction's account of each stepped element. */
  steprad_log: string[];
}

/** 4nec2's auto-segmentation (segments per half wave; null for off) and stepped-radius correction. */
export interface ModelOptions {
  autoseg: number | null;
  step_radius: boolean;
  /** The frequency to segment for; the one being solved when omitted. */
  mhz?: number | null;
}

export interface InputView {
  segment: number;
  tag: number;
  z: [number, number];
  swr: number;
  power: number;
}

export interface PowerView {
  input: number;
  radiated: number;
  structure_loss: number;
  network_loss: number;
  efficiency: number;
}

/** One lobe of a cut: a local maximum. */
export interface Lobe {
  angle: number;
  gain: number;
}

/** What 4nec2's pattern info reports for one cut and one component. */
export interface CutStats {
  peak: number;
  peak_angle: number;
  /** Between the −3 dB points; null when the gain does not fall that far on one side. */
  beamwidth: number | null;
  half_power: [number | null, number | null];
  /** Strongest first. */
  lobes: Lobe[];
  /** The strongest lobe after the main one, relative to it, dB. */
  side_lobe: number | null;
  front_to_back: number | null;
  front_to_rear: number | null;
}

/** The gain components a cut carries. */
export type Component = "gain" | "vertical" | "horizontal" | "rhcp" | "lhcp";

export interface Cut {
  angle: number[];
  gain: number[];
  /** Vertical (θ) and horizontal (φ) components, dBi. */
  vertical: number[];
  horizontal: number[];
  /** Right- and left-hand circular components, dBi. */
  rhcp: number[];
  lhcp: number[];
  at: number;
  stats: {
    total: CutStats | null;
    vertical: CutStats | null;
    horizontal: CutStats | null;
    rhcp: CutStats | null;
    lhcp: CutStats | null;
  };
}

/** One frequency of a sweep, kept for play-back. */
export interface Frame {
  azimuth: Cut;
  elevation: Cut;
  currents: number[] | null;
}

export interface PolarizationView {
  vertical: number;
  horizontal: number;
  axial_ratio: number;
  tilt: number;
  sense: "none" | "linear" | "right" | "left";
}

export interface PatternView {
  thetas: number[];
  phis: number[];
  gain: number[][];
  /** Right- and left-hand circular gain on the same grid. */
  rhcp: number[][];
  lhcp: number[][];
  peak_gain: number;
  peak_azimuth: number;
  peak_elevation: number;
  polarization: PolarizationView;
  peak_rhcp: number;
  peak_lhcp: number;
  front_to_back: number;
  front_to_rear: number;
  average_gain: number;
  azimuth_cut: Cut;
  elevation_cut: Cut;
}

export interface SolveView {
  mhz: number;
  inputs: InputView[];
  currents: number[];
  currents_complex: [number, number][];
  /** Charge per unit length, [re, im] C/m. */
  charges: [number, number][];
  patch_currents: number[];
  wrote_ngf: string | null;
  /** With a matching network in the deck, the antenna's own impedance. */
  antenna_z: [number, number] | null;
  power: PowerView;
  pattern: PatternView | null;
  millis: number;
}

// Mirrors of the serde types of the nec-engine crate.

export interface Direction {
  azimuth: number;
  elevation: number | null;
}

export interface EvalOptions {
  z0: number;
  pattern: Direction | null;
  model?: ModelOptions;
  /** What to keep of each point for play-back. */
  capture?: "none" | "patterns" | "patterns_and_currents";
}

export interface Metrics {
  mhz: number;
  z: [number, number];
  swr: number;
  gain: number | null;
  front_to_back: number | null;
  front_to_rear: number | null;
  elevation: number | null;
  efficiency: number;
}

/** One streamed sweep point; `x` is MHz, or the swept variable's value. */
export interface SweepEvent {
  job: number;
  index: number;
  x: number;
  metrics: Metrics | null;
  error: string | null;
  /** Kept for play-back when the sweep asked for it. */
  frame: Frame | null;
}

export type GoalKind = "swr" | "gain" | "front_to_back" | "front_to_rear" | "resistance" | "reactance" | "efficiency";

export interface Goal {
  kind: GoalKind;
  target: number | null;
  weight: number;
}

export interface Parameter {
  name: string;
  min: number;
  max: number;
}

export type Method = "nelder_mead" | "differential_evolution";

export interface OptimizeSpec {
  parameters: Parameter[];
  frequencies_mhz: number[];
  goals: Goal[];
  z0: number;
  direction: Direction;
  method: Method;
  max_evaluations: number;
  seed: number;
  model?: ModelOptions;
}

/** The optimizer set-up a project keeps; the reference impedance is the project's. */
export interface OptimizerSettings {
  parameters: Parameter[];
  frequencies_mhz: number[];
  goals: Goal[];
  direction?: Direction | null;
  method: Method;
  max_evaluations: number;
}

export type Units = "metres" | "feet" | "inches";

export interface ProjectSettings {
  frequency_mhz?: number | null;
  z0: number;
  units: Units;
  sweep?: { start_mhz: number; stop_mhz: number; steps: number; pattern: boolean } | null;
  optimizer?: OptimizerSettings | null;
  model?: ModelOptions;
}

/** A NEC Studio project: a deck plus the settings a deck has no cards for. */
export interface Project {
  format: number;
  name: string;
  notes: string;
  deck: string;
  settings: ProjectSettings;
}

export interface OptimizeProgress {
  evaluations: number;
  iteration: number;
  cost: number;
  values: number[];
  metrics: Metrics[];
}

export interface OptimizeOutcome {
  initial: OptimizeProgress;
  best: OptimizeProgress;
  converged: boolean;
  cancelled: boolean;
}

export type SlicePlane = "xy" | "xz" | "yz";

export interface SliceRequest {
  plane: SlicePlane;
  offset: number;
  u: [number, number];
  v: [number, number];
  n_u: number;
  n_v: number;
  magnetic: boolean;
}

/** Field magnitudes on a slice, row by row (n_u values per row). */
export interface Slice {
  values: number[];
  max: number;
}

export interface ModelFile {
  path: string;
  rel: string;
  description: string;
}
