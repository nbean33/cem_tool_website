<script lang="ts">
  import Help from "./Help.svelte";
  import { tip } from "./tip";
  import { hint } from "./help";
  // Tune the deck's SY variables against goals, then write the result back into the deck.
  import { onMount } from "svelte";
  import * as api from "./api";
  import LineChart from "./LineChart.svelte";
  import type { Analysis, Goal, GoalKind, Method, ModelOptions, OptimizeOutcome, OptimizeProgress, OptimizerSettings } from "./types";

  let {
    analysis,
    text,
    z0,
    azimuth = $bindable(0),
    band,
    model,
    onapply,
  }: {
    analysis: Analysis | null;
    text: string;
    z0: number;
    azimuth?: number;
    band: [number, number];
    model: ModelOptions;
    onapply: (text: string) => void;
  } = $props();

  const KINDS: { kind: GoalKind; label: string; unit: string; placeholder: string }[] = [
    { kind: "swr", label: "SWR ≤", unit: "", placeholder: "1" },
    { kind: "gain", label: "Gain ≥", unit: "dBi", placeholder: "max" },
    { kind: "front_to_back", label: "F/B ≥", unit: "dB", placeholder: "max" },
    { kind: "front_to_rear", label: "F/R ≥", unit: "dB", placeholder: "max" },
    { kind: "resistance", label: "R =", unit: "Ω", placeholder: "50" },
    { kind: "reactance", label: "X =", unit: "Ω", placeholder: "0" },
    { kind: "efficiency", label: "Efficiency ≥", unit: "%", placeholder: "100" },
  ];
  const label = (k: GoalKind) => KINDS.find((x) => x.kind === k)!;

  interface Range {
    tune: boolean;
    min: number;
    max: number;
  }
  let ranges = $state<Record<string, Range>>({});
  let goals = $state<Goal[]>([
    { kind: "swr", target: 1.5, weight: 1 },
    { kind: "gain", target: null, weight: 1 },
    { kind: "front_to_back", target: 20, weight: 0.5 },
  ]);
  let freqText = $state("");
  /** The frequencies follow the sweep band until they are typed or read from a project. */
  let followBand = $state(true);
  let elevation = $state<number | null>(null);
  let method = $state<Method>("nelder_mead");
  let budget = $state(600);
  let running = $state(false);
  let error = $state<string | null>(null);
  let job = 0;
  let history = $state<{ evals: number[]; cost: number[] }>({ evals: [], cost: [] });
  let live = $state<OptimizeProgress | null>(null);
  let outcome = $state<OptimizeOutcome | null>(null);
  let tuned = $state<string[]>([]);
  let undoText = $state<string | null>(null);

  const key = (n: string) => n.toLowerCase();
  const round = (v: number, d = 4) => +v.toPrecision(d);

  // New variables get a ±10 % range; existing ranges are kept across edits.
  $effect(() => {
    for (const v of analysis?.variables ?? []) {
      const k = key(v.name);
      if (ranges[k]) continue;
      const span = Math.abs(v.value) * 0.1 || 1;
      ranges[k] = { tune: false, min: round(v.value - span), max: round(v.value + span) };
    }
  });

  $effect(() => {
    if (followBand && band[0] > 0) useBand();
  });

  function useBand() {
    const [a, b] = band;
    freqText = [a, (a + b) / 2, b].map((f) => round(f, 6)).join(", ");
  }

  let frequencies = $derived(
    freqText
      .split(/[\s,;]+/)
      .map(Number)
      .filter((f) => Number.isFinite(f) && f > 0),
  );
  let selected = $derived((analysis?.variables ?? []).filter((v) => ranges[key(v.name)]?.tune));
  let needsPattern = $derived(goals.some((g) => g.kind === "gain" || g.kind === "front_to_back" || g.kind === "front_to_rear"));
  let shown = $derived(live ?? outcome?.best ?? null);

  async function run() {
    if (!selected.length || !frequencies.length || !goals.length) return;
    error = null;
    outcome = null;
    live = null;
    undoText = null;
    history = { evals: [], cost: [] };
    tuned = selected.map((v) => v.name);
    running = true;
    try {
      outcome = await api.optimize(text, {
        parameters: selected.map((v) => ({ name: v.name, min: ranges[key(v.name)].min, max: ranges[key(v.name)].max })),
        frequencies_mhz: frequencies,
        goals: $state.snapshot(goals),
        z0,
        direction: { azimuth, elevation },
        method,
        max_evaluations: Math.max(10, Math.round(budget)),
        seed: (Math.random() * 2 ** 32) >>> 0,
        model,
      });
      live = null;
    } catch (e) {
      error = String(e);
    } finally {
      running = false;
    }
  }

  /** The set-up, for a project file. */
  export function settings(): OptimizerSettings {
    return {
      parameters: Object.entries(ranges)
        .filter(([, r]) => r.tune)
        .map(([k, r]) => ({ name: (analysis?.variables ?? []).find((v) => key(v.name) === k)?.name ?? k, min: r.min, max: r.max })),
      frequencies_mhz: followBand ? [] : frequencies,
      goals: $state.snapshot(goals),
      direction: { azimuth, elevation },
      method,
      max_evaluations: Math.max(10, Math.round(budget)),
    };
  }

  /** Restore a set-up from a project file; `null` resets to the defaults for a new model. */
  export function restore(s: OptimizerSettings | null | undefined) {
    ranges = {};
    outcome = null;
    live = null;
    undoText = null;
    error = null;
    history = { evals: [], cost: [] };
    if (!s) {
      followBand = true;
      return;
    }
    for (const p of s.parameters) ranges[key(p.name)] = { tune: true, min: p.min, max: p.max };
    if (s.goals.length) goals = s.goals.map((g) => ({ ...g, target: g.target ?? null }));
    followBand = !s.frequencies_mhz.length;
    if (!followBand) freqText = s.frequencies_mhz.join(", ");
    if (s.direction) {
      azimuth = s.direction.azimuth;
      elevation = s.direction.elevation ?? null;
    }
    method = s.method;
    budget = s.max_evaluations;
  }

  async function apply() {
    const best = outcome?.best;
    if (!best) return;
    undoText = text;
    onapply(await api.setVariables(text, tuned.map((n, i) => [n, best.values[i]] as [string, number])));
  }

  onMount(() => {
    const off = api.onOptimizeProgress((j, p) => {
      if (j < job) return;
      if (j > job) {
        job = j;
        history = { evals: [], cost: [] };
      }
      live = p;
      history.evals.push(p.evaluations);
      history.cost.push(p.cost);
    });
    return () => void off.then((f) => f());
  });

  const fmt = (v: number | null | undefined, d = 2) => (v == null || !Number.isFinite(v) ? "–" : v.toFixed(d));
  const z = (m: { z: [number, number] }) => `${m.z[0].toFixed(1)} ${m.z[1] < 0 ? "−" : "+"} j${Math.abs(m.z[1]).toFixed(1)}`;
</script>

<section class="card">
  <h3>Variables<Help term="tuneVariables" /></h3>
  {#if analysis?.variables.length}
    <table class="vars">
      <thead><tr><th></th><th>Name</th><th>Value</th><th>Min</th><th>Max</th></tr></thead>
      <tbody>
        {#each analysis.variables as v}
          {@const r = ranges[key(v.name)]}
          {#if r}
            <tr class:formula={!v.tunable}>
              <td><input type="checkbox" bind:checked={r.tune} disabled={running} /></td>
              <td title={v.tunable ? "" : `= ${v.expression} — tuning this replaces its formula with a number`}>
                {v.name}{#if !v.tunable}<span class="f">ƒ</span>{/if}
              </td>
              <td class="num">{round(v.value, 6)}</td>
              <td><input type="number" step="any" bind:value={r.min} disabled={!r.tune || running} /></td>
              <td><input type="number" step="any" bind:value={r.max} disabled={!r.tune || running} /></td>
            </tr>
          {/if}
        {/each}
      </tbody>
    </table>
  {:else}
    <p class="hint">This deck has no <code>SY</code> variables. Write dimensions as variables
      (<code>SY len=5.1</code>, then <code>GW 1 21 0 -len 10 0 len 10 #12</code>) to tune them.</p>
  {/if}
</section>

<section class="card">
  <h3>Goals<Help term="tuneGoals" /></h3>
  {#each goals as g, i}
    <div class="goal">
      <select bind:value={g.kind} disabled={running}>
        {#each KINDS as k}<option value={k.kind}>{k.label}</option>{/each}
      </select>
      <input
        type="number"
        step="any"
        placeholder={label(g.kind).placeholder}
        value={g.target ?? ""}
        oninput={(e) => {
          const v = (e.currentTarget as HTMLInputElement).value;
          g.target = v === "" ? null : Number(v);
        }}
        disabled={running}
      />
      <span class="unit">{label(g.kind).unit}</span>
      <label class="w" use:tip={hint("goalWeight")}>× <input type="number" step="0.1" min="0" bind:value={g.weight} disabled={running} /></label>
      <button class="icon" onclick={() => goals.splice(i, 1)} disabled={running} title="Remove">✕</button>
    </div>
  {/each}
  <button class="small" onclick={() => goals.push({ kind: "swr", target: 1.5, weight: 1 })} disabled={running}>+ Goal</button>

  <div class="row">
    <label class="grow" use:tip={hint("tuneFrequencies")}>At MHz <input type="text" bind:value={freqText} oninput={() => (followBand = false)} disabled={running} /></label>
    <button class="small" onclick={() => ((followBand = true), useBand())} disabled={running} title="The sweep's start, centre and stop">Sweep band</button>
  </div>
  {#if needsPattern}
    <div class="row">
      <label use:tip={hint("forwardAz")}>Forward az <input type="number" class="short" bind:value={azimuth} disabled={running} />°</label>
      <label use:tip={hint("forwardEl")}
        >elevation <input
          type="number"
          class="short"
          placeholder="best"
          value={elevation ?? ""}
          oninput={(e) => {
            const v = (e.currentTarget as HTMLInputElement).value;
            elevation = v === "" ? null : Number(v);
          }}
          disabled={running}
        />°</label
      >
    </div>
  {/if}
  <div class="row">
    <select bind:value={method} disabled={running} use:tip={hint("tuneMethod")}>
      <option value="nelder_mead">Nelder–Mead (local)</option>
      <option value="differential_evolution">Differential evolution (global)</option>
    </select>
    <label use:tip={hint("tuneBudget")}>max <input type="number" class="short" step="100" min="10" bind:value={budget} disabled={running} /> evals</label>
  </div>
  <div class="row">
    {#if running}
      <button class="warn" onclick={() => api.cancel("optimize")}>Stop</button>
    {:else}
      <button class="primary" onclick={run} disabled={!selected.length || !frequencies.length || !goals.length} use:tip={hint("tuneRun")}>
        Optimize {selected.length ? `${selected.length} variable${selected.length > 1 ? "s" : ""}` : ""}
      </button>
    {/if}
    {#if shown}
      <span class="stat">{shown.evaluations} evals · cost {fmt(shown.cost, 3)}</span>
    {/if}
  </div>
  {#if error}<p class="error">{error}</p>{/if}
</section>

{#if history.evals.length > 1}
  <section class="card">
    <LineChart
      x={history.evals}
      series={[{ label: "cost", values: history.cost, color: "#ffae45", digits: 3 }]}
      height={120}
      xDigits={0}
      xLabel="evals"
    />
  </section>
{/if}

{#if shown}
  <section class="card">
    <h3>
      {running ? "Best so far" : outcome?.cancelled ? "Stopped — best found" : outcome?.converged ? "Converged" : "Best found"}
    </h3>
    <table class="out">
      <thead><tr><th>Variable</th><th>Start</th><th>Best</th></tr></thead>
      <tbody>
        {#each tuned as name, i}
          <tr>
            <td>{name}</td>
            <td class="num">{outcome ? round(outcome.initial.values[i], 6) : "…"}</td>
            <td class="num best">{round(shown.values[i], 6)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
    <table class="out">
      <thead><tr><th>MHz</th><th>Z (Ω)</th><th>SWR</th><th>Gain</th><th>F/B</th><th>F/R</th></tr></thead>
      <tbody>
        {#each shown.metrics as m, i}
          {@const before = outcome?.initial.metrics[i]}
          <tr>
            <td class="num">{round(m.mhz, 6)}</td>
            <td class="num">{z(m)}</td>
            <td class="num">{fmt(m.swr)}{#if before}<small>{fmt(before.swr)}</small>{/if}</td>
            <td class="num">{fmt(m.gain)}{#if before}<small>{fmt(before.gain)}</small>{/if}</td>
            <td class="num">{fmt(m.front_to_back, 1)}{#if before}<small>{fmt(before.front_to_back, 1)}</small>{/if}</td>
            <td class="num">{fmt(m.front_to_rear, 1)}{#if before}<small>{fmt(before.front_to_rear, 1)}</small>{/if}</td>
          </tr>
        {/each}
      </tbody>
    </table>
    {#if outcome}
      <p class="hint">Small figures are the starting design.</p>
      <div class="row">
        <button class="primary" onclick={apply} use:tip={hint("tuneApply")}>Apply to deck</button>
        {#if undoText != null}
          <button
            onclick={() => {
              onapply(undoText!);
              undoText = null;
            }}>Undo apply</button
          >
        {/if}
      </div>
    {/if}
  </section>
{/if}

<style>
  h3 {
    margin: 0 0 8px;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.8px;
    color: var(--muted);
    font-weight: 600;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 12px;
  }
  th {
    text-align: left;
    color: var(--muted);
    font-weight: 500;
    padding: 2px 4px;
  }
  td {
    padding: 2px 4px;
    border-top: 1px solid #1c2533;
  }
  .num {
    font-variant-numeric: tabular-nums;
    text-align: right;
  }
  .vars input[type="number"] {
    width: 78px;
    padding: 2px 5px;
  }
  tr.formula td:nth-child(2) {
    color: var(--muted);
    font-style: italic;
  }
  .f {
    margin-left: 4px;
    color: #9cc2ff;
    font-style: normal;
  }
  .goal {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 6px;
  }
  .goal select {
    width: 112px;
  }
  .goal input[type="number"] {
    width: 70px;
  }
  .goal .w input {
    width: 52px;
  }
  .unit {
    width: 28px;
    color: var(--muted);
    font-size: 11px;
  }
  .w {
    color: var(--muted);
    font-size: 12px;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 8px;
    font-size: 12px;
    color: var(--muted);
  }
  .row label {
    display: flex;
    align-items: center;
    gap: 5px;
  }
  .grow {
    flex: 1;
  }
  .grow input {
    flex: 1;
    min-width: 0;
  }
  input.short {
    width: 58px;
  }
  select {
    font: inherit;
    font-size: 12px;
    color: var(--text);
    background: #0e141d;
    border: 1px solid #2c3a4f;
    border-radius: 6px;
    padding: 4px 5px;
  }
  button.small {
    padding: 3px 9px;
    font-size: 12px;
  }
  button.icon {
    padding: 2px 7px;
    background: none;
    border-color: transparent;
    color: var(--muted);
  }
  .stat {
    margin-left: auto;
    font-variant-numeric: tabular-nums;
  }
  .best {
    color: #ffcf6a;
  }
  small {
    display: block;
    color: #5d6c82;
    font-size: 10px;
  }
  .out + .out {
    margin-top: 10px;
  }
  .hint {
    color: var(--muted);
    font-size: 11.5px;
    margin: 6px 0 0;
  }
  .error {
    color: #ff8a9a;
    font-size: 12px;
  }
  code {
    font-size: 11px;
    color: #c9d6e8;
  }
</style>
