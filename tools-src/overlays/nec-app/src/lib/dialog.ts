// Stands in for "@tauri-apps/plugin-dialog" in the website build (aliased in vite.config.ts),
// with the same three functions and return values, so the components that use it are
// unchanged.
//
// open: the browser's file (or folder) picker; the chosen files' text goes into vfs.ts and the
//       file's name is returned as its path.
// save: asks for a file name; the caller's saveModel/writeText then downloads under it.
// ask:  a confirm box.
import { put } from "./vfs";

interface Filter {
  name: string;
  extensions: string[];
}
interface OpenOptions {
  multiple?: boolean;
  directory?: boolean;
  filters?: Filter[];
}
interface SaveOptions {
  defaultPath?: string;
  filters?: Filter[];
}

/** Decks are plain ASCII and sometimes Latin-1; like the desktop app, decode leniently. */
async function readText(file: File): Promise<string> {
  return new TextDecoder("utf-8").decode(await file.arrayBuffer());
}

function pick(options: OpenOptions): Promise<FileList | null> {
  return new Promise((resolve) => {
    const input = document.createElement("input");
    input.type = "file";
    if (options.directory) {
      input.setAttribute("webkitdirectory", "");
      input.multiple = true;
    } else {
      input.multiple = !!options.multiple;
      const exts = (options.filters ?? []).flatMap((f) => f.extensions);
      if (exts.length) input.accept = [...new Set(exts.map((e) => "." + e.toLowerCase()))].join(",");
    }
    input.style.display = "none";
    input.addEventListener("change", () => {
      resolve(input.files && input.files.length ? input.files : null);
      input.remove();
    });
    input.addEventListener("cancel", () => {
      resolve(null);
      input.remove();
    });
    document.body.appendChild(input);
    input.click();
  });
}

export async function open(options: OpenOptions = {}): Promise<string | string[] | null> {
  const list = await pick(options);
  if (!list) return null;
  if (options.directory) {
    // Keep the decks and projects of the chosen folder; the folder's name is its path.
    let root: string | null = null;
    for (const f of Array.from(list)) {
      const rel = f.webkitRelativePath || f.name;
      root ??= rel.split("/")[0];
      if (/\.(nec|toml)$/i.test(rel)) put(rel, await readText(f));
    }
    return root;
  }
  const paths: string[] = [];
  for (const f of Array.from(list)) {
    put(f.name, await readText(f));
    paths.push(f.name);
  }
  return options.multiple ? paths : paths[0] ?? null;
}

export async function save(options: SaveOptions = {}): Promise<string | null> {
  const name = window.prompt("Save as (the file downloads with this name)", options.defaultPath ?? "untitled.nec");
  return name && name.trim() ? name.trim() : null;
}

export async function ask(message: string, _options?: { title?: string; kind?: string }): Promise<boolean> {
  return window.confirm(message);
}
