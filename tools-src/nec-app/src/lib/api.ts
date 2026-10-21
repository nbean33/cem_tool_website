// Typed wrappers over NEC Studio's engine commands: the website build.
//
// This replaces nec-rs's app/src/lib/api.ts. Every export keeps its name, signature and
// types, so the components are unchanged; only the transport differs. Where the desktop app
// calls Tauri's `invoke` and `listen`, this sends the same command name and argument object to
// a Web Worker running the nec-wasm build of the same backend (see worker.ts).
//
// Workers ("lanes"): interactive commands share one worker. Each kind of long job gets its own,
// as each ran on its own thread on the desktop, so a sweep never blocks the live re-solve.
// Starting a job cancels the previous job of its kind, and `cancel` stops it, by terminating
// that worker: a computation on a worker's only thread cannot see a flag set from outside.
import type {
  Analysis,
  EvalOptions,
  Metrics,
  ModelOptions,
  OptimizeOutcome,
  OptimizeProgress,
  OptimizeSpec,
  Project,
  ModelFile,
  Slice,
  SliceRequest,
  SolveView,
  SweepEvent,
} from "./types";
import { download, get, under } from "./vfs";

type UnlistenFn = () => void;

// ---- events (Tauri's listen) ------------------------------------------------------------

type Handler = (e: { payload: unknown }) => void;
const handlers = new Map<string, Set<Handler>>();

function listen<T>(event: string, f: (e: { payload: T }) => void): Promise<UnlistenFn> {
  const set = handlers.get(event) ?? new Set<Handler>();
  handlers.set(event, set);
  set.add(f as Handler);
  return Promise.resolve(() => void set.delete(f as Handler));
}

function dispatchEvent(event: string, payload: unknown) {
  handlers.get(event)?.forEach((f) => f({ payload }));
}

// ---- workers ------------------------------------------------------------------------------

type Lane = "interactive" | "checks" | JobKind;

/** Ends a pending call when its worker is terminated: the value to resolve with, or a rejection. */
type OnStop = () => { value: unknown } | { error: string };

interface Pending {
  resolve: (v: unknown) => void;
  reject: (e: unknown) => void;
  onStop?: OnStop;
}

class Engine {
  private worker: Worker | null = null;
  private pending = new Map<number, Pending>();
  private next = 0;

  private spawn(): Worker {
    const w = new Worker(new URL("./worker.ts", import.meta.url), { type: "module" });
    w.onmessage = (e: MessageEvent) => {
      const m = e.data;
      if (m.event) {
        dispatchEvent(m.event, JSON.parse(m.payload));
        return;
      }
      const p = this.pending.get(m.id);
      if (!p) return;
      this.pending.delete(m.id);
      if ("err" in m) p.reject(m.err);
      else p.resolve(JSON.parse(m.ok));
    };
    w.onerror = (e) => {
      // A worker that cannot start (or crashes) fails every call it holds.
      const message = e.message || "the engine stopped unexpectedly";
      this.pending.forEach((p) => p.reject(message));
      this.pending.clear();
      this.worker = null;
    };
    return w;
  }

  get busy() {
    return this.pending.size > 0;
  }

  request<T>(cmd: string, args: unknown, onStop?: OnStop): Promise<T> {
    this.worker ??= this.spawn();
    const id = ++this.next;
    return new Promise<T>((resolve, reject) => {
      this.pending.set(id, { resolve: resolve as (v: unknown) => void, reject, onStop });
      this.worker!.postMessage({ id, cmd, args });
    });
  }

  /** Terminate whatever is running; each pending call ends as its `onStop` says. */
  stop() {
    if (!this.worker) return;
    this.worker.terminate();
    this.worker = null;
    this.pending.forEach((p) => {
      const end = p.onStop?.() ?? { error: "cancelled" };
      if ("value" in end) p.resolve(end.value);
      else p.reject(end.error);
    });
    this.pending.clear();
  }
}

const lanes = new Map<Lane, Engine>();
const lane = (name: Lane) => {
  let e = lanes.get(name);
  if (!e) lanes.set(name, (e = new Engine()));
  return e;
};

/** The desktop numbered jobs from one counter; points of an older job are then ignored. */
let jobCounter = 0;

/** Start a long job of `kind`, replacing the one already running. */
function startJob<T>(kind: JobKind, cmd: string, args: Record<string, unknown>, onStop: (job: number) => OnStop) {
  const engine = lane(kind);
  if (engine.busy) engine.stop();
  const job = ++jobCounter;
  return engine.request<T>(cmd, { ...args, job }, onStop(job));
}

const invoke = <T>(cmd: string, args: unknown = {}) => lane("interactive").request<T>(cmd, args);

// ---- large models -------------------------------------------------------------------------

/** Above this many unknowns a solve takes seconds in the browser's single thread. */
const LARGE_MODEL = 2000;

function noteModelSize(a: Analysis) {
  const n = (a as unknown as { segments: unknown[]; patches?: unknown[] }).segments.length +
    ((a as unknown as { patches?: unknown[] }).patches?.length ?? 0);
  let note = document.getElementById("large-model-note");
  if (n <= LARGE_MODEL) {
    note?.remove();
    return;
  }
  if (!note) {
    note = document.createElement("div");
    note.id = "large-model-note";
    note.setAttribute("role", "status");
    document.body.appendChild(note);
  }
  note.textContent = `${n} unknowns: large models run single-threaded in the browser, so solves take longer here. The desktop app is faster.`;
}

// ---- the commands, as in the desktop api.ts ----------------------------------------------

/** `path` is the deck's file, whose folder holds the numerical Green's functions it reads and writes. */
export const analyze = (text: string, path: string | null, model: ModelOptions) =>
  invoke<Analysis>("analyze", { text, path, model }).then((a) => {
    noteModelSize(a);
    return a;
  });
export const solve = (text: string, mhz: number | null, z0: number, path: string | null, model: ModelOptions) =>
  invoke<SolveView>("solve", { text, mhz, z0, path, model });
export const readText = (path: string) => Promise.resolve().then(() => get(path));
export const writeText = (path: string, text: string) =>
  Promise.resolve().then(() => download(path, text));
/** A project file, or a `.nec` deck imported as one. */
export const openModel = (path: string) =>
  Promise.resolve().then(() => invoke<Project>("open_project", { name: path, text: get(path) }));
/** The whole project to a `.toml` path; only the deck to any other (a `.nec` export). */
export const saveModel = (path: string, project: Project) =>
  invoke<string>("project_text", { name: path, project }).then((text) => download(path, text));
/** The near field on a slice of space. */
export const nearField = (text: string, mhz: number | null, request: SliceRequest, path: string | null, model: ModelOptions) =>
  invoke<Slice>("near_field", { text, mhz, request, path, model });
/** A NEC-style text report of the solution. */
export const report = (text: string, mhz: number | null, path: string | null, model: ModelOptions) =>
  invoke<string>("report", { text, mhz, path, model });
/** There is no 4nec2 install to find in a browser; "Choose folder" picks one to read. */
export const modelsFolder = () => Promise.resolve<string | null>(null);
/** Every deck and project under a folder chosen in this session, with its first comment line. */
export const listModels = (root: string) =>
  Promise.resolve<ModelFile[]>(
    under(root)
      .map(([path, text]) => ({ path, rel: path.slice(root.length + 1), description: firstComment(text) }))
      .sort((a, b) => a.rel.toLowerCase().localeCompare(b.rel.toLowerCase())),
  );
/** A browser page has no command line to open a file from. */
export const startupFile = () => Promise.resolve<string | null>(null);
export const setVariables = (text: string, values: [string, number][]) =>
  invoke<string>("set_variables", { text, values });

/** A deck's first non-empty comment, as the desktop model browser shows it. */
function firstComment(text: string): string {
  for (const line of text.split(/\r?\n/)) {
    const m = /^\s*(CM|CE)\b(.*)$/i.exec(line);
    if (m && m[2].trim()) return m[2].trim();
  }
  return "";
}

export interface CardLine {
  line: number;
  mnemonic: string;
  fields: string[];
  values: (number | null)[];
  comment: string | null;
}
/** The deck's geometry cards, and the line a new card goes before (the GE card's). */
export const geometryCards = (text: string) =>
  invoke<{ cards: CardLine[]; insert_at: number | null }>("geometry_cards", { text });
/** Every data card of the deck, in order. */
export const allCards = (text: string) => invoke<CardLine[]>("all_cards", { text });
export const replaceCard = (text: string, line: number, mnemonic: string, fields: string[]) =>
  invoke<string>("replace_card", { text, line, mnemonic, fields });
export const insertCard = (text: string, line: number, mnemonic: string, fields: string[]) =>
  invoke<string>("insert_card", { text, line, mnemonic, fields });
export const deleteLine = (text: string, line: number) => invoke<string>("delete_line", { text, line });

export type Component =
  | { kind: "inductor"; henry: number; q: number }
  | { kind: "capacitor"; farad: number; q: number };
export interface Line {
  z0: number;
  vf: number;
}
export type Stage =
  | { stage: "series"; part: Component }
  | { stage: "shunt"; part: Component }
  | { stage: "line"; line: Line; length_m: number }
  | { stage: "stub"; line: Line; length_m: number; short: boolean; series: boolean };
export interface MatchNetwork {
  label: string;
  stages: Stage[];
}
export type MatchRequest =
  | { kind: "l"; q_l: number; q_c: number }
  | { kind: "pi"; q_net: number; q_l: number; q_c: number }
  | { kind: "tee"; q_net: number; q_l: number; q_c: number }
  | { kind: "stub"; line: Line; stub_line: Line; short: boolean; series: boolean }
  | { kind: "section"; line: Line; section: Line };
export interface MatchDesign {
  network: MatchNetwork;
  z_in: [number, number];
  swr: number;
  loss: number;
}
export interface MatchInDeck {
  dummy_tag: number;
  feed_tag: string;
  feed_segment: string;
  network: MatchNetwork;
}
export const designMatching = (z: [number, number], z0: number, mhz: number, request: MatchRequest) =>
  invoke<MatchDesign[]>("design_matching", { z, z0, mhz, request });
export const applyMatching = (text: string, tag: number, segment: number, network: MatchNetwork, mhz: number) =>
  invoke<string>("apply_matching", { text, tag, segment, network, mhz });
export const matchingNetworks = (text: string) => invoke<MatchInDeck[]>("matching_networks", { text });
export const removeMatching = (text: string, dummyTag: number) => invoke<string>("remove_matching", { text, dummyTag });

export type JobKind = "sweep" | "parameter" | "optimize" | "convergence";

export interface AverageGainTest {
  ratio: number;
  correction_db: number;
  grade: "accurate" | "usable" | "improvable" | "questionable";
  over_ground: boolean;
  mhz: number;
}
export interface ConvergenceStep {
  factor: number;
  segments: number;
  metrics: Metrics;
}
/** Not a cancellable job, but slow enough to get its own worker. */
export const averageGainTest = (text: string, mhz: number | null, path: string | null) =>
  lane("checks").request<AverageGainTest>("average_gain_test", { text, mhz, path });
/** Steps arrive as `convergence-step`; a stopped test resolves with the steps so far. */
export const convergenceTest = (text: string, mhz: number, steps: number, growth: number, options: EvalOptions) => {
  const done: ConvergenceStep[] = [];
  const off = listen<ConvergenceStep>("convergence-step", (e) => void done.push(e.payload));
  return startJob<ConvergenceStep[]>("convergence", "convergence_test", { text, mhz, steps, growth, options }, () => () => ({
    value: done.slice(),
  })).finally(() => void off.then((f) => f()));
};
export const onConvergenceStep = (f: (s: ConvergenceStep) => void): Promise<UnlistenFn> =>
  listen<ConvergenceStep>("convergence-step", (e) => f(e.payload));
export const cancel = (kind: JobKind) => Promise.resolve(lanes.get(kind)?.stop());

/** Frequency sweep; points arrive as `sweep-point` events. Resolves with the job number. */
export const sweep = (text: string, start: number, stop: number, steps: number, options: EvalOptions) =>
  startJob<number>("sweep", "sweep", { text, start, stop, steps, options }, (job) => () => ({ value: job }));

/** Parameter sweep; points arrive as `param-point` events. Resolves with the job number. */
export const parameterSweep = (
  text: string,
  name: string,
  start: number,
  stop: number,
  steps: number,
  mhz: number,
  options: EvalOptions,
) =>
  startJob<number>("parameter", "parameter_sweep", { text, name, start, stop, steps, mhz, options }, (job) => () => ({
    value: job,
  }));

/** Optimization; improvements arrive as `optimize-progress` events. A stopped run resolves with
 * the best design reported so far, as the desktop engine returns it when cancelled. */
export const optimize = (text: string, spec: OptimizeSpec) => {
  let initial: OptimizeProgress | null = null;
  let best: OptimizeProgress | null = null;
  let thisJob = 0;
  const off = listen<{ job: number; progress: OptimizeProgress }>("optimize-progress", (e) => {
    if (e.payload.job !== thisJob) return;
    const p = e.payload.progress;
    initial ??= p;
    if (!best || p.cost <= best.cost) best = p;
  });
  return startJob<OptimizeOutcome>("optimize", "optimize", { text, spec }, (job) => {
    thisJob = job;
    return () =>
      initial && best
        ? { value: { initial, best, converged: false, cancelled: true } satisfies OptimizeOutcome }
        : { error: "Stopped before the first design was evaluated" };
  }).finally(() => void off.then((f) => f()));
};

export const onSweepPoint = (f: (e: SweepEvent) => void): Promise<UnlistenFn> =>
  listen<SweepEvent>("sweep-point", (e) => f(e.payload));
export const onParamPoint = (f: (e: SweepEvent) => void): Promise<UnlistenFn> =>
  listen<SweepEvent>("param-point", (e) => f(e.payload));
export const onOptimizeProgress = (f: (job: number, p: OptimizeProgress) => void): Promise<UnlistenFn> =>
  listen<{ job: number; progress: OptimizeProgress }>("optimize-progress", (e) =>
    f(e.payload.job, e.payload.progress),
  );
