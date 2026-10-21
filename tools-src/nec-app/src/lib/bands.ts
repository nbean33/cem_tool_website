// US amateur bands, 160 m to 70 cm, with the FCC's edges (47 CFR §97.301, the band table for
// General class and above; Technician privileges are narrower on HF). 60 m is five channels
// (§97.303(h)), shown as the span from the first to the last channel centre.

export interface Band {
  /** The name hams use: "40 m". */
  name: string;
  /** Lower and upper edge, MHz. */
  lo: number;
  hi: number;
  /** A note shown with the band. */
  note?: string;
}

export const BANDS: Band[] = [
  { name: "160 m", lo: 1.8, hi: 2.0 },
  { name: "80 m", lo: 3.5, hi: 4.0, note: "3.6–4.0 MHz is often called 75 m (phone)" },
  { name: "60 m", lo: 5.3305, hi: 5.4035, note: "Five channels: 5.3305, 5.3465, 5.357, 5.3715 and 5.4035 MHz" },
  { name: "40 m", lo: 7.0, hi: 7.3 },
  { name: "30 m", lo: 10.1, hi: 10.15, note: "CW and data only" },
  { name: "20 m", lo: 14.0, hi: 14.35 },
  { name: "17 m", lo: 18.068, hi: 18.168 },
  { name: "15 m", lo: 21.0, hi: 21.45 },
  { name: "12 m", lo: 24.89, hi: 24.99 },
  { name: "10 m", lo: 28.0, hi: 29.7 },
  { name: "6 m", lo: 50.0, hi: 54.0 },
  { name: "2 m", lo: 144.0, hi: 148.0 },
  { name: "1.25 m", lo: 222.0, hi: 225.0 },
  { name: "70 cm", lo: 420.0, hi: 450.0 },
];

/** The middle of a band, rounded to a tidy frequency. */
export function centre(b: Band): number {
  const c = (b.lo + b.hi) / 2;
  const step = c < 30 ? 0.001 : 0.01;
  return Math.round(c / step) * step;
}

/** The band that contains `mhz`, if any. */
export function bandOf(mhz: number | null | undefined): Band | null {
  if (mhz == null || !Number.isFinite(mhz)) return null;
  return BANDS.find((b) => mhz >= b.lo - 1e-9 && mhz <= b.hi + 1e-9) ?? null;
}

/** The band nearest to `mhz` (by ratio), for suggestions. */
export function nearestBand(mhz: number): Band {
  let best = BANDS[0];
  let bestD = Infinity;
  for (const b of BANDS) {
    const d = Math.abs(Math.log(mhz / centre(b)));
    if (d < bestD) {
      bestD = d;
      best = b;
    }
  }
  return best;
}
