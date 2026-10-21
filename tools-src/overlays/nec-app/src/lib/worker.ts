// One engine in a Web Worker: the nec-wasm build of NEC Studio's backend.
//
// Messages in:  { id, cmd, args }      (the Tauri command name and its argument object)
// Messages out: { id, ok: json }       the command's result, as JSON text
//               { id, err: message }   the command's error
//               { event, payload }     a progress event (payload is JSON text)
//
// A call runs to completion on this worker's only thread, so the page cancels a job by
// terminating the worker; see api.ts.
import init, { call } from "./wasm/nec_wasm.js";

const ready = init();

const emit = (event: string, payload: string) => self.postMessage({ event, payload });

self.onmessage = async (e: MessageEvent<{ id: number; cmd: string; args: unknown }>) => {
  const { id, cmd, args } = e.data;
  try {
    await ready;
    self.postMessage({ id, ok: call(cmd, JSON.stringify(args ?? {}), emit) });
  } catch (err) {
    self.postMessage({ id, err: typeof err === "string" ? err : String(err) });
  }
};
