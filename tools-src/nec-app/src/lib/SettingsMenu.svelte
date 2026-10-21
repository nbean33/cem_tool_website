<script lang="ts">
  // The ⚙ Settings popover: what an experienced user changes and a beginner can leave alone.
  import Help from "./Help.svelte";
  import { tip } from "./tip";
  import { hint } from "./help";
  import { UNITS } from "./units";
  import type { Units } from "./types";

  let {
    z0 = $bindable(50),
    units = $bindable("metres"),
    sweepSteps = $bindable(41),
    autoseg = $bindable(true),
    segsPerHalfWave = $bindable(20),
    stepRadius = $bindable(false),
    onchange,
  }: {
    z0?: number;
    units?: Units;
    sweepSteps?: number;
    autoseg?: boolean;
    segsPerHalfWave?: number;
    stepRadius?: boolean;
    /** Something that changes the solution changed. */
    onchange: () => void;
  } = $props();

  let open = $state(false);
  let host: HTMLDivElement;
  function outside(e: MouseEvent) {
    if (open && host && !host.contains(e.target as Node)) open = false;
  }
  const clamp = (v: number, lo: number, hi: number) => (Number.isFinite(v) ? Math.min(hi, Math.max(lo, v)) : lo);
</script>

<svelte:window onmousedown={outside} onkeydown={(e) => e.key === "Escape" && (open = false)} />

<div class="settings" bind:this={host}>
  <button use:tip={hint("settings")} aria-expanded={open} onclick={() => (open = !open)}>⚙ Settings</button>
  {#if open}
    <div class="menu" role="dialog" aria-label="Settings">
      <label>
        <span>Feed impedance Z₀<Help term="z0" /></span>
        <span><input type="number" min="1" step="any" bind:value={z0} onchange={() => ((z0 = clamp(z0, 1, 10000)), onchange())} /> Ω</span>
      </label>
      <label>
        <span>Length units<Help term="units" /></span>
        <select bind:value={units}>
          {#each UNITS as u}<option value={u.id}>{u.label}</option>{/each}
        </select>
      </label>
      <label>
        <span>Sweep steps<Help term="sweepSteps" /></span>
        <input type="number" min="2" max="1001" step="1" bind:value={sweepSteps} onchange={() => (sweepSteps = Math.round(clamp(sweepSteps, 2, 1001)))} />
      </label>
      <hr />
      <label class="check">
        <span><input type="checkbox" bind:checked={autoseg} onchange={onchange} /> Auto-segmentation<Help term="autoseg" /></span>
      </label>
      {#if autoseg}
        <label class="sub">
          <span>Segments per half wave<Help term="segsPerHalfWave" /></span>
          <input
            type="number"
            min="10"
            max="500"
            step="1"
            bind:value={segsPerHalfWave}
            onchange={() => ((segsPerHalfWave = Math.round(clamp(segsPerHalfWave, 10, 500))), onchange())}
          />
        </label>
      {/if}
      <label class="check">
        <span><input type="checkbox" bind:checked={stepRadius} onchange={onchange} /> Stepped-radius correction<Help term="stepRadius" /></span>
      </label>
    </div>
  {/if}
</div>

<style>
  .settings {
    position: relative;
  }
  .menu {
    position: absolute;
    right: 0;
    top: calc(100% + 6px);
    z-index: 50;
    width: 300px;
    background: var(--panel);
    border: 1px solid #34445c;
    border-radius: 8px;
    padding: 10px 12px;
    box-shadow: 0 8px 28px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  label {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
    font-size: 12px;
  }
  label.sub {
    padding-left: 22px;
  }
  label.check {
    justify-content: flex-start;
  }
  input[type="number"] {
    width: 70px;
  }
  hr {
    border: none;
    border-top: 1px solid var(--line);
    margin: 2px 0;
    width: 100%;
  }
</style>
