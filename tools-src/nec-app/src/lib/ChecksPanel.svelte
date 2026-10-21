<script lang="ts">
  import Help from "./Help.svelte";
  import { tip } from "./tip";
  import { hint } from "./help";
  // One-click model checks: the average-gain test and the convergence test.
  import { onMount } from "svelte";
  import * as api from "./api";
  import type { AverageGainTest, ConvergenceStep } from "./api";
  import type { SolveView } from "./types";
  import LineChart, { type Series } from "./LineChart.svelte";

  let {
    text,
    path,
    result,
    z0,
    azimuth,
  }: {
    text: string;
    path: string | null;
    result: SolveView | null;
    z0: number;
    azimuth: number;
  } = $props();

  let agt = $state<AverageGainTest | null>(null);
  let agtBusy = $state(false);
  let steps = $state(5);
  let growth = $state(30);
  let conv = $state<ConvergenceStep[]>([]);
  let convBusy = $state(false);
  let error = $state<string | null>(null);

  async function runAgt() {
    error = null;
    agtBusy = true;
    try {
      agt = await api.averageGainTest(text, result?.mhz ?? null, path);
    } catch (e) {
      error = String(e);
    } finally {
      agtBusy = false;
    }
  }

  async function runConv() {
    if (!result) return;
    error = null;
    conv = [];
    convBusy = true;
    try {
      conv = await api.convergenceTest(text, result.mhz, Math.max(2, Math.round(steps)), growth / 100, {
        z0,
        pattern: { azimuth, elevation: null },
      });
    } catch (e) {
      if (!String(e).includes("cancelled")) error = String(e);
    } finally {
      convBusy = false;
    }
  }

  onMount(() => {
    const off = api.onConvergenceStep((s) => {
      if (convBusy) conv = [...conv, s];
    });
    return () => void off.then((f) => f());
  });

  let x = $derived(conv.map((s) => s.segments));
  let series = $derived<Series[]>([
    { label: "R Ω", values: conv.map((s) => s.metrics.z[0]), color: "#58b7ff", digits: 2 },
    { label: "X Ω", values: conv.map((s) => s.metrics.z[1]), color: "#b07cff", dash: true, digits: 2 },
    { label: "Gain dBi", values: conv.map((s) => s.metrics.gain), color: "#6fd39a", axis: "right", digits: 2 },
  ]);
  /** The last step's change against the one before, as a fraction of |Z|. */
  let settled = $derived.by(() => {
    if (conv.length < 2) return null;
    const [a, b] = conv.slice(-2).map((s) => s.metrics.z);
    return Math.hypot(b[0] - a[0], b[1] - a[1]) / Math.hypot(b[0], b[1]);
  });
</script>

<section class="card">
  <h3>Model accuracy tests</h3>
  <div class="row">
    <button class="small primary" onclick={runAgt} disabled={!result || agtBusy} use:tip={hint("agt")}>{agtBusy ? "Testing…" : "Average-gain test"}</button>
    <Help term="agt" />
    {#if agt}
      <span class="grade {agt.grade}">{agt.grade}</span>
      <span>ratio {agt.ratio.toFixed(4)} ({agt.correction_db >= 0 ? "+" : ""}{agt.correction_db.toFixed(2)} dB){agt.over_ground ? " · over perfect ground" : ""}</span>
    {/if}
  </div>
  <p class="hint">Losses removed and ground made perfect, the average gain should be exactly 1: 0.95–1.05 accurate, 0.90–1.10 usable, 0.80–1.20 improvable.</p>

  <div class="row">
    <label use:tip={hint("convSteps")}>steps <input type="number" class="n" bind:value={steps} min="2" max="12" /></label>
    <label use:tip={hint("convGrowth")}>+<input type="number" class="n" bind:value={growth} min="5" max="100" /> % per step</label>
    {#if convBusy}
      <button class="small warn" onclick={() => api.cancel("convergence")}>Stop</button>
    {:else}
      <button class="small primary" onclick={runConv} disabled={!result} use:tip={hint("convergence")}>Convergence test</button>
      <Help term="convergence" />
    {/if}
  </div>
  {#if conv.length}
    <LineChart {x} {series} height={150} xLabel="segments" />
    <table>
      <thead><tr><th>Segments</th><th>R Ω</th><th>X Ω</th><th>Gain dBi</th><th>Eff. %</th></tr></thead>
      <tbody>
        {#each conv as s}
          <tr>
            <td>{s.segments}</td><td>{s.metrics.z[0].toFixed(2)}</td><td>{s.metrics.z[1].toFixed(2)}</td>
            <td>{s.metrics.gain?.toFixed(2) ?? "–"}</td><td>{s.metrics.efficiency.toFixed(1)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
    {#if settled != null && !convBusy}
      <p class="hint">The last refinement changed the impedance by {(settled * 100).toFixed(2)} %{settled < 0.01 ? " — converged." : " — not yet settled; refine the model."}</p>
    {/if}
  {/if}
  {#if error}<p class="err">{error}</p>{/if}
</section>

<style>
  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
    font-size: 12px;
    margin: 6px 0;
  }
  input.n {
    width: 52px;
  }
  .grade {
    font-weight: 600;
    text-transform: capitalize;
  }
  .grade.accurate {
    color: #6fd39a;
  }
  .grade.usable {
    color: #ffae45;
  }
  .grade.improvable,
  .grade.questionable {
    color: #ff7a8c;
  }
  table {
    width: 100%;
    font-size: 12px;
    border-collapse: collapse;
    margin-top: 6px;
  }
  th {
    text-align: left;
    color: var(--muted);
    font-weight: 500;
  }
  .hint {
    font-size: 12px;
    color: var(--muted);
    margin: 4px 0;
  }
  .err {
    color: #ff7a8c;
    font-size: 12px;
  }
</style>
