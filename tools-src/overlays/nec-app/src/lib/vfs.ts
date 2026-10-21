// In-memory files for the browser build.
//
// The desktop app passes file paths between its dialogs and its backend. In the browser a
// "path" is the file's name (or its path inside an opened folder), the text lives here, and
// saving downloads a file under that name. Components keep using paths unchanged.

const files = new Map<string, string>();

export function put(path: string, text: string) {
  files.set(path, text);
}

export function get(path: string): string {
  const text = files.get(path);
  if (text === undefined) throw `${path}: not opened in this browser session`;
  return text;
}

/** Every stored path under `root/`, as `[path, text]`. */
export function under(root: string): [string, string][] {
  const prefix = root.endsWith("/") ? root : root + "/";
  return [...files].filter(([p]) => p.startsWith(prefix));
}

/** Save `text` as a download named after the last part of `path`. */
export function download(path: string, text: string) {
  const name = path.split(/[\\/]/).pop() || "untitled.txt";
  const url = URL.createObjectURL(new Blob([text], { type: "text/plain;charset=utf-8" }));
  const a = document.createElement("a");
  a.href = url;
  a.download = name;
  document.body.appendChild(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 10_000);
}
