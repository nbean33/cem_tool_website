// The engine's error messages, said plainly: what is wrong, on which line, and what to do. The
// original message is kept for the details.

const CARD_NAMES: Record<string, string> = {
  GW: "straight wire",
  GA: "arc",
  GH: "helix",
  GM: "move/copy",
  GX: "reflection",
  GR: "rotation",
  GS: "scale",
  GC: "taper",
  GE: "end of geometry",
  SP: "patch",
  SM: "patch grid",
  SC: "patch corner",
  EX: "source",
  LD: "load",
  TL: "transmission line",
  NT: "network",
  FR: "frequency",
  GN: "ground",
  GD: "second ground",
  RP: "radiation pattern",
  SY: "variable",
};

const name = (m: string) => (CARD_NAMES[m] ? `${CARD_NAMES[m]} (${m})` : m);

export interface Explained {
  /** The plain explanation. */
  text: string;
  /** The original message, when the explanation differs from it. */
  raw: string | null;
}

const RULES: [RegExp, (m: RegExpMatchArray) => string][] = [
  [/line (\d+): (\w+) card needs at least (\d+) fields, found (\d+)/, (m) => `Line ${m[1]}: a ${name(m[2])} card needs ${m[3]} numbers, but has ${m[4]}. Check for a missing value.`],
  [/line (\d+): cannot parse field "(.*)" on (\w+) card/, (m) => `Line ${m[1]}: "${m[2]}" on the ${name(m[3])} card is not a number or a formula. Check for a typo, or a comma instead of a decimal point.`],
  [/line (\d+): undefined symbol "(.*)"/, (m) => `Line ${m[1]} uses "${m[2]}", which is not defined. Add an SY line for it (SY ${m[2]}=…) above, or check the spelling.`],
  [/line (\d+): unknown function "(.*)"/, (m) => `Line ${m[1]} uses a function "${m[2]}" that NEC Studio does not know. These work: sin and cos (in degrees), sqr (square root), log (natural logarithm) and int (whole part).`],
  [/line (\d+): malformed card mnemonic "(.*)"/, (m) => `Line ${m[1]} does not start with a card name NEC knows, such as GW, EX or FR ("${m[2]}").`],
  [/line (\d+): malformed SY assignment "(.*)"/, (m) => `Line ${m[1]}: a variable line should read SY name=value (found "${m[2]}").`],
  [/line (\d+): bad expression "(.*)": (.*)/, (m) => `Line ${m[1]}: the formula "${m[2]}" cannot be worked out: ${m[3]}.`],
  [/line (\d+): GW tag (-?\d+) requests (-?\d+) segments/, (m) => `Line ${m[1]}: wire ${m[2]} has ${m[3]} segments; it needs at least 1.`],
  [/line (\d+): GW tag (-?\d+) has zero length/, (m) => `Line ${m[1]}: wire ${m[2]} starts and ends at the same point, so it has no length.`],
  [/model has (\d+) unknowns .*supports (\d+)/, (m) => `This model is too big to solve: ${m[1]} segments (patches count twice), and NEC Studio handles up to ${m[2]}. Use fewer segments, or fewer segments per half wave in Settings.`],
  [/line (\d+): EX tag (-?\d+) segment (-?\d+) does not exist/, (m) => `Line ${m[1]}: the source is on wire ${m[2]}, segment ${m[3]}, which does not exist. Check the wire's tag and its number of segments.`],
  [/no EX card|has no voltage source/, () => "There is no source, so nothing feeds the antenna. Add one on the Feed & ground tab (+ Source)."],
  [/line (\d+): (\w+) is not a NEC-2 geometry card(.*)/, (m) => `Line ${m[1]}: ${m[2]} is not a NEC-2 card${m[3] ? m[3] : ""}. NEC Studio reads NEC-2 models.`],
  [/line (\d+): (\w+) card is not implemented yet/, (m) => `Line ${m[1]}: the ${name(m[2])} card is not supported yet.`],
  [/only be solved there|can only be solved at/, () => "This model is built on a saved structure (a numerical Green's function), so it can only be solved at the frequency that was saved."],
];

export function explain(raw: string): Explained {
  const msg = raw.replace(/^Error:\s*/, "");
  for (const [re, say] of RULES) {
    const m = msg.match(re);
    if (m) return { text: say(m), raw: msg };
  }
  return { text: msg, raw: null };
}
