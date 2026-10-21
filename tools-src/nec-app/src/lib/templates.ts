// Ready-made antennas, sized for a frequency, a height and a ground.
//
// Every dimension is a plain-number SY variable with a label (its comment, ending in a unit), so
// the knobs and the optimizer can adjust it straight away. The starting sizes come from the
// handbook rules of thumb quoted in each deck's comments (lengths in feet for f in MHz, as hams
// quote them); they put the antenna near resonance, and the knobs finish the job.


/** A soil, as the ARRL and 4nec2 tabulate typical grounds; null means free space. */
export interface Soil {
  name: string;
  epsr: number;
  sigma: number;
}

export const SOILS: Soil[] = [
  { name: "Average", epsr: 13, sigma: 0.005 },
  { name: "Poor (sandy, dry)", epsr: 5, sigma: 0.001 },
  { name: "Good (rich, moist)", epsr: 20, sigma: 0.0303 },
  { name: "City / industrial", epsr: 3, sigma: 0.001 },
  { name: "Fresh water", epsr: 80, sigma: 0.001 },
  { name: "Sea water", epsr: 81, sigma: 5 },
];

export interface Site {
  mhz: number;
  /** Height of the antenna's lowest point (its feed for a dipole), metres. */
  height: number;
  /** null: free space, no ground at all. */
  soil: Soil | null;
}

export interface Template {
  id: string;
  name: string;
  /** What it is, in a line. */
  about: string;
  /** Why a ham would build it. */
  goodFor: string;
  /** A small sketch: SVG path data in a 40 × 24 box, y down. */
  icon: string;
  deck: (s: Site) => string;
}

/** Feet per MHz to metres per MHz. */
const FT = 0.3048;
const C = 299.8;
/** Wire and tubing radii, metres: #14 AWG copper, 1/2" and 1/4" tube or rod. */
export const AWG14 = 0.000814;
const HALF_INCH_TUBE = 0.00635;
const QUARTER_INCH_ROD = 0.003175;

const fmt = (v: number) => String(Number(v.toPrecision(5)));

/** The ground cards. Over real ground, Sommerfeld (GN 2) when any wire is lower than 0.2 λ —
 * the reflection-coefficient method's limit, as 4nec2's checks put it — else GN 0. */
function ground(s: Site, lowest: number): string {
  if (!s.soil) return "GE 0\nGN -1\n";
  const lambda = C / s.mhz;
  const kind = lowest < 0.2 * lambda ? 2 : 0;
  return `GE 0\nGN ${kind} 0 0 0 ${s.soil.epsr} ${s.soil.sigma}          ' ${s.soil.name} ground\n`;
}

function header(title: string, s: Site, notes: string[]): string {
  const where = s.soil ? `${fmt(s.height)} m up over ${s.soil.name.toLowerCase()} ground` : "in free space";
  return [`CM ${title} for ${fmt(s.mhz)} MHz, ${where}`, ...notes.map((n) => `CM ${n}`), "CE"].join("\n") + "\n";
}

const tail = (s: Site) => `FR 0 1 0 0 ${fmt(s.mhz)} 0\nEN\n`;
/** A sensible starting height: a quarter wave up or 10 m, whichever is lower, on HF; 6 m above. */
export const defaultHeight = (mhz: number) => (mhz < 30 ? Math.min(10, Number((C / mhz / 4).toFixed(1))) : 6);

export const TEMPLATES: Template[] = [
  {
    id: "dipole",
    name: "Half-wave dipole",
    about: "A straight wire, fed in the middle. The reference antenna everything is compared with.",
    goodFor: "The easiest antenna to build. Any band; strings between two trees or masts.",
    icon: "M2 12 H19 M21 12 H38 M19 12 V20 M21 12 V20",
    deck: (s) => {
      const L = (468 / s.mhz) * FT;
      return (
        header("Half-wave dipole", s, ["Starting length from the handbook rule 468/f feet. Adjust the length to move the SWR dip."]) +
        `SY L=${fmt(L)}          ' Overall length, m
SY h=${fmt(s.height)}          ' Height above ground, m
SY r=${AWG14}          ' Wire radius, m
GW 1 21 0 -L/2 h 0 L/2 h r
` +
        ground(s, s.height) +
        `EX 0 1 11 0 1 0
` +
        tail(s)
      );
    },
  },
  {
    id: "invv",
    name: "Inverted V",
    about: "A dipole hung from one support in the middle, its ends sloping down.",
    goodFor: "Needs only one tall support. Radiates more evenly in all directions than a flat dipole.",
    icon: "M4 20 L20 6 L36 20 M20 6 V3",
    deck: (s) => {
      const leg = ((468 / s.mhz) * FT) / 2;
      // Keep the ends at least 2 m up.
      const h = Math.max(s.height, leg * Math.sin((30 * Math.PI) / 180) + 2);
      return (
        header("Inverted V", { ...s, height: h }, [
          "Each leg starts at half of the handbook 468/f feet. Height is the apex; the ends stay at least 2 m up.",
        ]) +
        `SY leg=${fmt(leg)}          ' Each leg, m
SY h=${fmt(h)}          ' Apex height, m
SY droop=30          ' Legs below horizontal, deg
SY r=${AWG14}          ' Wire radius, m
SY dy=leg*cos(droop), dz=leg*sin(droop)
GW 1 11 0 -dy h-dz 0 0 h r
GW 2 11 0 0 h 0 dy h-dz r
` +
        ground(s, h - leg * 0.5) +
        `EX 0 1 11 0 1 0
` +
        tail(s)
      );
    },
  },
  {
    id: "gp",
    name: "Ground-plane vertical",
    about: "A quarter-wave vertical with four sloping radials, raised on a mast.",
    goodFor: "Low-angle radiation in all directions, for working far-away stations.",
    icon: "M20 3 V14 M20 14 L6 21 M20 14 L34 21 M20 14 L14 22 M20 14 L26 22",
    deck: (s) => {
      const q = (234 / s.mhz) * FT;
      // The radials slope 35° down from the base: keep their tips at least 1 m up.
      const drop = q * Math.sin((35 * Math.PI) / 180);
      const h = Math.max(s.height, drop + 1);
      return (
        header("Quarter-wave ground plane", { ...s, height: h }, [
          "Radiator and radials start at the handbook 234/f feet. Sloping the radials down raises the feed impedance toward 50 Ω.",
        ]) +
        `SY q=${fmt(q)}          ' Radiator length, m
SY rl=${fmt(q)}          ' Radial length, m
SY h=${fmt(h)}          ' Base height above ground, m
SY slope=35          ' Radials below horizontal, deg
SY r=${AWG14}          ' Wire radius, m
SY rx=rl*cos(slope), rz=h-rl*sin(slope)
GW 1 11 0 0 h 0 0 h+q r
GW 2 9 0 0 h rx 0 rz r
GW 3 9 0 0 h -rx 0 rz r
GW 4 9 0 0 h 0 rx rz r
GW 5 9 0 0 h 0 -rx rz r
` +
        ground(s, h - drop) +
        `EX 0 1 1 0 1 0
` +
        tail(s)
      );
    },
  },
  {
    id: "yagi3",
    name: "3-element Yagi",
    about: "A reflector, a driven element and a director on a boom: a beam.",
    goodFor: "Gain toward one direction and rejection of signals from behind. Rotate it to aim.",
    icon: "M8 4 V20 M20 6 V18 M31 7 V17 M4 12 H36",
    deck: (s) => {
      const lam = C / s.mhz;
      const r = s.mhz < 30 ? HALF_INCH_TUBE : QUARTER_INCH_ROD;
      return (
        header("3-element Yagi", s, [
          "Starting elements from the handbook rules: reflector 500/f, driven element 473/f, director 455/f feet;",
          "spacings 0.15 wavelength. Tune with the Tune tab for the gain, front-to-back and SWR you want.",
        ]) +
        `SY Lr=${fmt((500 / s.mhz) * FT)}          ' Reflector length, m
SY Ld=${fmt((473 / s.mhz) * FT)}          ' Driven element length, m
SY Lf=${fmt((455 / s.mhz) * FT)}          ' Director length, m
SY Sr=${fmt(0.15 * lam)}          ' Reflector spacing, m
SY Sf=${fmt(0.15 * lam)}          ' Director spacing, m
SY h=${fmt(s.height)}          ' Height above ground, m
SY r=${r}          ' Element radius, m
GW 1 21 -Sr -Lr/2 h -Sr Lr/2 h r
GW 2 21 0 -Ld/2 h 0 Ld/2 h r
GW 3 21 Sf -Lf/2 h Sf Lf/2 h r
` +
        ground(s, s.height) +
        `EX 0 2 11 0 1 0
` +
        tail(s)
      );
    },
  },
  {
    id: "quad",
    name: "Quad loop",
    about: "A square loop one wavelength around, standing vertically, fed at the bottom.",
    goodFor: "A little gain over a dipole, quiet on receive, and a single support.",
    icon: "M10 4 H30 V20 H10 Z M20 20 V23",
    deck: (s) => {
      const side = (1005 / s.mhz) * FT / 4;
      return (
        header("Full-wave quad loop", s, ["Starting perimeter from the handbook rule 1005/f feet. Height is the bottom wire."]) +
        `SY s=${fmt(side)}          ' Side length, m
SY h=${fmt(s.height)}          ' Bottom height, m
SY r=${AWG14}          ' Wire radius, m
GW 1 11 0 -s/2 h 0 s/2 h r
GW 2 11 0 s/2 h 0 s/2 h+s r
GW 3 11 0 s/2 h+s 0 -s/2 h+s r
GW 4 11 0 -s/2 h+s 0 -s/2 h r
` +
        ground(s, s.height) +
        `EX 0 1 6 0 1 0
` +
        tail(s)
      );
    },
  },
  {
    id: "delta",
    name: "Delta loop",
    about: "A triangular loop one wavelength around, point up, fed in the middle of the bottom wire.",
    goodFor: "Like a quad loop but needs only one tall support at the apex.",
    icon: "M6 20 L20 3 L34 20 Z M20 20 V23",
    deck: (s) => {
      const side = (1005 / s.mhz) * FT / 3;
      return (
        header("Full-wave delta loop", s, ["Starting perimeter from the handbook rule 1005/f feet. Height is the bottom wire."]) +
        `SY s=${fmt(side)}          ' Side length, m
SY h=${fmt(s.height)}          ' Bottom height, m
SY r=${AWG14}          ' Wire radius, m
SY top=h+s*sin(60)
GW 1 11 0 -s/2 h 0 s/2 h r
GW 2 11 0 s/2 h 0 0 top r
GW 3 11 0 0 top 0 -s/2 h r
` +
        ground(s, s.height) +
        `EX 0 1 6 0 1 0
` +
        tail(s)
      );
    },
  },
  {
    id: "jpole",
    name: "J-pole",
    about: "A half-wave vertical on top of a quarter-wave matching stub, all in one piece.",
    goodFor: "A popular home-built vertical for 2 m and 70 cm that needs no radials.",
    icon: "M17 2 V22 M23 13 V22 M17 22 H23 M17 19 H23",
    deck: (s) => {
      const lam = C / s.mhz;
      const long = 0.735 * lam;
      const stub = 0.24 * lam;
      const gap = 0.02 * lam;
      const tap = 0.02 * lam;
      // About 40 segments per wavelength on every conductor, so parallel parts match.
      const seg = (len: number) => Math.max(1, Math.round(len / (lam / 40)));
      return (
        header("J-pole", s, [
          "Proportions from an NEC optimisation of this model for SWR 1 at 146 and 435 MHz: long element 0.735 λ,",
          "stub 0.24 λ, spacing and feed tap 0.02 λ. Move the tap to bring the SWR down; trim the long element to move it in frequency.",
        ]) +
        `SY long=${fmt(long)}          ' Long element, m
SY stub=${fmt(stub)}          ' Stub length, m
SY gap=${fmt(gap)}          ' Spacing between the two, m
SY tap=${fmt(tap)}          ' Feed height above the bottom, m
SY h=${fmt(s.height)}          ' Bottom height above ground, m
SY r=${fmt(Math.min(QUARTER_INCH_ROD, lam / 650))}          ' Rod radius, m
GW 1 ${seg(tap)} 0 0 h 0 0 h+tap r
GW 2 ${seg(long - tap)} 0 0 h+tap 0 0 h+long r
GW 3 ${seg(tap)} gap 0 h gap 0 h+tap r
GW 4 ${seg(stub - tap)} gap 0 h+tap gap 0 h+stub r
GW 5 1 0 0 h gap 0 h r
GW 6 1 0 0 h+tap gap 0 h+tap r
` +
        ground(s, s.height) +
        `EX 0 6 1 0 1 0
` +
        tail(s)
      );
    },
  },
];

