<script lang="ts">
  import { fieldProblem } from "./guards";
  import Help from "./Help.svelte";
  import { tip } from "./tip";
  import { CARD_HINTS, FIELD_HINTS, hint } from "./help";
  // The deck's geometry cards as a list; the selected one opens as a form. Every edit rewrites
  // that one line of the deck, so fields may hold SY expressions as well as numbers.
  import * as api from "./api";
  import type { CardLine } from "./api";
  import type { Analysis, Units } from "./types";
  import { length } from "./units";

  let {
    text,
    analysis,
    cards,
    insertAt,
    mhz,
    selected = $bindable(null),
    dragNote,
    units = "metres",
    onedit,
  }: {
    text: string;
    analysis: Analysis | null;
    cards: CardLine[];
    insertAt: number | null;
    mhz: number | null;
    selected?: number | null;
    /** Why the selected wire cannot be dragged in the 3-D view, if it cannot. */
    dragNote: string | null;
    /** Display units for lengths read off the built structure. */
    units?: Units;
    onedit: (text: string) => void;
  } = $props();

  /** Field labels per card, from the NEC-2 user's guide. */
  const LABELS: Record<string, string[]> = {
    GW: ["Tag", "Segments", "X1", "Y1", "Z1", "X2", "Y2", "Z2", "Radius"],
    GA: ["Tag", "Segments", "Arc radius", "Angle 1°", "Angle 2°", "Wire radius"],
    GH: ["Tag", "Segments", "Turn spacing", "Length", "Radius X1", "Radius Y1", "Radius X2", "Radius Y2", "Wire radius"],
    GM: ["Tag step", "Copies", "Rot X°", "Rot Y°", "Rot Z°", "Move X", "Move Y", "Move Z", "From tag"],
    GX: ["Tag step", "Planes (XYZ)"],
    GR: ["Tag step", "Copies"],
    GS: ["", "", "Scale"],
    GC: ["", "", "Length ratio", "Radius 1", "Radius 2"],
    SP: ["", "Shape", "X1", "Y1", "Z1", "X2 / elev°", "Y2 / azim°", "Z2 / area"],
    SM: ["Columns", "Rows", "X1", "Y1", "Z1", "X2", "Y2", "Z2"],
    SC: ["", "Shape", "X3", "Y3", "Z3", "X4", "Y4", "Z4"],
    GE: ["Ground"],
    GF: ["", "File"],
  };
  const DESCRIBE: Record<string, string> = {
    GW: "Straight wire",
    GA: "Arc",
    GH: "Helix",
    GM: "Move / copy",
    GX: "Reflect",
    GR: "Rotate about Z",
    GS: "Scale",
    GC: "Taper",
    SP: "Patch",
    SM: "Patch grid",
    SC: "Patch corners",
    GE: "End of geometry",
    GF: "Green's function",
  };

  let error = $state<string | null>(null);
  let card = $derived(cards.find((c) => c.line === selected) ?? null);
  let labels = $derived(card ? (LABELS[card.mnemonic] ?? []) : []);
  /** The form's fields: as many as the card has labels or written fields. */
  let width = $derived(card ? Math.max(labels.length, card.fields.length) : 0);
  /** Length of the wire the selected card built, end to end along its segments, in metres. */
  let built = $derived.by(() => {
    if (!card || !analysis) return null;
    const segs = analysis.segments.filter((s) => s.origin === card!.line);
    if (!segs.length) return null;
    return segs.reduce((t, s) => t + Math.hypot(s.b[0] - s.a[0], s.b[1] - s.a[1], s.b[2] - s.a[2]), 0);
  });
  let segCount = $derived((line: number) => analysis?.segments.filter((s) => s.origin === line).length ?? 0);

  const isNumber = (f: string) => /^[-+]?(\d+\.?\d*|\.\d+)([eE][-+]?\d+)?$/.test(f.trim());
  const fmt = (v: number) => (Math.abs(v) < 1e-12 ? "0" : String(Number(v.toPrecision(7))));
  const nextTag = () => Math.max(0, ...cards.map((c) => (["GW", "GA", "GH"].includes(c.mnemonic) ? Number(c.fields[0]) || 0 : 0))) + 1;

  async function run(f: () => Promise<string>) {
    error = null;
    try {
      onedit(await f());
    } catch (e) {
      error = String(e);
    }
  }

  function setField(i: number, value: string) {
    if (!card) return;
    const problem = fieldProblem(card.mnemonic, i, value);
    if (problem) {
      error = problem;
      return;
    }
    const fields = [...card.fields];
    while (fields.length <= i) fields.push("0");
    fields[i] = value.trim() || "0";
    // Trailing zeros NEC would default anyway are dropped beyond the written width.
    const c = card;
    void run(() => api.replaceCard(text, c.line, c.mnemonic, fields));
  }

  /** A new straight wire: a copy of the selected one moved sideways, or a half-wave dipole. */
  function addWire() {
    const at = insertAt ?? cards.length + 1;
    let fields: string[];
    if (card?.mnemonic === "GW") {
      const d = [0, 1, 2].map((k) => (card!.values[5 + k] ?? 0) - (card!.values[2 + k] ?? 0));
      // Sideways by about a quarter of the wire's length, rounded to 1, 2 or 5, along the axis
      // the wire runs least along.
      const want = Math.max(Math.hypot(...d) * 0.25, 1e-3);
      const decade = 10 ** Math.floor(Math.log10(want));
      const step = [1, 2, 5, 10].map((m) => m * decade).find((v) => v >= want * 0.75) ?? want;
      const axis = d.map(Math.abs).indexOf(Math.min(...d.map(Math.abs)));
      fields = card.fields.map((f, i) => {
        if (i === 0) return String(nextTag());
        if (i === 2 + axis || i === 5 + axis) return isNumber(f) ? fmt(Number(f) + step) : `(${f})+${fmt(step)}`;
        return f;
      });
    } else {
      const quarter = 300 / (mhz ?? analysis?.frequencies_mhz[0] ?? 300) / 4;
      const z = analysis?.has_ground ? fmt(quarter * 2) : "0";
      fields = [String(nextTag()), "21", "0", fmt(-quarter), z, "0", fmt(quarter), z, ".001"];
    }
    void run(async () => {
      const t = await api.insertCard(text, at, "GW", fields);
      selected = at;
      return t;
    });
  }

  function remove() {
    if (!card) return;
    const c = card;
    selected = null;
    void run(() => api.deleteLine(text, c.line));
  }
</script>

<section class="card">
  <div class="head">
    <h3>Wires and shape<Help term="shape" /></h3>
    <button class="small" onclick={addWire} title="A copy of the selected wire moved sideways, or a half-wave dipole">+ Wire</button>
  </div>
  {#if !cards.length}
    <p class="hint">No geometry cards.</p>
  {/if}
  <ul class="cards">
    {#each cards as c (c.line)}
      <li>
        <button class="row" class:on={c.line === selected} onclick={() => (selected = selected === c.line ? null : c.line)} use:tip={CARD_HINTS[c.mnemonic]}>
          <span class="m">{c.mnemonic}</span>
          <span class="f">{c.fields.join(" ")}</span>
          <span class="d">{DESCRIBE[c.mnemonic] ?? ""}{#if segCount(c.line)} · {segCount(c.line)} seg{/if}</span>
        </button>
      </li>
    {/each}
  </ul>
</section>

{#if card}
  <section class="card">
    <div class="head">
      <h3>{card.mnemonic} · line {card.line}</h3>
      <button class="small warn" onclick={remove} use:tip={hint("deleteCard")}>Delete</button>
    </div>
    <div class="form">
      {#each Array(width) as _, i (i)}
        <label>
          <span use:tip={FIELD_HINTS[card.mnemonic]?.[i]}>{labels[i] || `Field ${i + 1}`}</span>
          <input
            type="text"
            value={card.fields[i] ?? ""}
            placeholder="0"
            onchange={(e) => setField(i, (e.currentTarget as HTMLInputElement).value)}
            onkeydown={(e) => e.key === "Enter" && (e.currentTarget as HTMLInputElement).blur()}
          />
          {#if card.fields[i] != null && !isNumber(card.fields[i]) && card.values[i] != null}
            <span class="val">= {fmt(card.values[i]!)}</span>
          {/if}
        </label>
      {/each}
    </div>
    {#if built != null}
      <p class="hint">As built: {length(built, units, 4)} of wire{units === "metres" ? "" : ` (${length(built, "metres", 4)})`}.</p>
    {/if}
    {#if card.comment}<p class="hint">' {card.comment}</p>{/if}
    {#if card.mnemonic === "GW"}
      <p class="hint">{dragNote ?? "Drag the wire's ends in the 3-D view. They snap to other wires' ends, or to the grid; hold Shift for neither."}</p>
    {/if}
  </section>
{/if}
{#if error}<p class="err">{error}</p>{/if}

<style>
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 6px;
  }
  .cards {
    list-style: none;
    margin: 0;
    padding: 0;
    max-height: 260px;
    overflow: auto;
  }
  .row {
    display: grid;
    grid-template-columns: 28px minmax(0, 1fr) auto;
    gap: 8px;
    width: 100%;
    text-align: left;
    background: none;
    border: 0;
    border-radius: 4px;
    padding: 3px 6px;
    color: var(--text);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }
  .row:hover {
    background: rgba(255, 255, 255, 0.04);
  }
  .row.on {
    background: rgba(64, 240, 255, 0.12);
  }
  .m {
    font-weight: 600;
    color: #40f0ff;
  }
  .f {
    font-family: ui-monospace, monospace;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .d {
    color: var(--muted);
  }
  .form {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 6px 8px;
  }
  .form label {
    display: flex;
    flex-direction: column;
    font-size: 11px;
    color: var(--muted);
    gap: 2px;
  }
  .form input {
    width: 100%;
    box-sizing: border-box;
    font-family: ui-monospace, monospace;
  }
  .val {
    font-size: 10px;
  }
  .hint {
    font-size: 12px;
    color: var(--muted);
    margin: 8px 0 0;
  }
  .err {
    color: #ff7a8c;
    font-size: 12px;
  }
</style>
