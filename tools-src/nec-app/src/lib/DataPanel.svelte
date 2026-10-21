<script lang="ts">
  import Help from "./Help.svelte";
  import { tip } from "./tip";
  import { hint } from "./help";
  // The solution as numbers: current and charge tables, exports (CSV and a NEC-style report),
  // and the near field on a slice of space shown in the 3-D view.
  import { save } from "@tauri-apps/plugin-dialog";
  import * as api from "./api";
  import type { Analysis, Metrics, ModelOptions, Slice, SlicePlane, SliceRequest, SolveView, Units } from "./types";
  import { fromUnit, length, toUnit, unitOf } from "./units";

  let {
    analysis,
    result,
    text,
    path,
    units,
    swept,
    model,
    highlight = $bindable(null),
    nearField = $bindable(null),
  }: {
    analysis: Analysis | null;
    result: SolveView | null;
    text: string;
    path: string | null;
    units: Units;
    swept: Metrics[];
    model: ModelOptions;
    highlight?: number | null;
    nearField?: { request: SliceRequest; slice: Slice } | null;
  } = $props();

  let status = $state<string | null>(null);
  const base = $derived((path?.split(/[\\/]/).pop() ?? "untitled").replace(/\.nec\.toml$|\.toml$|\.nec$/i, ""));

  // ---- Exports.
  async function saveAs(name: string, ext: string, kind: string, body: () => Promise<string> | string) {
    status = null;
    const p = await save({ defaultPath: `${base}-${name}.${ext}`, filters: [{ name: kind, extensions: [ext] }] });
    if (!p) return;
    try {
      await api.writeText(p, await body());
      status = `Wrote ${p}`;
    } catch (e) {
      status = String(e);
    }
  }
  const csv = (rows: (string | number | null)[][]) =>
    rows.map((r) => r.map((v) => (v == null ? "" : typeof v === "number" ? String(v) : `"${v.replace(/"/g, '""')}"`)).join(",")).join("\n") + "\n";

  const exportReport = () =>
    saveAs("report", "txt", "Text report", () => api.report(text, result?.mhz ?? null, path, model));

  function currentsCsv() {
    if (!result || !analysis) return "";
    const u = unitOf(units).label;
    const rows: (string | number | null)[][] = [
      ["segment", "tag", `x (${u})`, `y (${u})`, `z (${u})`, `length (${u})`, "I re (A)", "I im (A)", "|I| (A)", "phase (deg)", "q re (C/m)", "q im (C/m)"],
    ];
    analysis.segments.forEach((s, i) => {
      const c = result.currents_complex[i];
      const q = result.charges[i] ?? [0, 0];
      const mid = [0, 1, 2].map((k) => toUnit((s.a[k] + s.b[k]) / 2, units));
      const len = toUnit(Math.hypot(...[0, 1, 2].map((k) => s.b[k] - s.a[k])), units);
      rows.push([i + 1, s.tag, ...mid, len, c[0], c[1], Math.hypot(c[0], c[1]), (Math.atan2(c[1], c[0]) * 180) / Math.PI, q[0], q[1]]);
    });
    return csv(rows);
  }

  function patternCsv() {
    const p = result?.pattern;
    if (!p) return "";
    const rows: (string | number | null)[][] = [["theta (deg)", "phi (deg)", "elevation (deg)", "gain (dBi)"]];
    p.phis.forEach((ph, j) => p.thetas.forEach((th, i) => rows.push([th, ph, 90 - th, p.gain[j]?.[i] ?? null])));
    return csv(rows);
  }

  function sweepCsv() {
    const rows: (string | number | null)[][] = [["MHz", "R (ohm)", "X (ohm)", "SWR", "gain (dBi)", "F/B (dB)", "F/R (dB)", "elevation (deg)", "efficiency (%)"]];
    for (const m of swept) rows.push([m.mhz, m.z[0], m.z[1], m.swr, m.gain, m.front_to_back, m.front_to_rear, m.elevation, m.efficiency]);
    return csv(rows);
  }

  // ---- Near field.
  let plane = $state<SlicePlane>("xy");
  let offsetText = $state("");
  let magnetic = $state(false);
  let resolution = $state(61);
  let computing = $state(false);

  /** The structure's bounding box, padded by a third (and at least a tenth of a wavelength). */
  let box = $derived.by(() => {
    if (!analysis?.segments.length && !analysis?.patches.length) return null;
    const lo = [Infinity, Infinity, Infinity];
    const hi = [-Infinity, -Infinity, -Infinity];
    const add = (p: number[]) => p.forEach((v, k) => ((lo[k] = Math.min(lo[k], v)), (hi[k] = Math.max(hi[k], v))));
    analysis!.segments.filter((s) => !s.auxiliary).forEach((s) => (add(s.a), add(s.b)));
    analysis!.patches.forEach((p) => p.outline.forEach(add));
    const wl = 299.792458 / (result?.mhz ?? analysis!.frequencies_mhz[0] ?? 300);
    const size = Math.max(...hi.map((h, k) => h - lo[k]));
    const pad = Math.max(size / 3, wl / 10);
    return { lo: lo.map((v) => v - pad), hi: hi.map((v) => v + pad), centre: lo.map((v, k) => (v + hi[k]) / 2) };
  });
  const axes: Record<SlicePlane, [number, number, number]> = { xy: [0, 1, 2], xz: [0, 2, 1], yz: [1, 2, 0] };

  async function showNearField() {
    if (!box || !result) return;
    const [ui, vi, ni] = axes[plane];
    const offset = offsetText.trim() === "" ? box.centre[ni] : fromUnit(Number(offsetText), units);
    if (!Number.isFinite(offset)) return;
    const request: SliceRequest = {
      plane,
      offset,
      u: [box.lo[ui], box.hi[ui]],
      v: [box.lo[vi], box.hi[vi]],
      n_u: resolution,
      n_v: resolution,
      magnetic,
    };
    computing = true;
    status = null;
    const t0 = performance.now();
    try {
      const slice = await api.nearField(text, result.mhz, request, path, model);
      nearField = { request, slice };
      status = `${resolution * resolution} points in ${((performance.now() - t0) / 1000).toFixed(1)} s`;
    } catch (e) {
      status = String(e);
    } finally {
      computing = false;
    }
  }

  // ---- Current table, a page at a time.
  const PAGE = 100;
  let page = $state(0);
  let n = $derived(result?.currents.length ?? 0);
  let pages = $derived(Math.max(1, Math.ceil(n / PAGE)));
  $effect(() => {
    if (page >= pages) page = 0;
  });
  let peak = $derived(Math.max(1e-30, ...(result?.currents ?? [0])));
  const deg = (c: [number, number]) => ((Math.atan2(c[1], c[0]) * 180) / Math.PI).toFixed(1);
  const sci = (v: number) => (v === 0 ? "0" : v.toExponential(3));
</script>

<section class="card">
  <h3>Export</h3>
  <div class="row">
    <button class="small" onclick={exportReport} disabled={!result} use:tip={hint("exportReport")}>Report…</button>
    <button class="small" onclick={() => saveAs("currents", "csv", "CSV", currentsCsv)} disabled={!result} use:tip={hint("exportCsv")}>Currents CSV</button>
    <button class="small" onclick={() => saveAs("pattern", "csv", "CSV", patternCsv)} disabled={!result?.pattern} use:tip={hint("exportCsv")}>Pattern CSV</button>
    <button class="small" onclick={() => saveAs("sweep", "csv", "CSV", sweepCsv)} disabled={!swept.length} use:tip={hint("exportCsv")}>Sweep CSV</button>
  </div>
</section>

<section class="card">
  <h3>Near field<Help term="nearField" /></h3>
  <div class="row">
    <select bind:value={plane} use:tip={hint("nearPlane")}>
      <option value="xy">Horizontal (XY)</option>
      <option value="xz">Vertical (XZ)</option>
      <option value="yz">Vertical (YZ)</option>
    </select>
    <label use:tip={hint("nearAt")}>at {plane === "xy" ? "z" : plane === "xz" ? "y" : "x"} =
      <input type="text" class="num" placeholder="centre" bind:value={offsetText} /> {unitOf(units).label}</label>
  </div>
  <div class="row">
    <label use:tip={hint("nearKind")}><input type="radio" bind:group={magnetic} value={false} /> E</label>
    <label use:tip={hint("nearKind")}><input type="radio" bind:group={magnetic} value={true} /> H</label>
    <select bind:value={resolution} use:tip={hint("nearResolution")}>
      <option value={31}>31 × 31</option>
      <option value={61}>61 × 61</option>
      <option value={101}>101 × 101</option>
    </select>
    <button class="small primary" onclick={showNearField} disabled={!result || computing}>{computing ? "Computing…" : "Show"}</button>
    {#if nearField}<button class="small" onclick={() => (nearField = null)}>Hide</button>{/if}
  </div>
  {#if nearField}
    <p class="hint">
      Peak |{nearField.request.magnetic ? "H" : "E"}| {sci(nearField.slice.max)} {nearField.request.magnetic ? "A/m" : "V/m"};
      colours span 40 dB below it. Plane at {length(nearField.request.offset, units, 3)}.
    </p>
  {:else}
    <p class="hint">Peak field magnitude on a plane through the structure, at the solved frequency and drive.</p>
  {/if}
</section>

<section class="card">
  <div class="head">
    <h3>Currents and charges<Help term="currentsTable" /></h3>
    {#if pages > 1}
      <span class="pager">
        <button class="icon" onclick={() => (page = Math.max(0, page - 1))} disabled={page === 0}>‹</button>
        {page * PAGE + 1}–{Math.min(n, (page + 1) * PAGE)} of {n}
        <button class="icon" onclick={() => (page = Math.min(pages - 1, page + 1))} disabled={page >= pages - 1}>›</button>
      </span>
    {/if}
  </div>
  {#if result && analysis}
    <table>
      <thead><tr><th>Seg</th><th>Tag</th><th>|I| mA</th><th>Phase°</th><th>|q| C/m</th><th></th></tr></thead>
      <tbody>
        {#each result.currents_complex.slice(page * PAGE, (page + 1) * PAGE) as c, k (page * PAGE + k)}
          {@const i = page * PAGE + k}
          {@const q = result.charges[i] ?? [0, 0]}
          <tr class:on={highlight === i + 1} onclick={() => (highlight = highlight === i + 1 ? null : i + 1)}>
            <td>{i + 1}</td>
            <td>{analysis.segments[i]?.tag ?? ""}</td>
            <td class="num">{(Math.hypot(c[0], c[1]) * 1e3).toFixed(3)}</td>
            <td class="num">{deg(c)}</td>
            <td class="num">{sci(Math.hypot(q[0], q[1]))}</td>
            <td><span class="bar" style:width="{(48 * Math.hypot(c[0], c[1])) / peak}px"></span></td>
          </tr>
        {/each}
      </tbody>
    </table>
  {:else}
    <p class="hint">No solution yet.</p>
  {/if}
</section>
{#if status}<p class="hint">{status}</p>{/if}

<style>
  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
    margin-bottom: 6px;
    font-size: 12px;
  }
  .num {
    font-family: ui-monospace, monospace;
    text-align: right;
  }
  input.num {
    width: 70px;
  }
  .head {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .pager {
    font-size: 12px;
    color: var(--muted);
  }
  table {
    width: 100%;
    font-size: 12px;
    border-collapse: collapse;
  }
  th {
    text-align: left;
    color: var(--muted);
    font-weight: 500;
  }
  tr {
    cursor: pointer;
  }
  tr.on {
    background: rgba(255, 51, 230, 0.15);
  }
  tbody tr:hover {
    background: rgba(255, 255, 255, 0.04);
  }
  .bar {
    display: inline-block;
    height: 6px;
    border-radius: 3px;
    background: #ffae45;
  }
  .hint {
    font-size: 12px;
    color: var(--muted);
    margin: 6px 0 0;
  }
</style>
