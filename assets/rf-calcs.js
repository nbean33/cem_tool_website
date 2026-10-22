/* Machshev Labs site calculators: the closed forms pcb_toolkit (tools/pcb/) does not cover.
 *
 * A page declares <div data-calc="patch"></div> (or ifa, qwt, smith, wavelength) and loads
 * this file; it renders the inputs, live results and a step-by-step breakdown. Lengths are
 * mm, frequencies GHz, impedances ohm.
 */
(function () {
  "use strict";

  const C0 = 299792458; // m/s
  const ETA0 = 376.730313; // ohm, free-space wave impedance
  const PI = Math.PI;

  // ---- Microstrip (Hammerstad-Jensen, with Wheeler's thickness correction) -----------------
  function weff(W, h, t, er) {
    if (t <= 0) return W;
    const u = W / h, th = t / h;
    const coth = 1 / Math.tanh(Math.sqrt(6.517 * u));
    const du1 = (th / PI) * Math.log(1 + (4 * Math.E) / (th * coth * coth));
    const dur = 0.5 * (1 + 1 / Math.cosh(Math.sqrt(Math.max(er - 1, 0)))) * du1;
    return W + h * dur;
  }
  function microstrip(W, h, t, er) {
    const u = weff(W, h, t, er) / h;
    const a = 1 + Math.log((u ** 4 + (u / 52) ** 2) / (u ** 4 + 0.432)) / 49 + Math.log(1 + (u / 18.1) ** 3) / 18.7;
    const b = 0.564 * ((er - 0.9) / (er + 3)) ** 0.053;
    const eeff = (er + 1) / 2 + ((er - 1) / 2) * (1 + 10 / u) ** (-a * b);
    const fu = 6 + (2 * PI - 6) * Math.exp(-((30.666 / u) ** 0.7528));
    const z01 = (ETA0 / (2 * PI)) * Math.log(fu / u + Math.sqrt(1 + 4 / (u * u)));
    return { eeff, z0: z01 / Math.sqrt(eeff) };
  }
  function widthFor(Z, h, t, er) {
    let lo = 0.001 * h, hi = 50 * h;
    for (let i = 0; i < 80; i++) {
      const mid = Math.sqrt(lo * hi);
      if (microstrip(mid, h, t, er).z0 > Z) lo = mid;
      else hi = mid;
    }
    return Math.sqrt(lo * hi);
  }

  // ---- numerics ------------------------------------------------------------------------------
  /** Composite Simpson's rule over [a, b] with n (even) intervals. */
  function simpson(f, a, b, n = 400) {
    const h = (b - a) / n;
    let s = f(a) + f(b);
    for (let i = 1; i < n; i++) s += f(a + i * h) * (i % 2 ? 4 : 2);
    return (s * h) / 3;
  }
  /** Bessel J0 from its integral form, J0(x) = (1/pi) * integral_0^pi cos(x sin t) dt. */
  const j0 = (x) => simpson((t) => Math.cos(x * Math.sin(t)), 0, PI, 64) / PI;

  const fmt = (v, d = 3) => (Number.isFinite(v) ? v.toFixed(d) : "-");

  // ---- the calculators ------------------------------------------------------------------------
  const CALCS = {
    patch: {
      inputs: [
        { key: "f", label: "Resonant frequency (GHz)", value: 2.45 },
        { key: "er", label: "Substrate εr", value: 4.4 },
        { key: "h", label: "Substrate height h (mm)", value: 1.6 },
        { key: "z0", label: "Feed impedance (Ω), for the inset", value: 50 },
      ],
      refs: "C. A. Balanis, Antenna Theory, 4th ed., ch. 14 (transmission-line model). Bandwidth: D. R. Jackson and N. G. Alexopoulos, IEEE Trans. AP-39, 1991.",
      solve({ f, er, h, z0 }) {
        const fHz = f * 1e9, hm = h / 1000;
        const lambda0 = C0 / fHz, k0 = (2 * PI) / lambda0;
        const W = (C0 / (2 * fHz)) * Math.sqrt(2 / (er + 1));
        const eeff = (er + 1) / 2 + ((er - 1) / 2) / Math.sqrt(1 + (12 * hm) / W);
        const dL = 0.412 * hm * ((eeff + 0.3) * (W / hm + 0.264)) / ((eeff - 0.258) * (W / hm + 0.8));
        const L = C0 / (2 * fHz * Math.sqrt(eeff)) - 2 * dL;
        // Radiating-slot conductance G1 and mutual conductance G12 (Balanis 14-12, 14-18a).
        const slot = (th) => {
          const c = Math.cos(th);
          const x = (k0 * W) / 2;
          const s = Math.abs(c) < 1e-9 ? x : Math.sin(x * c) / c;
          return s * s * Math.sin(th) ** 3;
        };
        const G1 = simpson(slot, 0, PI, 600) / (120 * PI * PI);
        const G12 = simpson((th) => slot(th) * j0(k0 * L * Math.sin(th)), 0, PI, 600) / (120 * PI * PI);
        const Rin = 1 / (2 * (G1 + G12));
        const inset = z0 < Rin ? (L / PI) * Math.acos(Math.sqrt(z0 / Rin)) : NaN;
        const bw = 3.77 * ((er - 1) / (er * er)) * (W / L) * (hm / lambda0);
        return {
          results: [
            ["Patch width W", `${fmt(W * 1000, 2)} mm`, true],
            ["Patch length L", `${fmt(L * 1000, 2)} mm`, true],
            ["εeff", fmt(eeff, 3)],
            ["Length extension ΔL", `${fmt(dL * 1000, 3)} mm`],
            ["Edge resistance", `${fmt(Rin, 1)} Ω`],
            [`Inset depth for ${fmt(z0, 0)} Ω`, Number.isFinite(inset) ? `${fmt(inset * 1000, 2)} mm` : "not needed"],
            ["Bandwidth (VSWR < 2)", `${fmt(bw * 100, 2)} %`],
            ["Free-space λ0", `${fmt(lambda0 * 1000, 1)} mm`],
          ],
          steps: [
            `λ0 = c / f = ${fmt(lambda0 * 1000, 2)} mm`,
            `W = c / (2f) · √(2 / (εr + 1)) = ${fmt(W * 1000, 3)} mm`,
            `εeff = (εr + 1)/2 + (εr − 1)/2 · (1 + 12h/W)^-½ = ${fmt(eeff, 4)}`,
            `ΔL = 0.412 h (εeff + 0.3)(W/h + 0.264) / ((εeff − 0.258)(W/h + 0.8)) = ${fmt(dL * 1000, 4)} mm`,
            `L = c / (2f √εeff) − 2ΔL = ${fmt(L * 1000, 3)} mm`,
            `G1 = ${fmt(G1 * 1000, 4)} mS, G12 = ${fmt(G12 * 1000, 4)} mS (slot integrals)`,
            `R_edge = 1 / (2 (G1 + G12)) = ${fmt(Rin, 2)} Ω`,
            Number.isFinite(inset)
              ? `y0 = (L/π) · acos(√(Z0 / R_edge)) = ${fmt(inset * 1000, 3)} mm`
              : `Z0 ≥ R_edge: feed at the edge, no inset`,
          ],
        };
      },
    },

    ifa: {
      inputs: [
        { key: "f", label: "Frequency (GHz)", value: 2.44 },
        { key: "er", label: "Substrate εr", value: 4.4 },
        { key: "H", label: "Arm height above ground edge H (mm)", value: 4 },
        { key: "w", label: "Trace width (mm)", value: 0.5 },
      ],
      refs: "Quarter-wave inverted-F rules of thumb (e.g. TI DN007 and AN043 style PCB IFAs). Starting points only: finish the design with a full-wave solve.",
      solve({ f, er, H, w }) {
        const fHz = f * 1e9;
        const lambda0 = (C0 / fHz) * 1000;
        // A printed arm sits on the board edge: half its field is in the substrate.
        const eeff = (er + 1) / 2;
        const total = lambda0 / (4 * Math.sqrt(eeff));
        const arm = total - H;
        const feed = H / 2;
        return {
          results: [
            ["Total length (short to open end)", `${fmt(total, 2)} mm`, true],
            ["Horizontal arm", `${fmt(arm, 2)} mm`, true],
            ["Feed spacing from the short", `${fmt(feed, 2)} mm`],
            ["Ground cutout (length × depth)", `${fmt(arm + 2, 1)} × ${fmt(H + 1, 1)} mm`],
            ["εeff (edge-mounted)", fmt(eeff, 3)],
            ["Free-space λ0", `${fmt(lambda0, 1)} mm`],
          ],
          steps: [
            `εeff ≈ (εr + 1) / 2 = ${fmt(eeff, 3)} for an arm on the board edge`,
            `Total = λ0 / (4 √εeff) = ${fmt(total, 3)} mm, measured along the trace centreline (width ${fmt(w, 2)} mm)`,
            `Arm = total − H = ${fmt(arm, 3)} mm`,
            `Feed spacing ≈ H / 2 = ${fmt(feed, 2)} mm. Moving the feed away from the short raises the input resistance`,
            `Keep copper out from under the arm: cutout ≥ arm + 2 mm long and H + 1 mm deep`,
            `A longer arm lowers the resonance; trim it once a solve shows where the dip lands`,
          ],
        };
      },
    },

    qwt: {
      inputs: [
        { key: "z1", label: "Source impedance Z1 (Ω)", value: 50 },
        { key: "z2", label: "Load impedance Z2 (Ω)", value: 100 },
        { key: "f", label: "Frequency (GHz)", value: 2.4 },
        { key: "er", label: "Substrate εr", value: 4.3 },
        { key: "h", label: "Substrate height h (mm)", value: 1.6 },
        { key: "t", label: "Copper thickness (µm)", value: 35 },
        { key: "gm", label: "Acceptable |Γ| for bandwidth", value: 0.1 },
      ],
      refs: "D. M. Pozar, Microwave Engineering, 4th ed., sec. 5.4. Microstrip lines: Hammerstad and Jensen, IEEE MTT-S Digest, 1980.",
      solve({ z1, z2, f, er, h, t, gm }) {
        const tm = t / 1000;
        const zt = Math.sqrt(z1 * z2);
        const wt = widthFor(zt, h, tm, er), w1 = widthFor(z1, h, tm, er), w2 = widthFor(z2, h, tm, er);
        const { eeff } = microstrip(wt, h, tm, er);
        const lg = (C0 / (f * 1e9)) / Math.sqrt(eeff) * 1000;
        const ratio = (gm / Math.sqrt(1 - gm * gm)) * ((2 * Math.sqrt(z1 * z2)) / Math.abs(z2 - z1));
        const bw = ratio >= 1 ? NaN : 2 - (4 / PI) * Math.acos(ratio);
        return {
          results: [
            ["Transformer impedance Zt", `${fmt(zt, 2)} Ω`, true],
            ["Length λg / 4", `${fmt(lg / 4, 2)} mm`, true],
            ["Width W1 (Z1 line)", `${fmt(w1, 3)} mm`],
            ["Width Wt (transformer)", `${fmt(wt, 3)} mm`],
            ["Width W2 (Z2 line)", `${fmt(w2, 3)} mm`],
            ["εeff of the transformer", fmt(eeff, 3)],
            [`Bandwidth for |Γ| ≤ ${fmt(gm, 2)}`, Number.isFinite(bw) ? `${fmt(bw * 100, 1)} %` : "whole band"],
          ],
          steps: [
            `Zt = √(Z1 · Z2) = ${fmt(zt, 3)} Ω`,
            `Microstrip widths from Hammerstad-Jensen: W1 ${fmt(w1, 3)}, Wt ${fmt(wt, 3)}, W2 ${fmt(w2, 3)} mm`,
            `λg = c / (f √εeff) = ${fmt(lg, 3)} mm, so the section is ${fmt(lg / 4, 3)} mm`,
            `Δf / f0 = 2 − (4/π) acos( Γm/√(1 − Γm²) · 2√(Z1 Z2)/|Z2 − Z1| ) = ${Number.isFinite(bw) ? fmt(bw, 4) : "n/a"}`,
          ],
        };
      },
    },

    smith: {
      inputs: [
        { key: "r", label: "Resistance R (Ω)", value: 25 },
        { key: "x", label: "Reactance X (Ω)", value: 30 },
        { key: "z0", label: "System impedance Z0 (Ω)", value: 50 },
      ],
      refs: "Reflection coefficient Γ = (Z − Z0) / (Z + Z0); D. M. Pozar, Microwave Engineering, 4th ed., ch. 2.",
      chart: true,
      solve({ r, x, z0 }) {
        const zr = r / z0, zi = x / z0;
        const den = (zr + 1) ** 2 + zi ** 2;
        const gr = (zr * zr + zi * zi - 1) / den, gi = (2 * zi) / den;
        const mag = Math.hypot(gr, gi), ang = (Math.atan2(gi, gr) * 180) / PI;
        const vswr = mag < 1 ? (1 + mag) / (1 - mag) : Infinity;
        const rl = -20 * Math.log10(mag), ml = -10 * Math.log10(1 - mag * mag);
        const yden = r * r + x * x;
        return {
          gamma: [gr, gi],
          results: [
            ["|Γ|", fmt(mag, 4), true],
            ["∠Γ", `${fmt(ang, 2)}°`, true],
            ["VSWR", Number.isFinite(vswr) ? fmt(vswr, 3) : "∞"],
            ["Return loss", `${fmt(rl, 2)} dB`],
            ["Mismatch loss", `${fmt(ml, 3)} dB`],
            ["Γ (rectangular)", `${fmt(gr, 4)} ${gi < 0 ? "−" : "+"} j${fmt(Math.abs(gi), 4)}`],
            ["Normalized z", `${fmt(zr, 3)} ${zi < 0 ? "−" : "+"} j${fmt(Math.abs(zi), 3)}`],
            ["Admittance Y", yden ? `${fmt((r / yden) * 1000, 3)} ${x > 0 ? "−" : "+"} j${fmt((Math.abs(x) / yden) * 1000, 3)} mS` : "∞"],
          ],
          steps: [
            `z = Z / Z0 = ${fmt(zr, 4)} ${zi < 0 ? "−" : "+"} j${fmt(Math.abs(zi), 4)}`,
            `Γ = (z − 1) / (z + 1) = ${fmt(gr, 4)} ${gi < 0 ? "−" : "+"} j${fmt(Math.abs(gi), 4)}`,
            `VSWR = (1 + |Γ|) / (1 − |Γ|) = ${Number.isFinite(vswr) ? fmt(vswr, 4) : "∞"}`,
            `RL = −20 log10 |Γ| = ${fmt(rl, 3)} dB; ML = −10 log10 (1 − |Γ|²) = ${fmt(ml, 4)} dB`,
          ],
        };
      },
    },

    wavelength: {
      inputs: [{ key: "f", label: "Frequency (GHz)", value: 2.4 }],
      refs: "λ = c / f with c = 299 792 458 m/s exactly.",
      solve({ f }) {
        const lam = (C0 / (f * 1e9)) * 1000;
        return {
          results: [
            ["Wavelength λ0", `${fmt(lam, 3)} mm`, true],
            ["λ0 / 2", `${fmt(lam / 2, 3)} mm`, true],
            ["λ0 / 4", `${fmt(lam / 4, 3)} mm`],
            ["λ0 / 10", `${fmt(lam / 10, 3)} mm`],
            ["λ0 / 20", `${fmt(lam / 20, 3)} mm`],
            ["Period", `${fmt(1 / f, 4)} ns`],
          ],
          steps: [`λ0 = c / f = 299.792458 / ${fmt(f, 4)} GHz = ${fmt(lam, 4)} mm`],
        };
      },
    },
  };

  // ---- Smith chart drawing --------------------------------------------------------------------
  function smithSvg(gamma) {
    const R = 120;
    const grid = [0.2, 0.5, 1, 2, 5];
    const toXY = ([gr, gi]) => [gr * R, -gi * R];
    let g = "";
    for (const r of grid) g += `<circle cx="${(r / (1 + r)) * R}" cy="0" r="${R / (1 + r)}"/>`;
    for (const x of grid) g += `<circle cx="${R}" cy="${-R / x}" r="${R / x}"/><circle cx="${R}" cy="${R / x}" r="${R / x}"/>`;
    const [px, py] = toXY(gamma);
    const mag = Math.hypot(...gamma);
    return `<svg viewBox="-135 -135 270 270" role="img" aria-label="Smith chart with the load plotted">
      <defs><clipPath id="rim"><circle r="${R}"/></clipPath></defs>
      <circle r="${R}" fill="#0d0f13" stroke="rgba(255,255,255,.35)"/>
      <g clip-path="url(#rim)" fill="none" stroke="rgba(255,255,255,.12)">${g}<line x1="${-R}" x2="${R}" y1="0" y2="0"/></g>
      ${mag < 1 ? `<circle r="${mag * R}" fill="none" stroke="rgba(77, 148, 255,.5)" stroke-dasharray="3 3"/>` : ""}
      <line x1="0" y1="0" x2="${px}" y2="${py}" stroke="#4d94ff" stroke-width="1.5"/>
      <circle cx="${px}" cy="${py}" r="4.5" fill="#e7eaee"/>
      <text x="${-R - 2}" y="12" fill="#737c88" font-size="9" text-anchor="start">short</text>
      <text x="${R - 2}" y="12" fill="#737c88" font-size="9" text-anchor="end">open</text>
    </svg>`;
  }

  // ---- page --------------------------------------------------------------------------------
  function mount(host) {
    const calc = CALCS[host.dataset.calc];
    if (!calc) return;
    host.innerHTML = `
      <div class="calc-tool">
        <form class="card" onsubmit="return false">
          ${calc.inputs.map((i) => `<div class="field"><label for="in-${i.key}">${i.label}</label><input id="in-${i.key}" type="number" step="any" value="${i.value}"></div>`).join("")}
        </form>
        <div>
          ${calc.chart ? '<div class="card smith-card" data-chart></div>' : ""}
          <div class="result-grid" data-results></div>
          <div class="card" style="margin-top:16px">
            <h3 style="margin:0 0 10px">Math steps</h3>
            <div class="steps" data-steps></div>
            <p class="dim small" style="margin-top:14px">${calc.refs}</p>
          </div>
        </div>
      </div>`;
    const run = () => {
      const v = {};
      for (const i of calc.inputs) v[i.key] = parseFloat(host.querySelector(`#in-${i.key}`).value);
      if (Object.values(v).some((x) => !Number.isFinite(x))) return;
      const r = calc.solve(v);
      host.querySelector("[data-results]").innerHTML = r.results
        .map(([k, val, primary]) => `<div class="result"><div class="k">${k}</div><div class="v${primary ? " accent" : ""}">${val}</div></div>`)
        .join("");
      host.querySelector("[data-steps]").innerHTML = r.steps.map((s, i) => `<div>${i + 1}. ${s}</div>`).join("");
      const chart = host.querySelector("[data-chart]");
      if (chart && r.gamma) chart.innerHTML = smithSvg(r.gamma);
    };
    host.querySelector("form").addEventListener("input", run);
    run();
  }

  document.querySelectorAll("[data-calc]").forEach(mount);
})();
