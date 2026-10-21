// Preferences remembered between sessions, in localStorage. Every read and write is guarded:
// storage can be missing or refuse writes, and the app must work the same without it.

const KEY = "nec-studio.prefs.v1";

export interface Prefs {
  units: "metres" | "feet" | "inches";
  z0: number;
  /** Auto-segmentation on, and its segments per half wave. */
  autoseg: boolean;
  segsPerHalfWave: number;
  stepRadius: boolean;
  sweepSteps: number;
  /** The deck text panel open. */
  deckOpen: boolean;
  /** The welcome has been shown once. */
  welcomed: boolean;
}

export const DEFAULTS: Prefs = {
  units: "metres",
  z0: 50,
  // 4nec2's default; it only ever adds segments, so it cannot break a deck.
  autoseg: true,
  segsPerHalfWave: 20,
  stepRadius: false,
  sweepSteps: 41,
  deckOpen: false,
  welcomed: false,
};

export function loadPrefs(): Prefs {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return { ...DEFAULTS };
    const p = JSON.parse(raw) as Partial<Prefs>;
    return { ...DEFAULTS, ...p };
  } catch {
    return { ...DEFAULTS };
  }
}

export function savePrefs(p: Prefs) {
  try {
    localStorage.setItem(KEY, JSON.stringify(p));
  } catch {
    // Not remembered; nothing else depends on it.
  }
}
