import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { fileURLToPath } from "node:url";

// cem_tool_website build of NEC Studio.
//
// The components import the Tauri dialog plugin; it is aliased to src/lib/dialog.ts (browser
// file pickers with the same functions), and src/lib/api.ts talks to Web Workers instead of
// Tauri, so the vendored Svelte components stay unchanged. `base: "./"` keeps every URL,
// worker and wasm included, relative to the page, so the build works under the GitHub Pages
// sub-path (/cem_tool_website/tools/antenna/).
export default defineConfig({
  plugins: [svelte()],
  base: "./",
  resolve: {
    alias: {
      "@tauri-apps/plugin-dialog": fileURLToPath(new URL("./src/lib/dialog.ts", import.meta.url)),
    },
  },
  worker: { format: "es" },
  build: {
    outDir: fileURLToPath(new URL("../../tools/antenna", import.meta.url)),
    emptyOutDir: true,
    target: "es2022",
    chunkSizeWarningLimit: 1200,
  },
});
