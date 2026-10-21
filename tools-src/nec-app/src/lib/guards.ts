// Values the editors refuse before they reach the model, with a reason a beginner can act on:
// segment counts that are not whole positive numbers, negative radii and frequencies. Formulas
// (anything not a plain number) and AWG gauges (#14) are left to the engine to judge.

const plain = (v: string) => /^[-+]?(\d+\.?\d*|\.\d+)([eE][-+]?\d+)?$/.test(v.trim());

/** Why `value` cannot go in field `i` of a `mnemonic` card, or null when it can. */
export function fieldProblem(mnemonic: string, i: number, value: string): string | null {
  const v = value.trim();
  if (!plain(v)) return null;
  const x = Number(v);
  const segments = (mnemonic === "GW" || mnemonic === "GA" || mnemonic === "GH") && i === 1;
  if (segments) {
    if (!Number.isInteger(x) || Math.abs(x) < 1) return "The number of segments must be a whole number, 1 or more (a leading + fixes it).";
    if (x < 0 && !v.startsWith("+")) return "The number of segments can't be negative.";
  }
  const radius = (mnemonic === "GW" && i === 8) || (mnemonic === "GA" && i === 5) || (mnemonic === "GH" && i === 8);
  if (radius && x < 0) return "A wire's radius can't be negative.";
  if (radius && x === 0 && mnemonic !== "GW") return "A wire's radius must be more than zero.";
  if (mnemonic === "FR" && i === 4 && !(x > 0)) return "The frequency must be more than 0 MHz.";
  if (mnemonic === "FR" && i === 1 && !(Number.isInteger(x) && x >= 1)) return "The number of frequencies must be a whole number, 1 or more.";
  if ((mnemonic === "EX" || mnemonic === "LD") && i === 2 && x < 0) return "A segment number can't be negative.";
  return null;
}
