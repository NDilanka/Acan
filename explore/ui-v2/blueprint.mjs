import * as k from "/home/factory-user/.factory/skills/anatomy/kit/iso-kit.mjs";
import { ASSETS } from "./assets.mjs";
import { renderScene, projectionFor } from "./scene.mjs";
import { GHOSTS, STAGES, CATALOG_DEFAULTS } from "./acquit.mjs";

const MACHINES = ["intake", "splitter", "gate", "assembler", "bank", "hopper", "tank", "buffer", "kiln", "dock", "clock", "radar"];
const FITTINGS = ["breaker", "valve", "retry"];

function thumb(kind) {
  const scene = { entities: [{ id: "t", kind, x: 0, y: 0, state: "ok", params: kind === "bank" ? { cells: 3, busy: 0, queued: 0 } : {} }] };
  const W = 84, H = 50;
  const P = projectionFor(k, scene, ASSETS, { width: W, height: H, azimuth: 35, elevation: 30, pad: 6 });
  return renderScene(k, ASSETS, scene, P, { width: W, height: H, label: ASSETS[kind].label }).svg;
}
const tile = (kind) => `<button class="tile" data-kind="${kind}" title="${ASSETS[kind].label} · ${ASSETS[kind].maps}"><div class="iso th">${thumb(kind)}</div><span>${ASSETS[kind].label.replace(" + conveyor", "")}</span></button>`;

export const BP_HTML = `
<div class="fl drawer" id="drawer">
  <div class="dh"><b>Catalog</b><span>drag, click, or <kbd>A</kbd></span></div>
  <h6>Machines</h6><div class="tiles">${MACHINES.map(tile).join("")}</div>
  <h6>Fittings <small>snap onto a belt</small></h6><div class="tiles fit">${FITTINGS.map(tile).join("")}</div>
</div>
<div class="qa" id="qa"><input id="qaIn" placeholder="Add a machine or fitting" autocomplete="off" spellcheck="false"><div class="qal" id="qaList"></div></div>
<div class="phint" id="phint"></div>
<div class="toast" id="toast"></div>`;

export const BP_CSS = `
body:not([data-mode=live]) .hint{opacity:0}
.drawer{left:16px;top:72px;bottom:178px;width:212px;flex-direction:column;align-items:stretch;gap:0;padding:10px 8px 8px;overflow:auto;scrollbar-width:thin;scrollbar-color:#333 transparent;opacity:0;translate:-12px 0;pointer-events:none}
body[data-mode=blueprint] .drawer{opacity:1;translate:0 0;pointer-events:auto}
.drawer .dh{display:flex;align-items:baseline;gap:8px;padding:0 4px 6px}
.drawer .dh b{font-weight:600;font-size:12.5px;color:#e8eef5}
.drawer .dh span{margin-left:auto;font:10px var(--mono);color:var(--dim);white-space:nowrap}
.drawer h6{margin:10px 4px 6px;font:500 9.5px var(--mono);letter-spacing:.14em;text-transform:uppercase;color:var(--dim)}
.drawer h6 small{letter-spacing:.02em;text-transform:none;color:var(--faint);margin-left:4px}
.tiles{display:grid;grid-template-columns:1fr 1fr;gap:2px}
.tile{display:flex;flex-direction:column;align-items:center;gap:1px;min-width:0;padding:3px 2px 5px;border-radius:9px;border:1px solid transparent;color:var(--dim);font-size:10.5px;line-height:1.15;text-align:center;transition:background .15s,border-color .15s,color .15s;touch-action:none}
.tiles.fit{grid-template-columns:repeat(3,1fr)}.tiles.fit .th,.tiles.fit .th svg{width:60px;height:40px}
.tile:hover{background:rgba(140,190,240,.07);border-color:rgba(140,190,240,.16);color:#dbe8f5}
.tile[aria-pressed=true]{background:rgba(122,184,230,.12);border-color:rgba(122,184,230,.45);color:#fff}
.tile .th{width:84px;height:50px;background:transparent;pointer-events:none}.tile .th svg{width:84px;height:50px}
body.placing .stage,body.connecting .stage{cursor:crosshair}
.ent.placing{pointer-events:none}
.ent.placing .iso-line{stroke:#bfe3ff;opacity:1}
.ent.placing.bad .iso-line{stroke:var(--fail)}
.belt[data-ghost] .iso-line{stroke:var(--ghost);opacity:.9}
.belt[data-ghost] .iso-dots circle{fill:var(--ghost)}
.belt[data-ghost].lit .iso-line{stroke:#d4ecff}
.phint{position:fixed;z-index:9;pointer-events:none;transform:translate(16px,14px);padding:4px 8px;border-radius:7px;background:rgba(8,16,26,.86);border:1px solid rgba(122,184,230,.3);font:10.5px var(--mono);color:#bcd9ef;white-space:nowrap;opacity:0;transition:opacity .15s}
.phint.on{opacity:1}.phint.bad{border-color:rgba(232,96,74,.5);color:#f3b2a6}
.qa{position:fixed;z-index:10;width:280px;border-radius:12px;background:var(--glass-hi);backdrop-filter:blur(20px);-webkit-backdrop-filter:blur(20px);border:1px solid var(--line2);box-shadow:var(--shadow);padding:6px;display:none}
.qa.open{display:block}
.qa input{width:100%;height:30px;border-radius:8px;border:1px solid var(--line2);background:rgba(0,0,0,.35);color:#eee;font:12.5px var(--sans);padding:0 10px;outline:none}
.qa input:focus{border-color:rgba(122,184,230,.5)}
.qal{display:flex;flex-direction:column;margin-top:4px;max-height:240px;overflow:auto}
.qal button{display:flex;align-items:baseline;gap:8px;white-space:nowrap;padding:6px 8px;border-radius:7px;text-align:left;font-size:12px;color:#ddd}
.qal button small{font:10px var(--mono);color:var(--dim);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.qal button[aria-selected=true]{background:rgba(122,184,230,.14)}
.chip.prop{opacity:1;translate:0 0;border-color:rgba(225,164,60,.6);border-style:dashed;color:#f1d5a2}
.chip.prop .mono{background:transparent;border:1px dashed var(--strain);color:var(--strain)}
.chip.prop::after{content:"proposed";font:9.5px var(--mono);letter-spacing:.04em;text-transform:none;color:#c9a466;font-weight:400}
.firstlight{position:absolute;left:0;top:0;width:10px;height:10px;margin:-5px 0 0 -5px;border-radius:50%;background:#fff;box-shadow:0 0 10px 4px rgba(255,236,190,.85),0 0 28px 10px rgba(255,220,150,.35);offset-rotate:0deg;pointer-events:none}
.inkfront{stroke:#ffe6b0;stroke-width:1.3;filter:drop-shadow(0 0 3px rgba(255,220,150,.9))}
.toast{position:fixed;left:50%;bottom:var(--slot);z-index:9;transform:translate(-50%,8px);padding:7px 12px;border-radius:9px;background:var(--glass-hi);border:1px solid var(--line2);box-shadow:var(--shadow);font-size:12px;color:#e6e6e6;opacity:0;pointer-events:none;transition:opacity .25s,transform .35s var(--ease)}
.toast.on{opacity:1;transform:translate(-50%,0)}
.steps{display:grid;grid-template-columns:repeat(4,1fr);gap:4px}
.steps div{display:flex;flex-direction:column;gap:5px;font-size:10.5px;color:var(--faint);line-height:1.2}
.steps i{height:3px;border-radius:2px;background:#262626}
.steps .done{color:var(--dim)}.steps .done i{background:#8fb7d6}
.steps .cur{color:#e6f1fb}.steps .cur i{background:linear-gradient(90deg,#8fd3ff,#8fd3ff 60%,#2a3a48)}
body[data-mode=live] .steps .done i{background:#9a9a9a}
.stepnote{margin-top:6px;font:10px var(--mono);color:var(--faint)}
.propcard{padding:10px;border-radius:10px;border:1px dashed rgba(225,164,60,.45);background:rgba(225,164,60,.05);font-size:12px;color:#ddd;display:flex;flex-direction:column;gap:6px}
.propcard .who{font:10px var(--mono);letter-spacing:.1em;text-transform:uppercase;color:#d9b06a}
.propcard small{font:10px var(--mono);color:var(--dim)}
.propcard .row{display:flex;gap:6px;margin-top:2px}
.propcard .row button{flex:1;height:28px;border-radius:7px;border:1px solid var(--line2);font-size:12px}
.propcard .row .yes{background:#e8e8e8;color:#0a0a0a;border-color:#e8e8e8}
.flow{display:flex;align-items:center;gap:8px;padding:6px 0;border-top:1px solid var(--line);font-size:12px;color:#d6d6d6}
.flow:first-of-type{border-top:0}
.flow .arrow{font:11px var(--mono);color:var(--dim)}
.flow button{margin-left:auto;height:22px;min-width:22px;padding:0 6px;border-radius:6px;border:1px solid var(--line2);font:11px var(--mono);color:var(--dim)}
.flow button:hover{color:#fff}
.flow.sug{color:var(--dim)}.flow.sug .nm{border-bottom:1px dashed #444}
.addbelt{margin-top:6px;width:100%;height:28px;border-radius:7px;border:1px dashed rgba(122,184,230,.4);color:#bcd9ef;font-size:12px}
.defs{display:grid;grid-template-columns:1fr auto;gap:4px 10px;font-size:12px;color:#cfcfcf}
.defs span:nth-child(2n){font:11px var(--mono);color:#e6e6e6}
.lowc{font:9.5px var(--mono);padding:1px 6px;border-radius:4px;border:1px dashed var(--line2);color:var(--dim);text-transform:none;letter-spacing:0}
.bud{display:flex;align-items:center;gap:8px;font-size:12px;color:#d6d6d6}
.bud input{width:64px;height:26px;border-radius:6px;border:1px solid var(--line2);background:rgba(0,0,0,.3);color:#fff;font:12px var(--mono);padding:0 8px;outline:none}
.bud input:focus{border-color:rgba(122,184,230,.55)}
.checks div{display:flex;gap:8px;padding:4px 0;font-size:12px;color:#cfcfcf}
.checks i{font-style:normal;font:11px var(--mono);width:14px;color:var(--faint)}.checks .ok i{color:var(--ok)}
.checks small{margin-left:auto;font:10px var(--mono);color:var(--faint)}
.yaml{font:10.5px/1.55 var(--mono);color:#9aa7b4;background:rgba(0,0,0,.28);border:1px solid var(--line);border-radius:9px;padding:8px 10px;white-space:pre;overflow:auto}
.yaml .f{color:var(--dim);display:block;margin:2px 0}.yaml .f:not(:first-child){margin-top:8px}
.yaml .add{color:#9fd6b4;display:block}.yaml .add::before{content:"+ ";color:#4f8a66}
.yaml .ctx{display:block;color:#5d6873}.yaml .ctx::before{content:"  "}
.agentpick{display:flex;flex-direction:column;gap:2px;margin-top:6px;padding:4px;border-radius:9px;border:1px solid var(--line2);background:rgba(0,0,0,.25)}
.agentpick button{display:flex;align-items:center;gap:8px;padding:6px 8px;border-radius:7px;font-size:12px;color:#ddd;text-align:left}
.agentpick button:hover:not(:disabled){background:rgba(255,255,255,.06)}
.agentpick button:disabled{color:#555;cursor:default}
.agentpick small{margin-left:auto;font:10px var(--mono);color:var(--dim)}
.simres{font-size:12px;color:#d6d6d6;padding:8px 10px;border-radius:9px;background:rgba(170,230,190,.05);border:1px solid rgba(170,230,190,.15)}
.simres small{display:block;margin-top:4px;font:10px var(--mono);color:var(--dim)}
.simres .simrun{margin-top:6px;height:24px;padding:0 8px;border-radius:7px;border:1px solid rgba(170,230,190,.3);font-size:11.5px;color:#cfe0d4;display:inline-flex;align-items:center;gap:6px}
.acts button.pri{background:#e8e8e8;color:#0a0a0a;border-color:#e8e8e8}
body[data-mode=blueprint] .acts button.pri{background:#cfe6fb;border-color:#cfe6fb}
`;

export const BP_SCRIPT = `
const GHOST = ${JSON.stringify(GHOSTS)};
const STAGES = ${JSON.stringify(STAGES)};
const DEFAULTS = ${JSON.stringify(CATALOG_DEFAULTS)};
const PLACEABLE = ${JSON.stringify([...MACHINES, ...FITTINGS])};
const SLUG = { intake: "clients", splitter: "proxy", gate: "auth", assembler: "service", bank: "workers", hopper: "queue", tank: "db", buffer: "cache", kiln: "model-call", dock: "external", clock: "schedule", radar: "exporter", breaker: "breaker", valve: "limiter", retry: "retry" };
const AGENTS = [{ name: "Droid", perm: "can build" }, { name: "Claude Code", perm: "can build" }, { name: "Codex", perm: "passive · no MCP", off: true }];
const BP = { placing: null, connecting: null, ink: null, mouse: [innerWidth / 2, innerHeight / 2], picker: false, qa: 0, qaMode: "add", pt: null, undo: [] };
const foot = (e) => { const a = ASSETS[e.kind], s = e.size || 1; return { hw: (a.w * s) / 2, hd: (a.d * s) / 2 }; };
const isFloor = (e) => ASSETS[e.kind].family === "floor";
const isFit = (e) => ASSETS[e.kind].family === "fitting";

function toast(t) { const el = $("#toast"); el.textContent = t; el.classList.add("on"); clearTimeout(toast.t); toast.t = setTimeout(() => el.classList.remove("on"), 2600); }
function phint(t, bad) { const el = $("#phint"); el.textContent = t || ""; el.classList.toggle("on", !!t); el.classList.toggle("bad", !!bad); const at = BP.placing && BP.pt ? BP.pt : BP.mouse; el.style.left = at[0] + "px"; el.style.top = at[1] + "px"; }
function waiting(d) {
  const b = $(".bl .wait b"), n = Math.max(0, +b.textContent + d), you = ENT.you;
  b.textContent = n; you.params.waiting = n; you.reading = n + " waiting"; dirtyScene = true; animate();
}
function agentRow(name, text, since) { for (const r of document.querySelectorAll(".bl .agent")) if (r.querySelector("strong").textContent === name) { r.classList.toggle("off", /idle|not connected/.test(text)); r.childNodes[2].textContent = " " + text; r.querySelector("span").textContent = since || ""; } }
function ghostCounts() {
  const g = Object.entries(GHOST).filter(([id]) => ENT[id]);
  const live = g.filter(([, v]) => v.stage !== "built" && v.stage !== "proposed").length, prop = g.filter(([, v]) => v.stage === "proposed").length;
  MODES.blueprint = "BLUEPRINT · " + live + " GHOST" + (live === 1 ? "" : "S") + (prop ? " · " + prop + " PROPOSED" : "") + " · .orrery/system.yaml";
  if (document.body.dataset.mode === "blueprint") $("#watermark").textContent = MODES.blueprint;
}
function addChip(e) {
  const d = document.createElement("div");
  d.className = "chip"; d.dataset.id = e.id; d.dataset.state = e.state;
  d.innerHTML = '<span class="mono">' + e.mono + "</span>" + e.name + '<span class="rd">' + e.reading + "</span>";
  $("#chips").appendChild(d);
}
function syncChip(id) {
  const e = ENT[id], c = $('.chip[data-id="' + id + '"]'); if (!c || window.__redacting) return;
  c.dataset.state = e.state; c.childNodes[1].textContent = e.name; c.querySelector(".rd").textContent = e.reading;
  c.classList.toggle("prop", GHOST[id] && GHOST[id].stage === "proposed");
}

function arm(kind, cx = BP.mouse[0], cy = BP.mouse[1]) {
  if (document.body.dataset.mode !== "blueprint") setMode("blueprint");
  disarm(); if (selected) select(null);
  const e = { id: "__place", kind, x: 0, y: 0, size: 1, state: "ghost", params: kind === "bank" ? { cells: 3, busy: 0, queued: 0 } : {} };
  SCENE.entities.push(e); ENT.__place = e; BP.placing = e;
  document.body.classList.add("placing");
  for (const t of document.querySelectorAll(".tile")) t.setAttribute("aria-pressed", t.dataset.kind === kind);
  movePlace(cx, cy);
}
function disarm() {
  if (!BP.placing) return;
  SCENE.entities.splice(SCENE.entities.indexOf(BP.placing), 1); delete ENT.__place; BP.placing = null;
  document.body.classList.remove("placing"); phint("");
  for (const t of document.querySelectorAll(".tile")) t.setAttribute("aria-pressed", false);
  dirtyScene = true; animate();
}
function snapToBelt(x, y) {
  let best = null;
  for (const b of SCENE.belts) {
    if (b.ghost) continue;
    for (let i = 0; i < b.pts.length - 1; i++) {
      const [a, c] = [b.pts[i], b.pts[i + 1]], dx = c[0] - a[0], dy = c[1] - a[1], L = dx * dx + dy * dy || 1;
      const t = Math.max(0.12, Math.min(0.88, ((x - a[0]) * dx + (y - a[1]) * dy) / L));
      const px = a[0] + dx * t, py = a[1] + dy * t, d = Math.hypot(x - px, y - py);
      if (d < 22 && (!best || d < best.d)) best = { d, x: Math.round(px), y: Math.round(py), belt: b };
    }
  }
  return best;
}
function hit(e, x, y, pad) {
  const f = foot(e);
  for (const o of SCENE.entities) {
    if (o === e || isFloor(o) || o.id.endsWith("~g")) continue;
    const g = foot(o);
    if (Math.abs(o.x - x) < f.hw + g.hw + pad && Math.abs(o.y - y) < f.hd + g.hd + pad) return o;
  }
  return null;
}
function movePlace(cx, cy) {
  const e = BP.placing; if (!e) return;
  BP.pt = [cx, cy];
  let [x, y] = toWorld(cx, cy), why = "";
  if (isFit(e)) {
    const s = snapToBelt(x, y), o = s && hit(e, s.x, s.y, 2);
    if (s) { x = s.x; y = s.y; }
    e.on = s ? s.belt : null; e.bad = !s || !!o;
    why = !s ? "fittings snap onto a belt" : o ? "too close to " + o.name : "on " + ENT[s.belt.from].name + " → " + ENT[s.belt.to].name + " · Enter or click places";
  } else {
    x = Math.round(x / 6) * 6; y = Math.round(y / 6) * 6;
    const o = hit(e, x, y, 6); e.bad = !!o;
    why = o ? "overlaps " + o.name : "Enter or click places · arrows nudge · Esc cancels";
  }
  e.x = x; e.y = y;
  phint(ASSETS[e.kind].label + " · " + why, e.bad);
  dirtyScene = true; animate();
}
function bpSnap(label) {
  BP.undo.push({ label, ents: SCENE.entities.filter((e) => e.id !== "__place").map((e) => ({ ...e, params: e.params && { ...e.params } })), belts: SCENE.belts.map((b) => ({ ...b })), ghost: JSON.parse(JSON.stringify(GHOST)) });
  if (BP.undo.length > 50) BP.undo.shift();
}
function bpUndo() {
  const u = BP.undo.pop(); if (!u) return toast("Nothing to undo on the blueprint.");
  disarm(); endConnect(); closeQA();
  SCENE.entities = u.ents; SCENE.belts = u.belts;
  for (const id of Object.keys(ENT)) delete ENT[id];
  for (const e of SCENE.entities) ENT[e.id] = e;
  for (const id of Object.keys(GHOST)) delete GHOST[id];
  Object.assign(GHOST, u.ghost);
  for (const c of document.querySelectorAll(".chip")) if (!ENT[c.dataset.id]) c.remove();
  for (const e of SCENE.entities) if (GHOST[e.id] && !$('.chip[data-id="' + e.id + '"]')) addChip(e);
  dirtyScene = true; renderPlant(); ghostCounts();
  if (selected && !ENT[selected]) select(null); else refreshInspector();
  toast("Undid " + u.label + ". system.yaml and layout.yaml are back to before.");
}
function slugFor(kind) { const base = SLUG[kind]; let s = base, n = 2; while (ENT[s]) s = base + "-" + n++; return s; }
function commit() {
  const e = BP.placing; if (!e) return;
  if (e.bad) { toast("Can't place here. " + $("#phint").textContent.split(" · ")[1]); return; }
  const id = slugFor(e.kind);
  bpSnap("adding " + id);
  SCENE.entities.splice(SCENE.entities.indexOf(e), 1); delete ENT.__place; BP.placing = null;
  const ne = { id, kind: e.kind, x: e.x, y: e.y, size: 1, state: "ghost", params: e.params, mono: id.replace(/[^a-z]/g, "").slice(0, 2).toUpperCase(), name: id, reading: "ghost", sub: "drawn by you · not built" };
  SCENE.entities.push(ne); ENT[id] = ne;
  const d = DEFAULTS[e.kind] && DEFAULTS[e.kind][0], m = d && /^([\\d.,]+)\\s*(.*)$/.exec(d[1]);
  GHOST[id] = { stage: "ghost", by: "you", on: e.on ? ENT[e.on.from].name + " → " + ENT[e.on.to].name : null, budget: m ? { metric: d[0], value: +(parseFloat(m[1].replace(",", "")) * 2).toFixed(2), unit: m[2] } : null };
  document.body.classList.remove("placing"); phint("");
  for (const t of document.querySelectorAll(".tile")) t.setAttribute("aria-pressed", false);
  addChip(ne); renderPlant(); select(id); ghostCounts();
  toast("Wrote " + id + " to .orrery/system.yaml and layout.yaml");
}

function routeCost(pts, a, b) {
  const segs = pts.slice(1).map((p, i) => [pts[i], p]);
  const box = ([p, q]) => [Math.min(p[0], q[0]), Math.max(p[0], q[0]), Math.min(p[1], q[1]), Math.max(p[1], q[1])];
  let cost = segs.reduce((n, [p, q]) => n + Math.abs(q[0] - p[0]) + Math.abs(q[1] - p[1]), 0);
  for (const o of SCENE.entities) {
    if (o === a || o === b || isFloor(o) || o.id.startsWith("__") || o.id.endsWith("~g")) continue;
    const f = foot(o);
    for (const sg of segs) { const [x0, x1, y0, y1] = box(sg); if (x1 > o.x - f.hw - 3 && x0 < o.x + f.hw + 3 && y1 > o.y - f.hd - 3 && y0 < o.y + f.hd + 3) cost += 1000; }
  }
  for (const bt of SCENE.belts) for (let i = 0; i < bt.pts.length - 1; i++) {
    const [x0, x1, y0, y1] = box([bt.pts[i], bt.pts[i + 1]]);
    for (const sg of segs) { const [u0, u1, v0, v1] = box(sg); if (u1 >= x0 && u0 <= x1 && v1 >= y0 && v0 <= y1) cost += 60; }
  }
  return cost;
}
function route(a, b) {
  const A = foot(a), B = foot(b), dx = b.x - a.x, dy = b.y - a.y, sx = Math.sign(dx) || 1, sy = Math.sign(dy) || 1, c = [];
  const ylo = Math.max(a.y - A.hd, b.y - B.hd) + 6, yhi = Math.min(a.y + A.hd, b.y + B.hd) - 6;
  if (ylo <= yhi && Math.abs(dx) > A.hw + B.hw) { const y = Math.round((ylo + yhi) / 2); c.push([[a.x + sx * A.hw, y], [b.x - sx * B.hw, y]]); }
  const xlo = Math.max(a.x - A.hw, b.x - B.hw) + 6, xhi = Math.min(a.x + A.hw, b.x + B.hw) - 6;
  if (xlo <= xhi && Math.abs(dy) > A.hd + B.hd) { const x = Math.round((xlo + xhi) / 2); c.push([[x, a.y + sy * A.hd], [x, b.y - sy * B.hd]]); }
  c.push([[a.x + sx * A.hw, a.y], [b.x, a.y], [b.x, b.y - sy * B.hd]]);
  c.push([[a.x, a.y + sy * A.hd], [a.x, b.y], [b.x - sx * B.hw, b.y]]);
  return c.map((pts) => [routeCost(pts, a, b), pts]).sort((p, q) => p[0] - q[0])[0][1];
}
function addBelt(from, to) {
  if (SCENE.belts.some((b) => (b.from === from && b.to === to) || (b.from === to && b.to === from))) return;
  bpSnap("the belt " + ENT[from].name + " → " + ENT[to].name);
  SCENE.belts.push({ from, to, ghost: true, pts: route(ENT[from], ENT[to]) });
  dirtyScene = true; animate(); refreshInspector();
}
function suggestions(id) {
  const e = ENT[id];
  return SCENE.entities.filter((o) => o.id !== id && !isFloor(o) && !isFit(o) && !o.id.startsWith("__") && !o.id.endsWith("~g") && !SCENE.belts.some((b) => (b.from === id && b.to === o.id) || (b.to === id && b.from === o.id)))
    .sort((p, q) => Math.hypot(p.x - e.x, p.y - e.y) - Math.hypot(q.x - e.x, q.y - e.y)).slice(0, 2)
    .map((o) => (o.x <= e.x ? [o.id, id] : [id, o.id]));
}
function removeGhost(id) {
  const g = GHOST[id]; if (!g) return;
  bpSnap("removing " + ENT[id].name);
  SCENE.entities = SCENE.entities.filter((e) => e.id !== id);
  SCENE.belts = SCENE.belts.filter((b) => b.from !== id && b.to !== id);
  delete ENT[id]; delete GHOST[id];
  const c = $('.chip[data-id="' + id + '"]'); if (c) c.remove();
  select(null); dirtyScene = true; animate(); ghostCounts();
}

function stageIndex(g) { return g.stage === "proposed" ? -1 : STAGES.findIndex((s) => s.id === g.stage); }
function yamlFor(id) {
  const e = ENT[id], g = GHOST[id], flows = SCENE.belts.filter((b) => b.from === id || b.to === id), merged = g.stage === "built";
  const L = (t) => '<span class="' + (merged ? "ctx" : "add") + '">' + t + "</span>";
  let s = '<span class="f">.orrery/system.yaml</span><span class="ctx">entities:</span>' + L("  " + id + ":") + L("    kind: " + e.kind);
  s += L("    status: " + (merged ? "live" : g.stage === "proposed" ? "proposed" : "ghost")) + L("    designed_by: " + (g.by === "you" ? "architect" : g.by.toLowerCase().replace(" ", "-")));
  if (g.on) s += L("    on: " + g.on);
  if (g.budget) s += L("    budgets:") + L("      " + g.budget.metric.replace(/ /g, "_") + ": < " + g.budget.value + (g.budget.unit ? " " + g.budget.unit : ""));
  if (flows.length) { s += L("    flows:"); for (const b of flows) s += L("      - " + (b.from === id ? "to: " + b.to : "from: " + b.from)); }
  if (g.agent) s += L("    territory: " + g.branch);
  s += '<span class="f">.orrery/layout.yaml</span>' + L("  " + id + ": [" + Math.round(e.x) + ", " + Math.round(e.y) + "]");
  return s;
}
function ghostHtml(e) {
  const id = e.id, g = GHOST[id], a = ASSETS[e.kind], idx = stageIndex(g);
  let h = '<div class="fig" id="fig"><div class="iso" id="figPlant"></div><div class="read">' + e.reading + "<small>" + (g.stage === "built" ? "live" : "not built") + "</small></div>";
  h += '<div class="tools"><button>Open in system.yaml</button></div></div>';
  if (g.stage === "proposed") {
    h += '<div class="propcard"><span class="who">Proposed by ' + g.by + " · waiting on you</span>" + g.why + "<small>" + g.evidence + "</small><small>confidence " + g.conf + " · reversal " + g.cost + " · risk " + ((1 - g.conf) * 0.5).toFixed(2) + "</small>";
    h += '<div class="row"><button class="yes" data-act="approve">Approve</button><button data-act="reject">Reject</button></div><small>Agents propose. Only you approve ghosts (A6).</small></div>';
  }
  h += '<div class="sec"><h5>Ghost to live<i>' + (idx < 0 ? "not yet a ghost" : STAGES[Math.min(idx, 3)].note) + "</i></h5><div class=\\"steps\\">" + STAGES.map((s, i) => '<div class="' + (i < idx || g.stage === "built" ? "done" : i === idx ? "cur" : "") + '"><i></i>' + s.label + (i === 1 && idx > 1 && !g.sim && g.by === "you" ? " · skipped" : "") + "</div>").join("") + "</div>";
  if (g.by !== "you" && g.stage !== "proposed") h += '<div class="stepnote">proposed by ' + g.by + " · approved by you</div>";
  h += "</div>";
  if (g.stage === "ghost" || g.stage === "simulated") {
    h += '<div class="acts"><button data-act="sim">' + (g.sim ? "Estimate again" : "Quick estimate") + '</button><button class="pri" data-act="hand">Hand to agent</button></div>';
    if (BP.picker) h += '<div class="agentpick">' + AGENTS.map((ag) => { const busy = ag.name === "Droid" && GHOST.radar && GHOST.radar.stage === "building"; return '<button data-act="give" data-agent="' + ag.name + '"' + (ag.off || busy ? " disabled" : "") + ">" + DRONE_ICON_JS + ag.name + "<small>" + (ag.off ? ag.perm : busy ? "busy on otel radar" : ag.perm) + "</small></button>"; }).join("") + "</div>";
  }
  if (g.stage === "building") h += '<div class="acts"><button data-do="branch">Open branch diff</button><button class="pri" data-act="merge">Merge PR (demo)</button></div>';
  if (g.sim) h += '<div class="simres">' + g.sim + "<small>Catalog defaults, low confidence.</small><button class='simrun' data-act='simrun'>Open the full run <kbd>3</kbd></button></div>";
  if (isFit(e)) h += '<div class="sec"><h5>On belt</h5><div class="flow"><span class="nm">' + (g.on || "not on a belt") + "</span></div></div>";
  else {
    const flows = SCENE.belts.filter((b) => b.from === id || b.to === id);
    h += '<div class="sec"><h5>Flows<i>' + flows.length + " drawn</i></h5>";
    h += flows.map((b) => '<div class="flow"><span class="nm">' + ENT[b.from].name + '</span><span class="arrow">→</span><span class="nm">' + ENT[b.to].name + "</span>" + (b.ghost && g.stage !== "built" ? '<button data-act="flip" data-from="' + b.from + '" data-to="' + b.to + '" title="Reverse">⇄</button><button data-act="unbelt" data-from="' + b.from + '" data-to="' + b.to + '" title="Remove">×</button>' : "") + "</div>").join("");
    if (g.stage === "ghost" || g.stage === "simulated" || g.stage === "proposed") {
      h += suggestions(id).map(([f, t]) => '<div class="flow sug"><span class="nm">' + ENT[f].name + '</span><span class="arrow">→</span><span class="nm">' + ENT[t].name + '</span><button data-act="belt" data-from="' + f + '" data-to="' + t + '">+ add</button></div>').join("");
      h += '<button class="addbelt" data-act="connect">Draw a belt to any machine <kbd>B</kbd></button>';
    }
    h += "</div>";
  }
  if (DEFAULTS[e.kind] && g.stage !== "built") h += '<div class="sec"><h5>Catalog defaults<i><span class="lowc">low confidence</span></i></h5><div class="defs">' + DEFAULTS[e.kind].map(([k2, v]) => "<span>" + k2 + "</span><span>" + v + "</span>").join("") + "</div></div>";
  if (g.budget) h += '<div class="sec"><h5>Budget<i>agents hillclimb against it</i></h5><div class="bud">' + g.budget.metric + ' under <input data-act="budget" value="' + g.budget.value + '"> ' + g.budget.unit + '</div><div class="lock">You set this. Agents read it and can only propose a change (K16).</div></div>';
  const declared = g.stage === "built" ? (g.declared || "branch merged").replace("branch ", "") : g.declared || "no code yet", observed = g.stage === "built" ? e.reading : "no traffic yet";
  h += '<div class="sec"><h5>Three sources</h5>' + [["designed", true, "system.yaml · " + id, "status " + (g.stage === "built" ? "live" : g.stage)], ["declared", g.stage === "built" || !!g.declared, declared, g.stage === "building" ? "open branch" : ""], ["observed", g.stage === "built", observed, ""]].map(([s, ok, w, d]) => '<div class="src"><span class="n">' + s + '</span><span class="' + (ok ? "ok" : "no") + '">' + (ok ? "●" : "○") + "</span><b>" + w + "</b><span class='d'>" + d + "</span></div>").join("") + "</div>";
  if (g.agent) {
    h += '<div class="sec"><h5>Territory<i>' + g.agent + " · " + g.branch + "</i></h5>";
    if (g.checks) h += '<div class="checks">' + g.checks.map((c) => '<div class="' + (c.ok ? "ok" : "") + '"><i>' + (c.ok ? "✓" : "·") + "</i>" + c.text + (c.locked ? "<small>🔒 K16</small>" : "") + "</div>").join("") + "</div>";
    h += "</div>";
  }
  h += '<div class="sec"><h5>In the file<i>' + (g.stage === "built" ? "merged" : "written on every edit") + '</i></h5><div class="yaml" id="yaml">' + yamlFor(id) + "</div></div>";
  if (g.stage === "ghost" || g.stage === "simulated") h += '<div class="acts"><button data-act="remove">Remove ghost <kbd>Del</kbd></button></div>';
  return h;
}
function refreshInspector() {
  if (!selected || !GHOST[selected]) return;
  const top = $("#ibody").scrollTop;
  $("#ibody").innerHTML = ghostHtml(ENT[selected]); $("#ibody").scrollTop = top;
  $("#iKind").textContent = ASSETS[ENT[selected].kind].label + " · " + ASSETS[ENT[selected].kind].maps;
  $("#ih").dataset.state = ENT[selected].state;
  requestAnimationFrame(drawCloseup);
}

function simulate(id) {
  const e = ENT[id], g = GHOST[id], up = SCENE.belts.find((b) => b.to === id), down = SCENE.belts.find((b) => b.from === id);
  const who = up ? ENT[up.from].name : down ? ENT[down.to].name : null;
  g.sim = who ? "Predicted: " + who + " p99 182 → 173 ms. Budgets met 2 of 2. Confidence 0.35." : "Nothing flows through it yet, so the run has nothing to predict. Draw a belt first.";
  if (who) g.stage = "simulated";
  refreshInspector();
}
function give(id, agent) {
  const e = ENT[id], g = GHOST[id], a = ASSETS[e.kind], f = foot(e);
  Object.assign(g, { stage: "building", agent, since: "now", branch: "ghost/" + id, declared: "branch ghost/" + id, checks: [{ ok: false, text: "Code matches the drawing: kind, flows, name" }, ...(g.budget ? [{ ok: false, text: "Budget: " + g.budget.metric + " under " + g.budget.value + " " + g.budget.unit, locked: true }] : []), { ok: false, text: "Harness passes in CI" }] });
  BP.picker = false;
  const sc = { id: "scaffold-" + id, of: id, kind: "scaffold", x: e.x, y: e.y, state: "ok", params: { w: Math.min(140, a.w + 10), d: a.d + 10, h: Math.max(26, a.top * 0.8) } };
  const bot = { id: "bot-" + id, of: id, kind: "bot", x: ENT.you.x, y: ENT.you.y, size: 1, state: "ok", params: { target: [e.x, e.y, a.top * 0.6] }, name: agent };
  SCENE.entities.push(sc, bot); ENT[sc.id] = sc; ENT[bot.id] = bot;
  fly(bot, e.x + f.hw + 22, e.y - f.hd - 4, 1600);
  e.reading = "under construction"; syncChip(id);
  agentRow(agent, "building " + id, "now");
  refreshInspector(); toast(agent + " claimed the territory for " + id);
}
function fly(ent, x, y, ms, done) {
  const x0 = ent.x, y0 = ent.y, t0 = performance.now();
  const step = (now) => {
    const t = Math.min(1, (now - t0) / ms), q = t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
    ent.x = x0 + (x - x0) * q; ent.y = y0 + (y - y0) * q - Math.sin(Math.PI * t) * 14;
    dirtyScene = true; animate();
    if (t < 1) requestAnimationFrame(step); else if (done) done();
  };
  requestAnimationFrame(step);
}

function ink(id) {
  const e = ENT[id], g = GHOST[id];
  g.stage = "built"; if (g.checks) for (const c of g.checks) c.ok = true;
  const copy = { ...e, id: id + "~g", of: id, state: "ghost" };
  SCENE.entities.splice(SCENE.entities.indexOf(e), 0, copy);
  e.state = "ok"; e.reading = g.afterReading || "first light"; syncChip(id);
  BP.ink = { id, t0: performance.now(), ms: 1900 };
  const sc = ENT[id === "radar" ? "scaffold" : "scaffold-" + id], bot = ENT[id === "radar" ? "bot" : "bot-" + id];
  if (sc) { SCENE.entities = SCENE.entities.filter((x) => x !== sc); delete ENT[sc.id]; }
  if (bot) fly(bot, bot.x - 40, bot.y - 34, 1400, () => { bot.state = "idle"; dirtyScene = true; animate(); });
  agentRow(g.agent, "idle · finished " + ENT[id].name, "");
  renderPlant(); inkLoop();
}
function inkBox(id) {
  const e = ENT[id], a = ASSETS[e.kind], f = foot(e), pts = [];
  for (const sx of [-1, 1]) for (const sy of [-1, 1]) for (const z of [0, a.top * (e.size || 1)]) pts.push(k.iso([e.x + sx * f.hw, e.y + sy * f.hd, z], P));
  const xs = pts.map((p) => p[0]), ys = pts.map((p) => p[1]);
  return { x0: Math.min(...xs) - 8, x1: Math.max(...xs) + 8, y0: Math.min(...ys) - 8, y1: Math.max(...ys) + 8 };
}
function applyInk() {
  if (!BP.ink) return;
  const svg = plant.querySelector("svg"), { id, t0, ms } = BP.ink, t = Math.min(1, (performance.now() - t0) / ms), q = t < 0.5 ? 2 * t * t : 1 - Math.pow(-2 * t + 2, 2) / 2;
  const b = inkBox(id), front = b.y1 - (b.y1 - b.y0) * q;
  let defs = svg.querySelector("#inkdefs");
  if (!defs) {
    svg.insertAdjacentHTML("afterbegin", '<defs id="inkdefs"><clipPath id="inkSolid" clipPathUnits="userSpaceOnUse"><rect id="inkR1"/></clipPath><clipPath id="inkGhost" clipPathUnits="userSpaceOnUse"><rect id="inkR2"/></clipPath></defs>');
    svg.insertAdjacentHTML("beforeend", '<line class="inkfront" id="inkLine"/>');
    for (const g of svg.querySelectorAll('.ent[data-id="' + id + '"]')) g.setAttribute("clip-path", "url(#inkSolid)");
    for (const g of svg.querySelectorAll('.ent[data-id="' + id + '~g"]')) g.setAttribute("clip-path", "url(#inkGhost)");
  }
  const set = (el, o) => { for (const key in o) el.setAttribute(key, o[key]); };
  set(svg.querySelector("#inkR1"), { x: b.x0, y: front, width: b.x1 - b.x0, height: b.y1 - front + 2 });
  set(svg.querySelector("#inkR2"), { x: b.x0, y: b.y0 - 40, width: b.x1 - b.x0, height: front - b.y0 + 40 });
  set(svg.querySelector("#inkLine"), { x1: b.x0 + 14, x2: b.x1 - 14, y1: front, y2: front, opacity: t < 1 ? 1 : 0 });
}
function inkLoop() {
  applyInk();
  const { id, t0, ms } = BP.ink;
  if (performance.now() - t0 < ms) return requestAnimationFrame(inkLoop);
  BP.ink = null;
  SCENE.entities = SCENE.entities.filter((x) => x.id !== id + "~g");
  for (const b of SCENE.belts) if (b.from === id || b.to === id) b.ghost = false;
  renderPlant(); ghostCounts(); refreshInspector();
  setTimeout(() => firstLight(id), 120);
}
function firstLight(id) {
  for (const b of SCENE.belts.filter((x) => x.from === id || x.to === id)) {
    const pts = b.from === id ? b.pts.slice().reverse() : b.pts;
    const d = "M" + pts.map((p) => k.iso([p[0], p[1], 1.7], P).map((v) => v.toFixed(1)).join(" ")).join("L");
    const dot = document.createElement("div"); dot.className = "firstlight"; dot.style.offsetPath = "path('" + d + "')";
    world.appendChild(dot);
    dot.animate([{ offsetDistance: "0%", opacity: 0 }, { offsetDistance: "8%", opacity: 1, offset: 0.08 }, { offsetDistance: "100%", opacity: 1, offset: 0.85 }, { offsetDistance: "100%", opacity: 0, transform: "scale(2.4)" }], { duration: 1500, easing: "cubic-bezier(.4,0,.2,1)" }).onfinish = () => dot.remove();
  }
  toast("First light on " + ENT[id].name + ". " + (GHOST[id].afterReading || "Traffic is flowing."));
}

function afterRender() {
  for (const g of plant.querySelectorAll(".ent")) {
    const id = g.dataset.id;
    if (id === "__place") { g.classList.add("placing"); g.classList.toggle("bad", !!BP.placing && BP.placing.bad); }
  }
  for (const id in GHOST) syncChip(id);
  if (BP.ink) applyInk();
}

$("#ibody").addEventListener("click", (ev) => {
  const b = ev.target.closest("[data-act]"); if (!b || !selected) return;
  const id = selected, act = b.dataset.act;
  if (act === "approve") { GHOST[id].stage = "ghost"; ENT[id].reading = "ghost"; ENT[id].sub = "proposed by " + GHOST[id].by + " · approved"; waiting(-1); ghostCounts(); refreshInspector(); toast("Approved. " + ENT[id].name + " is now a ghost on the blueprint."); }
  if (act === "simrun") { setMode("simulate"); return; }
  if (act === "reject") { waiting(-1); removeGhost(id); toast("Rejected. Droid gets the reason as a notification."); }
  if (act === "belt") addBelt(b.dataset.from, b.dataset.to);
  if (act === "unbelt" || act === "flip") { const i = SCENE.belts.findIndex((x) => x.from === b.dataset.from && x.to === b.dataset.to); bpSnap((act === "flip" ? "reversing " : "removing ") + "the belt " + ENT[b.dataset.from].name + " → " + ENT[b.dataset.to].name); const old = SCENE.belts.splice(i, 1)[0]; if (act === "flip") SCENE.belts.push({ ...old, from: old.to, to: old.from, pts: old.pts.slice().reverse() }); dirtyScene = true; animate(); refreshInspector(); }
  if (act === "connect") startConnect();
  if (act === "sim") simulate(id);
  if (act === "hand") { BP.picker = !BP.picker; refreshInspector(); }
  if (act === "give") give(id, b.dataset.agent);
  if (act === "merge") ink(id);
  if (act === "remove") { removeGhost(id); toast("Removed " + id + ". Ctrl Z brings it back."); }
});
$("#ibody").addEventListener("input", (ev) => {
  if (ev.target.dataset.act !== "budget" || !selected) return;
  GHOST[selected].budget.value = ev.target.value; $("#yaml").innerHTML = yamlFor(selected);
});
function startConnect() { if (!selected) return; BP.connecting = selected; document.body.classList.add("connecting"); phint("pick from the list or click a machine · Esc cancels"); }
function endConnect() { BP.connecting = null; document.body.classList.remove("connecting"); phint(""); }

for (const t of document.querySelectorAll(".tile")) {
  t.addEventListener("pointerdown", (ev) => { ev.preventDefault(); arm(t.dataset.kind, ev.clientX, ev.clientY); BP.fromTile = true; });
}
addEventListener("pointermove", (ev) => {
  BP.mouse = [ev.clientX, ev.clientY];
  if (BP.placing) { if (!BP.pending) { BP.pending = true; requestAnimationFrame(() => { BP.pending = false; movePlace(BP.mouse[0], BP.mouse[1]); }); } }
  else if (BP.connecting) phint($("#phint").textContent);
}, true);
addEventListener("pointerdown", (ev) => {
  if ((BP.placing || BP.connecting) && ev.button === 0 && ev.target.closest("#stage")) { ev.stopImmediatePropagation(); }
}, true);
addEventListener("pointerup", (ev) => {
  if (ev.button !== 0) return;
  if (BP.placing) {
    if (ev.target.closest("#stage")) { ev.stopImmediatePropagation(); commit(); }
    BP.fromTile = false; return;
  }
  if (BP.connecting && ev.target.closest("#stage")) {
    ev.stopImmediatePropagation();
    const g = ev.target.closest(".ent"), from = BP.connecting, to = g && (ENT[g.dataset.id].of || g.dataset.id);
    endConnect(); closeQA();
    if (to && to !== from && !isFloor(ENT[to]) && !isFit(ENT[to])) addBelt(ENT[to].x <= ENT[from].x ? to : from, ENT[to].x <= ENT[from].x ? from : to);
  }
}, true);

function besideSelection() {
  if (!selected || !anchors[selected]) return BP.mouse.slice();
  const [ax, ay] = anchors[selected];
  return [Math.min(innerWidth - 480, view.x + ax * view.s + 90), view.y + ay * view.s + 30];
}
function openQA(mode = "add", at = BP.mouse.slice()) {
  if (document.body.dataset.mode !== "blueprint") setMode("blueprint");
  BP.qaMode = mode;
  const q = $("#qa"); q.classList.add("open");
  q.style.left = Math.min(innerWidth - 300, at[0]) + "px"; q.style.top = Math.min(innerHeight - 300, at[1]) + "px";
  $("#qaIn").placeholder = mode === "belt" ? "Belt from " + ENT[BP.connecting].name + " to… nearest first" : "Add a machine or fitting";
  BP.qaAt = at; $("#qaIn").value = ""; BP.qa = 0; listQA(); $("#qaIn").focus();
}
function closeQA() { $("#qa").classList.remove("open"); $("#qaIn").blur(); }
const NOT_TARGET = new Set(["scaffold", "bot", "inspection", "crane"]);
function beltTargets() {
  const e = ENT[BP.connecting]; if (!e) return [];
  return SCENE.entities.filter((o) => o !== e && !isFloor(o) && !isFit(o) && !NOT_TARGET.has(o.kind) && !o.id.startsWith("__") && !o.id.endsWith("~g") && !SCENE.belts.some((b) => (b.from === e.id && b.to === o.id) || (b.to === e.id && b.from === o.id)))
    .sort((p, q) => Math.hypot(p.x - e.x, p.y - e.y) - Math.hypot(q.x - e.x, q.y - e.y)).map((o) => o.id);
}
function qaItems() {
  const s = $("#qaIn").value.trim().toLowerCase();
  if (BP.qaMode === "belt") return beltTargets().filter((id) => !s || (ENT[id].name + " " + ASSETS[ENT[id].kind].label).toLowerCase().includes(s));
  return PLACEABLE.filter((kd) => !s || (ASSETS[kd].label + " " + ASSETS[kd].maps + " " + kd).toLowerCase().includes(s));
}
function freeSpot(near) {
  const e = BP.placing, o = ENT[near];
  if (!e || !e.bad || !o) return;
  const toS = (x, y) => { const p = k.iso([x, y, 0], P); return [p[0] * view.s + view.x, p[1] * view.s + view.y]; };
  for (const r of [70, 100, 130, 170]) for (let a = 0; a < 8; a++) {
    movePlace(...toS(o.x + Math.cos((a * Math.PI) / 4) * r, o.y + Math.sin((a * Math.PI) / 4) * r));
    if (!e.bad) return;
  }
}
function qaChoose(v) {
  closeQA();
  if (BP.qaMode === "add") { const near = selected; arm(v, ...BP.qaAt); return freeSpot(near); }
  const from = BP.connecting; endConnect();
  if (from && ENT[v]) addBelt(ENT[v].x <= ENT[from].x ? v : from, ENT[v].x <= ENT[from].x ? from : v);
}
function listQA() {
  const items = qaItems(); BP.qa = Math.min(BP.qa, Math.max(0, items.length - 1));
  const row = BP.qaMode === "belt" ? (id) => ENT[id].name + "<small>" + ASSETS[ENT[id].kind].label.toLowerCase() + " · " + Math.round(Math.hypot(ENT[id].x - ENT[BP.connecting].x, ENT[id].y - ENT[BP.connecting].y)) + " away</small>" : (kd) => ASSETS[kd].label + "<small>" + ASSETS[kd].maps + "</small>";
  $("#qaList").innerHTML = items.map((v, i) => '<button data-kind="' + v + '" aria-selected="' + (i === BP.qa) + '">' + row(v) + "</button>").join("") || "<button disabled>" + (BP.qaMode === "belt" ? "No machine left to connect" : "No match in the v1 catalog") + "</button>";
}
$("#qaIn").addEventListener("input", () => { BP.qa = 0; listQA(); });
$("#qaIn").addEventListener("keydown", (ev) => {
  const items = qaItems();
  if (ev.key === "ArrowDown") { BP.qa = Math.min(items.length - 1, BP.qa + 1); listQA(); ev.preventDefault(); }
  if (ev.key === "ArrowUp") { BP.qa = Math.max(0, BP.qa - 1); listQA(); ev.preventDefault(); }
  if (ev.key === "Enter" && items[BP.qa]) qaChoose(items[BP.qa]);
  if (ev.key === "Escape") { closeQA(); endConnect(); }
  ev.stopPropagation();
});
$("#qaList").addEventListener("click", (ev) => { const b = ev.target.closest("[data-kind]"); if (b) qaChoose(b.dataset.kind); });

addEventListener("keydown", (ev) => {
  if (typing(ev)) return;
  if (ev.key === "Escape" && (BP.placing || BP.connecting || $("#qa").classList.contains("open"))) { disarm(); endConnect(); closeQA(); ev.stopImmediatePropagation(); return; }
  const key = ev.key.toLowerCase();
  if ((ev.ctrlKey || ev.metaKey) && key === "z" && document.body.dataset.mode === "blueprint") { ev.preventDefault(); ev.stopImmediatePropagation(); bpUndo(); return; }
  if (BP.placing && (ev.key.startsWith("Arrow") || ev.key === "Enter")) {
    ev.preventDefault(); ev.stopImmediatePropagation();
    if (ev.key === "Enter") return commit();
    const d = ev.shiftKey ? 40 : 10, [x, y] = BP.pt;
    return movePlace(x + (ev.key === "ArrowLeft" ? -d : ev.key === "ArrowRight" ? d : 0), y + (ev.key === "ArrowUp" ? -d : ev.key === "ArrowDown" ? d : 0));
  }
  if (!bare(ev)) return;
  if (key === "a") { ev.preventDefault(); openQA("add", besideSelection()); ev.stopImmediatePropagation(); }
  if (key === "b" && selected && GHOST[selected]) { ev.preventDefault(); startConnect(); openQA("belt", besideSelection()); ev.stopImmediatePropagation(); }
  if ((ev.key === "Delete" || ev.key === "Backspace") && selected && GHOST[selected] && ["ghost", "simulated"].includes(GHOST[selected].stage)) { const n = ENT[selected].name; removeGhost(selected); toast("Removed " + n + ". Ctrl Z brings it back."); }
}, true);

function bpInit(params) {
  ghostCounts();
  if (params.get("draft")) {
    BP.mouse = [0, 0];
    const at = (x, y) => { const p = k.iso([x, y, 0], P); return [p[0] * view.s + view.x, p[1] * view.s + view.y]; };
    arm("buffer", ...at(48, 168)); commit();
    addBelt("api", "cache"); if (params.get("draft") === "sim") simulate("cache");
    if (params.get("draft") === "give") { give("cache", "Claude Code"); }
  }
  if (params.get("place")) { const [x, y] = (params.get("at") || "0.5,0.5").split(",").map(Number); BP.mouse = [innerWidth * x, innerHeight * y]; arm(params.get("place")); }
  if (params.get("qa")) { BP.mouse = [innerWidth * 0.42, innerHeight * 0.4]; openQA(); $("#qaIn").value = params.get("qa"); listQA(); }
  if (params.get("bsel")) select(params.get("bsel"));
  if (params.get("ink")) { select("radar"); ink("radar"); }
}
`;
