<script lang="ts">
  import { tip } from "./tip";
  import { hint } from "./help";
  // The structure in 3-D, wires and patches coloured by current magnitude, with the gain surface
  // around it. Clicking a wire selects the card that made it; a selected wire's end points can be
  // dragged, snapping to other wires' ends (to make a junction) or to a grid (Shift: no snap).
  import { onMount } from "svelte";
  import * as THREE from "three";
  import { OrbitControls } from "three/addons/controls/OrbitControls.js";
  import type { Analysis, PatternView, Slice, SliceRequest } from "./types";
  import { arrl, ramp, rampGradient } from "./colormap";

  let {
    analysis,
    currents = null,
    patchCurrents = null,
    pattern = null,
    highlight = null,
    selected = null,
    handles = null,
    fitKey = 0,
    nearField = null,
    onpick,
    ondrag,
  }: {
    analysis: Analysis | null;
    currents?: number[] | null;
    patchCurrents?: number[] | null;
    pattern?: PatternView | null;
    highlight?: number | null;
    /** A near-field slice to draw as a coloured plane. */
    nearField?: { request: SliceRequest; slice: Slice } | null;
    /** Changes when a different model is loaded, which re-fits the camera; edits keep the view. */
    fitKey?: number;
    /** Deck line of the selected card; its segments are drawn highlighted. */
    selected?: number | null;
    /** The selected wire's two end points, when they may be dragged. */
    handles?: [Vec3, Vec3] | null;
    onpick?: (line: number | null) => void;
    ondrag?: (end: 0 | 1, point: Vec3) => void;
  } = $props();

  type Vec3 = [number, number, number];

  let showPattern = $state(true);
  let showCurrents = $state(true);
  let host: HTMLDivElement;
  let ready = $state(false);

  let renderer: THREE.WebGLRenderer;
  let camera: THREE.PerspectiveCamera;
  let controls: OrbitControls;
  const scene = new THREE.Scene();
  const wireGroup = new THREE.Group();
  const patchGroup = new THREE.Group();
  const patternGroup = new THREE.Group();
  const groundGroup = new THREE.Group();
  const handleGroup = new THREE.Group();
  const fieldGroup = new THREE.Group();
  scene.add(wireGroup, patchGroup, patternGroup, groundGroup, handleGroup, fieldGroup);
  let wireMesh: THREE.InstancedMesh | null = null;

  /** Size and centre of the structure, which set the camera, the pattern scale and the grid. */
  let extent = { centre: new THREE.Vector3(), size: 1 };
  let fittedFor: Analysis | null = null;
  let fittedKey = -1;

  const dispose = (g: THREE.Group) => {
    for (const o of g.children) {
      const m = o as THREE.Mesh;
      m.geometry?.dispose();
      (Array.isArray(m.material) ? m.material : [m.material]).forEach((x) => x?.dispose());
    }
    g.clear();
  };

  function measure(a: Analysis) {
    const box = new THREE.Box3();
    for (const s of a.segments) {
      if (s.auxiliary) continue;
      box.expandByPoint(new THREE.Vector3(...s.a));
      box.expandByPoint(new THREE.Vector3(...s.b));
    }
    for (const p of a.patches) for (const q of p.outline) box.expandByPoint(new THREE.Vector3(...q));
    const size = Math.max(box.getSize(new THREE.Vector3()).length(), 1e-3);
    return { centre: box.getCenter(new THREE.Vector3()), size };
  }

  /** Patches as flat polygons, coloured by surface current density, with their outlines. */
  function buildPatches(a: Analysis, cur: number[] | null) {
    dispose(patchGroup);
    if (!a.patches.length) return;
    const n = a.patches.length;
    const max = cur && cur.length === n ? cur.reduce((m, v) => Math.max(m, v), 1e-30) : 0;
    const pos: number[] = [];
    const col: number[] = [];
    const edges: number[] = [];
    a.patches.forEach((pt, i) => {
      const rgb: [number, number, number] =
        showCurrents && max > 0 && cur ? ramp(cur[i] / max) : [0.55, 0.6, 0.68];
      const o = pt.outline;
      for (let k = 1; k + 1 < o.length; k++) {
        for (const q of [o[0], o[k], o[k + 1]]) {
          pos.push(...q);
          col.push(...rgb);
        }
      }
      o.forEach((q, k) => edges.push(...q, ...o[(k + 1) % o.length]));
    });
    const geo = new THREE.BufferGeometry();
    geo.setAttribute("position", new THREE.Float32BufferAttribute(pos, 3));
    geo.setAttribute("color", new THREE.Float32BufferAttribute(col, 3));
    geo.computeVertexNormals();
    const mesh = new THREE.Mesh(
      geo,
      new THREE.MeshLambertMaterial({ vertexColors: true, side: THREE.DoubleSide, transparent: true, opacity: 0.85 }),
    );
    const line = new THREE.BufferGeometry();
    line.setAttribute("position", new THREE.Float32BufferAttribute(edges, 3));
    const outline = new THREE.LineSegments(line, new THREE.LineBasicMaterial({ color: 0x0e131c, transparent: true, opacity: 0.5 }));
    patchGroup.add(mesh, outline);
  }

  function buildWires(a: Analysis, cur: number[] | null, hi: number | null, sel: number | null) {
    dispose(wireGroup);
    wireMesh = null;
    const n = a.segments.length;
    if (!n) return;
    const max = cur && cur.length === n ? Math.max(...cur, 1e-30) : 0;
    const thick = extent.size * 0.005;
    const geo = new THREE.CylinderGeometry(1, 1, 1, 8, 1, false);
    geo.translate(0, 0.5, 0);
    const mesh = new THREE.InstancedMesh(geo, new THREE.MeshStandardMaterial({ roughness: 0.5, metalness: 0.2 }), n);
    const up = new THREE.Vector3(0, 1, 0);
    const m = new THREE.Matrix4();
    const c = new THREE.Color();
    a.segments.forEach((s, i) => {
      const pa = new THREE.Vector3(...s.a);
      const d = new THREE.Vector3(...s.b).sub(pa);
      const len = d.length();
      const q = new THREE.Quaternion().setFromUnitVectors(up, d.clone().normalize());
      const picked = sel != null && s.origin === sel;
      const r = Math.max(s.radius, thick) * (hi === i + 1 ? 3 : picked ? 1.8 : 1);
      // Dummy wires of current sources are drawn at zero size.
      m.compose(pa, q, s.auxiliary ? new THREE.Vector3(0, 0, 0) : new THREE.Vector3(r, len, r));
      mesh.setMatrixAt(i, m);
      if (hi === i + 1) c.setRGB(1, 0.2, 0.9);
      else if (picked) c.setRGB(0.25, 0.95, 1);
      else if (showCurrents && max > 0 && cur) c.setRGB(...ramp(cur[i] / max));
      else c.setRGB(0.75, 0.8, 0.88);
      mesh.setColorAt(i, c);
    });
    wireGroup.add(mesh);
    wireMesh = mesh;
    // Feed points.
    for (const seg of a.sources) {
      const s = a.segments[seg - 1];
      if (!s) continue;
      const ball = new THREE.Mesh(
        new THREE.SphereGeometry(thick * 3.2, 16, 12),
        new THREE.MeshStandardMaterial({ color: 0xff3b5c, emissive: 0x661020 }),
      );
      ball.position.set((s.a[0] + s.b[0]) / 2, (s.a[1] + s.b[1]) / 2, (s.a[2] + s.b[2]) / 2);
      wireGroup.add(ball);
    }
  }

  function buildGround(a: Analysis) {
    dispose(groundGroup);
    const span = extent.size * 2.2;
    const grid = new THREE.GridHelper(span, 22, 0x3a4a63, 0x243041);
    grid.rotation.x = Math.PI / 2;
    if (a.has_ground) {
      grid.position.set(extent.centre.x, extent.centre.y, 0);
      const plane = new THREE.Mesh(
        new THREE.PlaneGeometry(span, span),
        new THREE.MeshBasicMaterial({ color: 0x3b2f1f, transparent: true, opacity: 0.35, side: THREE.DoubleSide, depthWrite: false }),
      );
      plane.position.copy(grid.position);
      groundGroup.add(plane);
    } else {
      grid.position.copy(extent.centre);
      (grid.material as THREE.Material).opacity = 0.35;
      (grid.material as THREE.Material).transparent = true;
    }
    groundGroup.add(grid);
    const axes = new THREE.AxesHelper(extent.size * 0.15);
    axes.position.set(extent.centre.x - extent.size * 0.6, extent.centre.y - extent.size * 0.6, a.has_ground ? 0 : extent.centre.z);
    groundGroup.add(axes);
  }

  /** The gain surface on the ARRL scale, coloured over the 30 dB below the peak. */
  function buildPattern(p: PatternView | null) {
    dispose(patternGroup);
    if (!p || !showPattern) return;
    const range = 30;
    const scale = extent.size * 0.75;
    const nt = p.thetas.length;
    const np = p.phis.length;
    const pos: number[] = [];
    const col: number[] = [];
    const origin = extent.centre.clone();
    for (let j = 0; j < np; j++) {
      const ph = (p.phis[j] * Math.PI) / 180;
      for (let i = 0; i < nt; i++) {
        const th = (p.thetas[i] * Math.PI) / 180;
        const g = p.gain[j]?.[i] ?? -999;
        const t = Math.max(0, (g - p.peak_gain + range) / range);
        const r = arrl(g, p.peak_gain) * scale;
        pos.push(origin.x + r * Math.sin(th) * Math.cos(ph), origin.y + r * Math.sin(th) * Math.sin(ph), origin.z + r * Math.cos(th));
        col.push(...ramp(t));
      }
    }
    const idx: number[] = [];
    for (let j = 0; j + 1 < np; j++)
      for (let i = 0; i + 1 < nt; i++) {
        const k = j * nt + i;
        idx.push(k, k + nt, k + 1, k + 1, k + nt, k + nt + 1);
      }
    const geo = new THREE.BufferGeometry();
    geo.setAttribute("position", new THREE.Float32BufferAttribute(pos, 3));
    geo.setAttribute("color", new THREE.Float32BufferAttribute(col, 3));
    geo.setIndex(idx);
    geo.computeVertexNormals();
    const surf = new THREE.Mesh(
      geo,
      new THREE.MeshLambertMaterial({ vertexColors: true, transparent: true, opacity: 0.55, side: THREE.DoubleSide, depthWrite: false }),
    );
    const wire = new THREE.LineSegments(
      new THREE.WireframeGeometry(geo),
      new THREE.LineBasicMaterial({ color: 0xffffff, transparent: true, opacity: 0.07 }),
    );
    patternGroup.add(surf, wire);
  }

  /** Spheres on the selected wire's ends, and the line between them while one is dragged. */
  function buildHandles(h: [Vec3, Vec3] | null) {
    dispose(handleGroup);
    if (!h) return;
    const r = extent.size * 0.012;
    h.forEach((pt, end) => {
      const ball = new THREE.Mesh(
        new THREE.SphereGeometry(r, 16, 12),
        new THREE.MeshStandardMaterial({ color: 0x40f0ff, emissive: 0x0b4a55, transparent: true, opacity: 0.9 }),
      );
      ball.position.set(...pt);
      ball.userData.end = end;
      handleGroup.add(ball);
    });
  }

  /** The slice's magnitudes as a texture on its plane, over the 40 dB below the peak. */
  function buildNearField(nf: { request: SliceRequest; slice: Slice } | null) {
    for (const o of fieldGroup.children) {
      const m = o as THREE.Mesh;
      const mat = m.material as THREE.MeshBasicMaterial;
      mat.map?.dispose();
    }
    dispose(fieldGroup);
    if (!nf) return;
    const { request: r, slice } = nf;
    const data = new Uint8Array(r.n_u * r.n_v * 4);
    const floor = slice.max * 1e-2; // 40 dB down
    slice.values.forEach((v, i) => {
      const t = v > floor ? Math.log10(v / floor) / 2 : 0;
      const [cr, cg, cb] = ramp(Math.min(1, t));
      data.set([cr * 255, cg * 255, cb * 255, 40 + 190 * Math.min(1, t)], 4 * i);
    });
    const tex = new THREE.DataTexture(data, r.n_u, r.n_v, THREE.RGBAFormat);
    tex.magFilter = THREE.LinearFilter;
    tex.minFilter = THREE.LinearFilter;
    tex.needsUpdate = true;
    const w = r.u[1] - r.u[0];
    const h = r.v[1] - r.v[0];
    const mesh = new THREE.Mesh(
      new THREE.PlaneGeometry(w, h),
      new THREE.MeshBasicMaterial({ map: tex, transparent: true, side: THREE.DoubleSide, depthWrite: false }),
    );
    // PlaneGeometry lies in its own XY plane, u along x and v along y (texture row 0 at the bottom).
    const cu = (r.u[0] + r.u[1]) / 2;
    const cv = (r.v[0] + r.v[1]) / 2;
    if (r.plane === "xy") {
      mesh.position.set(cu, cv, r.offset);
    } else if (r.plane === "xz") {
      mesh.rotation.x = Math.PI / 2;
      mesh.position.set(cu, r.offset, cv);
    } else {
      // Z then X: local x to world y, local y to world z, the normal to world x.
      mesh.rotation.order = "ZXY";
      mesh.rotation.set(Math.PI / 2, 0, Math.PI / 2);
      mesh.position.set(r.offset, cu, cv);
    }
    fieldGroup.add(mesh);
  }

  // Picking and dragging.
  const ray = new THREE.Raycaster();
  let down: { x: number; y: number } | null = null;
  let drag: { end: 0 | 1; plane: THREE.Plane; ball: THREE.Mesh; point: THREE.Vector3; line: THREE.Line } | null = null;
  let snapped = $state<string | null>(null);

  function pointer(e: PointerEvent) {
    const r = renderer.domElement.getBoundingClientRect();
    return new THREE.Vector2(((e.clientX - r.left) / r.width) * 2 - 1, -((e.clientY - r.top) / r.height) * 2 + 1);
  }

  function toScreen(v: THREE.Vector3) {
    const r = renderer.domElement.getBoundingClientRect();
    const q = v.clone().project(camera);
    return new THREE.Vector2(((q.x + 1) / 2) * r.width, ((1 - q.y) / 2) * r.height);
  }

  /** Where a dragged end lands: on another wire's end within 10 px, else on the grid. */
  function snap(e: PointerEvent, raw: THREE.Vector3): THREE.Vector3 {
    snapped = null;
    if (e.shiftKey || !analysis) return raw;
    const at = toScreen(raw);
    let best: THREE.Vector3 | null = null;
    let bestD = 10;
    for (const sg of analysis.segments) {
      if (selected != null && sg.origin === selected) continue;
      for (const q of [sg.a, sg.b]) {
        const v = new THREE.Vector3(...q);
        const d = toScreen(v).distanceTo(at);
        if (d < bestD) {
          bestD = d;
          best = v;
        }
      }
    }
    if (best) {
      snapped = "end of another wire";
      return best;
    }
    const step = 10 ** (Math.floor(Math.log10(extent.size)) - 2);
    return new THREE.Vector3(...raw.toArray().map((x) => Math.round(x / step) * step));
  }

  function onDown(e: PointerEvent) {
    down = { x: e.clientX, y: e.clientY };
    if (!handles || !ondrag) return;
    ray.setFromCamera(pointer(e), camera);
    const hit = ray.intersectObjects(handleGroup.children, false)[0];
    if (!hit) return;
    const ball = hit.object as THREE.Mesh;
    const end = ball.userData.end as 0 | 1;
    const normal = camera.getWorldDirection(new THREE.Vector3());
    const plane = new THREE.Plane().setFromNormalAndCoplanarPoint(normal, ball.position);
    const other = new THREE.Vector3(...handles[1 - end]);
    const line = new THREE.Line(
      new THREE.BufferGeometry().setFromPoints([other, ball.position.clone()]),
      new THREE.LineBasicMaterial({ color: 0x40f0ff }),
    );
    handleGroup.add(line);
    drag = { end, plane, ball, point: ball.position.clone(), line };
    controls.enabled = false;
    renderer.domElement.setPointerCapture(e.pointerId);
  }

  function onMove(e: PointerEvent) {
    if (!drag) return;
    ray.setFromCamera(pointer(e), camera);
    const raw = ray.ray.intersectPlane(drag.plane, new THREE.Vector3());
    if (!raw) return;
    drag.point = snap(e, raw);
    drag.ball.position.copy(drag.point);
    const pos = drag.line.geometry.getAttribute("position") as THREE.BufferAttribute;
    pos.setXYZ(1, drag.point.x, drag.point.y, drag.point.z);
    pos.needsUpdate = true;
  }

  function onUp(e: PointerEvent) {
    const click = down && Math.hypot(e.clientX - down.x, e.clientY - down.y) < 4;
    down = null;
    if (drag) {
      const d = drag;
      drag = null;
      snapped = null;
      controls.enabled = true;
      renderer.domElement.releasePointerCapture(e.pointerId);
      if (!click) ondrag?.(d.end, d.point.toArray() as Vec3);
      else buildHandles(handles);
      return;
    }
    if (!click || !onpick || !analysis) return;
    ray.setFromCamera(pointer(e), camera);
    const hit = wireMesh ? ray.intersectObject(wireMesh, false)[0] : undefined;
    const seg = hit?.instanceId != null ? analysis.segments[hit.instanceId] : undefined;
    onpick(seg && seg.origin > 0 ? seg.origin : null);
  }

  function fit() {
    const d = extent.size * 1.9;
    camera.position.set(extent.centre.x + d * 0.8, extent.centre.y - d * 1.0, extent.centre.z + d * 0.6);
    camera.near = extent.size / 1000;
    camera.far = extent.size * 100;
    camera.updateProjectionMatrix();
    controls.target.copy(extent.centre);
    controls.update();
  }

  $effect(() => {
    if (!ready || !analysis) return;
    if (fittedFor !== analysis) {
      extent = measure(analysis);
      buildGround(analysis);
      if (fittedKey !== fitKey) fit();
      fittedFor = analysis;
      fittedKey = fitKey;
    }
    buildWires(analysis, currents, highlight, selected);
    buildPatches(analysis, patchCurrents);
    void showCurrents;
  });

  $effect(() => {
    if (!ready) return;
    void analysis;
    buildHandles(handles);
  });

  $effect(() => {
    if (!ready) return;
    buildNearField(nearField);
  });

  $effect(() => {
    if (!ready || !analysis) return;
    void showPattern;
    buildPattern(pattern);
  });

  onMount(() => {
    renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setPixelRatio(window.devicePixelRatio);
    host.appendChild(renderer.domElement);
    scene.background = new THREE.Color(0x0e131c);
    camera = new THREE.PerspectiveCamera(40, 1, 0.01, 1000);
    camera.up.set(0, 0, 1);
    controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;
    scene.add(new THREE.AmbientLight(0xffffff, 1.4));
    const sun = new THREE.DirectionalLight(0xffffff, 1.6);
    sun.position.set(1, -2, 3);
    scene.add(sun);
    const resize = () => {
      const { clientWidth: w, clientHeight: h } = host;
      renderer.setSize(w, h);
      camera.aspect = w / Math.max(h, 1);
      camera.updateProjectionMatrix();
    };
    const el = renderer.domElement;
    el.addEventListener("pointerdown", onDown);
    el.addEventListener("pointermove", onMove);
    el.addEventListener("pointerup", onUp);
    const ro = new ResizeObserver(resize);
    ro.observe(host);
    resize();
    ready = true;
    let frame = 0;
    const loop = () => {
      controls.update();
      renderer.render(scene, camera);
      frame = requestAnimationFrame(loop);
    };
    loop();
    return () => {
      cancelAnimationFrame(frame);
      ro.disconnect();
      renderer.dispose();
    };
  });
</script>

<div class="viewer">
  <div class="canvas" bind:this={host}></div>
  <div class="toolbar">
    <label use:tip={hint("showPattern")}><input type="checkbox" bind:checked={showPattern} /> Pattern</label>
    <label use:tip={hint("showCurrents")}><input type="checkbox" bind:checked={showCurrents} /> Currents</label>
    <button onclick={fit} use:tip={hint("fitView")}>Fit</button>
  </div>
  <div class="legend">
    <div class="bar" style:background={rampGradient()}></div>
    <div class="ticks">
      {#if showPattern && pattern}
        <span>{(pattern.peak_gain - 30).toFixed(0)}</span><span>gain, dBi</span><span>{pattern.peak_gain.toFixed(1)}</span>
      {:else}
        <span>0</span><span>|I| relative</span><span>max</span>
      {/if}
    </div>
  </div>
  {#if snapped}<div class="snap">Snapped to the {snapped}</div>{/if}
  {#if analysis}
    <div class="caption">
      {analysis.segments.length} segments · {analysis.wires} wires{#if analysis.patches.length}
        · {analysis.patches.length} patches{/if} · {analysis.ground}
    </div>
  {/if}
</div>

<style>
  .viewer {
    position: relative;
    height: 100%;
    min-height: 0;
    overflow: hidden;
    border-radius: 8px;
    border: 1px solid var(--line);
  }
  .canvas {
    position: absolute;
    inset: 0;
  }
  .toolbar {
    position: absolute;
    top: 10px;
    left: 10px;
    display: flex;
    gap: 6px;
  }
  .toolbar label,
  .toolbar button {
    background: rgba(20, 27, 39, 0.85);
    border: 1px solid var(--line);
    border-radius: 6px;
    padding: 4px 9px;
    font-size: 12px;
    color: var(--text);
    display: flex;
    align-items: center;
    gap: 5px;
    cursor: pointer;
  }
  .legend {
    position: absolute;
    right: 12px;
    bottom: 12px;
    width: 220px;
    font-size: 11px;
    color: var(--muted);
  }
  .bar {
    height: 8px;
    border-radius: 4px;
  }
  .ticks {
    display: flex;
    justify-content: space-between;
    margin-top: 3px;
  }
  .snap {
    position: absolute;
    top: 10px;
    right: 12px;
    font-size: 12px;
    color: #40f0ff;
  }
  .caption {
    position: absolute;
    left: 12px;
    bottom: 12px;
    font-size: 12px;
    color: var(--muted);
  }
</style>
