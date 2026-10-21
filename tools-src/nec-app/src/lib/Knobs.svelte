<script lang="ts" module>
  /** A variable's label and kind, from its SY line's comment: "Overall length, m". */
  export function describe(name: string, comment: string | null): { label: string; unit: "m" | "deg" | "MHz" | ""; radius: boolean } {
    const c = (comment ?? "").trim();
    const m = c.match(/^(.*?),\s*(m|deg|°|MHz)\s*$/i);
    const label = (m ? m[1] : c) || name;
    const raw = m ? m[2].toLowerCase() : "";
    const unit = raw === "m" ? "m" : raw === "deg" || raw === "°" ? "deg" : raw === "mhz" ? "MHz" : "";
    return { label, unit, radius: unit === "m" && /radius/i.test(label) };
  }

  /** AWG gauge n's radius, metres: diameter 0.127 mm × 92^((36 − n)/39), as the deck's #n reads it. */
  export const awgRadius = (n: number) => (0.127e-3 * 92 ** ((36 - n) / 39)) / 2;

  /** Wire and tubing a ham is likely to use. */
  export const WIRES: { label: string; radius: number }[] = [
    ...[10, 12, 14, 16, 18, 20, 22].map((n) => ({ label: `#${n} AWG wire`, radius: awgRadius(n) })),
    { label: '1/8" rod', radius: 0.0015875 },
    { label: '1/4" rod or tube', radius: 0.003175 },
    { label: '3/8" tube', radius: 0.0047625 },
    { label: '1/2" tube', radius: 0.00635 },
    { label: '3/4" tube', radius: 0.009525 },
    { label: '1" tube', radius: 0.0127 },
  ];
</script>

<script lang="ts">
  // The antenna's adjustable dimensions: one number box and slider per plain-number SY
  // variable, labelled from its comment. Edits rewrite the SY line and re-solve, shortly after
  // the slider stops.
  import * as api from "./api";
  import Help from "./Help.svelte";
  import { tip } from "./tip";
  import type { Analysis, Units, Variable } from "./types";
  import { fromUnit, toUnit, unitOf } from "./units";

  let {
    analysis,
    text,
    units,
    resetKey = 0,
    onedit,
  }: {
    analysis: Analysis | null;
    text: string;
    units: Units;
    /** Changes when another model is loaded, to re-centre the sliders. */
    resetKey?: number;
    onedit: (text: string) => void;
  } = $props();

  let vars = $derived((analysis?.variables ?? []).filter((v) => v.tunable && Number.isFinite(v.value)));

  // Slider ranges: ±30 % around the value the variable had when it first appeared, so a range
  // does not wander while it is dragged.
  let base = $state<Record<string, number>>({});
  $effect(() => {
    void resetKey;
    base = {};
  });
  $effect(() => {
    const next: Record<string, number> = {};
    for (const v of vars) next[v.name] = base[v.name] ?? v.value;
    const same = Object.keys(next).length === Object.keys(base).length && Object.keys(next).every((k) => k in base);
    if (!same) base = next;
  });

  // While a slider moves, show its value at once and write it a moment later.
  let pending = $state<Record<string, number>>({});
  let timer: ReturnType<typeof setTimeout> | undefined;
  let error = $state<string | null>(null);
  function set(name: string, value: number, now = false) {
    if (!Number.isFinite(value)) return;
    pending = { ...pending, [name]: value };
    clearTimeout(timer);
    timer = setTimeout(flush, now ? 0 : 250);
  }
  async function flush() {
    const values = Object.entries(pending) as [string, number][];
    if (!values.length) return;
    pending = {};
    try {
      error = null;
      onedit(await api.setVariables(text, values.map(([n, v]) => [n, Number(v.toPrecision(6))])));
    } catch (e) {
      error = String(e);
    }
  }

  const shown = (v: Variable) => pending[v.name] ?? v.value;
  /** The value in the units it is shown in. */
  function display(v: Variable): number {
    const d = describe(v.name, v.comment);
    const x = shown(v);
    if (d.radius) return Number((toUnit(x, units) * (units === "metres" ? 1000 : 1)).toPrecision(4));
    return d.unit === "m" ? Number(toUnit(x, units).toPrecision(5)) : Number(x.toPrecision(5));
  }
  function unitLabel(v: Variable): string {
    const d = describe(v.name, v.comment);
    if (d.radius) return units === "metres" ? "mm" : unitOf(units).label;
    return d.unit === "m" ? unitOf(units).label : d.unit === "deg" ? "°" : d.unit;
  }
  function typed(v: Variable, raw: string) {
    const d = describe(v.name, v.comment);
    let x = Number(raw);
    if (!Number.isFinite(x)) return;
    if (d.radius) x = fromUnit(units === "metres" ? x / 1000 : x, units);
    else if (d.unit === "m") x = fromUnit(x, units);
    if ((d.unit === "m" && x <= 0) || (d.radius && x <= 0)) {
      error = `${d.label} must be more than zero.`;
      return;
    }
    set(v.name, x, true);
  }
  const range = (v: Variable): [number, number] => {
    let b = base[v.name] ?? v.value;
    // A typed value outside the range centres it.
    if (Math.abs(shown(v) - b) > Math.abs(b) * 0.3) b = shown(v);
    const span = Math.abs(b) * 0.3 || 1;
    return [b - span, b + span];
  };
  /** The closest wire choice, if within 2 %. */
  const wireOf = (r: number) => WIRES.findIndex((w) => Math.abs(w.radius / r - 1) < 0.02);
</script>

<section class="knobs" id="knobs">
  <div class="head">
    <h2>Your antenna</h2>
    <Help term="knobs" />
  </div>
  {#if vars.length}
    <ul>
      {#each vars as v (v.name)}
        {@const d = describe(v.name, v.comment)}
        <li>
          <div class="row">
            <span class="label" use:tip={`${d.label} (variable ${v.name} in the deck)`}>{d.label}</span>
            {#if d.radius}
              {@const w = wireOf(shown(v))}
              <select
                value={w}
                onchange={(e) => {
                  const i = Number((e.currentTarget as HTMLSelectElement).value);
                  if (i >= 0) set(v.name, WIRES[i].radius, true);
                }}
                use:tip={"The wire or tubing; this sets its radius"}
              >
                {#if w < 0}<option value={-1}>custom</option>{/if}
                {#each WIRES as wire, i}<option value={i}>{wire.label}</option>{/each}
              </select>
              <Help term="wire" />
            {:else}
              <input
                class="num"
                data-knob={d.label}
                type="number"
                step="any"
                value={display(v)}
                onchange={(e) => typed(v, (e.currentTarget as HTMLInputElement).value)}
                onkeydown={(e) => e.key === "Enter" && (e.currentTarget as HTMLInputElement).blur()}
              />
              <span class="unit">{unitLabel(v)}</span>
            {/if}
          </div>
          {#if !d.radius}
            {@const [lo, hi] = range(v)}
            <input
              class="slider"
              type="range"
              min={lo}
              max={hi}
              step={(hi - lo) / 400}
              value={shown(v)}
              oninput={(e) => set(v.name, Number((e.currentTarget as HTMLInputElement).value))}
              onchange={(e) => set(v.name, Number((e.currentTarget as HTMLInputElement).value), true)}
              aria-label={d.label}
            />
          {/if}
        </li>
      {/each}
    </ul>
  {:else if analysis}
    <p class="hint">This model has no adjustable dimensions. Change its wires on the Shape tab, or start from a template (New…), which comes with knobs.</p>
  {/if}
  {#if error}<p class="err">{error}</p>{/if}
</section>

<style>
  .knobs {
    display: flex;
    flex-direction: column;
    min-height: 0;
    max-height: 45%;
    flex: none;
  }
  .head {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-bottom: 6px;
  }
  h2 {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.8px;
    color: var(--muted);
    margin: 0;
    font-weight: 600;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 6px 10px;
    overflow-y: auto;
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--panel);
  }
  li {
    padding: 4px 0;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .label {
    flex: 1;
    font-size: 12.5px;
  }
  .num {
    width: 86px;
    text-align: right;
  }
  .unit {
    width: 22px;
    color: var(--muted);
    font-size: 12px;
  }
  .slider {
    width: 100%;
    margin: 2px 0 0;
    accent-color: var(--accent);
  }
  select {
    max-width: 150px;
  }
  .hint {
    font-size: 12px;
    color: var(--muted);
    margin: 0;
  }
  .err {
    color: #ff8a9a;
    font-size: 12px;
    margin: 4px 0 0;
  }
</style>
