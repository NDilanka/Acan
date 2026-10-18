export function sceneBounds(scene, ASSETS) {
  const pts = [];
  for (const e of scene.entities) {
    const a = ASSETS[e.kind];
    for (const [sx, sy] of [[-1, -1], [1, -1], [1, 1], [-1, 1]]) for (const z of [0, a.top]) pts.push([e.x + (sx * a.w) / 2, e.y + (sy * a.d) / 2, z]);
  }
  return pts;
}

export function projectionFor(k, scene, ASSETS, { width, height, azimuth, elevation, scale, pivot, pad = 80 }) {
  const pts = sceneBounds(scene, ASSETS);
  if (!scale) scale = k.fitProjection(pts, width, height, { pad, azimuth, elevation }).scale;
  if (!pivot) {
    const xs = pts.map((p) => p[0]), ys = pts.map((p) => p[1]);
    pivot = [(Math.min(...xs) + Math.max(...xs)) / 2, (Math.min(...ys) + Math.max(...ys)) / 2, 20];
  }
  const p0 = k.iso(pivot, { origin: [0, 0], scale, azimuth, elevation });
  return { origin: [width / 2 - p0[0], height / 2 - p0[1]], scale, azimuth, elevation, pivot };
}

export function renderScene(k, ASSETS, scene, P0, { width, height, label = "" }) {
  const floor = [], upright = [], labels = [];
  let P = P0, focus = [0, 0, 0], grow = 1;
  const at = (p) => k.iso(p, P);
  const depth = (p) => k.depthOf([focus[0] + (p[0] - focus[0]) * grow, focus[1] + (p[1] - focus[1]) * grow, p[2] * grow], P);
  const view = k.towardViewer(P);
  const cam = k.cameraOf(P);
  const view3 = [cam.cosA * cam.cosE, cam.sinA * cam.cosE, cam.sinE];
  let tag = "";
  const wrap = (svg) => (tag ? `<g ${tag}>${svg}</g>` : svg);

  const c = {
    k, P, at, view, view3,
    label: (p, text, side = 1) => labels.push({ pt: at(p), text, side }),
    rightOf: (plan, z) => k.corners(plan, 0).map(([x, y]) => [x, y, z]).reduce((m, p) => (at(p)[0] > at(m)[0] ? p : m)),
    S: (paths, o) => k.solidSvg(paths, o),
    L: (d, o) => k.lineSvg(d, o),
    D: (pts, o) => k.dotsSvg(pts, o),
    box: (x, y, w, d, z, h, o = {}, steps = 3, bevel = 0.5) => k.solidSvg(k.slabOf({ x, y, w, d, r: o.r ?? 1.5 }, z, h, P, steps, bevel), o),
    cyl: (x, y, r, z, h, o = {}, steps = 28, bevel = 0) => k.solidSvg(k.cylinder(x, y, r, z, h, P, steps, bevel), o),
    prism: (ring, z, h, o = {}) => k.solidSvg(k.extrude(ring, z, h, P, { convex: true, bevel: 0.3 }), o),
    halo: (plan, z, h) => `<path class="iso-halo" d="${k.haloOf(plan, z, h, P)}"/>`,
    screws: (plan, inset, z) => k.dotsSvg(k.corners(plan, inset).map(([x, y]) => at([x, y, z])), { size: 0.5 }),
    lamp: (x, y, z, state) => `<g class="lamp"${state ? ` data-lamp="${state}"` : ""}>${k.solidSvg(k.cylinder(x, y, 2.4, z, 1.8, P, 16, 0), { tone: "hi", crease: "none" })}</g>`,
    floor: (svg) => floor.push({ d: 0, svg: wrap(svg) }),
    put: (center, svg, mounts = []) => {
      const inner = mounts.slice().sort((a, b) => depth(a[0]) - depth(b[0])).map((m) => m[1]).join("");
      upright.push({ d: depth(center), svg: wrap(svg + inner) });
    },
    plinth(cx, cy, w, d, h = 3) {
      const plan = { x: cx - w / 2, y: cy - d / 2, w, d, r: 3 };
      c.lastPlinth = plan;
      c.floor(c.halo(plan, 0, h) + c.S(k.slabOf(plan, 0, h, P, 4, 0.6), { tone: "lo" }) + c.L(k.planOutline(k.insetPlan(plan, 3), h, P, 3), { tone: "faint" }) + c.screws(plan, 5, h));
    },
    frontFace(hw, hd) {
      const n = [[1, 0], [-1, 0], [0, 1], [0, -1]].reduce((best, v) => (v[0] * view[0] + v[1] * view[1] > best[0] * view[0] + best[1] * view[1] ? v : best));
      return [n, n[0] ? hw : hd];
    },
    text(x, y, z, str, { size = 4, tone = "mid", anchor = "start", along } = {}) {
      const o0 = at([0, 0, 0]), a1 = at([1, 0, 0]), b1 = at([0, 1, 0]), ex = { x: [a1[0] - o0[0], a1[1] - o0[1]], y: [b1[0] - o0[0], b1[1] - o0[1]] };
      const dirs = (along ? [along, [-along[0], -along[1]]] : [[1, 0], [0, -1], [-1, 0], [0, 1]]).map((u) => [u, [u[0] * ex.x[0] + u[1] * ex.y[0], u[0] * ex.x[1] + u[1] * ex.y[1]]]);
      const [u, U] = dirs.reduce((b, d) => (d[1][0] / Math.hypot(...d[1]) > b[1][0] / Math.hypot(...b[1]) ? d : b));
      let V = [-u[1] * ex.x[0] + u[0] * ex.y[0], -u[1] * ex.x[1] + u[0] * ex.y[1]];
      if (V[1] < 0) V = [-V[0], -V[1]];
      const o = at([x, y, z]), f = (n) => n.toFixed(3);
      return k.faceTextSvg(`matrix(${f(U[0])} ${f(U[1])} ${f(V[0])} ${f(V[1])} ${f(o[0])} ${f(o[1])})`, str, { size, tone, anchor });
    },
  };

  for (const t of scene.territories || []) {
    const ents = t.entities.map((id) => scene.entities.find((e) => e.id === id)).filter(Boolean);
    if (!ents.length) continue;
    let [x0, y0, x1, y1] = [Infinity, Infinity, -Infinity, -Infinity];
    for (const e of ents) {
      const a = ASSETS[e.kind], s = e.size || 1;
      x0 = Math.min(x0, e.x - (a.w * s) / 2); x1 = Math.max(x1, e.x + (a.w * s) / 2);
      y0 = Math.min(y0, e.y - (a.d * s) / 2); y1 = Math.max(y1, e.y + (a.d * s) / 2);
    }
    [x0, y0, x1, y1] = [x0 - 12, y0 - 12, x1 + 12, y1 + 12];
    const ring = [[x0, y0], [x1, y0], [x1, y1], [x0, y1], [x0, y0]];
    const fill = ring.map((p, i) => (i ? "L" : "M") + at([p[0], p[1], 0]).map((v) => v.toFixed(1)).join(" ")).join("") + "Z";
    let svg = `<path class="terr-fill" d="${fill}"/>` + c.L(k.onTop(ring, 0.2, P), { tone: "mid", dashed: t.state === "unclaimed" });
    svg += c.text(x0 + 3, y1 + 7, 0, t.label, { size: 4.2, tone: "mid" });
    if (t.state === "passed") svg += `<g class="seal">${c.cyl(x1 - 10, y0 + 10, 6.5, 0, 1.4, { tone: "hi" }, 24)}${c.D([at([x1 - 10, y0 + 10, 1.4])], { size: 1.1, tone: "hi" })}</g>`;
    floor.push({ d: -2e6, svg: `<g class="terr" data-id="${t.id}" data-state="${t.state}"${t.stale ? " data-stale" : ""}>${svg}</g>` });
  }

  for (const b of scene.belts || []) {
    tag = `class="belt" data-from="${b.from}" data-to="${b.to}"${b.tone === "red" ? ' data-tone="red"' : ""}${b.ghost ? " data-ghost" : ""}`;
    if (b.ghost) {
      const ends = [b.pts[0], b.pts[b.pts.length - 1]].map(([x, y]) => at([x, y, 0.4]));
      floor.push({ d: -1e6, svg: `<g ${tag}>${c.L(k.onTop(b.pts, 0.4, P), { tone: "hi", dashed: true })}${c.D(ends, { size: 0.9, tone: "hi" })}</g>` });
      continue;
    }
    let svg = "";
    for (let i = 0; i < b.pts.length - 1; i++) {
      const [a, e] = [b.pts[i], b.pts[i + 1]];
      const x = Math.min(a[0], e[0]) - 5, y = Math.min(a[1], e[1]) - 5;
      svg += c.S(k.slabOf({ x, y, w: Math.abs(e[0] - a[0]) + 10, d: Math.abs(e[1] - a[1]) + 10, r: 1 }, 0, 1.6, P, 2, 0.3), { tone: "lo" });
    }
    for (let i = 0; i < b.pts.length - 1; i++) {
      const [a, e] = [b.pts[i], b.pts[i + 1]];
      const nx = Math.sign(e[1] - a[1]), ny = Math.sign(e[0] - a[0]);
      for (const off of [-3.4, 3.4]) svg += c.L(k.lineOnTop([a[0] + nx * off, a[1] - ny * off], [e[0] + nx * off, e[1] - ny * off], 1.6, P), { tone: "lo" });
    }
    svg += c.L(k.onTop(b.pts, 1.7, P), { tone: b.tone === "red" ? "red" : "hi", flow: true });
    floor.push({ d: -1e6, svg: `<g ${tag}>${svg}</g>` });
  }
  tag = "";

  if (scene.wall) {
    const { x: wx, y0, y1, gaps = [], text } = scene.wall;
    tag = `class="wall"`;
    c.floor(c.L(k.onTop([[wx, y0], [wx, y1]], 0, P), { tone: "mid", dashed: true }) + (text ? c.text(wx - 8, y1 - 4, 0, text, { size: 5, tone: "mid" }) : ""));
    for (let yy = y0; yy <= y1; yy += 22) if (!gaps.some(([a, b]) => yy > a - 4 && yy < b + 4)) c.put([wx, yy, 6], c.box(wx - 2.5, yy - 2.5, 5, 5, 0, 12, { tone: "mid" }, 2, 0.4) + c.D([at([wx, yy, 12])], { size: 0.5, tone: "hi" }));
    tag = "";
  }

  const anchors = {};
  for (const e of scene.entities) {
    const a = ASSETS[e.kind];
    tag = `class="ent" data-id="${e.id}" data-kind="${e.kind}" data-state="${e.state || "ok"}"`;
    c.lastPlinth = null;
    grow = e.size || 1; focus = [e.x, e.y, 0];
    if (grow !== 1) {
      const q0 = k.iso([e.x, e.y, 0], P0), qe = k.iso([e.x, e.y, 0], { ...P0, scale: P0.scale * grow, origin: [0, 0] });
      P = { ...P0, scale: P0.scale * grow, origin: [q0[0] - qe[0], q0[1] - qe[1]] };
    }
    c.P = P; c.id = e.id;
    const before = labels.length;
    a.build(c, { ...e.params, x: e.x, y: e.y, state: e.state, variant: e.variant, explode: e.explode || 0 });
    for (const l of labels.slice(before)) l.id = e.id;
    if (e.variant && c.lastPlinth) {
      const pl = c.lastPlinth, [n, half] = c.frontFace(pl.w / 2, pl.d / 2);
      const [px, py] = [pl.x + pl.w / 2 + n[0] * (half + 6), pl.y + pl.d / 2 + n[1] * (half + 6)];
      c.floor(c.text(px, py, 0, e.variant, { size: 5, tone: "mid", anchor: "middle", along: [-n[1], n[0]] }));
    }
    if (e.state === "fail") {
      const s = at([e.x, e.y, a.top - 4]);
      c.put([e.x, e.y, a.top + 10], `<g class="smoke">${[0, 1, 2, 3].map((i) => `<circle cx="${s[0].toFixed(1)}" cy="${s[1].toFixed(1)}" r="${(3 + i).toFixed(1)}" style="--i:${i}"/>`).join("")}</g>`);
    }
    anchors[e.id] = at([e.x, e.y, a.top + (a.explodes ? (e.explode || 0) * a.explodes : 0)]);
    P = P0; c.P = P0; grow = 1;
  }
  tag = "";

  floor.sort((a, b) => a.d - b.d);
  upright.sort((a, b) => a.d - b.d);
  const svg = k.figureSvg({ width, height, label, body: [...floor, ...upright].map((p) => p.svg) });
  return { svg, anchors, labels };
}

export function layoutCallouts(labels, { gap = 15, pad = 28, scale = 1, offset = [0, 0] } = {}) {
  const groups = {};
  for (const l of labels) (groups[l.id] ||= []).push(l);
  const out = [];
  for (const list of Object.values(groups)) {
    const pts = list.map((l) => ({ text: l.text, ax: offset[0] + l.pt[0] * scale, ay: offset[1] + l.pt[1] * scale }));
    const col = Math.max(...pts.map((p) => p.ax)) + pad;
    // Anchors farther from the label column sort lower, so leaders fan out instead of crossing when parts share a row.
    const key = (p) => p.ay + (col - p.ax) * 0.3;
    pts.sort((a, b) => key(a) - key(b));
    let prev = -Infinity;
    for (const p of pts) { p.ty = Math.max(p.ay, prev + gap); prev = p.ty; p.tx = col; }
    const shift = (pts[pts.length - 1].ty - pts[pts.length - 1].ay) / 2;
    for (const p of pts) p.ty -= Math.min(shift, p.ty - (pts[0].ay - gap * 2));
    out.push(...pts);
  }
  return out;
}

export function calloutHtml(callouts, width, height) {
  const lines = callouts.map((c) => `<path d="M${c.ax.toFixed(1)} ${c.ay.toFixed(1)}L${(c.tx - 8).toFixed(1)} ${c.ty.toFixed(1)}H${(c.tx - 2).toFixed(1)}"/><circle cx="${c.ax.toFixed(1)}" cy="${c.ay.toFixed(1)}" r="1.8"/>`).join("");
  const texts = callouts.map((c) => `<span class="xl" style="left:${c.tx.toFixed(1)}px;top:${c.ty.toFixed(1)}px">${c.text}</span>`).join("");
  return `<svg class="leaders" width="${width}" height="${height}">${lines}</svg>${texts}`;
}
