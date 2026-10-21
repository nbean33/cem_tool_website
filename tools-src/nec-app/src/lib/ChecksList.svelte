<script lang="ts">
  // The model checks, grouped by kind, each in plain words: what is wrong, where, what to do,
  // and a button when the fix is mechanical. The measured detail of every finding is kept under
  // "Details".
  import * as api from "./api";
  import Help from "./Help.svelte";
  import { tip } from "./tip";
  import { FINDINGS, SEVERITY_WORDS, type FixAction } from "./findings";
  import { explain } from "./errors";
  import type { Analysis, Finding, FindingKind } from "./types";

  let {
    findings,
    analysis,
    text,
    parseError,
    solveError,
    autoseg,
    highlight = $bindable(null),
    onedit,
    onautoseg,
    onraise,
  }: {
    findings: Finding[];
    analysis: Analysis | null;
    text: string;
    parseError: string | null;
    solveError: string | null;
    autoseg: boolean;
    highlight?: number | null;
    onedit: (text: string) => void;
    /** Turn auto-segmentation on. */
    onautoseg: () => void;
    /** Show the height knob. */
    onraise: () => void;
  } = $props();

  const rank = (s: string) => (s === "error" ? 2 : s === "warning" ? 1 : 0);

  interface Group {
    kind: FindingKind;
    severity: Finding["severity"];
    items: Finding[];
  }
  let groups = $derived.by(() => {
    const by = new Map<FindingKind, Group>();
    for (const f of findings) {
      const g = by.get(f.kind) ?? { kind: f.kind, severity: f.severity, items: [] };
      if (rank(f.severity) > rank(g.severity)) g.severity = f.severity;
      g.items.push(f);
      by.set(f.kind, g);
    }
    return [...by.values()].sort((a, b) => rank(b.severity) - rank(a.severity) || b.items.length - a.items.length);
  });

  /** "wire 3", "wires 1–3, 7": the wires a group's segments are on. */
  function where(items: Finding[]): string {
    const segs = analysis?.segments ?? [];
    const tags = [...new Set(items.map((f) => (f.segment ? segs[f.segment - 1]?.tag : undefined)).filter((t): t is number => t != null))].sort(
      (a, b) => a - b,
    );
    if (!tags.length) return "";
    const runs: string[] = [];
    for (let i = 0; i < tags.length; ) {
      let j = i;
      while (j + 1 < tags.length && tags[j + 1] === tags[j] + 1) j++;
      runs.push(j > i ? `${tags[i]}–${tags[j]}` : String(tags[i]));
      i = j + 1;
    }
    const shown = runs.length > 6 ? [...runs.slice(0, 6), "…"] : runs;
    return `${tags.length > 1 ? "wires" : "wire"} ${shown.join(", ")}`;
  }

  let busy = $state(false);
  let error = $state<string | null>(null);
  /** Whether a fix applies to this model as it stands. */
  function available(a: FixAction | undefined): boolean {
    if (!a) return false;
    if (a === "autoseg") return !autoseg;
    return true;
  }
  async function fix(a: FixAction) {
    error = null;
    if (a === "autoseg") return onautoseg();
    if (a === "raise") return onraise();
    busy = true;
    try {
      const cards = await api.allCards(text);
      const ge = cards.find((c) => c.mnemonic === "GE");
      if (a === "extended-kernel") {
        if (cards.some((c) => c.mnemonic === "EK")) return;
        const at = ge ? ge.line + 1 : (cards.find((c) => c.mnemonic === "EX")?.line ?? 1);
        onedit(await api.insertCard(text, at, "EK", ["0"]));
      } else if (a === "ground-no-bond") {
        if (!ge) return;
        onedit(await api.replaceCard(text, ge.line, "GE", ["-1", ...ge.fields.slice(1)]));
      } else if (a === "sommerfeld") {
        const gn = cards.find((c) => c.mnemonic === "GN");
        if (!gn) return;
        onedit(await api.replaceCard(text, gn.line, "GN", ["2", ...gn.fields.slice(1)]));
      }
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<ul>
  {#if parseError}
    {@const e = explain(parseError)}
    <li class="error">
      <div class="title"><span class="sev">{SEVERITY_WORDS.error}</span> <b>The model can't be read</b></div>
      <p class="fix">{e.text}</p>
      {#if e.raw}<details><summary>The message</summary><p class="why">{e.raw}</p></details>{/if}
    </li>
  {/if}
  {#if solveError && !parseError}
    {@const e = explain(solveError)}
    <li class="error">
      <div class="title"><span class="sev">{SEVERITY_WORDS.error}</span> <b>The model can't be solved</b></div>
      <p class="fix">{e.text}</p>
      {#if e.raw}<details><summary>The message</summary><p class="why">{e.raw}</p></details>{/if}
    </li>
  {/if}
  {#each groups as g (g.kind)}
    {@const h = FINDINGS[g.kind]}
    {@const first = g.items.find((f) => f.segment != null)?.segment ?? null}
    <li class={g.severity}>
      <button
        class="title"
        class:active={highlight != null && highlight === first}
        onclick={() => (highlight = highlight === first ? null : first)}
        use:tip={first != null ? "Highlight where this is in the 3-D view" : null}
      >
        <span class="sev">{SEVERITY_WORDS[g.severity]}</span>
        <b>{h.title}</b>
        <span class="where">{where(g.items)}{g.items.length > 1 ? ` · ${g.items.length} places` : ""}</span>
      </button>
      <p class="fix">{h.fix}</p>
      {#if available(h.action)}
        <button class="small fixit" disabled={busy} onclick={() => fix(h.action!)}>{h.actionLabel}</button>
      {/if}
      <details>
        <summary>Why, and the details</summary>
        <p class="why">{h.why}</p>
        <ul class="raw">
          {#each g.items.slice(0, 50) as f}
            <li>
              <button class="rawbtn" onclick={() => (highlight = f.segment)}>{f.segment ? `segment ${f.segment}: ` : ""}{f.message}</button>
            </li>
          {/each}
          {#if g.items.length > 50}<li>… and {g.items.length - 50} more</li>{/if}
        </ul>
      </details>
    </li>
  {/each}
  {#if !parseError && !solveError && !findings.length && analysis}
    <li class="ok">No problems found. <Help term="checks" /></li>
  {/if}
  {#if error}<li class="error"><p class="fix">{error}</p></li>{/if}
</ul>

<style>
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    overflow-y: auto;
    flex: 1;
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--panel);
  }
  li {
    border-bottom: 1px solid var(--line);
    border-left: 3px solid transparent;
    font-size: 12px;
    padding: 6px 8px;
  }
  li.error {
    border-left-color: #ff5c73;
  }
  li.warning {
    border-left-color: #ffb93b;
  }
  li.info {
    border-left-color: #4b8fe0;
  }
  li.ok {
    color: #6fd39a;
    padding: 8px 10px;
  }
  .title {
    all: unset;
    display: block;
    width: 100%;
    box-sizing: border-box;
    cursor: pointer;
    line-height: 1.35;
  }
  .title.active b {
    color: var(--accent);
  }
  .sev {
    font-size: 10.5px;
    text-transform: uppercase;
    letter-spacing: 0.4px;
    margin-right: 4px;
  }
  .error .sev {
    color: #ff8a9a;
  }
  .warning .sev {
    color: #ffcf6a;
  }
  .info .sev {
    color: #8fc0ff;
  }
  .title b {
    font-weight: 600;
  }
  .where {
    color: var(--muted);
    margin-left: 4px;
  }
  .fix {
    margin: 3px 0 0;
    line-height: 1.4;
  }
  .fixit {
    margin-top: 5px;
  }
  details {
    margin-top: 4px;
  }
  summary {
    cursor: pointer;
    color: var(--muted);
    font-size: 11.5px;
  }
  .why {
    margin: 4px 0;
    color: var(--muted);
    line-height: 1.4;
  }
  .raw {
    border: none;
    background: none;
    overflow: visible;
  }
  .raw li {
    border: none;
    padding: 1px 0;
    font-size: 11px;
    color: var(--muted);
  }
  .rawbtn {
    all: unset;
    cursor: pointer;
  }
  .rawbtn:hover {
    color: var(--text);
  }
</style>
