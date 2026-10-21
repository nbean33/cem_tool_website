// A perceptually ordered blue → cyan → green → yellow → red ramp for currents and gain.
const STOPS: [number, number, number][] = [
  [0.19, 0.07, 0.23],
  [0.27, 0.42, 0.93],
  [0.11, 0.81, 0.83],
  [0.39, 0.99, 0.42],
  [0.88, 0.86, 0.22],
  [0.98, 0.48, 0.13],
  [0.73, 0.08, 0.04],
];

/** Colour for `t` in [0, 1], as RGB in [0, 1]. */
export function ramp(t: number): [number, number, number] {
  const x = Math.min(1, Math.max(0, Number.isFinite(t) ? t : 0)) * (STOPS.length - 1);
  const i = Math.min(STOPS.length - 2, Math.floor(x));
  const f = x - i;
  const [a, b] = [STOPS[i], STOPS[i + 1]];
  return [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f, a[2] + (b[2] - a[2]) * f];
}

/** The ramp as a CSS linear-gradient, for legends. */
export const rampGradient = () =>
  `linear-gradient(to right, ${STOPS.map((c, i) => `rgb(${c.map((v) => Math.round(v * 255)).join(",")}) ${(100 * i) / (STOPS.length - 1)}%`).join(", ")})`;

/**
 * The ARRL log-periodic scale: radius 0.89 per 2 dB below the peak, so −3, −10 and −20 dB sit at
 * 0.84, 0.56 and 0.31 of the outer ring and front-to-back is visible without losing the nulls.
 */
export const arrl = (gain: number, peak: number) => Math.pow(0.89, Math.max(0, peak - gain) / 2);
