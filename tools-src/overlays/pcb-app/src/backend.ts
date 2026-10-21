// Stands in for "@tauri-apps/api/core" in the website build (see the alias in vite.config.ts).
//
// The desktop app sends `invoke("calculate", { id, params })` and friends to its Tauri backend.
// Here the same three commands run in WebAssembly: pcb-wasm wraps the desktop backend's own
// dispatch, generated from it by scripts/sync_vendor.py, so results are identical. Values
// cross as JSON text, exactly as they crossed Tauri's IPC.
import init, { calculate, list_awg, list_materials } from "./wasm/pcb_wasm.js";

let ready: Promise<unknown> | null = null;

export async function invoke<T>(cmd: string, args: Record<string, unknown> = {}): Promise<T> {
  ready ??= init();
  await ready;
  switch (cmd) {
    case "calculate":
      // A calculator error is thrown as its message string, as Tauri rejects with one.
      return JSON.parse(calculate(String(args.id), JSON.stringify(args.params ?? {}))) as T;
    case "list_materials":
      return JSON.parse(list_materials()) as T;
    case "list_awg":
      return JSON.parse(list_awg()) as T;
    default:
      throw `unknown command \`${cmd}\``;
  }
}
