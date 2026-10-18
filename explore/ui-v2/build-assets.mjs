import { writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { ISO_CSS } from "/home/factory-user/.factory/skills/anatomy/kit/iso-kit.mjs";
import { ASSETS, DEFERRED, FAMILIES } from "./assets.mjs";
import { runtimeScript, STATE_CSS, FOG_DEFS } from "./lib.mjs";

const HERE = dirname(fileURLToPath(import.meta.url));
const STATES = [["ok", "working"], ["idle", "idle"], ["strain", "strained"], ["fail", "failing"], ["ghost", "ghost"], ["dark", "dark"], ["fog", "fogged"]];

const one = (kind, extra = {}) => [{ id: kind, kind, x: 0, y: 0, ...extra }];
const cell = (id, title, lines, entities, opts = {}) =>
  `<figure class="cell${opts.wide ? " wide" : ""}" data-cell="${id}"${opts.fixed ? " data-fixed" : ""}${opts.explode ? " data-explode" : ""}${opts.twins ? " data-twins" : ""} data-scene='${JSON.stringify(entities)}'><div class="iso pane"></div><div class="over"></div><figcaption><b>${title}</b>${lines.map((l) => `<span>${l}</span>`).join("")}</figcaption></figure>`;

const sections = Object.entries(FAMILIES).map(([fam, name]) => {
  const items = Object.entries(ASSETS).filter(([, a]) => a.family === fam);
  return `<section><h2>${name} <small>${items.length}</small></h2><div class="grid">${items.map(([kind, a]) => cell(kind, a.label, [a.maps, `<i>${a.fixtures}</i>`], one(kind, kind === "bot" ? { params: { target: [30, 6, 6] } } : kind === "crate" ? { variant: "?" } : kind === "bank" ? { params: { cells: 4, busy: 1, queued: 7 } } : {}))).join("")}</div></section>`;
}).join("");

const TWINS = [
  { kind: "assembler", items: [
    { name: "acquit-api", variant: "NODE", mono: "JS", size: 1, params: { needle: 0.38 }, reading: "15 rps · p99 184 ms" },
    { name: "web", variant: "NODE", mono: "JS", size: 0.78, params: { needle: 0.12 }, reading: "2 rps · p99 40 ms" },
    { name: "checkout", variant: "GO", mono: "GO", size: 1.22, params: { needle: 0.74 }, reading: "80 rps · p99 410 ms" },
  ] },
  { kind: "tank", items: [
    { name: "acquit.db", variant: "SQLITE", mono: "SQ", size: 0.86, params: { extra: 2 }, reading: "48 MB · 3 files" },
    { name: "orders", variant: "POSTGRES", mono: "PG", size: 1.2, params: { extra: 0 }, reading: "31 GB · 1 file" },
    { name: "accounts", variant: "POSTGRES", mono: "PG", size: 1, params: { extra: 1 }, reading: "4 GB · 2 files" },
  ] },
  { kind: "dock", items: [
    { name: "paypal", variant: "PAYPAL", mono: "PP", size: 1, state: "fail", reading: "4.1% errors" },
    { name: "github app", variant: "GITHUB", mono: "GH", size: 0.85, reading: "0.3 rps" },
    { name: "stripe", variant: "STRIPE", mono: "ST", size: 1.1, state: "idle", reading: "idle" },
  ] },
];
const twins = `<section><h2>Telling twins apart <small>same drawing, different part</small></h2>
<p class="lede">Three layers, each owning a zoom range. Toggle them to compare. The shape always says the kind.</p>
<div class="seg toggles" id="twinCtl"><button data-t="form" aria-pressed="true">Telemetry form</button><button data-t="tile" aria-pressed="true">Recipe tile · Alt</button><button data-t="badge" aria-pressed="true">Engraved badge · close up</button></div>
<div class="legend"><div><b>Telemetry form</b><span>always on. Size follows load or data size. Counts follow config: cells are worker slots, side tanks are files, items on a belt are queue depth. Two services on the same runtime still look different because they carry different load.</span></div><div><b>Recipe tile</b><span>hold Alt, like Factorio's alt-mode. A monogram tile and the name float over every machine and stay upright at any orbit. It answers "which one is Postgres" from far away.</span></div><div><b>Engraved badge</b><span>the variant engraved on the floor in front of the machine. It is readable at close zoom and turns to face the camera.</span></div></div>
<div class="grid twins">${TWINS.map(({ kind, items }) => cell(`twins-${kind}`, `${ASSETS[kind].label} ×3`, [items.map((i) => i.name).join(" · ")], items.map((it, i) => ({ id: `${kind}-${i}`, kind, x: 0, y: (1 - i) * 150, state: it.state || "ok", ...it })), { wide: true, fixed: true, twins: true })).join("")}</div></section>`;

const EXPLODE = Object.entries(ASSETS).filter(([, a]) => a.explodes);
const WHOLE = [
  ["Dock", "external. Acan sees only the calls that cross the wall"],
  ["Intake, Splitter", "already flat. Their routes are visible from outside"],
  ["Buffer rack", "shelves are visible without lifting anything (v0.2 candidate: keyspaces)"],
  ["Cam clock, Radar, Crate", "single-purpose parts with nothing to open"],
  ["Fittings, Floor", "too small. Their story is on the belt or in the agent panel"],
];
const exploded = `<section><h2>Exploded views <small>${EXPLODE.length} assets</small></h2>
<p class="lede">An asset explodes when Acan can read its insides from code or config. Each layer gets a leader label taken from that source. Press X on a selected machine in the Live window.</p>
<div class="ctl inline"><label>explode <input id="ex" type="range" min="0" max="100" value="100"> <span id="exv"></span></label><button class="play" id="exPlay">Play</button></div>
<div class="grid ex">${EXPLODE.map(([kind, a]) => cell(`ex-${kind}`, a.label, [a.parts], one(kind), { explode: true, fixed: true })).join("")}</div>
<table class="whole">${WHOLE.map(([n, w]) => `<tr><td>${n}</td><td>stays whole: ${w}</td></tr>`).join("")}</table></section>`;

const states = `<section><h2>Universal states <small>7, from map.md</small></h2><p class="lede">Every asset takes every state. Color appears only when it is the subject.</p><div class="grid states">${STATES.map(([s, n]) => cell(`state-${s}`, n, [], one("assembler", { state: s }), { fixed: true })).join("")}</div></section>`;

const deferred = `<section><h2>Deferred from the K7 catalog <small>${DEFERRED.length}</small></h2><table>${DEFERRED.map((d) => `<tr><td>${d.label}</td><td>${d.maps}</td><td>${d.why}</td></tr>`).join("")}</table></section>`;

const css = `${ISO_CSS}${STATE_CSS}
*{box-sizing:border-box}
html,body{margin:0;background:#0a0a0a;color:#d6d6d6;font:13px/1.45 "Inter",system-ui,sans-serif;-webkit-font-smoothing:antialiased}
header{position:sticky;top:12px;z-index:5;display:flex;align-items:center;gap:18px;margin:12px auto 0;max-width:1480px;padding:10px 18px;border-radius:14px;background:rgba(18,18,18,.72);backdrop-filter:blur(14px);border:1px solid rgba(255,255,255,.08);box-shadow:0 12px 40px rgba(0,0,0,.45)}
header h1{margin:0;font-size:14px;font-weight:600;color:#eee}
header .meta{font:11px ui-monospace,monospace;color:#7c7c7c}
.ctl{margin-left:auto;display:flex;align-items:center;gap:14px;font:11px ui-monospace,monospace;color:#9a9a9a}
.ctl.inline{margin:0 0 14px}
.ctl input[type=range]{width:140px;accent-color:#d6d6d6}
.seg{display:flex;border:1px solid rgba(255,255,255,.1);border-radius:8px;overflow:hidden;width:max-content}
.seg button,.play{font:11px ui-monospace,monospace;color:#8a8a8a;background:none;border:0;padding:5px 9px;cursor:pointer}
.play{border:1px solid rgba(255,255,255,.12);border-radius:7px}
.seg button[aria-pressed=true]{background:#262626;color:#fff}
.toggles{margin:0 0 12px}
main{padding:8px 28px 60px;max-width:1500px;margin:0 auto}
h2{font:500 11px ui-monospace,monospace;letter-spacing:.14em;text-transform:uppercase;color:#9a9a9a;margin:38px 0 12px}
h2 small{color:#555;margin-left:8px;letter-spacing:.06em}
.lede{margin:-4px 0 14px;color:#7c7c7c;max-width:78ch}
.legend{display:grid;grid-template-columns:repeat(3,1fr);gap:10px;margin:0 0 12px}
.legend div{padding:10px 12px;border:1px solid rgba(255,255,255,.06);border-radius:10px;display:flex;flex-direction:column;gap:4px}
.legend b{font-size:12px;color:#e6e6e6}.legend span{font-size:12px;color:#7c7c7c}
.grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(230px,1fr));gap:10px}
.grid.twins{grid-template-columns:1fr}
.cell{position:relative;margin:0;border-radius:12px;background:radial-gradient(120% 90% at 50% 40%,#161616,#0e0e0e);border:1px solid rgba(255,255,255,.06);overflow:hidden;cursor:grab;transition:border-color .2s}
.cell:hover{border-color:rgba(255,255,255,.16)}
.cell .pane{height:190px;background:transparent}
.cell.wide .pane{height:320px}
.cell .pane svg{width:100%;height:100%;display:block}
.over{position:absolute;left:0;top:0;right:0;height:190px;pointer-events:none}
.cell.wide .over{height:320px}
.cell figcaption{display:flex;flex-direction:column;gap:2px;padding:0 14px 12px}
.cell figcaption b{font-size:12px;font-weight:600;color:#e8e8e8;letter-spacing:.02em}
.cell figcaption span{font:10.5px ui-monospace,monospace;color:#7c7c7c}
.cell figcaption i{font-style:normal;color:#565656}
.states .cell .pane{height:170px}
.xl{position:absolute;transform:translate(0,-50%);font:10px ui-monospace,monospace;color:#d4d4d4;white-space:nowrap}
.leaders{position:absolute;left:0;top:0;overflow:visible}
.leaders path{fill:none;stroke:rgba(255,255,255,.32);stroke-width:1}
.leaders circle{fill:#e6e6e6}
.grid.ex{grid-template-columns:repeat(auto-fill,minmax(440px,1fr))}
.grid.ex .cell .pane,.grid.ex .over{height:260px}
.tile{position:absolute;transform:translate(-50%,calc(-100% - 14px));display:flex;align-items:center;gap:6px;padding:3px 7px 3px 3px;border-radius:7px;background:rgba(14,14,14,.75);border:1px solid rgba(255,255,255,.14);backdrop-filter:blur(6px);font:600 10px Inter,system-ui;letter-spacing:.08em;text-transform:uppercase;color:#ddd;white-space:nowrap}
.tile b{display:grid;place-items:center;min-width:20px;height:20px;padding:0 3px;border-radius:5px;background:#e8e8e8;color:#0a0a0a;font:700 10px ui-monospace,monospace;letter-spacing:0}
.tile small{font:10px ui-monospace,monospace;color:#8a8a8a;letter-spacing:0;text-transform:none}
.tile[data-state=fail]{border-color:rgba(232,96,74,.55)}
table{border-collapse:collapse;width:100%;max-width:980px;font-size:12px}
table.whole{margin-top:12px}
td{padding:7px 10px;border-top:1px solid rgba(255,255,255,.06);color:#8a8a8a}
td:first-child{color:#d6d6d6;width:180px}td:nth-child(2){font:11px ui-monospace,monospace}
`;

const script = `${runtimeScript()}
let az = 35, el = 30, state = "ok", ex = 1;
const twin = { form: true, tile: true, badge: true };
const cells = [...document.querySelectorAll(".cell")];
function draw() {
  for (const cel of cells) {
    const pane = cel.querySelector(".pane"), wide = cel.classList.contains("wide"), isEx = "explode" in cel.dataset, CW = wide ? 1000 : isEx ? 440 : 300, CH = wide ? 320 : isEx ? 260 : 220;
    const base = JSON.parse(cel.dataset.scene);
    const entities = base.map((e) => {
      const a = ASSETS[e.kind], out = { ...e };
      if (!("fixed" in cel.dataset)) out.state = a.family === "floor" ? "ok" : state;
      if ("explode" in cel.dataset) out.explode = ex;
      if ("twins" in cel.dataset) { if (!twin.form) { out.size = 1; out.params = {}; } if (!twin.badge) out.variant = undefined; }
      return out;
    });
    const scene = { entities };
    const top = Math.max(...entities.map((e) => ASSETS[e.kind].top + (ASSETS[e.kind].explodes || 0) * (e.explode || 0)));
    if (!cel.scale) cel.scale = Math.min(4.6, projectionFor(k, { entities: base.map((e) => ({ ...e, size: e.size || 1 })) }, ASSETS, { width: CW, height: CH, azimuth: 45, elevation: 30, pad: 34 }).scale * (isEx ? 0.7 : 0.82));
    const P = projectionFor(k, scene, ASSETS, { width: CW, height: CH, azimuth: az, elevation: el, scale: cel.scale, pivot: [isEx ? 30 : 0, isEx ? -30 : 0, top * 0.42] });
    const out = renderScene(k, ASSETS, scene, P, { width: CW, height: CH });
    pane.innerHTML = out.svg;
    const svg = pane.querySelector("svg"), rect = pane.getBoundingClientRect();
    svg.setAttribute("preserveAspectRatio", "xMidYMid meet");
    const sx = Math.min(rect.width / CW, rect.height / CH), ox = (rect.width - CW * sx) / 2, oy = (rect.height - CH * sx) / 2;
    const pos = ([x, y]) => "left:" + (ox + x * sx).toFixed(1) + "px;top:" + (oy + y * sx).toFixed(1) + "px";
    let html = out.labels.length ? calloutHtml(layoutCallouts(out.labels, { scale: sx, offset: [ox, oy] }), rect.width, rect.height) : "";
    if ("twins" in cel.dataset && twin.tile) html += entities.map((e) => '<span class="tile" data-state="' + e.state + '" style="' + pos(out.anchors[e.id]) + '"><b>' + e.mono + "</b>" + e.name + (twin.form ? " <small>" + e.reading + "</small>" : "") + "</span>").join("");
    cel.querySelector(".over").innerHTML = html;
  }
  document.querySelector("#azv").textContent = Math.round(((az % 360) + 360) % 360) + "°";
  document.querySelector("#elv").textContent = Math.round(el) + "°";
  document.querySelector("#exv").textContent = Math.round(ex * 100) + "%";
  document.querySelector("#az").value = ((az % 360) + 360) % 360; document.querySelector("#el").value = el; document.querySelector("#ex").value = ex * 100;
}
let pending = false;
const redraw = () => { if (pending) return; pending = true; requestAnimationFrame(() => { pending = false; draw(); }); };
document.querySelector("#az").oninput = (e) => { az = +e.target.value; redraw(); };
document.querySelector("#el").oninput = (e) => { el = +e.target.value; redraw(); };
document.querySelector("#ex").oninput = (e) => { ex = +e.target.value / 100; redraw(); };
let playing = 0;
document.querySelector("#exPlay").onclick = () => {
  if (playing) { cancelAnimationFrame(playing); playing = 0; return; }
  const t0 = performance.now();
  const tick = (t) => { const p = ((t - t0) / 2400) % 2; ex = 0.5 - 0.5 * Math.cos(Math.PI * p); draw(); playing = requestAnimationFrame(tick); };
  playing = requestAnimationFrame(tick);
};
for (const b of document.querySelectorAll("#state button")) b.onclick = () => { state = b.dataset.s; for (const o of document.querySelectorAll("#state button")) o.setAttribute("aria-pressed", o === b); redraw(); };
for (const b of document.querySelectorAll("#twinCtl button")) b.onclick = () => { twin[b.dataset.t] = !twin[b.dataset.t]; b.setAttribute("aria-pressed", twin[b.dataset.t]); redraw(); };
let drag = null;
addEventListener("pointerdown", (e) => { if (e.target.closest(".cell")) { drag = { x: e.clientX, y: e.clientY, az, el }; e.preventDefault(); } });
addEventListener("pointermove", (e) => { if (!drag) return; az = drag.az - (e.clientX - drag.x) * 0.5; el = Math.min(89, Math.max(8, drag.el + (e.clientY - drag.y) * 0.3)); redraw(); });
addEventListener("pointerup", () => (drag = null));
addEventListener("resize", redraw);
const params = new URLSearchParams(location.hash.slice(1));
if (params.get("az")) az = +params.get("az");
if (params.get("el")) el = +params.get("el");
if (params.get("ex")) ex = +params.get("ex");
if (params.get("twins")) for (const t of Object.keys(twin)) { twin[t] = params.get("twins").includes(t); document.querySelector('#twinCtl [data-t="' + t + '"]').setAttribute("aria-pressed", twin[t]); }
if (params.get("state")) { state = params.get("state"); for (const o of document.querySelectorAll("#state button")) o.setAttribute("aria-pressed", o.dataset.s === state); }
draw();
`;

const html = `<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Acan · v1 asset set</title><style>${css}</style></head><body>${FOG_DEFS}
<header><h1>Acan · v1 asset set</h1><span class="meta">${Object.keys(ASSETS).length} assets · anatomy kit · drag any cell to orbit all</span>
<div class="ctl"><label>azimuth <input id="az" type="range" min="0" max="359" value="35"> <span id="azv"></span></label><label>tilt <input id="el" type="range" min="8" max="89" value="30"> <span id="elv"></span></label>
<div class="seg" id="state">${STATES.map(([s, n], i) => `<button data-s="${s}" aria-pressed="${i === 0}">${n}</button>`).join("")}</div></div></header>
<main>${sections}${twins}${exploded}${states}${deferred}</main><script>${script}</script></body></html>`;

writeFileSync(join(HERE, "assets.html"), html);
console.log("wrote assets.html", (html.length / 1024).toFixed(0) + " KB");
