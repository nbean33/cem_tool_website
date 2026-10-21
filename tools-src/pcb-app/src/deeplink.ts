// Calculator deep links for the website: tools/pcb/#impedance.microstrip opens that calculator.
import { CALCULATORS } from "./schema/calculators";

/** The calculator id named by the URL hash, if it is one. */
export function idFromHash(): string | undefined {
  const id = decodeURIComponent(location.hash.slice(1));
  return CALCULATORS.some((c) => c.id === id) ? id : undefined;
}
