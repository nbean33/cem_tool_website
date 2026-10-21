<script lang="ts">
  // Start a model: answer three questions (band, height, ground), then pick an antenna; or open
  // one of the example decks in a folder (4nec2's example models when 4nec2 is installed),
  // searchable by name and first comment line.
  import { onMount, untrack } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import * as api from "./api";
  import Help from "./Help.svelte";
  import { tip } from "./tip";
  import type { ModelFile, Units } from "./types";
  import { BANDS, bandOf, centre } from "./bands";
  import { SOILS, TEMPLATES, defaultHeight, type Template } from "./templates";
  import { fromUnit, toUnit, unitOf } from "./units";

  let {
    onopen,
    onnew,
    onclose,
    welcome = false,
    units = "metres",
    mhz: startMhz = null,
  }: {
    onopen: (path: string) => void;
    onnew: (deck: string) => void;
    onclose: () => void;
    /** First run: say hello and how this works. */
    welcome?: boolean;
    units?: Units;
    /** The frequency being looked at, to start from its band. */
    mhz?: number | null;
  } = $props();

  // ---- The three questions.
  // The dialog starts from the frequency and units in use when it opens.
  const startBand = untrack(() => bandOf(startMhz)) ?? BANDS.find((b) => b.name === "20 m")!;
  let bandName = $state(startBand.name);
  let mhz = $state(centre(startBand));
  let height = $state(Number(toUnit(defaultHeight(centre(startBand)), untrack(() => units)).toFixed(1)));
  let heightTouched = $state(false);
  /** Index into SOILS, or -1 for free space. */
  let soilIx = $state(0);

  function chooseBand(name: string) {
    const b = BANDS.find((x) => x.name === name);
    if (!b) return;
    bandName = name;
    mhz = centre(b);
    if (!heightTouched) height = Number(toUnit(defaultHeight(mhz), units).toFixed(1));
  }
  let mhzOk = $derived(Number.isFinite(mhz) && mhz > 0.01 && mhz < 3000);
  let heightOk = $derived(Number.isFinite(height) && height >= 0);
  let ready = $derived(mhzOk && (soilIx < 0 || heightOk));

  function make(t: Template) {
    if (!ready) return;
    onnew(
      t.deck({
        mhz: Number(mhz.toPrecision(6)),
        height: Math.max(0.5, fromUnit(height, units)),
        soil: soilIx < 0 ? null : SOILS[soilIx],
      }),
    );
  }

  // ---- Folder of decks.
  let root = $state<string | null>(null);
  let files = $state<ModelFile[]>([]);
  let query = $state("");
  let loading = $state(false);
  let error = $state<string | null>(null);

  async function load(dir: string) {
    root = dir;
    loading = true;
    error = null;
    try {
      files = await api.listModels(dir);
    } catch (e) {
      error = String(e);
      files = [];
    } finally {
      loading = false;
    }
  }

  async function chooseFolder() {
    const d = await open({ directory: true, multiple: false });
    if (typeof d === "string") await load(d);
  }

  let shown = $derived.by(() => {
    const words = query.toLowerCase().split(/\s+/).filter(Boolean);
    return files.filter((f) => words.every((w) => f.rel.toLowerCase().includes(w) || f.description.toLowerCase().includes(w))).slice(0, 400);
  });

  onMount(() => {
    api.modelsFolder().then((d) => {
      if (d) void load(d);
    });
  });
</script>

<div class="backdrop" role="presentation" onclick={onclose}>
  <div class="dialog" role="dialog" aria-modal="true" tabindex="-1" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.key === "Escape" && onclose()}>
    <header>
      <h2>{welcome ? "Welcome to NEC Studio" : "New antenna"}</h2>
      <button class="icon" onclick={onclose} use:tip={"Close (Esc)"}>✕</button>
    </header>
    {#if welcome}
      <div class="welcome">
        <p>NEC Studio shows how an antenna will work before you build it: its SWR across a band, its gain, and where it sends your signal.</p>
        <ol>
          <li><b>Answer three questions</b> and <b>pick an antenna</b> below.</li>
          <li><b>Adjust</b> its length, height or wire with the knobs on the left.</li>
          <li><b>Read the results</b> on the right. Hover over anything for an explanation; click a <span class="q">?</span> for more.</li>
        </ol>
      </div>
    {/if}
    <div class="cols">
      <section class="start">
        <div class="questions">
          <label>
            <span class="qn">1</span>
            <span class="what">Band<Help term="stepBand" /></span>
            <select value={bandName} onchange={(e) => chooseBand((e.currentTarget as HTMLSelectElement).value)}>
              {#each BANDS as b}<option value={b.name}>{b.name} ({b.lo}–{b.hi} MHz)</option>{/each}
            </select>
            <span class="mhz">
              <input type="number" step="any" min="0.01" bind:value={mhz} class:bad={!mhzOk} use:tip={"The exact frequency to design for, MHz"} /> MHz
            </span>
          </label>
          {#if !mhzOk}<p class="warn">Enter a frequency in MHz, such as 7.15.</p>{/if}
          <label>
            <span class="qn">2</span>
            <span class="what">Ground<Help term="groundKind" /></span>
            <select bind:value={soilIx}>
              {#each SOILS as s, i}<option value={i}>{s.name} ground</option>{/each}
              <option value={-1}>None (free space)</option>
            </select>
          </label>
          {#if soilIx >= 0}
            <label>
              <span class="qn">3</span>
              <span class="what">Height<Help term="height" /></span>
              <span class="mhz">
                <input type="number" step="any" min="0" bind:value={height} oninput={() => (heightTouched = true)} class:bad={!heightOk} />
                {unitOf(units).label} above the ground
              </span>
            </label>
            {#if !heightOk}<p class="warn">The height can't be negative.</p>{/if}
          {/if}
        </div>
        <h3>Pick an antenna</h3>
        <ul class="grid">
          {#each TEMPLATES as t (t.id)}
            <li>
              <button class="card" onclick={() => make(t)} disabled={!ready} use:tip={t.about}>
                <svg viewBox="0 0 40 24" aria-hidden="true"><path d={t.icon} /></svg>
                <b>{t.name}</b>
                <span>{t.goodFor}</span>
              </button>
            </li>
          {/each}
        </ul>
      </section>
      <section class="examples">
        <h3>Or open an example</h3>
        <div class="bar">
          <input type="search" placeholder="Search names and comments" bind:value={query} />
          <button class="small" onclick={chooseFolder} use:tip={"Choose a folder of .nec models to browse"}>Folder…</button>
        </div>
        <p class="hint">{root ? `${root} · ${files.length} models` : "Choose a folder of .nec models (4nec2's examples are found automatically)."}{loading ? " · reading…" : ""}</p>
        {#if error}<p class="err">{error}</p>{/if}
        <ul class="files">
          {#each shown as f (f.path)}
            <li>
              <button class="item" onclick={() => onopen(f.path)} title={f.path}>
                <b>{f.rel}</b><span>{f.description}</span>
              </button>
            </li>
          {/each}
        </ul>
      </section>
    </div>
  </div>
</div>

<style>
  .welcome {
    padding: 4px 2px 6px;
    font-size: 13px;
    line-height: 1.5;
    color: var(--text);
  }
  .welcome p {
    margin: 0 0 6px;
  }
  .welcome ol {
    margin: 0;
    padding-left: 20px;
  }
  .welcome .q {
    display: inline-block;
    width: 14px;
    height: 14px;
    border: 1px solid #3a4a62;
    border-radius: 50%;
    text-align: center;
    font-size: 10px;
    line-height: 12px;
    color: #8fa3bf;
  }
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(5, 8, 12, 0.6);
    display: grid;
    place-items: center;
    z-index: 50;
  }
  .dialog {
    width: min(1080px, 94vw);
    height: min(720px, 90vh);
    display: flex;
    flex-direction: column;
    background: var(--panel, #141b27);
    border: 1px solid var(--line);
    border-radius: 10px;
    padding: 14px 16px;
  }
  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  h2 {
    margin: 0;
    font-size: 16px;
  }
  .cols {
    display: grid;
    grid-template-columns: minmax(0, 1.5fr) minmax(0, 1fr);
    gap: 18px;
    min-height: 0;
    flex: 1;
    margin-top: 10px;
  }
  section {
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  h3 {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--muted);
    margin: 12px 0 8px;
  }
  .examples h3 {
    margin-top: 0;
  }
  .questions {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .questions label {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
  }
  .qn {
    display: inline-grid;
    place-items: center;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: var(--accent-dim);
    color: #fff;
    font-size: 11px;
    font-weight: 700;
  }
  .what {
    width: 74px;
    display: inline-flex;
    align-items: center;
  }
  .mhz {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--muted);
  }
  .mhz input {
    width: 90px;
  }
  input.bad {
    outline: 1px solid #ff5c73;
  }
  .warn {
    margin: 0 0 0 28px;
    font-size: 12px;
    color: #ffcf6a;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    overflow: auto;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(170px, 1fr));
    gap: 8px;
    padding-right: 4px;
  }
  .card {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
    width: 100%;
    height: 100%;
    text-align: left;
    border-radius: 8px;
    padding: 8px 10px;
    background: #172131;
  }
  .card:hover:not(:disabled) {
    border-color: var(--accent);
  }
  .card:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .card svg {
    width: 60px;
    height: 36px;
  }
  .card path {
    fill: none;
    stroke: var(--accent);
    stroke-width: 1.6;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .card b {
    font-size: 13px;
    font-weight: 600;
  }
  .card span {
    font-size: 11.5px;
    color: var(--muted);
    line-height: 1.35;
  }
  .item {
    display: flex;
    flex-direction: column;
    gap: 2px;
    width: 100%;
    text-align: left;
    background: none;
    border: 0;
    border-radius: 6px;
    padding: 6px 8px;
    color: var(--text);
    font: inherit;
    cursor: pointer;
  }
  .item:hover {
    background: rgba(255, 255, 255, 0.05);
  }
  .item b {
    font-size: 13px;
    font-weight: 500;
  }
  .item span {
    font-size: 11px;
    color: var(--muted);
  }
  .bar {
    display: flex;
    gap: 8px;
  }
  .bar input {
    flex: 1;
  }
  .hint {
    font-size: 11px;
    color: var(--muted);
    margin: 6px 0;
  }
  .err {
    color: #ff7a8c;
    font-size: 12px;
  }
  .files {
    flex: 1;
  }
</style>
