<script lang="ts">
  import Help from "./Help.svelte";
  import { tip } from "./tip";
  import { hint } from "./help";
  // Matching networks between Z0 and the antenna: L, Pi and T networks of lossy components, and
  // stub or series-section line matches. A chosen network is written into the deck (a dummy
  // source wire joined to the feed by an NT card) and the model re-solved; nec-rs recomputes the
  // network at every frequency, so a sweep shows the matched bandwidth.
  import * as api from "./api";
  import type { MatchDesign, MatchInDeck, MatchRequest, Stage } from "./api";
  import type { Analysis, SolveView, Units } from "./types";
  import { LINES } from "./tables";
  import { length } from "./units";

  let {
    text,
    analysis,
    result,
    z0,
    units,
    onedit,
  }: {
    text: string;
    analysis: Analysis | null;
    result: SolveView | null;
    z0: number;
    units: Units;
    onedit: (text: string) => void;
  } = $props();

  type Kind = MatchRequest["kind"];
  let kind = $state<Kind>("l");
  let qL = $state(250);
  let qC = $state(0);
  let qNet = $state(0);
  let lineIx = $state(LINES.findIndex((l) => l.name === "RG-213"));
  let stubIx = $state(LINES.findIndex((l) => l.name === "RG-213"));
  let sectionIx = $state(LINES.findIndex((l) => l.name === "Open wire 450 Ω"));
  let short = $state(true);
  let series = $state(false);
  let designs = $state<MatchDesign[]>([]);
  let existing = $state<MatchInDeck[]>([]);
  let error = $state<string | null>(null);

  $effect(() => {
    const t = text;
    api.matchingNetworks(t).then(
      (m) => {
        if (t === text) existing = m;
      },
      () => {},
    );
  });

  let source = $derived(result?.inputs[0] ?? null);
  /** The feed as (tag, segment within the tag). */
  let feed = $derived.by(() => {
    if (!source || !analysis) return null;
    const within = analysis.segments.slice(0, source.segment).filter((s) => s.tag === source.tag).length;
    return { tag: source.tag, segment: within };
  });
  let zAnt = $derived<[number, number] | null>(existing.length ? (result?.antenna_z ?? null) : (source?.z ?? null));

  function request(): MatchRequest {
    const ln = (i: number) => ({ z0: LINES[i].z0, vf: LINES[i].vf });
    switch (kind) {
      case "l":
        return { kind, q_l: qL, q_c: qC };
      case "pi":
      case "tee":
        return { kind, q_net: qNet, q_l: qL, q_c: qC };
      case "stub":
        return { kind, line: ln(lineIx), stub_line: ln(stubIx), short, series };
      case "section":
        return { kind, line: ln(lineIx), section: ln(sectionIx) };
    }
  }

  async function design() {
    error = null;
    if (!zAnt || !result) return;
    try {
      designs = await api.designMatching(zAnt, z0, result.mhz, request());
      if (!designs.length) error = "No network of this kind matches this load (try another kind or line).";
    } catch (e) {
      error = String(e);
    }
  }

  async function apply(d: MatchDesign) {
    if (!feed || !result) return;
    error = null;
    try {
      onedit(await api.applyMatching(text, feed.tag, feed.segment, d.network, result.mhz));
      designs = [];
    } catch (e) {
      error = String(e);
    }
  }

  async function remove(m: MatchInDeck) {
    error = null;
    try {
      onedit(await api.removeMatching(text, m.dummy_tag));
    } catch (e) {
      error = String(e);
    }
  }

  const eng = (v: number, unit: string) => {
    const p = [
      [1e-12, "p"],
      [1e-9, "n"],
      [1e-6, "µ"],
      [1e-3, "m"],
      [1, ""],
    ] as const;
    const [s, pre] = [...p].reverse().find(([s]) => Math.abs(v) >= s) ?? p[0];
    return `${(v / s).toPrecision(4)} ${pre}${unit}`;
  };
  function stageText(s: Stage): string {
    switch (s.stage) {
      case "series":
      case "shunt": {
        const c = s.part;
        const value = c.kind === "inductor" ? eng(c.henry, "H") : eng(c.farad, "F");
        const q = c.q > 0 ? ` (Q ${c.q})` : "";
        return `${s.stage} ${c.kind === "inductor" ? "L" : "C"} ${value}${q}`;
      }
      case "line":
        return `line ${s.line.z0} Ω × ${length(s.length_m, units, 3)}`;
      case "stub":
        return `${s.series ? "series" : "parallel"} ${s.short ? "shorted" : "open"} stub ${s.line.z0} Ω × ${length(s.length_m, units, 3)}`;
    }
  }
  const z = (v: [number, number]) => `${v[0].toFixed(2)} ${v[1] < 0 ? "−" : "+"} j${Math.abs(v[1]).toFixed(2)} Ω`;
</script>

<section class="card">
  <h3>Matching network<Help term="matching" /></h3>
  {#each existing as m}
    <div class="present">
      <div><b>{m.network.label}</b> at tag {m.feed_tag} seg {m.feed_segment}</div>
      <ol>
        {#each m.network.stages as s}<li>{stageText(s)}</li>{/each}
      </ol>
      {#if result?.antenna_z}<div class="hint">Antenna {z(result.antenna_z)} → source sees {z(result.inputs[0].z)}</div>{/if}
      <button class="small warn" onclick={() => remove(m)} use:tip={hint("matchRemove")}>Remove</button>
    </div>
  {/each}
  {#if !existing.length}
    {#if zAnt && result}
      <p class="hint">Antenna {z(zAnt)} at {result.mhz.toFixed(4)} MHz, to {z0} Ω.</p>
    {:else}
      <p class="hint">Solve a deck with a voltage source first.</p>
    {/if}
    <div class="row">
      <select bind:value={kind} use:tip={hint("matchType")}>
        <option value="l">L network</option>
        <option value="pi">Pi network</option>
        <option value="tee">T network</option>
        <option value="stub">Stub</option>
        <option value="section">Series section</option>
      </select>
      {#if kind === "l" || kind === "pi" || kind === "tee"}
        <label use:tip={hint("qCoil")}>Q coil <input type="number" class="n" min="0" bind:value={qL} /></label>
        <label use:tip={hint("qCap")}>Q cap <input type="number" class="n" min="0" bind:value={qC} /></label>
        {#if kind !== "l"}<label use:tip={hint("qNet")}>network Q <input type="number" class="n" min="0" bind:value={qNet} /></label>{/if}
      {:else}
        <label use:tip={hint("feedLine")}>feed <select bind:value={lineIx}>{#each LINES as l, i}<option value={i}>{l.name}</option>{/each}</select></label>
        {#if kind === "stub"}
          <label use:tip={hint("stubLine")}>stub <select bind:value={stubIx}>{#each LINES as l, i}<option value={i}>{l.name}</option>{/each}</select></label>
          <label use:tip={hint("stubShorted")}><input type="checkbox" bind:checked={short} /> shorted</label>
          <label use:tip={hint("stubSeries")}><input type="checkbox" bind:checked={series} /> series</label>
        {:else}
          <label use:tip={hint("sectionLine")}>section <select bind:value={sectionIx}>{#each LINES as l, i}<option value={i}>{l.name}</option>{/each}</select></label>
        {/if}
      {/if}
      <button class="small primary" onclick={design} disabled={!zAnt} use:tip={hint("matchDesign")}>Design</button>
    </div>
    {#each designs as d}
      <div class="design">
        <div class="head"><b>{d.network.label}</b><button class="small" onclick={() => apply(d)} use:tip={hint("matchApply")}>Apply</button></div>
        <ol>
          {#each d.network.stages as s}<li>{stageText(s)}</li>{/each}
        </ol>
        <div class="hint">Source sees {z(d.z_in)}, SWR {d.swr.toFixed(2)}, loss {(d.loss * 100).toFixed(2)} %</div>
      </div>
    {/each}
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
    width: 56px;
  }
  .design,
  .present {
    border-top: 1px solid var(--line);
    padding: 6px 0;
    font-size: 12px;
  }
  .head {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  ol {
    margin: 4px 0;
    padding-left: 18px;
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
