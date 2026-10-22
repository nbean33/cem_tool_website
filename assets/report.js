/* Machshev Labs - the example board report.
 *
 * One example board, drawn as line art, with the high-speed connections a check finds on it.
 * Used by the home page preview, the Sample report page and the app screen mockups:
 *
 *   <div data-report="full"></div>       board + connection list + detail + graph
 *   <div data-report="preview"></div>    board only, the failing connection marked
 *   <div data-report="detail" data-conn="usb"></div>   one connection (or #hash)
 *   <tbody data-report-rows></tbody>     results table rows (app mockup)
 *
 * The board, its connections and every number below are an illustrative example, not a
 * measurement of a real design.
 */
(function () {
  "use strict";

  const BOARD = {
    file: "sensor-hub_revB.kicad_pcb",
    layers: 4,
    stackup: "JLCPCB JLC04161H-7628 (4 layers, 1.6 mm)",
  };

  // status: free check result. full: result of the paid full check (null = not run yet).
  const CONNECTIONS = [
    {
      id: "usb", name: "USB 2.0 data pair", nets: "USB_DP / USB_DN", kind: "Differential pair",
      target: 90, tol: 0.1, unit: "Ω differential", length: 48, status: "fail",
      summary: "Impedance drops to 71 Ω at via V3.",
      where: "Via V3, where the pair moves from the top layer (L1) to the bottom layer (L4), 31 mm from the USB connector J1.",
      why: "The clearance holes around V3 on the inner ground layers are too small. The via barrels sit close to the ground plane, which adds capacitance and pulls the impedance down to 71 Ω over about 1.2 mm.",
      fix: "Enlarge the clearance (antipad) around V3 on L2 and L3 to 0.9 mm, or keep the pair on L1 and remove the layer change.",
      profile: { base: 90, start: [3, 84], dips: [{ x: 30.5, w: 0.55, to: 71 }] }, marker: 30.5, markerLabel: "V3",
      full: { status: "fail", text: "Confirmed: this will fail. With the vias, connector and chip package included, the reflection at V3 closes the eye at the receiver below the USB 2.0 mask at 480 Mb/s." },
      path: [["M54 163H146", "M146 163H196L232 181", true], ["M54 170H146", "M146 170H192L232 188", true]],
      vias: [[146, 163], [146, 170]],
    },
    {
      id: "eth-tx", name: "Ethernet transmit pair", nets: "ETH_TX+ / ETH_TX−", kind: "Differential pair",
      target: 100, tol: 0.1, unit: "Ω differential", length: 36, status: "warn",
      summary: "Impedance is 109 Ω, close to the 110 Ω limit.",
      where: "The whole run from the PHY U2 to the magnetics in J2, on L1.",
      why: "The 0.15 mm traces with 0.2 mm spacing give 109 Ω on this stackup: inside the window, but with almost no margin for etching tolerance.",
      fix: "Widen the traces to 0.18 mm to centre the pair on 100 Ω, or leave it if the full check passes.",
      profile: { base: 109, wobble: 1.2 }, marker: null,
      full: { status: "pass", text: "Cleared: this will work. Return loss meets the 100BASE-TX limit with 4 dB to spare." },
      path: [["M440 182H468L498 92H510"], ["M440 188H471L501 98H510"]],
      vias: [],
    },
    {
      id: "eth-rx", name: "Ethernet receive pair", nets: "ETH_RX+ / ETH_RX−", kind: "Differential pair",
      target: 100, tol: 0.1, unit: "Ω differential", length: 37, status: "pass",
      summary: "Impedance is 101 Ω along the whole run.",
      where: "From J2 to the PHY U2, on L1.", why: "Within the 90–110 Ω window.", fix: "Nothing to change.",
      profile: { base: 101, wobble: 0.8 }, marker: null, full: null,
      path: [["M440 205H478L506 112H510"], ["M440 211H481L509 118H510"]],
      vias: [],
    },
    {
      id: "hdmi", name: "HDMI data and clock (4 pairs)", nets: "TMDS_D0–D2, TMDS_CLK", kind: "Differential pairs",
      target: 100, tol: 0.15, unit: "Ω differential", length: 22, status: "pass",
      summary: "All four pairs between 96 and 102 Ω.",
      where: "From U1 to the HDMI connector J3, on L1.", why: "Within the 85–115 Ω window.", fix: "Nothing to change.",
      profile: { base: 99, wobble: 1 }, marker: null, full: null,
      path: [["M268 240V300"], ["M273 240V300"], ["M286 240V300"], ["M291 240V300"], ["M304 240V300"], ["M309 240V300"], ["M322 240V300"], ["M327 240V300"]],
      vias: [],
    },
    {
      id: "ant", name: "2.4 GHz antenna feed", nets: "RF_ANT", kind: "Single-ended, 50 Ω",
      target: 50, tol: 0.1, unit: "Ω", length: 18, status: "warn",
      summary: "Feed trace is 58 Ω, outside the 45–55 Ω window.",
      where: "The feed from U1 to the chip antenna ANT1, on L1.",
      why: "At 0.28 mm the trace is too narrow for this stackup; 0.36 mm gives 50 Ω. On a run this short the mismatch may not matter, which is what the full check decides.",
      fix: "Widen the feed to 0.36 mm, or run the full check to see whether the match at the antenna suffers.",
      profile: { base: 58, wobble: 0.6 }, marker: null, full: null,
      path: [["M232 142L186 98H98V60"]],
      vias: [],
    },
  ];

  const STATUS = {
    pass: { label: "Pass", cls: "pass" },
    warn: { label: "Check", cls: "warn" },
    fail: { label: "Fail", cls: "fail" },
  };

  const esc = (t) => String(t).replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
  const byId = (id) => CONNECTIONS.find((c) => c.id === id) || CONNECTIONS[0];
  const pill = (status, text) => `<span class="pill ${STATUS[status].cls}">${esc(text || STATUS[status].label)}</span>`;

  /* ---------------- The board ---------------- */
  function boardSVG(selected, opts = {}) {
    const sel = selected ? byId(selected) : null;
    const trace = (c) => {
      const on = sel && sel.id === c.id;
      const color = on ? `var(--${c.status === "pass" ? "pass" : c.status})` : "var(--copper)";
      const w = c.id === "ant" ? 3 : 2;
      return `<g class="conn${on ? " on" : ""}" data-conn="${c.id}" tabindex="0" role="button" aria-label="${esc(c.name)}">
        ${c.path.map(([top, bottom]) => `<path d="${top}" stroke-width="${on ? w + 0.6 : w}" fill="none" stroke-linecap="round" stroke-linejoin="round" style="stroke:${color}"/>${bottom ? `<path d="${bottom}" stroke-width="${on ? w + 0.6 : w}" stroke-dasharray="5 3" fill="none" stroke-linecap="round" stroke-linejoin="round" style="stroke:${color}"/>` : ""}`).join("")}
        ${c.vias.map(([x, y]) => `<circle cx="${x}" cy="${y}" r="3.4" stroke-width="1.6" style="fill:var(--bg);stroke:${color}"/>`).join("")}
        <path d="${c.path.map(([t, b]) => t + (b || "")).join("")}" stroke="transparent" stroke-width="16" fill="none"/>
      </g>`;
    };
    const marker = sel && sel.status === "fail" && sel.vias.length
      ? `<circle cx="${sel.vias[0][0]}" cy="${(sel.vias[0][1] + sel.vias[sel.vias.length - 1][1]) / 2}" r="15" fill="none" stroke-width="2" style="stroke:var(--fail)"/>
         <path d="M${sel.vias[0][0] + 11} ${sel.vias[0][1] - 11}L${sel.vias[0][0] + 34} ${sel.vias[0][1] - 40}H${sel.vias[0][0] + 44}" stroke-width="1.4" fill="none" style="stroke:var(--fail)"/>`
      : "";
    const note = opts.note && sel
      ? `<g class="board-note"><rect x="${sel.vias[0][0] + 44}" y="${sel.vias[0][1] - 56}" width="${opts.note.length * 6.4 + 20}" height="30" rx="6" style="fill:var(--surface-2);stroke:var(--fail)"/>
         <text x="${sel.vias[0][0] + 54}" y="${sel.vias[0][1] - 36}" font-size="12" font-family="IBM Plex Sans,sans-serif" style="fill:var(--text)">${esc(opts.note)}</text></g>`
      : "";
    return `<svg class="board" viewBox="0 0 600 360" role="img" aria-label="Example board ${esc(BOARD.file)} with its high-speed connections">
      <rect x="20" y="20" width="560" height="320" rx="14" stroke="#3a3f48" stroke-width="1.5" style="fill:var(--bg-2)"/>
      ${[[40, 40], [560, 40], [40, 320], [560, 320]].map(([x, y]) => `<circle cx="${x}" cy="${y}" r="6" fill="none" stroke="#3a3f48" stroke-width="1.5"/>`).join("")}
      <g fill="none" stroke="#3a3f48" stroke-width="1.2">
        <rect x="20" y="146" width="34" height="48" rx="3"/>
        <rect x="232" y="132" width="108" height="108" rx="4"/>
        <rect x="380" y="168" width="60" height="52" rx="3"/>
        <rect x="510" y="72" width="70" height="66" rx="3"/>
        <rect x="256" y="300" width="84" height="40" rx="3"/>
        <rect x="70" y="42" width="56" height="18" rx="2"/>
        <rect x="40" y="30" width="110" height="46" rx="4" stroke-dasharray="3 3" opacity=".7"/>
        <path d="M340 160H380M340 176H380M340 192H380M340 208H380" stroke-width="1" opacity=".8"/>
        <path d="M232 200H190V250H150M232 214H200V270H120M300 132V96H330" stroke-width="1" opacity=".8"/>
        <rect x="140" y="236" width="22" height="14" rx="2"/><rect x="110" y="262" width="22" height="14" rx="2"/><rect x="322" y="82" width="22" height="14" rx="2"/>
      </g>
      <g style="fill:var(--dim)" font-size="10" font-family="IBM Plex Mono,monospace">
        <text x="26" y="208">J1 USB</text><text x="240" y="150">U1</text><text x="386" y="234">U2 PHY</text>
        <text x="516" y="152">J2 RJ45</text><text x="346" y="322">J3 HDMI</text><text x="132" y="56">ANT1</text>
      </g>
      ${CONNECTIONS.map(trace).join("")}
      ${marker}${note}
    </svg>`;
  }

  /* ---------------- Impedance profile ---------------- */
  function profileValue(c, x) {
    const p = c.profile;
    let z = p.base;
    if (p.start && x < p.start[0]) z = p.start[1] + (p.base - p.start[1]) * (x / p.start[0]) ** 3;
    if (p.wobble) z += p.wobble * Math.sin(x * 1.7) + 0.5 * p.wobble * Math.sin(x * 4.3 + 1);
    for (const d of p.dips || []) z += (d.to - p.base) * Math.exp(-(((x - d.x) / d.w) ** 2));
    return z;
  }

  function profileSVG(c) {
    const W = 640, H = 240, m = { l: 48, r: 16, t: 14, b: 38 };
    const lo = c.target * (1 - c.tol), hi = c.target * (1 + c.tol);
    const y0 = Math.floor(Math.min(lo, ...Array.from({ length: 200 }, (_, i) => profileValue(c, c.length * i / 199))) / 10) * 10 - 5;
    const y1 = Math.ceil(Math.max(hi, ...Array.from({ length: 200 }, (_, i) => profileValue(c, c.length * i / 199))) / 10) * 10 + 5;
    const X = (x) => m.l + x / c.length * (W - m.l - m.r);
    const Y = (z) => m.t + (y1 - z) / (y1 - y0) * (H - m.t - m.b);
    let d = "";
    for (let i = 0; i <= 400; i++) {
      const x = c.length * i / 400;
      d += (i ? "L" : "M") + X(x).toFixed(1) + " " + Y(profileValue(c, x)).toFixed(1);
    }
    const yt = [];
    for (let z = Math.ceil(y0 / 10) * 10; z <= y1; z += 10) yt.push(z);
    const xs = c.length > 30 ? 10 : 5, xt = [];
    for (let x = 0; x <= c.length; x += xs) xt.push(x);
    const mk = c.marker != null
      ? `<circle cx="${X(c.marker)}" cy="${Y(profileValue(c, c.marker))}" r="4" style="fill:var(--fail)"/>
         <text x="${X(c.marker) + 8}" y="${Y(profileValue(c, c.marker)) + 4}" font-size="11" font-family="IBM Plex Mono,monospace" style="fill:var(--fail)">${esc(c.markerLabel)} · ${Math.round(profileValue(c, c.marker))} Ω</text>`
      : "";
    return `<svg class="profile" viewBox="0 0 ${W} ${H}" role="img" aria-label="Impedance along ${esc(c.name)}">
      <rect x="${m.l}" y="${Y(hi)}" width="${W - m.l - m.r}" height="${Y(lo) - Y(hi)}" fill="rgba(255,255,255,.05)"/>
      <g stroke="rgba(255,255,255,.07)">${yt.map((z) => `<line x1="${m.l}" x2="${W - m.r}" y1="${Y(z)}" y2="${Y(z)}"/>`).join("")}</g>
      <line x1="${m.l}" x2="${W - m.r}" y1="${Y(c.target)}" y2="${Y(c.target)}" stroke-dasharray="4 3" style="stroke:var(--dim)"/>
      <text x="${W - m.r - 4}" y="${Y(hi) - 5}" text-anchor="end" font-size="10.5" font-family="IBM Plex Mono,monospace" style="fill:var(--dim)">target ${c.target} Ω ±${Math.round(c.tol * 100)}%</text>
      <g style="fill:var(--dim)" font-size="10.5" font-family="IBM Plex Mono,monospace">
        ${yt.map((z) => `<text x="${m.l - 8}" y="${Y(z) + 4}" text-anchor="end">${z}</text>`).join("")}
        ${xt.map((x) => `<text x="${X(x)}" y="${H - m.b + 16}" text-anchor="middle">${x}</text>`).join("")}
      </g>
      <text x="${(m.l + W - m.r) / 2}" y="${H - 4}" text-anchor="middle" font-size="11.5" font-family="IBM Plex Sans,sans-serif" style="fill:var(--muted)">Distance along the connection (mm)</text>
      <text transform="translate(12 ${(m.t + H - m.b) / 2}) rotate(-90)" text-anchor="middle" font-size="11.5" font-family="IBM Plex Sans,sans-serif" style="fill:var(--muted)">Impedance (Ω)</text>
      <path d="${d}" fill="none" stroke-width="2" style="stroke:var(--accent)"/>
      ${mk}
    </svg>`;
  }

  /* ---------------- Detail panel ---------------- */
  function detailHTML(c, opts = {}) {
    const full = c.full
      ? `<div class="full-result ${c.full.status}"><b>Full check</b> ${pill(c.full.status, c.full.status === "pass" ? "Cleared" : "Confirmed")}<p>${esc(c.full.text)}</p></div>`
      : c.status === "pass" ? ""
      : `<div class="full-result pending"><b>Full check</b> <span class="pill">Not run</span><p>The free check flags this connection. A full check simulates it with the vias and connectors included and tells you whether it will actually fail.</p>
         ${opts.app ? `<a class="btn" href="#" data-run-full>Run full check · about 2 minutes · $—</a>` : ""}</div>`;
    return `<div class="conn-detail">
      <div class="conn-head"><div><h3>${esc(c.name)}</h3><div class="mono dim small">${esc(c.nets)} · ${esc(c.kind)} · target ${c.target} ${esc(c.unit)}</div></div>${pill(c.status)}</div>
      <p class="conn-summary">${esc(c.summary)}</p>
      <dl class="conn-facts"><dt>Where</dt><dd>${esc(c.where)}</dd><dt>Why</dt><dd>${esc(c.why)}</dd><dt>Fix</dt><dd>${esc(c.fix)}</dd></dl>
      ${full}
      <div class="profile-card"><div class="small muted">Impedance along the connection</div>${profileSVG(c)}</div>
    </div>`;
  }

  function counts() {
    const n = { pass: 0, warn: 0, fail: 0 };
    CONNECTIONS.forEach((c) => n[c.status]++);
    return n;
  }

  function listHTML(selected, hrefFor) {
    return `<ul class="conn-list">${CONNECTIONS.map((c) => `<li>
      <a href="${hrefFor ? hrefFor(c) : "#" + c.id}" data-pick="${c.id}"${c.id === selected ? ' aria-current="true"' : ""}>
        <span class="dot ${c.status}"></span><span><b>${esc(c.name)}</b><span class="small dim">${esc(c.summary)}</span></span>
      </a></li>`).join("")}</ul>`;
  }

  /* ---------------- Mounting ---------------- */
  function mountFull(el) {
    let sel = (location.hash.slice(1) && CONNECTIONS.some((c) => c.id === location.hash.slice(1))) ? location.hash.slice(1) : "usb";
    const n = counts();
    el.innerHTML = `<div class="report">
      <div class="report-head">
        <div><div class="mono small dim">${esc(BOARD.file)} · ${BOARD.layers} layers · ${esc(BOARD.stackup)}</div>
        <div class="report-counts">${pill("fail", n.fail + " fail")}${pill("warn", n.warn + " to check")}${pill("pass", n.pass + " pass")}</div></div>
      </div>
      <div class="report-grid">
        <div class="report-board"></div>
        <div class="report-list"></div>
      </div>
      <div class="report-detail"></div>
    </div>`;
    const draw = () => {
      el.querySelector(".report-board").innerHTML = boardSVG(sel);
      el.querySelector(".report-list").innerHTML = listHTML(sel);
      el.querySelector(".report-detail").innerHTML = detailHTML(byId(sel));
    };
    const pick = (id) => { sel = id; history.replaceState(null, "", "#" + id); draw(); };
    el.addEventListener("click", (e) => {
      const t = e.target.closest("[data-pick], .conn");
      if (!t) return;
      e.preventDefault();
      pick(t.dataset.pick || t.dataset.conn);
    });
    el.addEventListener("keydown", (e) => {
      const t = e.target.closest(".conn");
      if (t && (e.key === "Enter" || e.key === " ")) { e.preventDefault(); pick(t.dataset.conn); }
    });
    draw();
  }

  function mountPreview(el) {
    el.innerHTML = boardSVG("usb", { note: "USB pair: impedance drops at this via." });
  }

  function mountDetail(el) {
    const id = el.dataset.conn || location.hash.slice(1) || "usb";
    const draw = (cid) => {
      el.innerHTML = `<div class="report-grid"><div class="report-board">${boardSVG(cid)}</div>
        <div class="report-list">${listHTML(cid, (c) => "app-connection.html#" + c.id)}</div></div>
        <div class="report-detail">${detailHTML(byId(cid), { app: true })}</div>`;
    };
    draw(byId(id).id);
    window.addEventListener("hashchange", () => draw(byId(location.hash.slice(1)).id));
    el.addEventListener("click", (e) => {
      const c = e.target.closest(".conn");
      if (c) location.hash = c.dataset.conn;
    });
  }

  function mountRows(el) {
    el.innerHTML = CONNECTIONS.map((c) => `<tr>
      <td><span class="dot ${c.status}"></span></td>
      <td><a href="app-connection.html#${c.id}"><b>${esc(c.name)}</b></a><div class="small dim mono">${esc(c.nets)}</div></td>
      <td class="muted">${esc(c.summary)}</td>
      <td>${c.full ? pill(c.full.status, c.full.status === "pass" ? "Cleared" : "Confirmed") : c.status === "pass" ? '<span class="dim">—</span>' : '<span class="pill">Not run</span>'}</td>
    </tr>`).join("");
  }

  window.MachshevReport = { BOARD, CONNECTIONS, counts };
  document.querySelectorAll('[data-report="full"]').forEach(mountFull);
  document.querySelectorAll('[data-report="preview"]').forEach(mountPreview);
  document.querySelectorAll('[data-report="detail"]').forEach(mountDetail);
  document.querySelectorAll("[data-report-rows]").forEach(mountRows);
})();
