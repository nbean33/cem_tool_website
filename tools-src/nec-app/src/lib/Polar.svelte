<script lang="ts" module>
  /** A pattern drawn over the main one, for comparison. */
  export interface Overlay {
    cut: import("./types").Cut;
    color: string;
    label: string;
  }
</script>

<script lang="ts">
  // A polar gain plot on the ARRL scale: the outer ring is the strongest gain shown.
  import type { Component, Cut } from "./types";
  import { arrl } from "./colormap";

  let {
    cut,
    title,
    half = false,
    component = "gain",
    split = false,
    overlays = [],
  }: {
    cut: Cut | null;
    title: string;
    half?: boolean;
    /** The component drawn. */
    component?: Component;
    /** Draw the vertical and horizontal components over the total. */
    split?: boolean;
    overlays?: Overlay[];
  } = $props();

  const R = 100;
  const RINGS = [0, 3, 10, 20, 30];
  const rad = (deg: number) => (deg * Math.PI) / 180;
  const max = (v: number[] | undefined) => (v?.length ? Math.max(...v) : -Infinity);

  // One scale for everything drawn.
  let peak = $derived(
    Math.max(max(cut?.[component]), ...overlays.map((o) => max(o.cut[component]))) || 0,
  );
  const radius = (g: number) => arrl(g, peak) * R;

  function trace(c: Cut | null, values: number[] | undefined) {
    if (!c || !c.angle.length || !values?.length) return "";
    const d = c.angle
      .map((a, i) => {
        const r = radius(values[i]);
        return `${i ? "L" : "M"}${(r * Math.cos(rad(a))).toFixed(2)},${(-r * Math.sin(rad(a))).toFixed(2)}`;
      })
      .join("");
    return half ? d : d + "Z";
  }
  let path = $derived(trace(cut, cut?.[component]));
  let vertical = $derived(split ? trace(cut, cut?.vertical) : "");
  let horizontal = $derived(split ? trace(cut, cut?.horizontal) : "");
  let others = $derived(overlays.map((o) => ({ d: trace(o.cut, o.cut[component]), color: o.color })));
  const spokes = Array.from({ length: 12 }, (_, i) => i * 30);
</script>

<figure>
  <figcaption>{title}</figcaption>
  <svg viewBox={half ? "-118 -118 236 132" : "-118 -118 236 236"}>
    {#each RINGS as db}
      <circle r={radius(peak - db)} class:outer={db === 0} />
    {/each}
    {#each spokes as s}
      {#if !half || s <= 180}
        <line x2={R * Math.cos(rad(s))} y2={-R * Math.sin(rad(s))} />
        <text x={(R + 9) * Math.cos(rad(s))} y={-(R + 9) * Math.sin(rad(s)) + 3} text-anchor="middle">{s}°</text>
      {/if}
    {/each}
    {#if half}<line x1={-R} x2={R} class="horizon" />{/if}
    {#each RINGS.slice(1) as db}
      <text class="db" x="2" y={-radius(peak - db) + 6}>−{db}</text>
    {/each}
    <path d={path} />
    {#if vertical}<path class="v" d={vertical} />{/if}
    {#if horizontal}<path class="h" d={horizontal} />{/if}
    {#each others as o}
      {#if o.d}<path class="overlay" d={o.d} style:stroke={o.color} />{/if}
    {/each}
  </svg>
  {#if split}
    <div class="key"><span class="t">— total</span><span class="vk">- - vertical</span><span class="hk">- - horizontal</span></div>
  {/if}
</figure>

<style>
  figure {
    margin: 0;
  }
  figcaption {
    font-size: 12px;
    color: var(--muted);
    margin-bottom: 2px;
  }
  svg {
    width: 100%;
    display: block;
  }
  circle {
    fill: none;
    stroke: #2a3648;
    stroke-width: 0.6;
  }
  circle.outer {
    stroke: #3e506a;
    fill: rgba(255, 255, 255, 0.02);
  }
  line {
    stroke: #222c3b;
    stroke-width: 0.5;
  }
  line.horizon {
    stroke: #6b5332;
    stroke-width: 1;
  }
  text {
    fill: #6f7f96;
    font-size: 7px;
  }
  text.db {
    font-size: 6px;
    fill: #536179;
  }
  path {
    fill: rgba(255, 170, 60, 0.18);
    stroke: #ffae45;
    stroke-width: 1.4;
    stroke-linejoin: round;
  }
  path.v,
  path.h {
    fill: none;
    stroke-width: 1;
    stroke-dasharray: 3 2;
  }
  path.v {
    stroke: #58b7ff;
  }
  path.h {
    stroke: #6fd39a;
  }
  path.overlay {
    fill: none;
    stroke-width: 1.1;
  }
  .key {
    display: flex;
    gap: 10px;
    justify-content: center;
    font-size: 10px;
  }
  .t {
    color: #ffae45;
  }
  .vk {
    color: #58b7ff;
  }
  .hk {
    color: #6fd39a;
  }
</style>
