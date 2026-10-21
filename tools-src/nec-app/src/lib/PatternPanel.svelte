<script lang="ts">
  import { tip } from "./tip";
  import { hint } from "./help";
  // The two polar cuts with 4nec2's pattern tools: a choice of component (total, vertical and
  // horizontal, right- and left-hand circular), a readout of beamwidth, lobes, front-to-back and
  // front-to-rear, up to five pinned patterns to compare, and play-back of a sweep's patterns.
  import Polar, { type Overlay } from "./Polar.svelte";
  import type { Component, Cut, CutStats, Frame, PatternView } from "./types";

  let {
    pattern,
    mhz,
    name,
    half,
    frames,
    component = $bindable("gain"),
    split = $bindable(false),
    playing = $bindable(null),
  }: {
    pattern: PatternView | null;
    mhz: number | null;
    /** The model's name, for pinned patterns' labels. */
    name: string;
    half: boolean;
    /** The last sweep's patterns, by frequency. */
    frames: { mhz: number; frame: Frame }[];
    component?: Component;
    split?: boolean;
    /** The frame being played back, or null for the solution. */
    playing?: number | null;
  } = $props();

  const COMPONENTS: { id: Component | "split"; label: string }[] = [
    { id: "gain", label: "Total" },
    { id: "split", label: "Total, V and H" },
    { id: "vertical", label: "Vertical" },
    { id: "horizontal", label: "Horizontal" },
    { id: "rhcp", label: "RHCP" },
    { id: "lhcp", label: "LHCP" },
  ];
  const COLORS = ["#58b7ff", "#6fd39a", "#ff7a8c", "#c9a0ff", "#e8e36a"];
  const MAX_PINNED = 5;

  let choice = $state<Component | "split">("gain");
  $effect(() => {
    component = choice === "split" ? "gain" : choice;
    split = choice === "split";
  });

  let frame = $derived(playing != null ? frames[playing] ?? null : null);
  let azimuth = $derived<Cut | null>(frame?.frame.azimuth ?? pattern?.azimuth_cut ?? null);
  let elevation = $derived<Cut | null>(frame?.frame.elevation ?? pattern?.elevation_cut ?? null);
  let shownMhz = $derived(frame?.mhz ?? mhz);

  // Pinned patterns.
  interface Pinned {
    label: string;
    color: string;
    azimuth: Cut;
    elevation: Cut;
  }
  let pinned = $state.raw<Pinned[]>([]);
  function pin() {
    if (!azimuth || !elevation || shownMhz == null) return;
    const used = new Set(pinned.map((p) => p.color));
    const keep = pinned.slice(-(MAX_PINNED - 1));
    const color = COLORS.find((c) => !keep.some((p) => p.color === c)) ?? COLORS[used.size % COLORS.length];
    pinned = [...keep, { label: `${name} · ${shownMhz.toFixed(3)} MHz`, color, azimuth, elevation }];
  }
  const unpin = (p: Pinned) => (pinned = pinned.filter((q) => q !== p));
  let azOverlays = $derived<Overlay[]>(pinned.map((p) => ({ cut: p.azimuth, color: p.color, label: p.label })));
  let elOverlays = $derived<Overlay[]>(pinned.map((p) => ({ cut: p.elevation, color: p.color, label: p.label })));

  // Play-back.
  let timer = $state<ReturnType<typeof setInterval> | null>(null);
  function play() {
    if (timer) {
      clearInterval(timer);
      timer = null;
      return;
    }
    if (!frames.length) return;
    if (playing == null || playing >= frames.length - 1) playing = 0;
    timer = setInterval(() => {
      if (playing == null || playing >= frames.length - 1) {
        clearInterval(timer!);
        timer = null;
        return;
      }
      playing += 1;
    }, 350);
  }
  function stop() {
    if (timer) clearInterval(timer);
    timer = null;
    playing = null;
  }
  // A new sweep (or none) ends play-back.
  $effect(() => {
    if (playing != null && playing >= frames.length) stop();
  });
  $effect(() => () => {
    if (timer) clearInterval(timer);
  });

  const statsOf = (c: Cut | null): CutStats | null => (c ? c.stats[component === "gain" ? "total" : component] : null);
  const deg = (v: number | null | undefined, digits = 1) => (v == null || !Number.isFinite(v) ? "–" : `${v.toFixed(digits)}°`);
  const db = (v: number | null | undefined, digits = 1) => (v == null || !Number.isFinite(v) ? "–" : `${v.toFixed(digits)} dB`);
  let rows = $derived(
    [
      { name: "Azimuth", s: statsOf(azimuth) },
      { name: "Elevation", s: statsOf(elevation) },
    ],
  );
</script>

<div class="patterns">
  <div class="cardhead">
    <label>
      <select bind:value={choice} use:tip={hint("patternComponent")}>
        {#each COMPONENTS as c}<option value={c.id}>{c.label}</option>{/each}
      </select>
    </label>
    <button onclick={pin} disabled={!azimuth} use:tip={hint("pinPattern")}>Pin</button>
    {#if frames.length > 1}
      <button onclick={play} title="Play the last sweep's patterns back, frequency by frequency (needs gain & F/B ticked for the sweep)">{timer ? "⏸" : "▶"}</button>
      <input
        type="range"
        min="0"
        max={frames.length - 1}
        value={playing ?? 0}
        oninput={(e) => (playing = +(e.currentTarget as HTMLInputElement).value)}
        title="Step through the sweep's frequencies (← and → keys once focused)"
      />
      {#if playing != null}<button onclick={stop} title="Back to the solution">✕</button>{/if}
    {/if}
  </div>
  {#if frame}
    <p class="playing">Sweep play-back: {frame.mhz.toFixed(4)} MHz</p>
  {/if}
  <div class="two">
    <Polar
      title={azimuth ? `Azimuth at ${azimuth.at.toFixed(0)}° elevation` : "Azimuth"}
      cut={azimuth}
      {component}
      {split}
      overlays={azOverlays}
    />
    <Polar
      title={elevation ? `Elevation toward ${elevation.at.toFixed(0)}° azimuth` : "Elevation"}
      cut={elevation}
      {half}
      {component}
      {split}
      overlays={elOverlays}
    />
  </div>
  {#if pinned.length}
    <div class="legend">
      {#each pinned as p}
        <span style:color={p.color}>— {p.label} <button class="x" onclick={() => unpin(p)} title="Unpin">×</button></span>
      {/each}
    </div>
  {/if}
  <table>
    <thead>
      <tr>
        <th></th>
        <th>Peak</th>
        <th title="Between the half-power (−3 dB) points">Beamwidth</th>
        <th>F/B</th>
        <th title="Front against the strongest gain within 90° of the back">F/R</th>
        <th>Lobes</th>
        <th title="The strongest lobe after the main one">Side lobe</th>
      </tr>
    </thead>
    <tbody>
      {#each rows as r}
        <tr>
          <th>{r.name}</th>
          {#if r.s}
            <td>{r.s.peak.toFixed(2)} dBi <small>at {deg(r.s.peak_angle, 0)}</small></td>
            <td>{deg(r.s.beamwidth)}</td>
            <td>{db(r.s.front_to_back)}</td>
            <td>{db(r.s.front_to_rear)}</td>
            <td>{r.s.lobes.length}</td>
            <td>{db(r.s.side_lobe)}</td>
          {:else}
            <td colspan="6">–</td>
          {/if}
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<style>
  .two {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }
  .cardhead {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 6px;
  }
  .cardhead input[type="range"] {
    flex: 1;
    min-width: 60px;
  }
  .playing {
    margin: 0 0 4px;
    font-size: 12px;
    color: #ffcf6a;
  }
  .legend {
    display: flex;
    flex-wrap: wrap;
    gap: 4px 12px;
    font-size: 11px;
    margin: 4px 0;
  }
  .x {
    all: unset;
    cursor: pointer;
    color: var(--muted);
    padding: 0 3px;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 11px;
    margin-top: 6px;
  }
  th,
  td {
    text-align: right;
    padding: 2px 4px;
    white-space: nowrap;
  }
  th:first-child {
    text-align: left;
  }
  thead th {
    color: var(--muted);
    font-weight: 500;
  }
  small {
    color: var(--muted);
  }
</style>
