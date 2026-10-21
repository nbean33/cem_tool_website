<script lang="ts">
  // A Smith chart normalized to 50 Ω: the sweep trace, with the solved frequency marked.
  let {
    points,
    marker = null,
    z0 = 50,
  }: { points: { z: [number, number] }[]; marker?: [number, number] | null; z0?: number } = $props();

  const R = 100;
  /** Γ = (z − 1)/(z + 1) for z = Z/Z0, in chart coordinates (y down). */
  const gamma = (re: number, im: number): [number, number] => {
    const [a, b] = [re / z0, im / z0];
    const d = (a + 1) ** 2 + b ** 2;
    return [((a * a + b * b - 1) / d) * R, ((2 * b) / d) * -R];
  };

  let trace = $derived(
    points
      .map((p, i) => {
        const [x, y] = gamma(p.z[0], p.z[1]);
        return `${i ? "L" : "M"}${x.toFixed(2)},${y.toFixed(2)}`;
      })
      .join(""),
  );
  let dot = $derived(marker ? gamma(marker[0], marker[1]) : null);

  const grid = [0.2, 0.5, 1, 2, 5];
  // Constant-reactance circles: centre (1, ±1/x), radius 1/x, clipped to the chart.
  const swr2 = R / 3;
</script>

<figure>
  <figcaption>Smith chart ({z0} Ω)</figcaption>
  <svg viewBox="-112 -112 224 224">
    <defs><clipPath id="smith-rim"><circle r={R} /></clipPath></defs>
    <circle r={R} class="rim" />
    {#each grid as r}<circle cx={(r / (1 + r)) * R} r={R / (1 + r)} />{/each}
    <g clip-path="url(#smith-rim)">
      {#each grid as x}<circle cx={R} cy={-R / x} r={R / x} /><circle cx={R} cy={R / x} r={R / x} />{/each}
    </g>
    <line x1={-R} x2={R} />
    <circle r={swr2} class="swr" />
    <text x={swr2 * 0.72} y={-swr2 * 0.72 - 2} class="swrlabel">SWR 2</text>
    {#each grid as r}<text x={((r - 1) / (r + 1)) * R + 1.5} y="-2">{r}</text>{/each}
    <path d={trace} class="trace" />
    {#if dot}<circle cx={dot[0]} cy={dot[1]} r="3.2" class="dot" />{/if}
  </svg>
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
  circle,
  path {
    fill: none;
    stroke: #26313f;
    stroke-width: 0.6;
  }
  line {
    stroke: #26313f;
    stroke-width: 0.6;
  }
  .rim {
    stroke: #3e506a;
    stroke-width: 1;
    fill: rgba(255, 255, 255, 0.02);
  }
  .swr {
    stroke: #3f7d5a;
    stroke-dasharray: 3 2;
  }
  .swrlabel {
    fill: #4f9c71;
  }
  text {
    fill: #6f7f96;
    font-size: 6px;
  }
  .trace {
    stroke: #58b7ff;
    stroke-width: 1.5;
  }
  .dot {
    fill: #ff3b5c;
    stroke: white;
    stroke-width: 0.8;
  }
</style>
