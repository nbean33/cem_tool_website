<script lang="ts">
  // A small "?" that explains a term: its short line, the longer explanation and any source.
  // Opens on hover, keyboard focus or click; Escape or clicking elsewhere closes it.
  import { HELP, type HelpKey } from "./help";

  let { term }: { term: HelpKey } = $props();
  let open = $state(false);
  let pinned = $state(false);
  let host: HTMLSpanElement;
  let pop = $state<HTMLDivElement>();
  let style = $state("");

  const h = $derived(HELP[term] as { short: string; long?: string; source?: string });

  function place() {
    if (!host || !pop) return;
    const r = host.getBoundingClientRect();
    const w = Math.min(340, window.innerWidth - 16);
    let x = Math.max(8, Math.min(r.left - 12, window.innerWidth - w - 8));
    let y = r.bottom + 6;
    const ph = pop.offsetHeight;
    if (y + ph > window.innerHeight - 8) y = Math.max(8, r.top - ph - 6);
    style = `left:${x}px;top:${y}px;width:${w}px`;
  }
  $effect(() => {
    if (open) requestAnimationFrame(place);
  });
  function outside(e: MouseEvent) {
    if (pinned && host && !host.contains(e.target as Node)) {
      pinned = false;
      open = false;
    }
  }
</script>

<svelte:window onmousedown={outside} />

<span class="help" bind:this={host}>
  <button
    type="button"
    class="q"
    aria-label={`Help: ${h.short}`}
    aria-expanded={open}
    onmouseenter={() => (open = true)}
    onmouseleave={() => (open = pinned)}
    onfocus={() => (open = true)}
    onblur={() => (open = pinned)}
    onclick={(e) => {
      e.stopPropagation();
      e.preventDefault();
      pinned = !pinned;
      open = pinned;
    }}
    onkeydown={(e) => {
      if (e.key === "Escape") {
        pinned = false;
        open = false;
      }
    }}>?</button
  >
  {#if open}
    <div class="pop" role="tooltip" bind:this={pop} {style}>
      <p class="short">{h.short}</p>
      {#if h.long}<p>{h.long}</p>{/if}
      {#if h.source}<p class="src">Source: {h.source}</p>{/if}
    </div>
  {/if}
</span>

<style>
  .help {
    display: inline-flex;
    vertical-align: middle;
  }
  .q {
    all: unset;
    box-sizing: border-box;
    width: 15px;
    height: 15px;
    border-radius: 50%;
    border: 1px solid #3a4a62;
    color: #8fa3bf;
    font-size: 10px;
    line-height: 13px;
    text-align: center;
    cursor: help;
    margin-left: 4px;
  }
  .q:hover,
  .q:focus-visible {
    border-color: var(--accent);
    color: var(--accent);
  }
  .pop {
    position: fixed;
    z-index: 1000;
    background: #18212e;
    border: 1px solid #34445c;
    border-radius: 8px;
    padding: 8px 10px;
    box-shadow: 0 6px 24px rgba(0, 0, 0, 0.45);
    font-size: 12px;
    line-height: 1.45;
    color: var(--text);
    text-align: left;
    white-space: normal;
    font-weight: normal;
  }
  .pop p {
    margin: 0 0 6px;
  }
  .pop p:last-child {
    margin-bottom: 0;
  }
  .short {
    font-weight: 600;
  }
  .src {
    color: var(--muted);
    font-size: 11px;
  }
</style>
