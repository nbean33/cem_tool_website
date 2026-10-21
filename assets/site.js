/* Machshev site - shared behavior.
 * Each page sets <body data-page="..."> and includes empty
 * <div id="site-header"></div> / <div id="site-footer"></div> mount points.
 */
(function () {
  "use strict";

  // Set to false once the S11 charts are fed real measured / simulated data.
  const SHOW_PLACEHOLDER_NOTE = true;

  const LOGO = `<svg viewBox="0 0 32 32" aria-hidden="true">
    <defs><linearGradient id="lg" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="#5eead4"/><stop offset="1" stop-color="#0ea5e9"/></linearGradient></defs>
    <rect x="3" y="3" width="26" height="26" rx="7" fill="none" stroke="url(#lg)" stroke-width="2.2"/>
    <path d="M8 16q2-7 4 0t4 0 4 0 4 0" fill="none" stroke="#eaf2fb" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
    <circle cx="24" cy="16" r="1.9" fill="#2dd4bf"/></svg>`;

  const NAV = [
    ["index", "Product", "index.html"],
    ["validation", "Validation", "validation.html"],
    ["benchmarks", "Benchmarks", "benchmarks.html"],
    ["pricing", "Pricing", "pricing.html"],
    ["docs", "Docs", "docs.html"],
    ["calculators", "RF Calculators", "rf-calculators.html"],
    ["antenna", "Antenna Studio", "tools/antenna/index.html"],
  ];

  // Pages below the site root (the tools in tools/*/) set data-root="../../" so the shared
  // header and footer links still point at the site's own pages.
  const ROOT = document.body.dataset.root || "";

  // Optional cookie tiers, as described in the Cookie Policy. The choice is only
  // remembered locally; no analytics or advertising scripts are wired up.
  const CONSENT_TIERS = [
    ["functional", "Functional", "Remembers preferences, such as your last-used settings."],
    ["analytics", "Analytics", "Anonymous page views and performance measurements."],
    ["thirdParty", "Third-party", "Analytics and advertising services run by other companies."],
  ];

  const page = document.body.dataset.page || "";
  const section = page.split("-")[0];

  function header() {
    const el = document.getElementById("site-header");
    if (!el) return;
    el.outerHTML = `
      <a class="skip" href="#main">Skip to content</a>
      <header class="site-header">
        <div class="container">
          <a class="brand" href="${ROOT}index.html">${LOGO}<span>Machshev</span></a>
          <nav class="nav" id="nav" aria-label="Primary">
            ${NAV.map(([k, label, href]) => `<a href="${ROOT}${href}"${k === section ? ' class="active" aria-current="page"' : ""}>${label}</a>`).join("")}
          </nav>
          <div class="header-actions">
            <a class="btn" href="${ROOT}rf-calculators.html">Try the online tools</a>
            <button class="menu-btn" aria-label="Menu" aria-expanded="false" aria-controls="nav">&#9776;</button>
          </div>
        </div>
      </header>`;
    const btn = document.querySelector(".menu-btn");
    const nav = document.getElementById("nav");
    btn.addEventListener("click", () => {
      const open = nav.classList.toggle("open");
      btn.setAttribute("aria-expanded", String(open));
    });
  }

  function footer() {
    const el = document.getElementById("site-footer");
    if (!el) return;
    el.outerHTML = `
      <footer class="site-footer">
        <div class="container">
          <div class="footer-grid">
            <div>
              <a class="brand" href="${ROOT}index.html">${LOGO}<span>Machshev</span></a>
              <p>Computational electromagnetics software from Machshev Labs LLC.</p>
            </div>
            <div><h4>Product</h4><ul>
              <li><a href="${ROOT}index.html">Features</a></li>
              <li><a href="${ROOT}benchmarks.html">Benchmarks</a></li>
              <li><a href="${ROOT}pricing.html">Pricing</a></li>
              <li><a href="${ROOT}rf-calculators.html">Online tools</a></li></ul></div>
            <div><h4>Docs</h4><ul>
              <li><a href="${ROOT}docs-install.html">Getting started</a></li>
              <li><a href="${ROOT}docs-file-formats.html">Reference</a></li>
              <li><a href="${ROOT}validation.html">Validation suite</a></li>
              <li><a href="${ROOT}blog.html">Blog</a></li></ul></div>
            <div><h4>Contact</h4><ul>
              <li><a href="mailto:support@machshev.com">Support<small>support@machshev.com</small></a></li>
              <li><a href="mailto:business@machshev.com">Business inquiries<small>business@machshev.com</small></a></li></ul></div>
            <div><h4>Legal</h4><ul>
              <li><a href="${ROOT}legal-terms.html">Terms</a></li>
              <li><a href="${ROOT}legal-privacy.html">Privacy</a></li>
              <li><a href="${ROOT}legal-cookies.html">Cookies</a></li></ul></div>
          </div>
          <div class="footer-bottom">
            <span>&copy; 2026 Machshev Labs LLC</span>
            <button type="button" id="cookie-prefs">Cookie preferences</button>
          </div>
        </div>
      </footer>
      <div class="cookie" id="cookie" hidden role="dialog" aria-label="Cookie consent">
        <div class="cookie-row">
          <span>Choose which optional cookies this site may use. <a href="${ROOT}legal-cookies.html">Cookie Policy</a></span>
          <button class="btn" data-consent="accept">Accept</button>
          <button class="btn ghost" data-consent="essential">Essentials only</button>
          <button class="btn ghost" data-consent="customize" aria-expanded="false">Customize</button>
        </div>
        <div class="cookie-prefs" hidden>
          <label class="tog"><input type="checkbox" checked disabled><span><b>Strictly necessary</b>Remembers your cookie choice. Always on.</span></label>
          ${CONSENT_TIERS.map(([k, name, desc]) => `<label class="tog"><input type="checkbox" data-tier="${k}"><span><b>${name}</b>${desc}</span></label>`).join("")}
          <div class="cookie-actions"><button class="btn" data-consent="save">Save choices</button></div>
        </div>
      </div>`;
    const banner = document.getElementById("cookie");
    const prefs = banner.querySelector(".cookie-prefs");
    const custom = banner.querySelector('[data-consent="customize"]');
    const boxes = [...banner.querySelectorAll("[data-tier]")];
    const store = (choice) => {
      try { localStorage.setItem("machshev-consent", JSON.stringify(choice)); } catch (err) { /* storage blocked */ }
    };
    let stored = null;
    try { stored = JSON.parse(localStorage.getItem("machshev-consent")); } catch (e) { stored = null; }
    if (!stored || typeof stored !== "object") banner.hidden = false;
    const sync = () => boxes.forEach((b) => { b.checked = !!(stored && stored[b.dataset.tier]); });
    const openPrefs = (open) => { prefs.hidden = !open; custom.setAttribute("aria-expanded", String(open)); };
    banner.addEventListener("click", (e) => {
      const v = e.target.dataset && e.target.dataset.consent;
      if (!v) return;
      if (v === "customize") { sync(); openPrefs(prefs.hidden); return; }
      const all = v === "accept";
      stored = Object.fromEntries(CONSENT_TIERS.map(([k]) => [k, v === "save" ? boxes.find((b) => b.dataset.tier === k).checked : all]));
      store(stored);
      banner.hidden = true;
      openPrefs(false);
    });
    document.getElementById("cookie-prefs").addEventListener("click", () => { banner.hidden = false; sync(); openPrefs(true); });
  }

  /* ---------------- Reveal + benchmark bars ---------------- */
  function reveal() {
    const els = document.querySelectorAll(".reveal, .bench");
    if (!("IntersectionObserver" in window)) {
      els.forEach((e) => { e.classList.add("in"); fillBars(e); });
      return;
    }
    const io = new IntersectionObserver((entries) => {
      entries.forEach((en) => {
        if (!en.isIntersecting) return;
        en.target.classList.add("in");
        fillBars(en.target);
        io.unobserve(en.target);
      });
    }, { threshold: 0.15 });
    els.forEach((e) => io.observe(e));
  }
  function fillBars(root) {
    if (!root.classList.contains("bench")) return;
    const rows = root.querySelectorAll("[data-rate]");
    const max = Math.max(...[...rows].map((r) => +r.dataset.rate));
    rows.forEach((r) => {
      const fill = r.querySelector(".bench-fill");
      if (fill) fill.style.width = (100 * (+r.dataset.rate) / max).toFixed(2) + "%";
    });
  }

  /* ---------------- Pricing toggle ---------------- */
  function pricing() {
    const seg = document.querySelector("[data-billing-toggle]");
    if (!seg) return;
    const set = (mode) => {
      seg.querySelectorAll("button").forEach((b) => b.classList.toggle("on", b.dataset.mode === mode));
      document.querySelectorAll("[data-yearly]").forEach((el) => {
        el.textContent = mode === "yearly" ? el.dataset.yearly : el.dataset.monthly;
      });
      document.querySelectorAll("[data-show]").forEach((el) => { el.hidden = el.dataset.show !== mode; });
    };
    seg.addEventListener("click", (e) => { if (e.target.dataset.mode) set(e.target.dataset.mode); });
    set("yearly");
  }

  /* ---------------- S11 charts ---------------- */
  // Synthetic S11 model: a sum of Lorentzian dips (in dB) on top of an optional
  // passband shelf. PLACEHOLDER data - replace DUTS[...] with real sweeps.
  function s11db(f, spec, wobble) {
    let db = spec.base != null ? spec.base : -0.4;
    if (spec.band) {
      const [fc, bw, floor] = spec.band;
      db += floor / (1 + Math.pow((f - fc) / bw, 8));
    }
    for (const d of spec.dips) db += -d.depth / (1 + Math.pow((f - d.f) / d.w, 2));
    if (wobble) db += 0.35 * Math.sin(f * 37) + 0.2 * Math.sin(f * 113 + 1);
    return Math.max(db, -45);
  }
  function simSpec(spec) {
    return {
      ...spec,
      dips: spec.dips.map((d) => ({ ...d, f: d.f * (1 + (d.df || 0) / 100), depth: d.depth * (1 + (d.dd || 15) / 100) })),
    };
  }

  const DUTS = {
    "EX-HOME": { range: [3, 5], ymin: -25, dips: [{ f: 4.02, depth: 21, w: 0.045, df: 0.4, dd: 12 }] },
    "EX-01": {
      range: [2.0, 2.7], ymin: -40, base: -0.3, band: [2.345, 0.16, -9],
      dips: [
        { f: 2.238, depth: 16, w: 0.012, df: 0.66, dd: 6.9 },
        { f: 2.305, depth: 20, w: 0.012, df: 0.38, dd: 9.7 },
        { f: 2.387, depth: 18, w: 0.012, df: -0.28, dd: 19.6 },
        { f: 2.453, depth: 22, w: 0.012, df: -0.05, dd: 14.8 },
      ],
    },
    "EX-02": { range: [4, 8], ymin: -35, dips: [{ f: 6.02, depth: 19, w: 0.08, df: 0.4 }] },
  };

  const NS = "http://www.w3.org/2000/svg";
  function niceStep(span, target) {
    const raw = span / target;
    const p = Math.pow(10, Math.floor(Math.log10(raw)));
    const m = raw / p;
    return (m < 1.5 ? 1 : m < 3.5 ? 2.5 : m < 7.5 ? 5 : 10) * p;
  }

  function makeChart(host) {
    const spec = DUTS[host.dataset.dut];
    if (!spec) return;
    const sim = simSpec(spec);
    const card = host.closest(".chart-card") || host.parentElement;
    let mode = "db";
    let view = spec.range.slice();
    const svg = document.createElementNS(NS, "svg");
    svg.setAttribute("class", "chart");
    svg.setAttribute("role", "img");
    svg.setAttribute("aria-label", `S11 magnitude, measured versus Machshev simulation, ${spec.range[0]} to ${spec.range[1]} GHz`);
    host.replaceWith(svg);

    function draw() {
      const W = svg.clientWidth || 700, H = svg.clientHeight || 340;
      const m = { l: 52, r: 14, t: 12, b: 40 };
      svg.setAttribute("viewBox", `0 0 ${W} ${H}`);
      const [x0, x1] = view;
      const lin = mode === "lin";
      const y0 = lin ? 0 : spec.ymin, y1 = lin ? 1 : 0;
      const X = (f) => m.l + (f - x0) / (x1 - x0) * (W - m.l - m.r);
      const Y = (v) => m.t + (y1 - v) / (y1 - y0) * (H - m.t - m.b);
      const val = (db) => (lin ? Math.pow(10, db / 20) : db);
      let out = `<g class="grid">`;
      const xs = niceStep(x1 - x0, 6), ys = lin ? 0.2 : 5;
      const xt = [], yt = [];
      for (let f = Math.ceil(x0 / xs) * xs; f <= x1 + 1e-9; f += xs) xt.push(+f.toFixed(6));
      for (let v = y0; v <= y1 + 1e-9; v += ys) yt.push(+v.toFixed(6));
      xt.forEach((f) => (out += `<line x1="${X(f)}" x2="${X(f)}" y1="${m.t}" y2="${H - m.b}"/>`));
      yt.forEach((v) => (out += `<line x1="${m.l}" x2="${W - m.r}" y1="${Y(v)}" y2="${Y(v)}"/>`));
      out += `</g><g class="axis">`;
      xt.forEach((f) => (out += `<text x="${X(f)}" y="${H - m.b + 16}" text-anchor="middle">${+f.toFixed(3)}</text>`));
      yt.forEach((v) => (out += `<text x="${m.l - 8}" y="${Y(v) + 4}" text-anchor="end">${lin ? v.toFixed(1) : v}</text>`));
      out += `</g>`;
      out += `<text class="axis-title" x="${(m.l + W - m.r) / 2}" y="${H - 6}" text-anchor="middle">Frequency (GHz)</text>`;
      out += `<text class="axis-title" transform="translate(13 ${(m.t + H - m.b) / 2}) rotate(-90)" text-anchor="middle">${lin ? "|S11|" : "|S11| (dB)"}</text>`;
      const N = 700;
      const path = (s, wob) => {
        let d = "";
        for (let i = 0; i <= N; i++) {
          const f = x0 + (x1 - x0) * i / N;
          const v = Math.min(Math.max(val(s11db(f, s, wob)), y0), y1);
          d += (i ? "L" : "M") + X(f).toFixed(1) + " " + Y(v).toFixed(1);
        }
        return d;
      };
      out += `<clipPath id="c-${host.dataset.dut}"><rect x="${m.l}" y="${m.t}" width="${W - m.l - m.r}" height="${H - m.t - m.b}"/></clipPath>`;
      out += `<g clip-path="url(#c-${host.dataset.dut})"><path class="meas" d="${path(spec, true)}"/><path class="sim" d="${path(sim, false)}"/></g>`;
      svg.innerHTML = out;
    }

    // dB / Linear toggle
    const seg = card.querySelector("[data-scale]");
    if (seg) seg.addEventListener("click", (e) => {
      if (!e.target.dataset.v) return;
      mode = e.target.dataset.v;
      seg.querySelectorAll("button").forEach((b) => b.classList.toggle("on", b === e.target));
      draw();
    });

    // ctrl/shift + wheel: zoom around pointer. drag: pan. double-click: reset.
    const fAt = (clientX) => {
      const r = svg.getBoundingClientRect();
      const t = Math.min(Math.max((clientX - r.left - 52) / (r.width - 66), 0), 1);
      return view[0] + t * (view[1] - view[0]);
    };
    const clamp = () => {
      const span = Math.min(view[1] - view[0], spec.range[1] - spec.range[0]);
      if (view[0] < spec.range[0]) view = [spec.range[0], spec.range[0] + span];
      if (view[1] > spec.range[1]) view = [spec.range[1] - span, spec.range[1]];
    };
    svg.addEventListener("wheel", (e) => {
      if (!e.ctrlKey && !e.shiftKey) return;
      e.preventDefault();
      const f = fAt(e.clientX);
      const k = (e.deltaY || e.deltaX) > 0 ? 1.18 : 1 / 1.18;
      const min = (spec.range[1] - spec.range[0]) / 200;
      if ((view[1] - view[0]) * k < min) return;
      view = [f - (f - view[0]) * k, f + (view[1] - f) * k];
      clamp(); draw();
    }, { passive: false });
    let drag = null;
    svg.addEventListener("pointerdown", (e) => { drag = { x: e.clientX, v: view.slice() }; svg.setPointerCapture(e.pointerId); });
    svg.addEventListener("pointermove", (e) => {
      if (!drag) return;
      const r = svg.getBoundingClientRect();
      const df = (e.clientX - drag.x) / (r.width - 66) * (drag.v[1] - drag.v[0]);
      view = [drag.v[0] - df, drag.v[1] - df];
      clamp(); draw();
    });
    svg.addEventListener("pointerup", () => (drag = null));
    svg.addEventListener("contextmenu", (e) => e.preventDefault());
    svg.addEventListener("dblclick", () => { view = spec.range.slice(); draw(); });
    window.addEventListener("resize", draw);
    draw();

    if (SHOW_PLACEHOLDER_NOTE && !card.querySelector(".ph-note")) {
      const n = document.createElement("div");
      n.className = "chart-hint ph-note";
      n.textContent = "Placeholder: sample curves. Replace with measured .s1p/.csv data in assets/site.js";
      svg.after(n);
    }
  }

  function charts() {
    const hosts = document.querySelectorAll("[data-dut]");
    if (!("IntersectionObserver" in window)) { hosts.forEach(makeChart); return; }
    const io = new IntersectionObserver((entries) => {
      entries.forEach((en) => {
        if (!en.isIntersecting) return;
        io.unobserve(en.target);
        makeChart(en.target);
      });
    }, { rootMargin: "200px" });
    hosts.forEach((h) => io.observe(h));
  }

  /* ---------------- Tabs ---------------- */
  function tabs() {
    document.querySelectorAll("[data-tabs]").forEach((group) => {
      const btns = group.querySelectorAll(".tabs button");
      const panels = group.querySelectorAll(".tab-panel");
      btns.forEach((b, i) => b.addEventListener("click", () => {
        btns.forEach((x, j) => { x.classList.toggle("on", i === j); x.setAttribute("aria-selected", String(i === j)); });
        panels.forEach((p, j) => p.classList.toggle("on", i === j));
      }));
    });
  }

  /* ---------------- Board illustrations (validation tabs) ---------------- */
  function boardShapes(kind, copper) {
    if (kind === "filter") {
      let s = "";
      for (let i = 0; i < 4; i++) {
        const x = 70 + i * 40, top = i % 2 === 0;
        s += `<rect x="${x}" y="${top ? 40 : 70}" width="16" height="110" fill="${copper}"/>`;
      }
      s += `<rect x="20" y="${40}" width="50" height="12" fill="${copper}"/><rect x="230" y="168" width="50" height="12" fill="${copper}"/>`;
      s += `<rect x="20" y="30" width="260" height="8" fill="${copper}" opacity=".7"/><rect x="20" y="182" width="260" height="8" fill="${copper}" opacity=".7"/>`;
      return s;
    }
    const dual = kind === "dual";
    const w = dual ? 120 : 90, h = dual ? 92 : 72, x = 150 - w / 2, y = 52;
    const inset = 22, gap = 6, feedW = 10;
    return `<path d="M${x} ${y}h${w}v${h}H${150 + feedW / 2 + gap}v-${inset}h-${gap}v${inset + 1}H${150 - feedW / 2}v-${inset + 1}h-${gap}v${inset}H${x}z" fill="${copper}"/>
      ${dual ? `<rect x="${x + 18}" y="${y + 20}" width="${w - 36}" height="6" fill="#0000"/>` : ""}
      <rect x="${150 - feedW / 2}" y="${y + h - inset}" width="${feedW}" height="${220 - (y + h - inset)}" fill="${copper}"/>`;
  }
  function boardSVG(kind, mode) {
    if (mode === "hardware") {
      return `<svg viewBox="0 0 300 220"><rect width="300" height="220" fill="#22201c"/>
        <rect x="10" y="10" width="280" height="200" rx="4" fill="#e9e4d6"/>
        ${boardShapes(kind, "#c79a4b")}
        <rect x="132" y="200" width="36" height="20" fill="#b8b8bc"/></svg>`;
    }
    if (mode === "mesh") {
      let g = "";
      for (let i = 0; i <= 300; i += 6) g += `<line x1="${i}" y1="0" x2="${i}" y2="220"/>`;
      for (let j = 0; j <= 220; j += 6) g += `<line x1="0" y1="${j}" x2="300" y2="${j}"/>`;
      return `<svg viewBox="0 0 300 220"><rect width="300" height="220" fill="#0c1a2e"/>
        ${boardShapes(kind, "rgba(45, 212, 191,.55)")}
        <g stroke="rgba(255,255,255,.09)" stroke-width=".6">${g}</g></svg>`;
    }
    return `<svg viewBox="0 0 300 220"><rect width="300" height="220" fill="#11223a"/>
      <rect x="0" y="0" width="300" height="16" fill="#1a2e4b"/>
      <circle cx="9" cy="8" r="2.5" fill="#f87171"/><circle cx="18" cy="8" r="2.5" fill="#fbbf24"/><circle cx="27" cy="8" r="2.5" fill="#4ade80"/>
      <rect x="0" y="16" width="44" height="204" fill="#152842"/><rect x="256" y="16" width="44" height="204" fill="#152842"/>
      ${[30, 44, 58, 72].map((y) => `<rect x="6" y="${y}" width="32" height="6" rx="2" fill="#2a4064"/>`).join("")}
      ${[30, 44, 58].map((y) => `<rect x="262" y="${y}" width="32" height="6" rx="2" fill="#2a4064"/>`).join("")}
      <g transform="translate(52 22) scale(.68)">${boardShapes(kind, "#d4a14f")}</g>
      <rect x="44" y="16" width="212" height="204" fill="none" stroke="#2d4468"/></svg>`;
  }
  function boards() {
    document.querySelectorAll("[data-board]").forEach((el) => {
      el.innerHTML = boardSVG(el.dataset.kind || "patch", el.dataset.board);
    });
  }

  /* ---------------- Hero "demo video" canvas ---------------- */
  function demo() {
    const box = document.querySelector("[data-demo]");
    if (!box) return;
    const cv = box.querySelector("canvas");
    const ctx = cv.getContext("2d");
    const btn = box.querySelector(".play");
    const clock = box.querySelector("[data-clock]");
    const fill = box.querySelector("[data-progress]");
    let playing = false, t = 0, last = 0, raf = 0;
    const reduce = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

    function color(v) {
      // diverging turbo-ish ramp from deep blue to red
      const s = Math.max(-1, Math.min(1, v));
      const a = (s + 1) / 2;
      const r = Math.round(255 * Math.min(1, Math.max(0, 1.6 * a - 0.35)));
      const g = Math.round(255 * Math.max(0, 1 - Math.abs(a - 0.5) * 2.1));
      const b = Math.round(255 * Math.min(1, Math.max(0, 1.3 - 1.7 * a)));
      return [r, g, b];
    }
    function frame() {
      const W = (cv.width = cv.clientWidth >> 1), H = (cv.height = cv.clientHeight >> 1);
      if (!W || !H) return;
      const img = ctx.createImageData(W, H);
      const cx = W * 0.5, cy = H * 0.52, pw = W * 0.18, ph = H * 0.26;
      for (let y = 0; y < H; y++) {
        for (let x = 0; x < W; x++) {
          const dx = (x - cx) / W, dy = (y - cy) / W;
          const r = Math.hypot(dx, dy);
          const inPatch = Math.abs(x - cx) < pw && Math.abs(y - cy) < ph;
          const edge = Math.exp(-Math.pow((Math.abs(x - cx) - pw) / (W * 0.02), 2)) * (Math.abs(y - cy) < ph ? 1 : 0);
          let v = Math.sin(r * 60 - t * 4) * Math.exp(-r * 3.2) * 0.9 + edge * Math.sin(t * 4) * 1.2;
          if (inPatch) v = Math.cos(((x - cx) / pw) * Math.PI / 2) * Math.sin(t * 4) * 0.3;
          const [R, G, B] = color(v);
          const i = (y * W + x) * 4, fade = 0.35 + 0.65 * Math.exp(-r * 1.4);
          img.data[i] = R * fade; img.data[i + 1] = G * fade; img.data[i + 2] = B * fade; img.data[i + 3] = 255;
        }
      }
      ctx.putImageData(img, 0, 0);
      ctx.strokeStyle = "rgba(255,220,160,.85)"; ctx.lineWidth = 1.5;
      ctx.strokeRect(cx - pw, cy - ph, pw * 2, ph * 2);
      const secs = Math.floor(t) % 60;
      clock.textContent = `0:${String(secs).padStart(2, "0")}`;
      fill.style.width = ((t % 60) / 60 * 100).toFixed(1) + "%";
    }
    function loop(ts) {
      if (!playing) return;
      if (last) t += (ts - last) / 1000;
      last = ts;
      frame();
      raf = requestAnimationFrame(loop);
    }
    btn.addEventListener("click", () => {
      playing = !playing;
      btn.hidden = playing;
      last = 0;
      if (playing) raf = requestAnimationFrame(loop);
    });
    cv.addEventListener("click", () => {
      if (!playing) return;
      playing = false; btn.hidden = false; cancelAnimationFrame(raf);
    });
    t = 1.3;
    frame();
    window.addEventListener("resize", frame);
    if (reduce) return;
  }

  /* ---------------- Docs sidebar + search ---------------- */
  // DOCS_NAV and DOCS_INDEX come from assets/docs-nav.js, generated by build_docs.py.
  const DOCS = window.DOCS_NAV || [];
  const INDEX = window.DOCS_INDEX || [];
  const current = location.pathname.split("/").pop() || "index.html";
  const esc = (s) => s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));

  // Full-text search: every term must appear in the page title or body.
  function searchDocs(term) {
    const words = term.toLowerCase().split(/\s+/).filter(Boolean);
    const hits = [];
    for (const page of INDEX) {
      const hay = (page.t + " " + page.x).toLowerCase();
      if (!words.every((w) => hay.includes(w))) continue;
      const inTitle = words.every((w) => page.t.toLowerCase().includes(w));
      const at = page.x.toLowerCase().indexOf(words[0]);
      let snippet = "";
      if (at >= 0) {
        const from = Math.max(0, at - 50);
        snippet = (from ? "..." : "") + page.x.slice(from, at + 110) + "...";
      }
      hits.push({ page, inTitle, snippet });
    }
    hits.sort((a, b) => Number(b.inTitle) - Number(a.inTitle));
    const mark = (s) => words.reduce((acc, w) =>
      acc.replace(new RegExp(w.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "gi"), (m) => `<mark>${m}</mark>`), esc(s));
    return hits.slice(0, 12).map(({ page, snippet }) =>
      `<a class="hit" href="${page.h}"><b>${mark(page.t)}</b><span class="sec">${esc(page.s)}</span>${snippet ? `<span class="snip">${mark(snippet)}</span>` : ""}</a>`).join("");
  }

  function docsNav() {
    const side = document.getElementById("docs-side");
    if (side) {
      side.innerHTML = `
        <input class="docs-search" type="search" placeholder="Search the docs" aria-label="Search the docs">
        <div class="docs-results" hidden aria-live="polite"></div>
        <button type="button" class="btn ghost docs-toggle" aria-expanded="false">All docs pages <span aria-hidden="true">&#9662;</span></button>
        <a class="video-link" href="docs.html">&#9654;&nbsp; Video walkthrough</a>
        ${DOCS.map(({ section, items }) => `<div class="grp"><h5>${section}</h5><ul>${items.map((it) =>
          `<li><a href="${it.h}"${it.h === current ? ' class="active" aria-current="page"' : ""}>${it.t}</a></li>`).join("")}</ul></div>`).join("")}`;
      const q = side.querySelector(".docs-search");
      const results = side.querySelector(".docs-results");
      q.addEventListener("input", () => {
        const term = q.value.trim();
        side.querySelectorAll(".grp").forEach((g) => { g.hidden = !!term; });
        side.querySelector(".video-link").hidden = !!term;
        results.hidden = !term;
        if (term) results.innerHTML = searchDocs(term) || '<p class="none">No pages match.</p>';
      });
      q.addEventListener("keydown", (e) => {
        if (e.key === "Escape") { q.value = ""; q.dispatchEvent(new Event("input")); }
        if (e.key === "Enter") { const first = results.querySelector("a"); if (first) location.href = first.href; }
      });
      const toggle = side.querySelector(".docs-toggle");
      toggle.addEventListener("click", () => {
        const open = side.classList.toggle("nav-open");
        toggle.setAttribute("aria-expanded", String(open));
      });
      const active = side.querySelector("a.active");
      if (active) active.scrollIntoView({ block: "nearest" });
    }
    const topics = document.getElementById("docs-topics");
    if (topics) {
      topics.innerHTML = DOCS.map(({ section, items }) => `<div class="card"><h4>${section}</h4><ul>${items.map((it) =>
        `<li><a href="${it.h}">${it.t}</a></li>`).join("")}</ul></div>`).join("");
    }
  }

  header();
  footer();
  docsNav();
  boards();
  tabs();
  pricing();
  demo();
  charts();
  reveal();
})();
