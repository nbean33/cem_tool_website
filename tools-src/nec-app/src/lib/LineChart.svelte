<script lang="ts" module>
  export interface Series {
    label: string;
    values: (number | null)[];
    color: string;
    /** Which y axis: the left one or the right one. */
    axis?: "left" | "right";
    dash?: boolean;
    digits?: number;
  }
</script>

<script lang="ts">
  // A small multi-series line chart (uPlot) with up to two y axes and an optional x marker.
  import { onMount } from "svelte";
  import uPlot from "uplot";
  import "uplot/dist/uPlot.min.css";

  let {
    x,
    series,
    marker = null,
    height = 180,
    xDigits = 3,
    xLabel = "",
    leftRange = null,
    onpick = null,
    logX = false,
    logLeft = false,
    smooth = false,
  }: {
    x: number[];
    series: Series[];
    marker?: number | null;
    height?: number;
    xDigits?: number;
    xLabel?: string;
    leftRange?: ((min: number, max: number) => [number, number]) | null;
    onpick?: ((x: number) => void) | null;
    /** Logarithmic x axis, or left y axis (values must be positive). */
    logX?: boolean;
    logLeft?: boolean;
    /** Draw monotone cubic splines through the points instead of straight lines (display only). */
    smooth?: boolean;
  } = $props();

  let host: HTMLDivElement;
  let plot: uPlot | null = null;
  /** The series layout the current plot was built for; a change rebuilds it. */
  let layout = "";

  const axis = { stroke: "#8392a8", grid: { stroke: "#1f2835", width: 1 }, ticks: { stroke: "#1f2835" } };
  const compact = (v: number) => (Math.abs(v) >= 1000 ? `${+(v / 1000).toFixed(1)}k` : `${+v.toFixed(2)}`);
  const fmt = (digits: number) => (_u: uPlot, v: number | null) => (v == null ? "–" : v.toFixed(digits));

  function build() {
    plot?.destroy();
    const right = series.some((s) => s.axis === "right");
    const opts: uPlot.Options = {
      width: host.clientWidth,
      height,
      cursor: { drag: { x: false, y: false } },
      scales: {
        x: { time: false, distr: logX ? 3 : 1 },
        left: { distr: logLeft ? 3 : 1, ...(leftRange && !logLeft ? { range: (_u: uPlot, min: number, max: number) => leftRange!(min, max) } : {}) },
        right: {},
      },
      axes: [
        { ...axis, values: (_u, v) => v.map((t) => t.toFixed(xDigits)) },
        { ...axis, scale: "left", size: 40, values: (_u, v) => v.map(compact) },
        ...(right
          ? [{ ...axis, scale: "right", side: 1, grid: { show: false }, size: 44, values: (_u: uPlot, v: number[]) => v.map(compact) }]
          : []),
      ],
      series: [
        { label: xLabel, value: fmt(xDigits + 1) },
        ...series.map((s) => ({
          label: s.label,
          scale: s.axis ?? "left",
          stroke: s.color,
          width: s.dash ? 1.3 : 1.8,
          dash: s.dash ? [5, 3] : undefined,
          spanGaps: true,
          paths: smooth ? uPlot.paths.spline!() : undefined,
          value: fmt(s.digits ?? 2),
        })),
      ],
      hooks: {
        draw: [
          (u) => {
            if (marker == null) return;
            const px = u.valToPos(marker, "x", true);
            const ctx = u.ctx;
            ctx.save();
            ctx.strokeStyle = "#ff3b5c";
            ctx.setLineDash([4, 3]);
            ctx.beginPath();
            ctx.moveTo(px, u.bbox.top);
            ctx.lineTo(px, u.bbox.top + u.bbox.height);
            ctx.stroke();
            ctx.restore();
          },
        ],
      },
    };
    plot = new uPlot(opts, data(), host);
    plot.over.addEventListener("click", () => {
      const i = plot?.cursor.idx;
      if (onpick && i != null && x[i] != null) onpick(x[i]);
    });
  }

  const data = (): uPlot.AlignedData => [x, ...series.map((s) => s.values)] as uPlot.AlignedData;

  onMount(() => {
    const ro = new ResizeObserver(() => plot?.setSize({ width: host.clientWidth, height }));
    ro.observe(host);
    return () => {
      ro.disconnect();
      plot?.destroy();
      plot = null;
    };
  });

  $effect(() => {
    const key = series.map((s) => `${s.label}/${s.axis}/${s.color}`).join("|") + `/${logX}/${logLeft}/${smooth}`;
    const d = data();
    void marker;
    if (!host) return;
    if (key !== layout || !plot) {
      layout = key;
      build();
    } else {
      plot.setData(d);
    }
  });
</script>

<div class="chart" class:pickable={!!onpick} bind:this={host}></div>

<style>
  .chart {
    width: 100%;
  }
  .pickable :global(.u-over) {
    cursor: crosshair;
  }
  .chart :global(.u-legend) {
    font-size: 11px;
    color: var(--muted);
  }
  .chart :global(.u-legend th) {
    font-weight: 500;
  }
</style>
