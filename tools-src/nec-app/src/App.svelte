<script lang="ts">
  import { onMount } from "svelte";
  import { ask, open, save } from "@tauri-apps/plugin-dialog";
  import * as api from "./lib/api";
  import type { Analysis, Component, ModelOptions, Project, ProjectSettings, Slice, SliceRequest, SolveView, Units } from "./lib/types";
  import Viewer3D from "./lib/Viewer3D.svelte";
  import LineChart, { type Series } from "./lib/LineChart.svelte";
  import Smith from "./lib/Smith.svelte";
  import PatternPanel from "./lib/PatternPanel.svelte";
  import Optimizer from "./lib/Optimizer.svelte";
  import ParamSweep from "./lib/ParamSweep.svelte";
  import GeometryEditor from "./lib/GeometryEditor.svelte";
  import ProgramEditor from "./lib/ProgramEditor.svelte";
  import MatchPanel from "./lib/MatchPanel.svelte";
  import DataPanel from "./lib/DataPanel.svelte";
  import ChecksPanel from "./lib/ChecksPanel.svelte";
  import ModelBrowser from "./lib/ModelBrowser.svelte";
  import Knobs from "./lib/Knobs.svelte";
  import ChecksList from "./lib/ChecksList.svelte";
  import { SOILS, TEMPLATES } from "./lib/templates";
  import { coverage, dBd, resonanceNear, swrSpan, swrVerdict, trim } from "./lib/readout";
  import SettingsMenu from "./lib/SettingsMenu.svelte";
  import Help from "./lib/Help.svelte";
  import { tip } from "./lib/tip";
  import { hint } from "./lib/help";
  import { BANDS, bandOf, centre } from "./lib/bands";
  import { loadPrefs, savePrefs } from "./lib/prefs";
  import type { CardLine } from "./lib/api";
  import { PointStream } from "./lib/stream.svelte";

  // The first model: a 20 m dipole from the templates, so there are knobs to try straight away.
  const EXAMPLE = TEMPLATES[0].deck({ mhz: 14.175, height: 10, soil: SOILS[0] });

  /** Re-sweep automatically after an edit only while a sweep stays quick. */
  const AUTO_SWEEP_SEGMENTS = 800;

  let text = $state(EXAMPLE);
  let path = $state<string | null>(null);
  let dirty = $state(false);
  let analysis = $state<Analysis | null>(null);
  let parseError = $state<string | null>(null);
  let result = $state<SolveView | null>(null);
  let solveError = $state<string | null>(null);
  let mhz = $state<number | null>(null);
  /** The band the frequency is in, for the band picker. */
  let band = $derived(bandOf(mhz));
  const prefs = loadPrefs();
  let z0 = $state(prefs.z0);
  let sweepStart = $state(14.0);
  let sweepStop = $state(14.35);
  let sweepSteps = $state(prefs.sweepSteps);
  let sweepPattern = $state(true);
  let azimuth = $state(0);
  let sweeping = $state(false);
  let busy = $state(false);
  let highlight = $state<number | null>(null);
  let status = $state("Ready");
  let tab = $state<"results" | "geometry" | "model" | "match" | "optimize" | "data">("results");
  let browsing = $state(false);
  let nearField = $state<{ request: SliceRequest; slice: Slice } | null>(null);
  let geo = $state<{ cards: CardLine[]; insert_at: number | null }>({ cards: [], insert_at: null });
  /** Deck line of the card selected in the geometry editor or the 3-D view. */
  let selectedLine = $state<number | null>(null);
  /** Counts loaded models, so the 3-D view re-fits for a new one but not for an edit. */
  let loads = $state(0);
  let units = $state<Units>(prefs.units);
  /** 4nec2's auto-segmentation and stepped-radius correction (its Settings menu). */
  let autoseg = $state(prefs.autoseg);
  let segsPerHalfWave = $state(prefs.segsPerHalfWave);
  let stepRadius = $state(prefs.stepRadius);
  /** The deck text panel open (beginners rarely need it). */
  let deckOpen = $state(prefs.deckOpen);
  /** Show the welcome with the New dialog. */
  let welcome = $state(false);
  // Remember the settings for next time.
  $effect(() => {
    savePrefs({
      units,
      z0,
      autoseg,
      segsPerHalfWave,
      stepRadius,
      sweepSteps,
      deckOpen,
      // This launch shows the welcome if it is the first, so from now on it has been seen.
      welcomed: true,
    });
  });
  const modelOptions = $derived<ModelOptions>({
    autoseg: autoseg ? Math.min(500, Math.max(10, Math.round(segsPerHalfWave))) : null,
    step_radius: stepRadius,
  });
  /** The pattern component shown (polar plots and the 3-D surface), and play-back. */
  let component = $state<Component>("gain");
  let splitComponents = $state(false);
  let playing = $state<number | null>(null);
  /** Sweep chart display: logarithmic frequency or SWR, spline smoothing. */
  let logF = $state(false);
  let logSwr = $state(false);
  let smooth = $state(false);
  /** The open project's name and notes, kept for saving. */
  let meta = $state({ name: "", notes: "" });
  /** Settings read with a project, applied once its deck has been analysed. */
  let restored: ProjectSettings | null = null;
  let optimizer: Optimizer | undefined = $state();
  const sweep = new PointStream();

  let fileName = $derived(path ? path.split(/[\\/]/).pop() : "untitled.nec");
  const isProject = (p: string) => /\.toml$/i.test(p);
  let pattern = $derived(result?.pattern ?? null);
  // The 3-D surface shows the circular component chosen, on the total gain's scale.
  let viewPattern = $derived(
    pattern && (component === "rhcp" || component === "lhcp") ? { ...pattern, gain: pattern[component] } : pattern,
  );
  let first = $derived(result?.inputs[0] ?? null);
  let findings = $derived([...(analysis?.findings ?? [])].sort((a, b) => rank(b.severity) - rank(a.severity)));
  // Counted per kind of check, as the list groups them.
  let counts = $derived.by(() => {
    const worst = new Map<string, string>();
    for (const f of findings) {
      const w = worst.get(f.kind);
      if (!w || rank(f.severity) > rank(w)) worst.set(f.kind, f.severity);
    }
    const v = [...worst.values()];
    return { error: v.filter((x) => x === "error").length, warning: v.filter((x) => x === "warning").length, info: v.filter((x) => x === "info").length };
  });

  // The sweep, as chart series.
  let swept = $derived(sweep.points.filter((p) => p.metrics).map((p) => p.metrics!));
  let sweepX = $derived(swept.map((m) => m.mhz));
  let impedanceSeries = $derived<Series[]>([
    { label: "SWR", values: swept.map((m) => Math.min(m.swr, 20)), color: "#ffae45" },
    { label: "R Ω", values: swept.map((m) => m.z[0]), color: "#58b7ff", axis: "right", digits: 1 },
    { label: "X Ω", values: swept.map((m) => m.z[1]), color: "#b07cff", axis: "right", dash: true, digits: 1 },
  ]);
  let sweptGain = $derived(swept.some((m) => m.gain != null));
  let gainSeries = $derived<Series[]>([
    { label: "Gain dBi", values: swept.map((m) => m.gain), color: "#6fd39a" },
    { label: "F/B dB", values: swept.map((m) => m.front_to_back), color: "#ff7a8c", axis: "right", dash: true, digits: 1 },
    { label: "F/R dB", values: swept.map((m) => m.front_to_rear), color: "#c9a0ff", axis: "right", dash: true, digits: 1 },
  ]);
  // The sweep's patterns (and currents) for play-back.
  let frames = $derived(sweep.points.filter((p) => p.frame).map((p) => ({ mhz: p.x, frame: p.frame! })));
  let playFrame = $derived(playing != null ? (frames[playing] ?? null) : null);
  let viewCurrents = $derived(
    playFrame?.frame.currents && analysis && playFrame.frame.currents.length === analysis.segments.length
      ? playFrame.frame.currents
      : (result?.currents ?? null),
  );
  // Plain-language readings: the SWR verdict, and what the sweep says about resonance and
  // bandwidth.
  let verdict = $derived(first ? swrVerdict(first.swr) : null);
  let resonant = $derived(result && swept.length > 2 ? resonanceNear(swept, result.mhz) : null);
  let span2 = $derived(swept.length > 2 ? swrSpan(swept, 2) : null);
  let covered = $derived(span2 && band ? coverage(span2, band) : null);
  let trimTo = $derived(result && swept.length > 2 ? trim(swept, result.mhz) : null);
  /** The polarization at the peak in words. */
  function polWords(pol: { sense: string; tilt: number; axial_ratio: number }): string {
    if (pol.sense === "none") return "–";
    if (pol.sense === "linear") {
      const t = Math.abs(pol.tilt);
      return t < 20 ? "Vertical" : t > 70 ? "Horizontal" : `Slanted ${pol.tilt.toFixed(0)}°`;
    }
    const hand = pol.sense === "right" ? "Right-hand" : "Left-hand";
    return pol.axial_ratio > 0.9 ? `${hand} circular` : `${hand} elliptical (AR ${pol.axial_ratio.toFixed(2)})`;
  }
  const f3 = (v: number) => v.toFixed(v < 30 ? 3 : 2);
  let sweepError = $derived(sweep.points.find((p) => p.error)?.error ?? null);
  const swrRange = (_min: number, max: number): [number, number] => [1, Math.max(3, Math.min(20, Math.ceil(max)))];

  const isNumber = (f: string) => /^[-+]?(\d+\.?\d*|\.\d+)([eE][-+]?\d+)?$/.test(f.trim());

  /** Whether the selected wire's ends can be dragged in 3-D: a GW card with numeric end points
   * whose segments sit where the card puts them, up to a uniform GS scale. */
  let drag = $derived.by((): { handles: [[number, number, number], [number, number, number]]; scale: number } | { note: string } | null => {
    const card = geo.cards.find((c) => c.line === selectedLine);
    if (!card || card.mnemonic !== "GW" || !analysis) return null;
    const segs = analysis.segments.filter((sg) => sg.origin === card.line);
    if (!segs.length) return null;
    if (!card.fields.slice(2, 8).every(isNumber) || card.fields.length < 8)
      return { note: "Its end points are written with variables, so edit them here rather than by dragging." };
    const c = card.values.slice(2, 8).map((v) => v ?? 0);
    const w = [...segs[0].a, ...segs[segs.length - 1].b];
    let num = 0;
    let den = 0;
    w.forEach((x, k) => {
      num += x * c[k];
      den += c[k] * c[k];
    });
    const scale = den > 0 ? num / den : 1;
    const size = Math.max(...w.map(Math.abs), 1e-9);
    if (w.some((x, k) => Math.abs(x - scale * c[k]) > 1e-6 * size))
      return { note: "A later GM, GX or GR card moves this wire, so edit it here rather than by dragging." };
    return { handles: [segs[0].a, segs[segs.length - 1].b], scale };
  });

  async function dragEnd(end: 0 | 1, point: [number, number, number]) {
    const card = geo.cards.find((c) => c.line === selectedLine);
    if (!card || !drag || !("handles" in drag)) return;
    const fields = [...card.fields];
    const fmt = (v: number) => (Math.abs(v) < 1e-12 ? "0" : String(Number(v.toPrecision(7))));
    point.forEach((x, k) => (fields[2 + 3 * end + k] = fmt(x / drag.scale)));
    try {
      replaceText(await api.replaceCard(text, card.line, card.mnemonic, fields));
    } catch (e) {
      status = String(e);
    }
  }

  function rank(s: string) {
    return s === "error" ? 2 : s === "warning" ? 1 : 0;
  }
  const round = (v: number, d = 3) => Math.round(v * 10 ** d) / 10 ** d;
  const sign = (v: number) => (v < 0 ? "−" : "+");
  const fmtW = (w: number) => (Math.abs(w) >= 1 ? `${w.toFixed(3)} W` : `${(w * 1e3).toFixed(3)} mW`);

  function setSweepRange(a: Analysis) {
    const f = a.frequencies_mhz;
    const b = bandOf(f[0]);
    if (f.length > 1) {
      sweepStart = round(f[0]);
      sweepStop = round(f[f.length - 1]);
    } else if (b) {
      // Across the band the frequency is in.
      sweepStart = b.lo;
      sweepStop = b.hi;
    } else if (f.length) {
      sweepStart = round(f[0] * 0.97);
      sweepStop = round(f[0] * 1.03);
    }
  }

  /** Look at a band: solve at its middle and sweep it edge to edge. */
  function chooseBand(name: string) {
    const b = BANDS.find((x) => x.name === name);
    if (!b) return;
    mhz = centre(b);
    sweepStart = b.lo;
    sweepStop = b.hi;
    void refresh(false);
  }

  let seq = 0;
  let solvedText = "";
  /** Re-read the deck, re-solve it, and (when cheap or asked) re-sweep it. */
  async function refresh(newModel = false, resweep = true) {
    const my = ++seq;
    let a: Analysis;
    try {
      a = await api.analyze(text, path, { ...modelOptions, mhz });
    } catch (e) {
      if (my === seq) {
        parseError = String(e);
        status = "The deck has errors";
      }
      return;
    }
    if (my !== seq) return;
    analysis = a;
    parseError = null;
    api.geometryCards(text).then(
      (g) => {
        if (my === seq) geo = g;
      },
      () => {},
    );
    if (newModel || mhz == null) {
      const r = restored;
      restored = null;
      mhz = r?.frequency_mhz ?? a.frequencies_mhz[0] ?? null;
      if (r?.sweep) {
        sweepStart = r.sweep.start_mhz;
        sweepStop = r.sweep.stop_mhz;
        sweepSteps = r.sweep.steps;
        sweepPattern = r.sweep.pattern;
      } else {
        setSweepRange(a);
      }
    }
    // A run built on a numerical Green's function can only be solved at its frequency.
    if (a.ngf.locked_mhz != null) {
      mhz = a.ngf.locked_mhz;
    }
    // Auto-segmented for another frequency than the one to solve: the geometry must match it.
    if (a.segmented_for != null && mhz != null && a.segmented_for !== mhz) {
      try {
        a = await api.analyze(text, path, { ...modelOptions, mhz });
      } catch (e) {
        if (my === seq) parseError = String(e);
        return;
      }
      if (my !== seq) return;
      analysis = a;
    }
    if (a.unsolvable) {
      result = null;
      solveError = a.unsolvable;
      status = "Cannot solve this deck yet";
      return;
    }
    busy = true;
    status = `Solving ${a.segments.length} segments…`;
    try {
      const r = await api.solve(text, mhz, z0, path, modelOptions);
      if (my !== seq) return;
      if (result && (r.mhz !== result.mhz || text !== solvedText)) nearField = null;
      result = r;
      solvedText = text;
      solveError = null;
      status = `Solved ${a.segments.length} segments${a.patches.length ? ` and ${a.patches.length} patches` : ""} at ${r.mhz.toFixed(4)} MHz in ${r.millis.toFixed(0)} ms`;
      if (r.wrote_ngf) status += ` · wrote ${r.wrote_ngf}`;
      if (newModel && r.pattern) azimuth = Math.round(r.pattern.peak_azimuth);
    } catch (e) {
      if (my !== seq) return;
      solveError = String(e);
      status = "Solve failed";
    } finally {
      if (my === seq) busy = false;
    }
    if (resweep && (newModel || a.segments.length <= AUTO_SWEEP_SEGMENTS)) void runSweep();
  }

  async function runSweep() {
    if (!analysis || analysis.unsolvable || !analysis.sources.length || analysis.ngf.locked_mhz != null) return;
    // A sweep runs upward from a positive start.
    if (!(sweepStart > 0) || !(sweepStop > 0)) {
      status = "The sweep's frequencies must be more than 0 MHz.";
      return;
    }
    if (sweepStart > sweepStop) [sweepStart, sweepStop] = [sweepStop, sweepStart];
    if (sweepStart === sweepStop) sweepStop = round(sweepStart * 1.01);
    sweeping = true;
    const t0 = performance.now();
    try {
      await api.sweep(text, sweepStart, sweepStop, Math.max(2, Math.round(sweepSteps)), {
        z0,
        pattern: sweepPattern ? { azimuth, elevation: null } : null,
        model: modelOptions,
        // Keep each point's cuts for play-back, and the currents while that stays small.
        capture: sweepPattern
          ? analysis.segments.length * Math.round(sweepSteps) <= 2_000_000
            ? "patterns_and_currents"
            : "patterns"
          : "none",
      });
      status = `Swept ${Math.round(sweepSteps)} frequencies in ${((performance.now() - t0) / 1000).toFixed(1)} s`;
    } catch (e) {
      status = `Sweep failed: ${e}`;
    } finally {
      sweeping = false;
    }
  }

  async function cancelSweep() {
    await api.cancel("sweep");
    sweeping = false;
  }

  /** The frequency box: a positive number, or back to what it was. */
  let lastMhz: number | null = null;
  function frequencyTyped() {
    if (mhz == null || !Number.isFinite(mhz) || mhz <= 0) {
      mhz = lastMhz;
      status = "The frequency must be more than 0 MHz.";
      return;
    }
    lastMhz = mhz;
    void refresh(false, false);
  }
  $effect(() => {
    if (mhz != null && mhz > 0) lastMhz = mhz;
  });

  /** Step 3: go where the antenna's dimensions are changed. */
  function adjust() {
    const knobs = document.getElementById("knobs");
    if (knobs) {
      knobs.scrollIntoView({ behavior: "smooth", block: "start" });
      knobs.querySelector<HTMLInputElement>("input")?.focus();
    } else {
      tab = "geometry";
    }
  }

  /** A fix that needs the antenna higher: the height knob, or the Shape tab. */
  function raiseKnob() {
    const el = [...document.querySelectorAll<HTMLInputElement>("#knobs input[data-knob]")].find((i) => /height|base|apex|bottom/i.test(i.dataset.knob ?? ""));
    if (el) {
      el.scrollIntoView({ behavior: "smooth", block: "center" });
      el.focus();
      el.select();
    } else {
      tab = "geometry";
    }
  }

  /** Jump the solution to a frequency picked on the sweep chart. */
  function solveAt(f: number) {
    mhz = round(f, 4);
    void refresh(false, false);
  }

  // ---- Undo and redo. Every change to the deck is a step: knobs, editors, fixes, the optimizer,
  // matching. Typing in the deck text counts as one step per pause.
  let past = $state<string[]>([]);
  let future = $state<string[]>([]);
  /** The deck as of the last step. */
  let shadow = "";
  const HISTORY = 100;
  /** Record the deck's change since the last step, if any. */
  function step() {
    if (text === shadow) return;
    past = [...past.slice(-(HISTORY - 1)), shadow];
    future = [];
    shadow = text;
  }
  /** Start the history over (a model loaded or created). */
  function resetHistory() {
    past = [];
    future = [];
    shadow = text;
  }
  function undo() {
    if (!past.length) return;
    step();
    future = [text, ...future];
    text = past[past.length - 1];
    past = past.slice(0, -1);
    shadow = text;
    dirty = true;
    void refresh(false);
  }
  function redo() {
    if (!future.length) return;
    past = [...past, text];
    text = future[0];
    future = future.slice(1);
    shadow = text;
    dirty = true;
    void refresh(false);
  }

  /** Replace the deck with an edited version (a knob, an editor, a fix, the optimizer). */
  function replaceText(t: string) {
    if (t === text) return;
    step();
    text = t;
    step();
    dirty = true;
    void refresh(false);
  }

  async function load(p: string) {
    if (!(await discardOk())) return;
    let proj: Project;
    try {
      proj = await api.openModel(p);
    } catch (e) {
      status = String(e);
      return;
    }
    text = proj.deck;
    meta = { name: proj.name, notes: proj.notes };
    if (isProject(p)) {
      z0 = proj.settings.z0;
      units = proj.settings.units;
      autoseg = proj.settings.model?.autoseg != null;
      segsPerHalfWave = proj.settings.model?.autoseg ?? segsPerHalfWave;
      stepRadius = proj.settings.model?.step_radius ?? false;
    }
    restored = proj.settings;
    optimizer?.restore(proj.settings.optimizer);
    path = p;
    dirty = false;
    resetHistory();
    selectedLine = null;
    nearField = null;
    loads++;
    result = null;
    sweep.clear();
    highlight = null;
    await refresh(true);
  }

  /** Whether unsaved edits may be dropped: true when there are none, else the user's answer. */
  async function discardOk(): Promise<boolean> {
    if (!dirty) return true;
    return ask(`${fileName} has unsaved changes. Discard them?`, { title: "NEC Studio", kind: "warning" });
  }

  /** Start an unsaved model from a deck (a template). */
  async function newModel(deck: string) {
    if (!(await discardOk())) return;
    browsing = false;
    text = deck;
    path = null;
    meta = { name: "", notes: "" };
    restored = null;
    optimizer?.restore(null);
    dirty = true;
    resetHistory();
    selectedLine = null;
    nearField = null;
    loads++;
    result = null;
    sweep.clear();
    highlight = null;
    await refresh(true);
  }

  async function openFile() {
    const p = await open({
      multiple: false,
      directory: false,
      filters: [
        { name: "Models and projects", extensions: ["nec", "NEC", "toml"] },
        { name: "NEC decks", extensions: ["nec", "NEC"] },
        { name: "NEC Studio projects", extensions: ["toml"] },
      ],
    });
    if (typeof p === "string") await load(p);
  }

  /** The model and its working settings, as a project file holds them. */
  function project(): Project {
    return {
      format: 1,
      name: meta.name,
      notes: meta.notes,
      deck: text,
      settings: {
        frequency_mhz: mhz,
        z0,
        units,
        sweep: { start_mhz: sweepStart, stop_mhz: sweepStop, steps: Math.round(sweepSteps), pattern: sweepPattern },
        optimizer: optimizer?.settings() ?? null,
        model: modelOptions,
      },
    };
  }

  /** Save to the current file, or choose one. A `.toml` path saves the whole project; any other
   * exports just the deck, which is all a `.nec` file can hold. */
  async function saveFile(as = false) {
    let p = path;
    if (!p || as) {
      const projectFirst = !p || isProject(p);
      const filters = [
        { name: "NEC Studio project", extensions: ["toml"] },
        { name: "NEC deck", extensions: ["nec"] },
      ];
      const base = (fileName ?? "untitled").replace(/\.nec\.toml$|\.toml$|\.nec$/i, "");
      const chosen = await save({
        defaultPath: projectFirst ? `${base}.nec.toml` : `${base}.nec`,
        filters: projectFirst ? filters : filters.reverse(),
      });
      if (!chosen) return;
      p = chosen;
    }
    try {
      await api.saveModel(p, project());
    } catch (e) {
      status = `Save failed: ${e}`;
      return;
    }
    path = p;
    dirty = false;
    status = isProject(p) ? `Saved project ${p}` : `Exported deck ${p}`;
  }

  let timer: ReturnType<typeof setTimeout> | undefined;
  function edited() {
    dirty = true;
    clearTimeout(timer);
    timer = setTimeout(() => {
      step();
      void refresh(false);
    }, 350);
  }

  function onKey(e: KeyboardEvent) {
    if (!(e.ctrlKey || e.metaKey)) return;
    // Typing fields keep their own undo; elsewhere Ctrl+Z / Ctrl+Y undo and redo deck changes.
    const t = e.target as HTMLElement | null;
    const typing = !!t && (t.tagName === "TEXTAREA" || (t.tagName === "INPUT" && (t as HTMLInputElement).type !== "checkbox" && (t as HTMLInputElement).type !== "range"));
    if (!typing && e.key.toLowerCase() === "z" && !e.shiftKey) {
      e.preventDefault();
      undo();
      return;
    }
    if (!typing && (e.key.toLowerCase() === "y" || (e.key.toLowerCase() === "z" && e.shiftKey))) {
      e.preventDefault();
      redo();
      return;
    }
    if (e.key === "o") {
      e.preventDefault();
      void openFile();
    } else if (e.key === "s") {
      e.preventDefault();
      void saveFile(e.shiftKey);
    }
  }

  onMount(() => {
    const off = api.onSweepPoint((e) => sweep.accept(e));
    resetHistory();
    (async () => {
      const start = await api.startupFile();
      if (start) await load(start);
      else {
        await refresh(true);
        // First run: offer the templates, with a word of welcome.
        if (!prefs.welcomed) {
          welcome = true;
          browsing = true;
        }
      }
    })();
    return () => void off.then((f) => f());
  });
</script>

<svelte:window onkeydown={onKey} />

{#if browsing}
  <ModelBrowser
    onopen={(p) => {
      browsing = false;
      welcome = false;
      void load(p);
    }}
    onnew={(deck) => {
      welcome = false;
      void newModel(deck);
    }}
    onclose={() => {
      browsing = false;
      welcome = false;
    }}
    {welcome}
    {units}
    {mhz}
  />
{/if}

<div class="app">
  <header>
    <div class="brand" title="NEC Studio"><span class="logo">◉</span> <span class="name">NEC Studio</span></div>
    <div class="step">
      <span class="n" use:tip={hint("stepAntenna")}>1</span>
      <button onclick={() => (browsing = true)} use:tip={hint("newModel")}>New…</button>
      <button onclick={openFile} use:tip={hint("openModel")}>Open…</button>
      <button onclick={() => saveFile(false)} use:tip={hint("saveModel")}>Save</button>
      <button onclick={() => saveFile(true)} use:tip={hint("saveAs")}>Save as…</button>
      <button class="icon" onclick={undo} disabled={!past.length} use:tip={"Undo the last change to the model (Ctrl+Z)"} aria-label="Undo">↶</button>
      <button class="icon" onclick={redo} disabled={!future.length} use:tip={"Redo (Ctrl+Y)"} aria-label="Redo">↷</button>
      <span class="file" title={path ?? "Not saved yet"}>{fileName}{dirty ? " •" : ""}</span>
    </div>
    <div class="step">
      <span class="n" use:tip={hint("stepBand")}>2</span>
      <label class="field" use:tip={hint("stepBand")}>
        Band
        <select value={band?.name ?? ""} onchange={(e) => chooseBand((e.currentTarget as HTMLSelectElement).value)} disabled={!analysis || analysis.ngf.locked_mhz != null}>
          {#if !band}<option value="">{mhz != null ? "other" : "–"}</option>{/if}
          {#each BANDS as b}<option value={b.name}>{b.name}</option>{/each}
        </select>
      </label>
      <label class="field" use:tip={hint("frequency")}>
        <input type="number" step="0.001" min="0.001" bind:value={mhz} onchange={frequencyTyped} disabled={!analysis || analysis.ngf.locked_mhz != null} />
        MHz
      </label>
      {#if mhz != null && !band}<span class="offband" use:tip={"This frequency is outside the US amateur bands. That's fine for experiments; choose a band to go back to one."}>not a ham band</span>{/if}
    </div>
    <div class="step">
      <span class="n" use:tip={hint("stepAdjust")}>3</span>
      <button onclick={adjust} use:tip={hint("stepAdjust")}>Adjust</button>
    </div>
    <div class="step">
      <span class="n" use:tip={hint("stepResults")}>4</span>
      <button onclick={() => (tab = "results")} use:tip={hint("stepResults")}>Results</button>
    </div>
    <div class="sep"></div>
    <label class="field" use:tip={hint("sweep")}>Sweep <input type="number" step="0.001" class="mid" bind:value={sweepStart} /></label>
    <label class="field">to <input type="number" step="0.001" class="mid" bind:value={sweepStop} /> MHz</label>
    {#if sweeping}
      <button class="warn" onclick={cancelSweep}>Cancel</button>
    {:else}
      <button class="primary" onclick={runSweep} disabled={analysis?.ngf.locked_mhz != null} use:tip={hint("sweep")}>Sweep</button>
    {/if}
    <SettingsMenu bind:z0 bind:units bind:sweepSteps bind:autoseg bind:segsPerHalfWave bind:stepRadius onchange={() => refresh(false)} />
    <div class="status" class:busy={busy || sweeping} use:tip={hint("status")}>{status}</div>
  </header>

  <aside class="left">
    <Knobs {analysis} {text} {units} resetKey={loads} onedit={replaceText} />
    <section class="checks">
      <div class="head">
        <h2>Model checks</h2>
        <Help term="checks" />
        <span class="counts">
          {#if counts.error}<span class="sev error">{counts.error} problem{counts.error > 1 ? "s" : ""}</span>{/if}
          {#if counts.warning}<span class="sev warning">{counts.warning} worth a look</span>{/if}
          {#if counts.info}<span class="sev info">{counts.info} note{counts.info > 1 ? "s" : ""}</span>{/if}
        </span>
      </div>
      <ChecksList
        {findings}
        {analysis}
        {text}
        {parseError}
        {solveError}
        {autoseg}
        bind:highlight
        onedit={replaceText}
        onautoseg={() => {
          autoseg = true;
          void refresh(false);
        }}
        onraise={raiseKnob}
      />
      {#if analysis?.autoseg_log.length}
        <details class="log">
          <summary>Auto-segmentation at {analysis.segmented_for} MHz: {analysis.autoseg_log.length} wire{analysis.autoseg_log.length > 1 ? "s" : ""} refined</summary>
          <ul>
            {#each analysis.autoseg_log as c}<li>line {c.line}, tag {c.tag}: {c.from} → {c.to} segments</li>{/each}
          </ul>
        </details>
      {/if}
      {#if stepRadius && analysis?.steprad_log.length}
        <details class="log">
          <summary>Stepped-radius correction</summary>
          <ul>
            {#each analysis.steprad_log as l}<li>{l}</li>{/each}
          </ul>
        </details>
      {/if}
    </section>
    <section class="editor" class:closed={!deckOpen}>
      <div class="head">
        <button class="disclose" aria-expanded={deckOpen} onclick={() => (deckOpen = !deckOpen)} use:tip={deckOpen ? "Hide the deck text" : "Show the model as NEC cards, to read or type directly"}>
          <span class="chev">{deckOpen ? "▾" : "▸"}</span>
          <h2>Deck <span class="adv">(advanced)</span></h2>
        </button>
        <Help term="deck" />
        {#if analysis?.lowered}<span class="pill" title="4nec2 extension cards were translated to NEC-2">4nec2</span>{/if}
      </div>
      {#if deckOpen}
        <textarea bind:value={text} oninput={edited} spellcheck="false" autocomplete="off"></textarea>
      {/if}
    </section>
  </aside>

  <main>
    <Viewer3D
      {analysis}
      currents={viewCurrents}
      patchCurrents={result?.patch_currents ?? null}
      pattern={viewPattern}
      {highlight}
      selected={selectedLine}
      fitKey={loads}
      {nearField}
      handles={drag && "handles" in drag ? drag.handles : null}
      onpick={(line) => {
        selectedLine = line;
        if (line != null) tab = "geometry";
      }}
      ondrag={dragEnd}
    />
  </main>

  <aside class="right">
    <nav class="tabs">
      <button class:on={tab === "results"} onclick={() => (tab = "results")} use:tip={hint("tabResults")}>Results</button>
      <button class:on={tab === "geometry"} onclick={() => (tab = "geometry")} use:tip={hint("tabShape")}>Shape</button>
      <button class:on={tab === "model"} onclick={() => (tab = "model")} use:tip={hint("tabFeed")}>Feed &amp; ground</button>
      <button class:on={tab === "match"} onclick={() => (tab = "match")} use:tip={hint("tabMatch")}>Match</button>
      <button class:on={tab === "optimize"} onclick={() => (tab = "optimize")} use:tip={hint("tabTune")}>Tune</button>
      <button class:on={tab === "data"} onclick={() => (tab = "data")} use:tip={hint("tabData")}>Checks &amp; data</button>
    </nav>

    <div class="pane" class:hidden={tab !== "geometry"}>
      <GeometryEditor
        {text}
        {analysis}
        cards={geo.cards}
        insertAt={geo.insert_at}
        {mhz}
        bind:selected={selectedLine}
        dragNote={drag && "note" in drag ? drag.note : null}
        {units}
        onedit={replaceText}
      />
    </div>

    <div class="pane" class:hidden={tab !== "model"}>
      <ProgramEditor {text} {analysis} segment={highlight} onedit={replaceText} />
    </div>

    <div class="pane" class:hidden={tab !== "match"}>
      <MatchPanel {text} {analysis} {result} {z0} {units} onedit={replaceText} />
    </div>

    <div class="pane" class:hidden={tab !== "data"}>
      <ChecksPanel {text} {path} {result} {z0} {azimuth} />
      <DataPanel {analysis} {result} {text} {path} {units} {swept} model={modelOptions} bind:highlight bind:nearField />
    </div>

    <div class="pane" class:hidden={tab !== "results"}>
      {#if analysis && (analysis.ngf.writes || analysis.ngf.reads || analysis.ngf.runs > 1)}
        <section class="card notice">
          {#if analysis.ngf.reads}
            <div>Built on the numerical Green's function <b>{analysis.ngf.reads}</b>
              ({analysis.ngf.base_segments} segments), so it solves only at {analysis.ngf.locked_mhz} MHz.</div>
          {/if}
          {#if analysis.ngf.writes}
            <div>This deck writes the numerical Green's function <b>{analysis.ngf.writes}</b> next to it when solved.</div>
          {/if}
          {#if analysis.ngf.runs > 1}<div>{analysis.ngf.runs} runs (NX); showing the last.</div>{/if}
        </section>
      {/if}
      <section class="card results">
        {#if first && result && verdict}
          <div class="hero">
            <div>
              <div class="label">SWR at {result.mhz.toFixed(3)} MHz against {z0} Ω<Help term="swr" /></div>
              <div class="swrline">
                <span class="big {verdict.level}">{Number.isFinite(first.swr) ? first.swr.toFixed(2) : "∞"}</span>
                <span class="verdict {verdict.level}">{verdict.text}</span>
                <Help term="swrVerdict" />
              </div>
              <div class="zline">
                Impedance {first.z[0].toFixed(1)} {sign(first.z[1])} j{Math.abs(first.z[1]).toFixed(1)} Ω<Help term="impedance" />
              </div>
            </div>
          </div>
          {#if swept.length > 2}
            <ul class="facts">
              <li>
                {#if resonant != null}Resonant at <b>{f3(resonant)} MHz</b>{:else}Not resonant between {f3(swept[0].mhz)} and {f3(swept[swept.length - 1].mhz)} MHz{/if}<Help term="resonance" />
              </li>
              <li>
                {#if span2}
                  SWR 2:1 or better from <b>{f3(span2.lo)} to {f3(span2.hi)} MHz</b>
                  ({((span2.hi - span2.lo) * 1000).toFixed(0)} kHz{span2.openLo || span2.openHi ? ", or more past the end of the sweep" : ""})
                {:else}
                  The SWR stays above 2:1 across the sweep
                {/if}<Help term="bandwidth" />
              </li>
              {#if band}
                <li>
                  {#if covered?.whole}<span class="ok">Covers all of {band.name}</span>
                  {:else if covered}Covers {f3(covered.lo)}–{f3(covered.hi)} of {band.name} ({band.lo}–{band.hi} MHz)
                  {:else}<span class="warn">Doesn't reach 2:1 anywhere in {band.name}</span>{/if}
                </li>
              {/if}
              {#if trimTo}
                <li class="hint">
                  → {trimTo.direction === "lengthen" ? "Lengthen" : "Shorten"} it{Number.isFinite(trimTo.percent) ? ` by about ${trimTo.percent.toFixed(1)} %` : ""}
                  {trimTo.resonance != null ? `to move the resonance from ${trimTo.estimated ? "about " : ""}${f3(trimTo.resonance)} to ${f3(result.mhz)} MHz` : `to bring the resonance toward ${f3(result.mhz)} MHz`}.<Help term="trimHint" />
                </li>
              {/if}
            </ul>
          {/if}
          {#if pattern}
            <p class="sentence">
              {#if pattern.peak_elevation > 70}
                Strongest <b>almost straight up</b> ({pattern.peak_elevation.toFixed(0)}° elevation): high-angle radiation, good for nearby
                contacts<Help term="takeoff" />.
              {:else}
                Strongest toward <b>{pattern.peak_azimuth.toFixed(0)}°</b> azimuth<Help term="direction" />,
                <b>{pattern.peak_elevation.toFixed(0)}°</b> above the horizon<Help term="takeoff" />.
              {/if}
            </p>
          {/if}
          <div class="grid">
            {#if pattern}
              <div><span>Gain<Help term="gain" /></span><b>{pattern.peak_gain.toFixed(2)} dBi <small>({dBd(pattern.peak_gain).toFixed(1)} dBd)</small></b></div>
              <div><span>Front/back<Help term="frontToBack" /></span><b>{pattern.front_to_back.toFixed(1)} dB</b></div>
              <div><span>Polarization<Help term="polarization" /></span><b>{polWords(pattern.polarization)}</b></div>
            {/if}
            <div><span>Efficiency<Help term="efficiency" /></span><b>{result.power.efficiency.toFixed(1)} %</b></div>
          </div>
          <details class="more">
            <summary>More details</summary>
            <div class="grid">
              {#if pattern}
                {@const pol = pattern.polarization}
                <div><span>Front/rear<Help term="frontToRear" /></span><b>{Number.isFinite(pattern.front_to_rear) ? `${pattern.front_to_rear.toFixed(1)} dB` : "–"}</b></div>
                <div><span>Takeoff angle</span><b>{pattern.peak_elevation.toFixed(0)}°</b></div>
                {#if Number.isFinite(pattern.average_gain)}
                  <div><span>Average gain<Help term="averageGain" /></span><b>{(10 * Math.log10(pattern.average_gain)).toFixed(2)} dB</b></div>
                {/if}
                <div><span>Tilt</span><b>{pol.tilt.toFixed(0)}°</b></div>
                <div><span>Vertical</span><b>{pol.vertical < -99 ? "–" : `${pol.vertical.toFixed(2)} dBi`}</b></div>
                <div><span>Horizontal</span><b>{pol.horizontal < -99 ? "–" : `${pol.horizontal.toFixed(2)} dBi`}</b></div>
                <div><span>RHCP</span><b>{pattern.peak_rhcp < -99 ? "–" : `${pattern.peak_rhcp.toFixed(2)} dBi`}</b></div>
                <div><span>LHCP</span><b>{pattern.peak_lhcp < -99 ? "–" : `${pattern.peak_lhcp.toFixed(2)} dBi`}</b></div>
                {#if pol.sense !== "linear" && pol.sense !== "none"}<div><span>Axial ratio</span><b>{pol.axial_ratio.toFixed(2)}</b></div>{/if}
              {/if}
              <div><span>Radiated</span><b>{fmtW(result.power.radiated)}</b></div>
              <div><span>Solve time</span><b>{result.millis.toFixed(0)} ms</b></div>
            </div>
          </details>
          {#if result.inputs.length > 1}
            <table>
              <thead><tr><th>Tag</th><th>Seg</th><th>Z (Ω)</th><th>SWR</th><th>Power</th></tr></thead>
              <tbody>
                {#each result.inputs as i}
                  <tr>
                    <td>{i.tag}</td><td>{i.segment}</td>
                    <td>{i.z[0].toFixed(1)} {sign(i.z[1])} j{Math.abs(i.z[1]).toFixed(1)}</td>
                    <td>{i.swr.toFixed(2)}</td>
                    <td>{fmtW(i.power)}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
            <p class="note">The figures above are for the first source; the sweep and the optimizer use it too.</p>
          {/if}
        {:else if result}
          <div class="label">Solved at {result.mhz.toFixed(4)} MHz — the model has no voltage source, so there is no SWR or impedance to show. Add one on the Feed &amp; ground tab.</div>
        {:else}
          <div class="label">{busy ? "Solving…" : "No solution yet."}</div>
        {/if}
      </section>

      <section class="card">
        <div class="cardhead">
          <span class="label">Sweep — click to solve at a frequency</span>
          <label use:tip={hint("sweepGain")}><input type="checkbox" bind:checked={sweepPattern} /> gain &amp; F/B</label>
          <label use:tip={hint("sweepAz")}>az <input type="number" class="tiny" bind:value={azimuth} />°</label>
        </div>
        <LineChart x={sweepX} series={impedanceSeries} marker={playFrame?.mhz ?? result?.mhz ?? null} leftRange={swrRange} xLabel="MHz" onpick={solveAt} logX={logF} logLeft={logSwr} {smooth} />
        {#if sweptGain}
          <LineChart x={sweepX} series={gainSeries} marker={playFrame?.mhz ?? result?.mhz ?? null} height={150} xLabel="MHz" onpick={solveAt} logX={logF} {smooth} />
        {/if}
        <div class="chartopts">
          <label use:tip={hint("logFrequency")}><input type="checkbox" bind:checked={logF} /> log frequency</label>
          <label use:tip={hint("logSwr")}><input type="checkbox" bind:checked={logSwr} /> log SWR</label>
          <label use:tip={hint("smooth")}><input type="checkbox" bind:checked={smooth} /> smooth</label>
        </div>
        {#if sweepError}<p class="sweeperror">{sweepError}</p>{/if}
      </section>

      <section class="card">
        <PatternPanel
          {pattern}
          mhz={result?.mhz ?? null}
          name={fileName ?? "model"}
          half={analysis?.has_ground ?? false}
          {frames}
          bind:component
          bind:split={splitComponents}
          bind:playing
        />
      </section>
      <section class="card smith" use:tip={hint("smith")}>
        <Smith points={swept} marker={first ? first.z : null} {z0} />
      </section>
    </div>

    <div class="pane" class:hidden={tab !== "optimize"}>
      <Optimizer bind:this={optimizer} {analysis} {text} {z0} bind:azimuth band={[sweepStart, sweepStop]} model={modelOptions} onapply={replaceText} />
      <ParamSweep {analysis} {text} {mhz} {z0} {azimuth} model={modelOptions} onapply={replaceText} />
    </div>
  </aside>
</div>

<style>
  .app {
    display: grid;
    grid-template-columns: 340px minmax(0, 1fr) 440px;
    grid-template-rows: auto minmax(0, 1fr);
    grid-template-areas: "top top top" "left main right";
    height: 100vh;
    gap: 10px;
    padding: 0 10px 10px;
  }
  header {
    grid-area: top;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px 8px;
    padding: 8px 4px;
    border-bottom: 1px solid var(--line);
    margin: 0 -10px;
    padding-inline: 14px;
    background: var(--panel);
  }
  .brand {
    font-weight: 650;
    letter-spacing: 0.2px;
    white-space: nowrap;
  }
  header :global(button),
  header label {
    white-space: nowrap;
  }

  .logo {
    color: var(--accent);
  }
  /* On narrower windows the header keeps to one row by dropping the name. */
  @media (max-width: 1640px) {
    .brand .name {
      display: none;
    }
  }
  .file {
    color: var(--muted);
    max-width: 150px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .sep {
    width: 1px;
    height: 22px;
    background: var(--line);
  }
  .field {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--muted);
    font-size: 12px;
  }
  .field input {
    width: 86px;
  }
  .status {
    margin-left: auto;
    flex: 1 1 100px;
    min-width: 0;
    text-align: right;
    color: var(--muted);
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .status.busy {
    color: var(--accent);
  }
  aside {
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .left {
    grid-area: left;
  }
  .right {
    grid-area: right;
    gap: 8px;
  }
  .tabs {
    display: flex;
    gap: 4px;
    border-bottom: 1px solid var(--line);
  }
  .tabs button {
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    border-radius: 0;
    padding: 5px 7px 7px;
    color: var(--muted);
    white-space: nowrap;
  }
  .tabs button.on {
    color: var(--text);
    border-bottom-color: var(--accent);
  }
  .pane {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding-right: 2px;
  }
  .pane.hidden {
    display: none;
  }
  .notice {
    border-color: #2b4a6b;
    background: #12202f;
    font-size: 12px;
    color: #a9c8ea;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .cardhead {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 4px;
    font-size: 12px;
    color: var(--muted);
  }
  .cardhead .label {
    margin-right: auto;
  }
  .cardhead label {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  input.tiny {
    width: 52px;
    padding: 2px 5px;
  }
  .sweeperror {
    color: #ff8a9a;
    font-size: 12px;
    margin: 4px 0 0;
  }
  main {
    grid-area: main;
    min-height: 0;
  }
  .head {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 6px;
  }
  h2 {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.8px;
    color: var(--muted);
    margin: 0;
    font-weight: 600;
  }
  .editor {
    flex: 3;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .editor.closed {
    flex: none;
  }
  .disclose {
    all: unset;
    display: flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
  }
  .disclose:focus-visible {
    outline: 1px solid var(--accent-dim);
  }
  .chev {
    color: var(--muted);
    font-size: 11px;
    width: 10px;
  }
  .adv {
    text-transform: none;
    letter-spacing: 0;
    font-weight: 400;
  }
  .step {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .step .n {
    display: inline-grid;
    place-items: center;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--accent-dim);
    color: #fff;
    font-size: 11px;
    font-weight: 700;
    cursor: help;
  }
  .field input.mid {
    width: 72px;
  }
  .offband {
    font-size: 11px;
    color: #ffcf6a;
    white-space: nowrap;
    cursor: help;
  }
  header .icon {
    padding: 4px 8px;
  }
  textarea {
    flex: 1;
    resize: none;
    font-family: "Cascadia Mono", Consolas, monospace;
    font-size: 12.5px;
    line-height: 1.5;
    background: var(--panel);
    color: var(--text);
    border: 1px solid var(--line);
    border-radius: 8px;
    padding: 10px;
    white-space: pre;
    tab-size: 8;
  }
  textarea:focus {
    outline: 1px solid var(--accent-dim);
  }
  .pill {
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 8px;
    background: #2b3a52;
    color: #9cc2ff;
  }
  .checks {
    flex: 1.3;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .counts {
    margin-left: auto;
    display: flex;
    gap: 6px;
  }
  .sev {
    font-size: 11px;
    padding: 1px 7px;
    border-radius: 8px;
  }
  .sev.error {
    background: #4a1d24;
    color: #ff8a9a;
  }
  .sev.warning {
    background: #4a3a17;
    color: #ffcf6a;
  }
  .sev.info {
    background: #1d3049;
    color: #8fc0ff;
  }
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
  }
  details.log {
    margin-top: 6px;
    font-size: 12px;
    color: var(--muted);
  }
  details.log summary {
    cursor: pointer;
    padding: 2px 0;
  }
  details.log ul {
    max-height: 140px;
    flex: none;
  }
  details.log li {
    padding: 4px 8px;
    line-height: 1.35;
  }
  .finding {
    all: unset;
    display: block;
    width: 100%;
    box-sizing: border-box;
    padding: 6px 8px;
    cursor: pointer;
    line-height: 1.35;
  }
  .finding:hover,
  .finding.active {
    background: #1c2533;
  }
  li b {
    color: var(--text);
    margin-right: 4px;
  }
  .card {
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 8px;
    padding: 10px 12px;
  }
  .card.smith {
    display: flex;
    justify-content: center;
  }
  .card.smith > :global(*) {
    width: 60%;
  }
  .chartopts {
    display: flex;
    gap: 12px;
    font-size: 11px;
    color: var(--muted);
    margin-top: 4px;
  }
  .label {
    font-size: 11px;
    color: var(--muted);
  }
  .hero {
    margin-bottom: 8px;
  }
  .swrline {
    display: flex;
    align-items: baseline;
    gap: 10px;
  }
  .big {
    font-size: 30px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }
  .verdict {
    font-size: 13px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 10px;
  }
  .big.excellent,
  .big.good {
    color: #6fd39a;
  }
  .big.tuner {
    color: #ffcf6a;
  }
  .big.poor {
    color: #ff7a8c;
  }
  .verdict.excellent,
  .verdict.good {
    background: #173a2a;
    color: #8ff0b8;
  }
  .verdict.tuner {
    background: #4a3a17;
    color: #ffcf6a;
  }
  .verdict.poor {
    background: #4a1d24;
    color: #ff8a9a;
  }
  .zline {
    font-size: 13px;
    color: var(--muted);
    margin-top: 2px;
  }
  .facts {
    list-style: none;
    margin: 0 0 8px;
    padding: 8px 10px;
    border: 1px solid var(--line);
    border-radius: 8px;
    background: #0f1620;
    font-size: 12.5px;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .facts .hint {
    color: #ffcf6a;
  }
  .facts .ok {
    color: #6fd39a;
  }
  .facts .warn {
    color: #ff8a9a;
  }
  .sentence {
    margin: 0 0 6px;
    font-size: 12.5px;
  }
  .more {
    margin-top: 6px;
    font-size: 12px;
  }
  .more summary {
    cursor: pointer;
    color: var(--muted);
    padding: 2px 0 4px;
  }
  .grid small {
    color: var(--muted);
    font-weight: 400;
  }
  .grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 4px 16px;
    font-size: 12.5px;
  }
  .grid div {
    display: flex;
    justify-content: space-between;
    border-bottom: 1px dashed #222c3b;
    padding: 3px 0;
  }
  .grid span {
    color: var(--muted);
  }
  .grid b {
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  table {
    width: 100%;
    margin-top: 10px;
    font-size: 12px;
    border-collapse: collapse;
  }
  th {
    text-align: left;
    color: var(--muted);
    font-weight: 500;
  }
  td,
  th {
    padding: 2px 4px;
  }
</style>
