// Plain-language readings off a sweep: where the antenna is resonant, where its SWR is low
// enough to use, how much of a band that covers, and which way to trim it. Pure functions over
// sweep points sorted by frequency.

import type { Band } from "./bands";

export interface Point {
  mhz: number;
  z: [number, number];
  swr: number;
}

/** Frequencies where the reactance crosses zero, interpolated linearly between points. */
export function resonances(points: Point[]): number[] {
  const out: number[] = [];
  for (let i = 1; i < points.length; i++) {
    const [a, b] = [points[i - 1], points[i]];
    const [xa, xb] = [a.z[1], b.z[1]];
    if (xa === 0) out.push(a.mhz);
    else if (Math.sign(xa) !== Math.sign(xb) && xb !== 0) out.push(a.mhz + ((b.mhz - a.mhz) * -xa) / (xb - xa));
  }
  if (points.length && points[points.length - 1].z[1] === 0) out.push(points[points.length - 1].mhz);
  return out;
}

/** The resonance nearest `mhz`, if the sweep has one. */
export function resonanceNear(points: Point[], mhz: number): number | null {
  const r = resonances(points);
  if (!r.length) return null;
  return r.reduce((best, f) => (Math.abs(f - mhz) < Math.abs(best - mhz) ? f : best));
}

export interface Span {
  lo: number;
  hi: number;
  /** The span runs into the end of the sweep, so it may be wider. */
  openLo: boolean;
  openHi: boolean;
}

/** Where the SWR is at most `limit`, in the run of points around the lowest SWR (edges
 * interpolated). Null when no point is that low. */
export function swrSpan(points: Point[], limit = 2): Span | null {
  if (!points.length) return null;
  let k = 0;
  for (let i = 1; i < points.length; i++) if (points[i].swr < points[k].swr) k = i;
  if (!(points[k].swr <= limit)) return null;
  let i = k;
  while (i > 0 && points[i - 1].swr <= limit) i--;
  let j = k;
  while (j < points.length - 1 && points[j + 1].swr <= limit) j++;
  const cross = (a: Point, b: Point) => a.mhz + ((b.mhz - a.mhz) * (limit - a.swr)) / (b.swr - a.swr);
  return {
    lo: i > 0 ? cross(points[i], points[i - 1]) : points[0].mhz,
    hi: j < points.length - 1 ? cross(points[j], points[j + 1]) : points[points.length - 1].mhz,
    openLo: i === 0,
    openHi: j === points.length - 1,
  };
}

/** The part of `band` inside `span`, if any. */
export function coverage(span: Span, band: Band): { lo: number; hi: number; whole: boolean } | null {
  const lo = Math.max(span.lo, band.lo);
  const hi = Math.min(span.hi, band.hi);
  if (hi <= lo) return null;
  return { lo, hi, whole: lo <= band.lo + 1e-9 && hi >= band.hi - 1e-9 };
}

export interface Trim {
  /** "lengthen" when resonance is above the target (the antenna is short), else "shorten". */
  direction: "lengthen" | "shorten";
  /** By about this fraction, as a percentage. */
  percent: number;
  /** The resonance it is judged from, or null when the sweep only shows which side it is. */
  resonance: number | null;
  /** The resonance is outside the sweep, estimated by extending the reactance's slope. */
  estimated: boolean;
}

/**
 * Which way to trim a simple wire antenna to bring its resonance to `target`. Resonance scales
 * about inversely with length, so a dipole resonant 2 % high wants to be about 2 % longer; this is
 * a first-order rule for dipoles, verticals and loops, not for traps or multiband designs. Within
 * 0.5 % nothing is suggested. With no resonance in the sweep, the sign of the reactance says which
 * side it is: below its resonance a wire antenna is capacitive (X < 0); and the reactance's slope
 * at the nearer end of the sweep, extended to zero, estimates how far (trusted only within 15 %).
 */
export function trim(points: Point[], target: number): Trim | null {
  const r = resonanceNear(points, target);
  if (r != null) {
    const ratio = r / target;
    if (Math.abs(ratio - 1) < 0.005) return null;
    return { direction: ratio > 1 ? "lengthen" : "shorten", percent: Math.abs(ratio - 1) * 100, resonance: r, estimated: false };
  }
  if (points.length < 2) return null;
  const below = points.every((p) => p.z[1] < 0);
  const above = points.every((p) => p.z[1] > 0);
  if (!below && !above) return null;
  // Capacitive throughout: the resonance is above, so extend from the top end; inductive: below.
  const [a, b] = below ? [points[points.length - 2], points[points.length - 1]] : [points[0], points[1]];
  const slope = (b.z[1] - a.z[1]) / (b.mhz - a.mhz);
  const end = below ? b : a;
  let est: number | null = null;
  if (slope > 0) {
    const f = end.mhz - end.z[1] / slope;
    if (Math.abs(f / target - 1) < 0.15) est = f;
  }
  const direction = below ? "lengthen" : "shorten";
  if (est == null) return { direction, percent: NaN, resonance: null, estimated: false };
  return { direction, percent: Math.abs(est / target - 1) * 100, resonance: est, estimated: true };
}

export type Verdict = { level: "excellent" | "good" | "tuner" | "poor"; text: string };

/** What an SWR means for a typical station. The thresholds are the ones radios are built to:
 * see `HELP.swrVerdict` for the sources. */
export function swrVerdict(swr: number): Verdict {
  if (!Number.isFinite(swr)) return { level: "poor", text: "No match at all" };
  if (swr <= 1.5) return { level: "excellent", text: "Excellent match" };
  if (swr <= 2) return { level: "good", text: "Good: radios give full power" };
  if (swr <= 3) return { level: "tuner", text: "Usable with an antenna tuner" };
  return { level: "poor", text: "Needs matching" };
}

/** Gain over a half-wave dipole in free space (2.15 dBi). */
export const dBd = (dBi: number) => dBi - 2.15;
