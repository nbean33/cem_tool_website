// Reference tables used by the matching tools and utilities.

/** A transmission line type: characteristic impedance and velocity factor. */
export interface LineType {
  name: string;
  z0: number;
  vf: number;
}

/**
 * Feed and stub lines. Values are the nominal ones 4nec2 ships in `data\Coax.txt`, except
 * RG-213, which 4nec2 lists with a velocity factor of 0.86: RG-213/U has a solid polyethylene
 * dielectric (MIL-C-17), whose velocity factor is 0.66. LMR-400 is from Times Microwave's data
 * sheet (foam PE, 0.85).
 */
export const LINES: LineType[] = [
  { name: "RG-6", z0: 75, vf: 0.75 },
  { name: "RG-8", z0: 52, vf: 0.66 },
  { name: "RG-8X", z0: 50, vf: 0.78 },
  { name: "RG-8 foam", z0: 50, vf: 0.78 },
  { name: "RG-11", z0: 75, vf: 0.66 },
  { name: "RG-11 foam", z0: 75, vf: 0.78 },
  { name: "RG-58", z0: 53.5, vf: 0.66 },
  { name: "RG-58A/C", z0: 50, vf: 0.66 },
  { name: "RG-58A foam", z0: 53.5, vf: 0.78 },
  { name: "RG-59", z0: 75, vf: 0.66 },
  { name: "RG-59 foam", z0: 75, vf: 0.79 },
  { name: "RG-213", z0: 50, vf: 0.66 },
  { name: "LMR-400", z0: 50, vf: 0.85 },
  { name: "Twin lead 75 Ω", z0: 75, vf: 0.67 },
  { name: "Twin lead 300 Ω flat", z0: 300, vf: 0.82 },
  { name: "Twin lead 300 Ω tubular", z0: 300, vf: 0.8 },
  { name: "Open wire 300 Ω", z0: 300, vf: 0.95 },
  { name: "Open wire 450 Ω", z0: 450, vf: 0.95 },
];
