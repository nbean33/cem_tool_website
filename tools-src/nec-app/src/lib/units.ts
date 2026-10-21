// Display units for lengths. The deck and the engine always work in metres; these only change
// what is shown and what typed lengths mean.
import type { Units } from "./types";

export const UNITS: { id: Units; label: string; metres: number }[] = [
  { id: "metres", label: "m", metres: 1 },
  { id: "feet", label: "ft", metres: 0.3048 },
  { id: "inches", label: "in", metres: 0.0254 },
];

export const unitOf = (u: Units) => UNITS.find((x) => x.id === u) ?? UNITS[0];

/** A length in metres, in the display unit, with its unit label. */
export function length(m: number, u: Units, digits = 4): string {
  const x = unitOf(u);
  return `${(m / x.metres).toFixed(digits)} ${x.label}`;
}

/** Metres to the display unit. */
export const toUnit = (m: number, u: Units) => m / unitOf(u).metres;
/** The display unit to metres. */
export const fromUnit = (v: number, u: Units) => v * unitOf(u).metres;
