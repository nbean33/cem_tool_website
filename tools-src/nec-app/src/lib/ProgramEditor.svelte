<script lang="ts">
  import { fieldProblem } from "./guards";
  import Help from "./Help.svelte";
  import { tip } from "./tip";
  import { CARD_HINTS, FIELD_HINTS, hint } from "./help";
  // The deck's program cards — sources, loads, transmission lines, networks, frequency and
  // ground — listed and edited one card at a time, like the geometry editor. Ground has its own
  // form with typical soils.
  import * as api from "./api";
  import type { CardLine } from "./api";
  import type { Analysis } from "./types";

  let {
    text,
    analysis,
    segment = null,
    onedit,
  }: {
    text: string;
    analysis: Analysis | null;
    /** A segment picked elsewhere (1-based), where new sources and loads go. */
    segment?: number | null;
    onedit: (text: string) => void;
  } = $props();

  const EDITED = ["EX", "LD", "TL", "NT", "FR", "GN", "GD", "EK", "KH"];
  const LABELS: Record<string, string[]> = {
    EX: ["Type", "Tag", "Segment", "Print", "V real / I real", "V imag / I imag", "Field 7", "Field 8", "Field 9", "Field 10"],
    LD: ["Type", "Tag", "From seg", "To seg", "R Ω / σ S/m", "L H / X Ω", "C F"],
    TL: ["Tag 1", "Seg 1", "Tag 2", "Seg 2", "Z₀ Ω", "Length m", "Y1 real", "Y1 imag", "Y2 real", "Y2 imag"],
    NT: ["Tag 1", "Seg 1", "Tag 2", "Seg 2", "Y11 real", "Y11 imag", "Y12 real", "Y12 imag", "Y22 real", "Y22 imag"],
    FR: ["Type", "Steps", "", "", "Start MHz", "Step MHz"],
    GN: ["Type", "Radials", "", "", "εr", "σ S/m", "Screen radius", "Wire radius"],
    GD: ["", "", "", "", "εr 2", "σ 2", "Distance", "Depth"],
    EK: ["Flag"],
    KH: ["", "", "", "", "Distance λ"],
  };
  const DESCRIBE: Record<string, (f: string[]) => string> = {
    EX: (f) =>
      ({ "0": "Voltage source", "1": "Plane wave", "2": "Plane wave (right)", "3": "Plane wave (left)", "4": "Current element", "5": "Voltage source (slope)", "6": "Current source" })[f[0]] ?? "Excitation",
    LD: (f) =>
      ({ "-1": "Clear loads", "0": "Series RLC", "1": "Parallel RLC", "2": "Series RLC per m", "3": "Parallel RLC per m", "4": "Impedance", "5": "Wire conductivity", "6": "Trap (4nec2)", "7": "Insulation (4nec2)" })[f[0]] ?? "Load",
    TL: () => "Transmission line",
    NT: () => "Network",
    FR: (f) => (Number(f[1] ?? 1) > 1 ? `${f[1]} frequencies` : "Frequency"),
    GN: (f) => ({ "-1": "Free space", "0": "Real ground", "1": "Perfect ground", "2": "Sommerfeld ground", "3": "MiniNEC ground" })[f[0]] ?? "Ground",
    GD: () => "Second ground medium",
    EK: () => "Extended kernel",
    KH: () => "Interaction approximation",
  };

  /** Typical grounds, as 4nec2 and the ARRL tabulate them. */
  const SOILS = [
    { name: "Average", epsr: 13, sigma: 0.005 },
    { name: "Poor (sandy, dry)", epsr: 5, sigma: 0.001 },
    { name: "Good (rich soil)", epsr: 20, sigma: 0.0303 },
    { name: "City / industrial", epsr: 3, sigma: 0.001 },
    { name: "Fresh water", epsr: 80, sigma: 0.001 },
    { name: "Sea water", epsr: 81, sigma: 5 },
  ];

  let cards = $state<CardLine[]>([]);
  let selected = $state<number | null>(null);
  let error = $state<string | null>(null);

  $effect(() => {
    const t = text;
    api.allCards(t).then(
      (c) => {
        if (t === text) cards = c;
      },
      () => {},
    );
  });

  let ge = $derived(cards.find((c) => c.mnemonic === "GE")?.line ?? null);
  let program = $derived(cards.filter((c) => EDITED.includes(c.mnemonic) && (ge == null || c.line > ge)));
  let card = $derived(program.find((c) => c.line === selected) ?? null);
  let labels = $derived(card ? (LABELS[card.mnemonic] ?? []) : []);
  let width = $derived(card ? Math.max(labels.length, card.fields.length) : 0);
  let gn = $derived(program.find((c) => c.mnemonic === "GN") ?? null);

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
    const c = card;
    void run(() => api.replaceCard(text, c.line, c.mnemonic, fields));
  }

  /** Where a new card of \`mnemonic\` goes: after the last of its kind, else before the first card
   * that runs the solution (FR, RP, XQ, NE, NH, EN), else at the end. */
  function insertAt(mnemonic: string): number {
    const same = program.filter((c) => c.mnemonic === mnemonic);
    if (same.length) return same[same.length - 1].line + 1;
    const after = ge ?? 0;
    const run = cards.find((c) => c.line > after && ["FR", "RP", "XQ", "NE", "NH", "EN", "PT", "PQ", "NX", "CP"].includes(c.mnemonic));
    return run ? run.line : (cards.at(-1)?.line ?? 0) + 1;
  }

  /** Tag and segment-within-tag of the picked segment, or the first wire's centre. */
  function place(): [string, string] {
    const segs = analysis?.segments ?? [];
    const k = segment != null && segment >= 1 ? segment - 1 : -1;
    const tag = k >= 0 ? segs[k]?.tag : segs[0]?.tag;
    if (tag == null) return ["1", "1"];
    const ofTag = segs.map((s, i) => [s.tag, i] as const).filter(([t]) => t === tag);
    const within = k >= 0 ? ofTag.findIndex(([, i]) => i === k) : Math.floor(ofTag.length / 2);
    return [String(tag), String(within + 1)];
  }

  function add(mnemonic: string, fields: string[]) {
    const at = insertAt(mnemonic);
    void run(async () => {
      const t = await api.insertCard(text, at, mnemonic, fields);
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

  // Ground form.
  let groundKind = $state("0");
  let epsr = $state(13);
  let sigma = $state(0.005);
  $effect(() => {
    // Follow the deck's GN card.
    const f = gn?.fields;
    groundKind = f ? (f[0] ?? "0") : "-1";
    if (f && f[4] != null && Number.isFinite(Number(f[4]))) epsr = Number(f[4]);
    if (f && f[5] != null && Number.isFinite(Number(f[5]))) sigma = Number(f[5]);
  });
  function applyGround() {
    const fields =
      groundKind === "-1" ? ["-1"] : groundKind === "1" ? ["1"] : [groundKind, "0", "0", "0", String(epsr), String(sigma)];
    if (gn) {
      const keep = gn.fields.slice(fields.length);
      const line = gn.line;
      // Keep a radial screen's fields when only the soil changes.
      void run(() => api.replaceCard(text, line, "GN", groundKind === "0" ? [...fields, ...keep] : fields));
    } else {
      const at = ge != null ? ge + 1 : insertAt("GN");
      void run(() => api.insertCard(text, at, "GN", fields));
    }
  }
</script>

<section class="card">
  <h3>Ground<Help term="groundKind" /></h3>
  <div class="row">
    <select bind:value={groundKind}>
      <option value="-1">Free space</option>
      <option value="1">Perfect ground</option>
      <option value="0">Real ground (reflection coefficient)</option>
      <option value="2">Real ground (Sommerfeld)</option>
    </select>
  </div>
  {#if groundKind === "0" || groundKind === "2"}
    <div class="row">
      <select onchange={(e) => {
        const s = SOILS[Number((e.currentTarget as HTMLSelectElement).value)];
        if (s) ((epsr = s.epsr), (sigma = s.sigma));
      }}>
        <option value="">Soil…</option>
        {#each SOILS as s, i}<option value={i}>{s.name} ({s.epsr}, {s.sigma})</option>{/each}
      </select>
      <label>εr <input type="number" step="any" class="n" bind:value={epsr} /></label>
      <label>σ <input type="number" step="any" class="n" bind:value={sigma} /> S/m</label>
    </div>
  {/if}
  <button class="small primary" onclick={applyGround}>Apply</button>
  <p class="hint">Wires that touch the ground connect to it only when the GE card says so (GE 1).</p>
</section>

<section class="card">
  <div class="head">
    <h3>Feed, loads and lines<Help term="feed" /></h3>
    <span class="adds">
      <button class="small" onclick={() => add("EX", ["0", ...place(), "0", "1", "0"])} use:tip={hint("addSource")}>+ Source</button>
      <button class="small" onclick={() => add("LD", ["0", ...place(), place()[1], "50", "0", "0"])} use:tip={hint("addLoad")}>+ Load</button>
      <button class="small" onclick={() => add("TL", [...place(), ...place(), "50", "0"])} use:tip={hint("addLine")}>+ Line</button>
    </span>
  </div>
  <ul class="cards">
    {#each program as c (c.line)}
      <li>
        <button class="rowbtn" class:on={c.line === selected} onclick={() => (selected = selected === c.line ? null : c.line)} use:tip={CARD_HINTS[c.mnemonic]}>
          <span class="m">{c.mnemonic}</span>
          <span class="f">{c.fields.join(" ")}</span>
          <span class="d">{DESCRIBE[c.mnemonic]?.(c.fields) ?? ""}</span>
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
        </label>
      {/each}
    </div>
    {#if card.comment}<p class="hint">' {card.comment}</p>{/if}
  </section>
{/if}
{#if error}<p class="err">{error}</p>{/if}

<style>
  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
    margin-bottom: 6px;
    font-size: 12px;
  }
  input.n {
    width: 70px;
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 6px;
  }
  .adds {
    display: flex;
    gap: 4px;
  }
  .cards {
    list-style: none;
    margin: 0;
    padding: 0;
    max-height: 240px;
    overflow: auto;
  }
  .rowbtn {
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
  .rowbtn:hover {
    background: rgba(255, 255, 255, 0.04);
  }
  .rowbtn.on {
    background: rgba(64, 240, 255, 0.12);
  }
  .m {
    font-weight: 600;
    color: #ffae45;
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
