// Plain-language help for every term and control, in one place so the wording stays
// consistent. `short` is the tooltip; `long` the "?" popover; `source` where a number or a
// verdict comes from.

export interface Help {
  /** One sentence: what it is. */
  short: string;
  /** More: why it matters, what to do. */
  long?: string;
  /** Where a threshold or rule comes from. */
  source?: string;
}

export const HELP = {
  // ---- The workflow strip.
  stepAntenna: {
    short: "Start here: pick a ready-made antenna, or open a model file.",
    long: "New… offers templates (dipole, vertical, Yagi, loops…) sized for the band you choose, and 4nec2's example models. Open… reads a .nec deck or a NEC Studio project.",
  },
  stepBand: {
    short: "The band you want the antenna to work on.",
    long: "Choosing a band solves the antenna at the middle of the band and sweeps it from edge to edge, so the charts show how it behaves across the whole band. Type an exact frequency in the MHz box to look at one spot.",
    source: "US band edges from the FCC rules, 47 CFR §97.301.",
  },
  stepAdjust: {
    short: "Change the antenna's dimensions with the knobs on the left.",
    long: "Templates come with knobs for their lengths, heights and wire size. Each change re-solves the antenna straight away.",
  },
  stepResults: {
    short: "See how well it works: SWR, gain, and where it radiates.",
  },
  newModel: { short: "Start a new antenna from a template, or open one of 4nec2's examples." },
  openModel: { short: "Open a .nec model or a NEC Studio project (Ctrl+O)." },
  saveModel: { short: "Save (Ctrl+S). A new model is saved as a project, which also keeps your settings." },
  saveAs: {
    short: "Save under a new name (Ctrl+Shift+S).",
    long: "A .toml project keeps the model and its settings (band, sweep, optimizer). A .nec file keeps only the model, in the format 4nec2 and other NEC programs read.",
  },
  frequency: {
    short: "The frequency the antenna is solved at, MHz.",
    long: "The results, the 3-D pattern and the currents are for this frequency. Click anywhere on a sweep chart to jump to that frequency.",
  },
  sweep: {
    short: "Solve the antenna at many frequencies across the band and chart SWR, impedance and gain.",
    long: "A sweep shows the antenna's bandwidth: where its SWR is low enough to use. Small models re-sweep by themselves after every change.",
  },
  settings: { short: "Settings for experienced users: reference impedance, units, segmentation." },
  status: { short: "What the program is doing, and how long the last solve took." },

  groundKind: {
    short: "What is under the antenna. The ground reflects part of the signal and changes the pattern and the impedance.",
    long: "Average ground suits most back yards. Poor ground is sandy or dry soil; good ground is rich, moist farmland; salt water is the best ground there is. Free space ignores the ground altogether, which is how antennas are often compared in books.",
    source: "Typical soil constants as the ARRL and 4nec2 tabulate them.",
  },
  height: {
    short: "How high the antenna is above the ground.",
    long: "For a dipole or Yagi, the height of the wire or boom; for an inverted V, the apex; for a loop or J-pole, its bottom; for a ground plane, the base. Higher usually means a lower radiation angle, better for long-distance contacts. A quarter to a half wavelength up is a common goal.",
  },
  wire: {
    short: "The wire or tubing the antenna is made of.",
    long: "Thicker wire has a little more bandwidth and slightly less loss. #14 AWG copper is the usual choice for wire antennas; aluminium tubing for Yagi elements.",
  },

  // ---- Results.
  swr: {
    short: "Standing-wave ratio: how well the antenna matches your feed line. 1.0 is perfect.",
    long: "SWR compares the antenna's impedance with Z₀ (50 Ω). At 1:1 all the power goes into the antenna; as SWR rises, more is reflected back toward the radio. Modern radios protect themselves by turning their power down when the SWR is high.",
  },
  swrVerdict: {
    short: "What this SWR means for a typical station.",
    long: "Up to 1.5:1 is an excellent match. Up to 2:1 most solid-state transceivers still deliver full power. Up to 3:1 a radio's built-in antenna tuner can usually match it. Above that, add a matching network (Match tab) or an external tuner, or adjust the antenna.",
    source: "Solid-state transceivers commonly reduce power above about 2:1 (the ARRL Antenna Book discusses this with SWR and transmission lines; see also your radio's manual); built-in tuners are commonly specified to 3:1, e.g. the Icom IC-7300's 16.7–150 Ω.",
  },
  impedance: {
    short: "The antenna's feed-point impedance, R + jX ohms.",
    long: "R (resistance) is the part that takes power, almost all of it radiated. X (reactance) stores energy and only gets in the way: negative X means the antenna is electrically too short, positive X too long. A resonant antenna has X = 0.",
  },
  resonance: {
    short: "The frequency where the reactance X is zero: the antenna is 'in tune'.",
    long: "Read from the sweep. Trimming a simple wire antenna moves it: longer lowers the resonance, shorter raises it.",
  },
  bandwidth: {
    short: "The range of frequencies where the SWR is 2:1 or less.",
    long: "Read from the sweep. Inside this range most radios work at full power without a tuner. If the range runs to the edge of the sweep, the antenna may be usable further; widen the sweep to see.",
  },
  trimHint: {
    short: "Which way to change the length to move the resonance to your frequency.",
    long: "Resonance moves about inversely with length: an antenna resonant 2 % too high wants to be about 2 % longer. This rule of thumb suits dipoles, verticals and loops; antennas with traps, coils or several bands need cut-and-try. Change the length knob and watch the SWR dip move.",
  },
  gain: {
    short: "How much stronger the signal is in the best direction than from an ideal antenna radiating equally everywhere (dBi).",
    long: "dBi compares with an isotropic antenna; dBd with a half-wave dipole in free space, which is 2.15 dBi. Over ground, reflections add up to about 6 dB to a horizontal antenna's gain, so a dipole over ground often shows 6–8 dBi.",
    source: "A half-wave dipole's free-space gain, 2.15 dBi.",
  },
  takeoff: {
    short: "The elevation angle of the strongest radiation, above the horizon.",
    long: "Low angles (under about 20°) favour long-distance (DX) contacts; high angles favour nearby stations via the ionosphere (NVIS). Raising a horizontal antenna lowers its takeoff angle.",
  },
  direction: { short: "The azimuth of the strongest radiation, degrees counter-clockwise from the +X axis (0° = +X, 90° = +Y)." },
  frontToBack: {
    short: "How much weaker the signal is directly behind than in front, dB.",
    long: "Matters for beams: a high front-to-back rejects signals and interference from behind. Over ground, 'behind' is the same elevation, opposite direction; in free space the exact opposite direction.",
  },
  frontToRear: {
    short: "Front against the strongest signal anywhere in the rear half, dB — a stricter version of front-to-back.",
  },
  efficiency: {
    short: "The share of the power reaching the antenna that is radiated rather than lost in the wire, loads and networks.",
    long: "Losses in the ground are not counted here (NEC reports them as part of the pattern instead). A copper wire antenna is close to 100 %; loading coils and small antennas lose more.",
  },
  averageGain: {
    short: "A check on the model: 0 dB for a lossless antenna in free space, about +3 dB over perfect ground.",
    long: "Far from those values, the model may be inaccurate. Run the average-gain test (Checks & data) to grade it.",
  },
  polarization: {
    short: "The orientation of the radiated electric field at the peak: linear (horizontal, vertical or tilted) or circular.",
    long: "Antennas talking to each other should share a polarization: horizontal to horizontal, vertical to vertical. Satellites often use circular (RHCP or LHCP).",
  },

  // ---- Matching.
  matching: {
    short: "Design a network of coils and capacitors, or a stub of feed line, so the antenna looks like 50 Ω to your radio.",
    long: "Pick a type and press Design: every arrangement that matches is listed, with its parts, the SWR the radio then sees and the power lost in it. Apply writes it into the model; the sweep then shows the matched bandwidth. Remove takes it out again.",
  },
  matchType: {
    short: "The kind of match.",
    long: "L network: one coil and one capacitor, the simplest. Pi and T networks: three parts, with a choice of how narrow the match is (its Q). Stub: a length of feed line connected across (or in series with) the line at the right distance from the antenna. Series section: a length of a different line set into the feed line.",
  },
  qCoil: {
    short: "How good the coil is (its Q). 250 is typical of a home-wound air coil; lower means more loss.",
    source: "4nec2's default.",
  },
  qCap: { short: "How good the capacitor is (its Q). 0 means lossless, which is close to true for air and ceramic capacitors." },
  qNet: { short: "How narrow the Pi or T match is (its loaded Q). 0 uses the lowest possible, which gives the widest bandwidth." },
  feedLine: { short: "The feed line between the radio and the antenna: its impedance and velocity factor set where the stub goes." },
  stubLine: { short: "The line the stub is cut from." },
  stubShorted: { short: "A shorted stub has its far end connected together; unticked, the far end is left open." },
  stubSeries: { short: "A series stub is cut into one conductor of the line; unticked, it is connected across the line (parallel), the usual way." },
  sectionLine: {
    short: "The line set into the feed line. Its impedance must be far from 50 Ω to match a high SWR.",
    long: "A series section can match only an SWR below (Z₁/Z₀)²; 450 Ω ladder line or 75 Ω coax are common choices.",
  },
  matchDesign: { short: "Work out every network of this type that matches the antenna at this frequency." },
  matchApply: { short: "Write this network into the model and re-solve; the source then sees the matched impedance." },
  matchRemove: { short: "Take the matching network out of the model, putting the source back on the antenna." },

  // ---- Tuning (optimizer and parameter sweep).
  tuneVariables: {
    short: "The dimensions the optimizer may change, each between a minimum and a maximum.",
    long: "Tick the ones to tune. Variables computed from a formula (ƒ) can be tuned too, but tuning replaces the formula with a number.",
  },
  tuneGoals: {
    short: "What the optimizer aims for. Each goal has a target and a weight (how much it matters).",
    long: "SWR, gain, front-to-back and so on. Over several frequencies each goal counts its worst frequency, so 'SWR ≤ 1.5' means across the whole band.",
  },
  goalWeight: { short: "How much this goal matters compared with the others. 1 is normal; 0.5 half as much." },
  tuneFrequencies: { short: "The frequencies the goals are judged at, MHz, separated by commas. Sweep band uses the sweep's start, middle and end." },
  forwardAz: { short: "The direction the beam should point, for gain and front-to-back goals: azimuth, degrees." },
  forwardEl: { short: "The elevation to judge gain at; blank finds the strongest elevation, which is what gain means over ground." },
  tuneMethod: {
    short: "How the optimizer searches.",
    long: "Nelder–Mead improves the design you have, step by step: fast, and best when you are close. Differential evolution searches the whole range with a population of designs: slower, but finds good designs further away.",
  },
  tuneBudget: { short: "The most designs the optimizer may try. Each solves the antenna at every goal frequency." },
  tuneRun: { short: "Start the search. The best design so far is shown as it goes; Stop keeps it." },
  tuneApply: { short: "Write the best design's values into the model." },
  paramSweep: {
    short: "Solve the antenna for a range of values of one dimension, and chart SWR, impedance and gain against it.",
    long: "A way to see how sensitive the antenna is to one dimension, or to pick a value by eye: click a point on the chart to write that value into the model.",
  },
  paramPattern: { short: "Also compute gain and front-to-back for each value (slower)." },

  // ---- Checks and data.
  agt: {
    short: "Average-gain test: a check of the model's accuracy, with all losses removed.",
    long: "With no losses all the input power must be radiated, so the average gain over the whole sphere should be exactly 1 (0 dB). 0.95–1.05 is accurate, 0.90–1.10 usable; further off, the model needs more segments or better junctions. The dB figure is a correction to apply to the model's gains.",
    source: "4nec2's average-gain test and its grades.",
  },
  convergence: {
    short: "Convergence test: solve the antenna again with more and more segments and see whether the results settle.",
    long: "A trustworthy model gives nearly the same impedance and gain with more segments. If the numbers keep moving, the model needs more segments everywhere.",
  },
  convSteps: { short: "How many times to add segments." },
  convGrowth: { short: "How many more segments each step adds, as a percentage (20–50 % is usual)." },
  exportCsv: { short: "Save the data as a spreadsheet file (CSV) to open in Excel, LibreOffice or a plotting program." },
  exportReport: { short: "Save a text report of the solution in the style NEC itself prints." },
  nearField: {
    short: "The electric (E) or magnetic (H) field strength on a slice through the space around the antenna, drawn in the 3-D view.",
    long: "Useful to see where the field is strongest — for example near the antenna, or why something nearby couples to it. Values are for 1 V at the source; scale them for your power.",
  },
  nearPlane: { short: "Which plane the slice lies in." },
  nearAt: { short: "Where the slice is, along the axis it cuts." },
  nearKind: { short: "E: the electric field, volts per metre. H: the magnetic field, amperes per metre." },
  nearResolution: { short: "Points along each side of the slice. More is smoother and slower." },
  currentsTable: { short: "The current on every segment: its size and phase, and the charge. The 3-D view colours the wires by current." },

  // ---- 3-D view and charts.
  showPattern: { short: "Draw the radiation pattern around the antenna: further out means stronger in that direction." },
  showCurrents: { short: "Colour the wires by the current flowing in them: red is strongest." },
  fitView: { short: "Zoom and centre the view on the antenna. Drag to rotate, right-drag to pan, scroll to zoom." },
  sweepGain: { short: "Also chart gain, front-to-back and front-to-rear across the sweep (slower). Needed for pattern play-back." },
  sweepAz: { short: "The direction, azimuth in degrees, that the sweep's gain and front-to-back are measured toward." },
  logFrequency: { short: "Spread the frequency axis logarithmically, for sweeps over a wide range." },
  logSwr: { short: "A logarithmic SWR axis, so high SWRs don't squash the low ones." },
  smooth: { short: "Draw smooth curves through the solved points. Display only: the points are what was computed." },
  smith: {
    short: "The Smith chart: the antenna's impedance across the sweep. The centre is a perfect match.",
    long: "The closer the curve passes to the centre, the lower the SWR; the circle marks 2:1. Above the middle line the antenna is inductive (too long), below it capacitive (too short).",
  },
  patternComponent: {
    short: "Which part of the radiated field to show.",
    long: "Total is what a station receives. Vertical and horizontal show each polarization; RHCP and LHCP the circular ones, which matter for satellites.",
  },
  pinPattern: { short: "Keep this pattern on the plots to compare later ones with (up to five)." },

  // ---- Editors.
  shape: {
    short: "The wires the antenna is made of, one card per wire or transformation.",
    long: "Select a wire here or in the 3-D view to edit its ends, thickness and segments. Fields can hold numbers or formulas over the model's variables.",
  },
  feed: {
    short: "Sources, loads, transmission lines, frequency and ground.",
    long: "+ Source puts a 1 V source on the segment picked in the 3-D view (or the middle of the first wire). + Load adds a coil, capacitor or resistor; + Line a transmission line between two segments.",
  },
  addSource: { short: "Add a source (where the feed line connects) on the picked segment, or the middle of the first wire." },
  addLoad: { short: "Add a load — a resistor, coil or capacitor — on the picked segment." },
  addLine: { short: "Add a transmission line between two segments." },
  deleteCard: { short: "Remove this card from the model." },

  // ---- Settings.
  z0: {
    short: "The impedance of your feed line and radio, Ω. Almost always 50.",
    long: "SWR is measured against this. Coax and nearly every amateur transceiver are 50 Ω; use 75 for 75 Ω coax, or 450 / 300 for ladder line and twin lead.",
  },
  units: {
    short: "Units for the lengths shown and typed in the editors and knobs.",
    long: "This only changes what you see. The model file keeps its own units.",
  },
  sweepSteps: {
    short: "How many frequencies a sweep solves.",
    long: "More steps draw smoother curves and take longer. 41 is plenty for one band.",
  },
  autoseg: {
    short: "Automatically cut wires into enough pieces for the frequency being solved.",
    long: "NEC splits every wire into segments, and needs enough of them per wavelength to be accurate. With this on, each wire gets at least the number of segments per half wavelength set here, at every frequency solved — so moving to a higher band never makes the model too coarse. It never takes segments away. A segment count written with a leading + in the deck is left alone.",
    source: "4nec2's auto-segmentation; 20 per half wave is its default.",
  },
  segsPerHalfWave: {
    short: "Segments per half wavelength for auto-segmentation (10 to 500).",
    long: "About 20 is accurate for simple antennas. Use more for sharp bends or close spacing; it makes solving slower.",
    source: "4nec2's help: 10 to 20 is sufficient for simple structures.",
  },
  stepRadius: {
    short: "Correct elements made of telescoping tubes of different diameters.",
    long: "NEC models a sudden change of wire thickness poorly, so a Yagi built of telescoping tubing comes out at the wrong frequency. This replaces each such element with an equivalent one of a single thickness before solving (David Leeson's method, as 4nec2 uses). Leave it off for wire antennas.",
    source: "W2DU, Physical Design of Yagi Antennas; 4nec2's Convert.exe.",
  },

  // ---- Panels and tabs.
  tabResults: { short: "How the antenna performs at the chosen frequency and across the band." },
  tabShape: { short: "The wires that make up the antenna: their ends, thickness and segments." },
  tabFeed: { short: "Where the antenna is fed, loads such as traps or coils, and the ground under it." },
  tabMatch: { short: "Design a matching network or stub so the antenna presents 50 Ω to your radio." },
  tabTune: { short: "Let the program adjust the dimensions for you, toward goals such as low SWR or high gain." },
  tabData: { short: "Model checks, current tables, near fields and exports." },
  knobs: {
    short: "The antenna's adjustable dimensions.",
    long: "Each knob is a variable in the model (an SY line). Type a value or drag the slider; the antenna re-solves as you go.",
  },
  checks: {
    short: "Problems with the model that could make the results wrong.",
    long: "NEC gives a plausible-looking answer even for a model outside its limits. These checks catch the common mistakes. Click one to highlight where it is.",
  },
  deck: {
    short: "The model itself, as NEC cards (text).",
    long: "Everything else in the window edits this text. You can also type here directly; the antenna re-solves as you type. The NEC-2 user's guide describes every card.",
  },
} satisfies Record<string, Help>;

export type HelpKey = keyof typeof HELP;

/** The tooltip text for a term: its short line. */
export const hint = (k: HelpKey): string => HELP[k].short;

/** What each card is, for the card lists. */
export const CARD_HINTS: Record<string, string> = {
  GW: "A straight wire between two points.",
  GA: "A wire bent into an arc of a circle.",
  GH: "A wire wound into a helix.",
  GM: "Moves, rotates or copies wires already defined.",
  GX: "Mirrors the wires in the X, Y or Z plane, as copies.",
  GR: "Copies the wires around the Z axis, like spokes.",
  GS: "Scales the whole structure: for example 0.3048 when the wires are written in feet.",
  GC: "Makes the wire before it tapered, thicker at one end.",
  SP: "A flat surface patch.",
  SM: "A grid of surface patches.",
  SC: "The corners of a patch.",
  GE: "Ends the geometry, and says whether wires touching the ground connect to it.",
  GF: "Reads a structure saved earlier (a numerical Green's function file).",
  EX: "A source: where the feed line connects.",
  LD: "A load on some segments: a resistor, coil, capacitor, trap, or the wire's own resistance.",
  TL: "A transmission line between two segments.",
  NT: "A two-port network between two segments (a matching network uses one).",
  FR: "The frequency or frequencies to solve at.",
  GN: "The ground under the antenna.",
  GD: "A second ground region, beyond a cliff or a shoreline.",
  EK: "Uses the extended thin-wire kernel, for thick wires with short segments.",
  KH: "Speeds up solving for parts far apart.",
};

/** Field hints per card, matching the field labels in the editors. */
export const FIELD_HINTS: Record<string, string[]> = {
  GW: [
    "A number naming this wire; sources and loads refer to it.",
    "How many pieces (segments) NEC divides the wire into. About 20 per half wavelength is accurate; auto-segmentation adds more when needed. A leading + fixes the count.",
    "End 1, X coordinate", "End 1, Y coordinate", "End 1, Z coordinate (height)",
    "End 2, X coordinate", "End 2, Y coordinate", "End 2, Z coordinate (height)",
    "The wire's radius — half its diameter. #14 means AWG 14 wire; 0 makes it tapered (a GC card follows).",
  ],
  GA: ["Tag number", "Segments", "Radius of the arc", "Angle where the arc starts, degrees", "Angle where it ends, degrees", "Wire radius"],
  GH: [
    "Tag number", "Segments", "Spacing between turns", "Total length of the helix (negative for a left-hand helix)",
    "Helix radius in X at the start", "Helix radius in Y at the start", "Helix radius in X at the end", "Helix radius in Y at the end", "Wire radius",
  ],
  GM: [
    "Added to each copy's tag numbers", "How many copies (0 moves the wires instead of copying)", "Rotation about X, degrees", "Rotation about Y, degrees",
    "Rotation about Z, degrees", "Move in X", "Move in Y", "Move in Z", "The first tag to move or copy (0 for all)",
  ],
  GX: ["Added to the mirrored copies' tag numbers", "Which mirrors to apply, as three digits: 100 turns x into −x, 010 y into −y, 001 z into −z; 110 does both of the first two"],
  GR: ["Added to each copy's tag numbers", "How many copies in all, around the full circle"],
  GS: ["", "", "Scale factor applied to all the coordinates"],
  GC: ["", "", "Ratio of each segment's length to the one before", "Radius of the first segment", "Radius of the last segment"],
  GE: ["0: no ground connection; 1: wires touching the ground connect to it; −1: ground present but nothing connects to it"],
  EX: [
    "0: voltage source (the usual feed); 6: current source", "Tag of the wire it is on",
    "Which segment of that wire, counted from end 1; 50% means the middle", "Print option (leave 0)", "Voltage, real part (1 V is fine: results are normalised)", "Voltage, imaginary part",
  ],
  LD: [
    "0: series R-L-C; 1: parallel R-L-C; 4: impedance; 5: wire conductivity (copper 5.8e7 S/m); 6: trap; 7: insulation",
    "Tag of the wire", "First segment loaded (0 for the whole wire)", "Last segment loaded", "Resistance in ohms, or conductivity in S/m for type 5", "Inductance in henries, or reactance in ohms for type 4", "Capacitance in farads",
  ],
  TL: [
    "Tag of the wire at end 1", "Segment at end 1", "Tag of the wire at end 2", "Segment at end 2",
    "Characteristic impedance, ohms (negative for a line with a half twist)", "Length, m (0 means the straight distance between the ends)",
  ],
  FR: ["0: steps add MHz; 1: steps multiply", "Number of frequencies", "", "", "First frequency, MHz", "Step, MHz (or the ratio)"],
  GN: [
    "−1: free space; 0: real ground, quick method; 1: perfect ground; 2: real ground, accurate (Sommerfeld)", "Radial wires on the ground (0 for none)", "", "",
    "Ground's dielectric constant (13 is average soil)", "Ground's conductivity, S/m (0.005 is average soil)", "Radius of the radial screen", "Radius of the radial wires",
  ],
};
