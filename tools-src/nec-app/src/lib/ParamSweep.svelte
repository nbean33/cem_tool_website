<script lang="ts">
  import Help from "./Help.svelte";
  import { tip } from "./tip";
  import { hint } from "./help";
  // Sweep one SY variable at a fixed frequency; click a point to set the variable to it.
  import { onMount } from "svelte";
  import * as api from "./api";
  import LineChart, { type Series } from "./LineChart.svelte";
  import { PointStream } from "./stream.svelte";
  import type { Analysis, ModelOptions } from "./types";

  let {
    analysis,
    text,
    mhz,
    z0,
    azimuth,
    model,
    onapply,
  }: {
    analysis: Analysis | null;
    text: string;
    mhz: number | null;
    z0: number;
    azimuth: number;
    model: ModelOptions;
    onapply: (text: string) => void;
  } = $props();

  let name = $state("");
  let start = $state(0);
  let stop = $state(1);
  let steps = $state(21);
  let withPattern = $state(true);
  let running = $state(false);
  let error = $state<string | null>(null);
  const stream = new PointStream();

  function pick(n: string) {
    name = n;
    const v = analysis?.variables.find((x) => x.name === n)?.value ?? 0;
    const span = Math.abs(v) * 0.2 || 1;
    start = +(v - span).toPrecision(4);
    stop = +(v + span).toPrecision(4);
  }

  $effect(() => {
    const vars = analysis?.variables ?? [];
    if (vars.length && !vars.some((v) => v.name === name)) pick((vars.find((v) => v.tunable) ?? vars[0]).name);
  });

  async function run() {
    if (!name || mhz == null) return;
    error = null;
    running = true;
    try {
      await api.parameterSweep(text, name, start, stop, Math.max(2, Math.round(steps)), mhz, {
        z0,
        pattern: withPattern ? { azimuth, elevation: null } : null,
        model,
      });
    } catch (e) {
      error = String(e);
    } finally {
      running = false;
    }
  }

  async function setTo(v: number) {
    onapply(await api.setVariables(text, [[name, +v.toPrecision(7)]]));
  }

  onMount(() => {
    const off = api.onParamPoint((e) => stream.accept(e));
    return () => void off.then((f) => f());
  });

  let ok = $derived(stream.points.filter((p) => p.metrics));
  let failed = $derived(stream.points.find((p) => p.error)?.error ?? null);
  let x = $derived(ok.map((p) => p.x));
  let hasGain = $derived(ok.some((p) => p.metrics!.gain != null));
  let series = $derived<Series[]>([
    { label: "SWR", values: ok.map((p) => Math.min(p.metrics!.swr, 20)), color: "#ffae45" },
    ...(hasGain
      ? [
          { label: "Gain dBi", values: ok.map((p) => p.metrics!.gain), color: "#6fd39a", axis: "right" as const },
          { label: "F/B dB", values: ok.map((p) => p.metrics!.front_to_back), color: "#ff7a8c", axis: "right" as const, dash: true, digits: 1 },
          { label: "F/R dB", values: ok.map((p) => p.metrics!.front_to_rear), color: "#c9a0ff", axis: "right" as const, dash: true, digits: 1 },
        ]
      : [
          { label: "R Ω", values: ok.map((p) => p.metrics!.z[0]), color: "#58b7ff", axis: "right" as const, digits: 1 },
          { label: "X Ω", values: ok.map((p) => p.metrics!.z[1]), color: "#b07cff", axis: "right" as const, dash: true, digits: 1 },
        ]),
  ]);
  let current = $derived(analysis?.variables.find((v) => v.name === name)?.value ?? null);
</script>

<section class="card">
  <h3>Parameter sweep<Help term="paramSweep" /></h3>
  {#if analysis?.variables.length}
    <div class="row">
      <select value={name} onchange={(e) => pick((e.currentTarget as HTMLSelectElement).value)} disabled={running}>
        {#each analysis.variables as v}<option value={v.name}>{v.name}{v.tunable ? "" : " (ƒ)"}</option>{/each}
      </select>
      <label>from <input type="number" step="any" bind:value={start} disabled={running} /></label>
      <label>to <input type="number" step="any" bind:value={stop} disabled={running} /></label>
    </div>
    <div class="row">
      <label>steps <input type="number" class="short" bind:value={steps} disabled={running} /></label>
      <label use:tip={hint("paramPattern")}><input type="checkbox" bind:checked={withPattern} disabled={running} /> gain &amp; F/B</label>
      <span class="at">at {mhz?.toFixed(3) ?? "–"} MHz</span>
      {#if running}
        <button class="warn" onclick={() => api.cancel("parameter")}>Stop</button>
      {:else}
        <button class="primary" onclick={run} disabled={mhz == null}>Run</button>
      {/if}
    </div>
    {#if error ?? failed}<p class="error">{error ?? failed}</p>{/if}
    {#if ok.length > 1}
      <LineChart {x} {series} marker={current} height={170} xDigits={3} xLabel={name} leftRange={(_a, b) => [1, Math.max(3, Math.min(20, Math.ceil(b)))]} onpick={setTo} />
      <p class="hint">Click the chart to set <b>{name}</b> to that value.</p>
    {/if}
  {:else}
    <p class="hint">No <code>SY</code> variables to sweep.</p>
  {/if}
</section>

<style>
  h3 {
    margin: 0 0 8px;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.8px;
    color: var(--muted);
    font-weight: 600;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
    font-size: 12px;
    color: var(--muted);
  }
  .row label {
    display: flex;
    align-items: center;
    gap: 5px;
  }
  .row input[type="number"] {
    width: 74px;
  }
  input.short {
    width: 50px !important;
  }
  .at {
    margin-left: auto;
  }
  select {
    font: inherit;
    font-size: 12px;
    color: var(--text);
    background: #0e141d;
    border: 1px solid #2c3a4f;
    border-radius: 6px;
    padding: 4px 5px;
    max-width: 120px;
  }
  .hint {
    color: var(--muted);
    font-size: 11.5px;
    margin: 4px 0 0;
  }
  .error {
    color: #ff8a9a;
    font-size: 12px;
  }
  code {
    font-size: 11px;
  }
</style>
