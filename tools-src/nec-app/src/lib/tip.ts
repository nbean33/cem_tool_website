// A Svelte action for styled tooltips: `<button use:tip={"What this does"}>`. One shared
// element follows the pointer's target; it opens after a short delay on hover or keyboard
// focus and closes on leave, blur, scroll or Escape.

let el: HTMLDivElement | null = null;
let timer: ReturnType<typeof setTimeout> | undefined;

function bubble(): HTMLDivElement {
  if (el) return el;
  el = document.createElement("div");
  el.className = "tip-bubble";
  el.setAttribute("role", "tooltip");
  document.body.appendChild(el);
  return el;
}

function show(target: HTMLElement, text: string) {
  const b = bubble();
  b.textContent = text;
  b.style.display = "block";
  const r = target.getBoundingClientRect();
  const bw = Math.min(320, window.innerWidth - 16);
  b.style.maxWidth = `${bw}px`;
  const w = b.offsetWidth;
  const h = b.offsetHeight;
  let x = r.left + r.width / 2 - w / 2;
  x = Math.max(8, Math.min(x, window.innerWidth - w - 8));
  let y = r.bottom + 6;
  if (y + h > window.innerHeight - 8) y = r.top - h - 6;
  b.style.left = `${x}px`;
  b.style.top = `${Math.max(8, y)}px`;
}

export function hideTip() {
  clearTimeout(timer);
  if (el) el.style.display = "none";
}

export function tip(node: HTMLElement, text: string | null | undefined) {
  let current = text ?? "";
  const enter = (e: Event) => {
    if (!current) return;
    // Focus from a click is not a request for help; focus from the keyboard is.
    if (e.type === "focus" && !node.matches(":focus-visible")) return;
    clearTimeout(timer);
    timer = setTimeout(() => show(node, current), 350);
  };
  const key = (e: KeyboardEvent) => e.key === "Escape" && hideTip();
  node.addEventListener("mouseenter", enter);
  node.addEventListener("focus", enter);
  node.addEventListener("mouseleave", hideTip);
  node.addEventListener("blur", hideTip);
  node.addEventListener("keydown", key);
  node.addEventListener("mousedown", hideTip);
  window.addEventListener("scroll", hideTip, true);
  if (current) node.setAttribute("aria-description", current);
  return {
    update(t: string | null | undefined) {
      current = t ?? "";
      if (current) node.setAttribute("aria-description", current);
      else node.removeAttribute("aria-description");
    },
    destroy() {
      node.removeEventListener("mouseenter", enter);
      node.removeEventListener("focus", enter);
      node.removeEventListener("mouseleave", hideTip);
      node.removeEventListener("blur", hideTip);
      node.removeEventListener("keydown", key);
      node.removeEventListener("mousedown", hideTip);
      window.removeEventListener("scroll", hideTip, true);
      hideTip();
    },
  };
}
