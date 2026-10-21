import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { fileURLToPath } from "node:url";

// cem_tool_website build of the PCB Toolkit desktop frontend.
//
// The app imports `invoke` from "@tauri-apps/api/core"; that import is aliased to
// src/backend.ts, which runs the same commands in WebAssembly, so the vendored components stay
// unchanged. `base: "./"` keeps every asset URL relative, so the build works under the
// GitHub Pages sub-path (/cem_tool_website/tools/pcb/) as well as at a domain root.
export default defineConfig({
  plugins: [react()],
  base: "./",
  resolve: {
    alias: {
      "@tauri-apps/api/core": fileURLToPath(new URL("./src/backend.ts", import.meta.url)),
    },
  },
  build: {
    outDir: fileURLToPath(new URL("../../tools/pcb", import.meta.url)),
    emptyOutDir: true,
    target: "es2022",
  },
});
