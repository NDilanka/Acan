import * as kit from "/home/factory-user/.factory/skills/anatomy/kit/iso-kit.mjs";
import { WORLD } from "./world-data.mjs";
import { ACQUIT } from "../acquit.mjs";
import { ASSETS } from "../assets.mjs";
import { renderScene, projectionFor, layoutCallouts, calloutHtml } from "../scene.mjs";

const AZ = 35, EL = 35;
const BOUND = { "acquit:verifier": "verifier", "acquit:OTLP boundary": "otlp", "acan:receiver": "receiver", "kvarm:train": "train", "format:reference validator": "validator", "site:static waitlist": "preview" };
const bindingKey = (plant, name) => BOUND[plant + ":" + name] || plant + ":" + name;
const entityId = (plant, name) => (plant.ids && plant.ids[name]) || name;
for (const p of WORLD.plants) for (const name of p.machines) {
  const key = bindingKey(p.id, name);
  if (WORLD.bindings[key]) continue;
  const acquit = p.id === "acquit" && ACQUIT.entities.find(e => e.id === entityId(p, name));
  WORLD.bindings[key] = acquit
    ? { entity: p.name + " / " + name, channel: "runtime.reading", value: acquit.reading, state: "stale", sourceClass: "observed", locator: "otel://sandbox/acquit/" + acquit.id, revision: p.revision, age: 41, unit: "reading", meaning: "Last reading, not current", policy: "stale after 5 s", reason: "OTLP export is 41 s old." }
    : { entity: p.name + " / " + name, channel: "runtime.status", value: null, state: "unknown", sourceClass: "observed", locator: "collector://not-attached/" + p.id + "/" + name.replaceAll(" ", "-"), revision: p.revision, age: null, unit: "status", meaning: "Runtime not reported", policy: "5 s heartbeat", reason: "Catalog shell is designed. Runtime evidence is absent." };
}
WORLD.machineBindings = Object.fromEntries(WORLD.plants.flatMap(p => p.machines.map(name => [p.id + ":" + name, bindingKey(p.id, name)])));
const DRAWN = { unknown: "fog", stale: "stale", zero: "zero", stopped: "stopped", simulated: "sim", valid: "fog" };

function plantScene(plant) {
  if (plant.id !== "acquit") {
    const scene = structuredClone(plant.scene);
    for (const e of scene.entities) e.state = DRAWN[WORLD.bindings[bindingKey(plant.id, e.id)].state];
    return scene;
  }
  const scene = structuredClone(ACQUIT);
  scene.entities = scene.entities.filter(e => !["bot", "scaffold", "crane", "inspection"].includes(e.kind));
  for (const e of scene.entities) if (e.state !== "ghost") e.state = "stale";
  const verifier = scene.entities.find(e => e.id === "verifier");
  verifier.params = { cells: WORLD.bindings.verifier.value, busy: 0, queued: 0 };
  verifier.state = "fog";
  delete scene.wall;
  return scene;
}

function territoryLabels(scene, P) {
  return (scene.territories || []).map(t => {
    const ents = t.entities.map(id => scene.entities.find(e => e.id === id));
    const x0 = Math.min(...ents.map(e => e.x - ASSETS[e.kind].w * (e.size || 1) / 2)) - 12;
    const y1 = Math.max(...ents.map(e => e.y + ASSETS[e.kind].d * (e.size || 1) / 2)) + 12;
    return { id: t.id, state: t.state, label: t.label, pt: kit.iso([x0 + 3, y1 + 7, 0], P) };
  });
}

function draw(scene, box, scale, pad) {
  const P = projectionFor(kit, scene, ASSETS, { width: box[0], height: box[1], azimuth: AZ, elevation: EL, scale, pad });
  const out = renderScene(kit, ASSETS, scene, P, { width: box[0], height: box[1] });
  const svg = out.svg.replace(/data-id="g1" data-state="unclaimed"/, 'data-id="g1" data-state="unmatched"');
  return { svg, anchors: out.anchors, labels: out.labels, P };
}

const scenes = Object.fromEntries(WORLD.plants.map(p => [p.id, plantScene(p)]));
const fit = (scene, box, pad) => projectionFor(kit, scene, ASSETS, { width: box[0], height: box[1], azimuth: AZ, elevation: EL, pad }).scale;
const THUMB = [460, 265], STAGE = [1100, 620];
const thumbScale = fit(scenes.acquit, THUMB, 14), stageScale = fit(scenes.acquit, STAGE, 60);
const worldArt = { thumb: {}, stage: {}, cut: {} };
for (const p of WORLD.plants) {
  const scene = scenes[p.id];
  const thumb = draw(scene, THUMB, thumbScale);
  worldArt.thumb[p.id] = { svg: thumb.svg, anchors: thumb.anchors };
  const stage = draw(scene, STAGE, Math.min(stageScale * 1.35, fit(scene, STAGE, 80)));
  worldArt.stage[p.id] = { svg: stage.svg, anchors: stage.anchors, territories: territoryLabels(scene, stage.P) };
  for (const name of p.machines) {
    const id = entityId(p, name), e = scene.entities.find(x => x.id === id);
    if (!ASSETS[e.kind].explodes) continue;
    const solo = { entities: [{ ...e, explode: 1 }] };
    const cut = draw(solo, STAGE, Math.min(stageScale * 2.2, fit(solo, STAGE, 150)));
    const labels = cut.labels.map(l => ({ ...l, text: l.text.replace(/ · (busy|idle)$/, " · occupancy not reported").replace(/^\d+ waiting$/, "waiting tray · not reported") }));
    worldArt.cut[p.id + "/" + name] = { svg: cut.svg + calloutHtml(layoutCallouts(labels, { gap: 22, pad: 36 }), STAGE[0], STAGE[1]), anchor: cut.anchors[id] };
  }
}

function worldMount(D, art) {
  if (!conceptActive("world")) return;
  const root = document.querySelector("#worldApp");
  document.body.dataset.world = "active";
  document.title = "Acan · My world";
  const q = s => root.querySelector(s), all = s => [...root.querySelectorAll(s)];
  const esc = v => String(v).replace(/[&<>"']/g, c => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));
  const params = new URLSearchParams(location.hash.slice(1));
  const plants = Object.fromEntries(D.plants.map(p => [p.id, p]));
  const number = id => String(D.plants.findIndex(p => p.id === id) + 1).padStart(2, "0");
  const entity = (p, name) => (p.ids && p.ids[name]) || name;
  const bindingOf = (plant, name) => D.machineBindings[plant + ":" + name] || plant + ":" + name;
  const COL = { left: 1186, top: 128, bottom: 852 }, STAGE = { w: 1100, h: 620, top: 128 };
  const RANK = { "blocks publication": 3, question: 2, "recommended hold": 1 };
  const KIND = { "blocks publication": { glyph: "✕", one: "blocks publication", many: "block publication" }, question: { glyph: "?", one: "question", many: "questions" }, "recommended hold": { glyph: "◇", one: "recommended hold", many: "recommended holds" } };
  const loaded = performance.now(), kvStart = params.get("kv") || "ok";
  const state = window.worldState = {
    focus: null, level: "world", selection: null, mode: "live", floor: "All plants",
    camera: { x: 0, y: 0, s: 1 }, history: { i: 0, n: 1 }, bookmarks: [], queueOpen: false, queue: [...D.queue], fresh: [],
    kvarm: { state: kvStart, pending: [], sent: false, checked: false, contact: loaded - ({ ok: 8, stale: 31, offline: 40 }[kvStart] ?? 8) * 1000 },
    egressBytes: 0, network: [], sites: Object.fromEntries(D.plants.map(p => [p.id, [p.x, p.y]])), bindings: structuredClone(D.bindings), belts: true,
    appearance: params.get("theme") || "system", theme: "dark", overlay: null, question: null, contract: null, arrival: false
  };
  let drag = null, alt = false, searchIndex = 0, searchResults = [], opener = null, underSearch = null, announceTimer = 0, stageCache = "", cardCache = "";
  const wheel = { sum: 0, last: -1e9, locked: false };
  const current = () => ({ focus: state.focus, level: state.level, selection: state.selection, mode: state.mode, floor: state.floor, camera: { ...state.camera }, question: state.question, contract: state.contract });
  const entries = [current()];
  function operation(op) { state.network.push({ ...op, destination: D.origin }); state.egressBytes += op.bytes; }
  if (kvStart === "ok" || kvStart === "stale") D.operations.forEach(operation);
  if (kvStart === "offline") state.kvarm.pending = structuredClone(D.pending);
  const kvAge = () => Math.round((performance.now() - state.kvarm.contact) / 1000);
  function refreshRemote() {
    const b = state.bindings.train = { ...D.bindings.train };
    if (state.kvarm.state === "ok") { state.kvarm.contact = performance.now() - 8000; return; }
    b.state = "stale"; b.age = kvAge();
    b.reason = state.kvarm.state === "offline" ? "Disconnected. Last authenticated value, not current." : "Two missed polls. Last authenticated value, not current.";
  }
  history.replaceState({ world: true, index: 0, view: current() }, "");
  function announce(text) {
    const el = q("#worldAnnouncement");
    el.textContent = text; clearTimeout(announceTimer);
    announceTimer = setTimeout(() => { el.textContent = ""; }, 2600);
  }
  function appearance(value) {
    state.appearance = value;
    state.theme = value === "system" ? (matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark") : value;
    root.dataset.theme = state.theme;
    document.body.dataset.worldTheme = state.theme;
    all(".iso").forEach(e => e.dataset.theme = state.theme);
    q("#worldAppearance").value = value;
  }
  const bindText = b => b.text || [b.entity, b.channel, String(b.value ?? "not reported"), b.state, b.sourceClass, b.locator, "revision " + b.revision, b.age === null ? "age unknown" : b.age + " s", b.reason || b.meaning].join(" · ");
  const markValue = b => b.state === "unknown" ? "?" : b.state === "stopped" ? "⏻" : b.state === "simulated" ? "SIM" : String(b.value);
  const stateWord = b => b.state === "stale" ? "stale · " + b.age + " s" : b.state === "stopped" ? "stopped · exit 0" : b.state;
  function stateMark(id, full = false) {
    const b = state.bindings[id];
    return `<span class="world-state-mark" data-binding="${id}" data-evidence-state="${b.state}" tabindex="0" aria-label="${esc(bindText(b))}"><span class="world-bezel">${esc(markValue(b))}</span>${full ? `<span class="world-mark-text">${esc(stateWord(b))}<small>${esc(b.entity)}</small></span>` : ""}</span>`;
  }
  const miniValue = b => ({ unknown: "?", stopped: "⏻ exit 0", simulated: "SIM", zero: "0", stale: b.value + " · " + b.age + " s" }[b.state] ?? b.value + " " + b.unit);
  function freshness(p) {
    const b = state.bindings[p.binding];
    if (p.id === "kvarm") return state.kvarm.state === "ok" ? "train depth · fresh 8 s" : "train depth · " + (state.kvarm.state === "offline" ? "offline" : "stale") + " · " + b.age + " s";
    return { unknown: "runtime not reported", stopped: "validator stopped · exit 0", simulated: "simulated · not live", stale: "OTLP stale · " + b.age + " s" }[b.state] ?? b.meaning;
  }
  const rowsOf = id => state.queue.filter(r => r.plant === id);
  const worst = rows => rows.slice().sort((a, b) => RANK[b.kind] - RANK[a.kind])[0];
  function chip(id, compact = false) {
    const rows = rowsOf(id), w = worst(rows);
    if (!w) return compact ? "" : `<span class="world-chip" data-kind="none">nothing waiting</span>`;
    const n = rows.filter(r => r.kind === w.kind).length, k = KIND[w.kind];
    if (compact) return `<span class="world-chip" data-kind="${w.kind}"><b>${k.glyph}</b>${rows.length}</span>`;
    return `<span class="world-chip" data-kind="${w.kind}"><b>${k.glyph}</b>${n} ${n > 1 ? k.many : k.one}${rows.length > n ? `<span>+${rows.length - n}</span>` : ""}</span>`;
  }
  function scopeLine() {
    const blocks = new Set(state.queue.filter(r => r.kind === "blocks publication").map(r => r.plant)).size;
    const stale = D.plants.filter(p => state.bindings[p.binding].state === "stale").length;
    return `${D.plants.length} plants · ${blocks} block publication · ${state.queue.length} waiting · ${stale} stale`;
  }
  function patch(el, list) {
    const old = new Map([...el.children].map(c => [c.dataset.key, c]));
    let prev = null;
    for (const { key, html } of list) {
      let n = old.get(key);
      if (!n || n.worldHtml !== html) {
        const t = document.createElement("template"); t.innerHTML = html.trim();
        const m = t.content.firstElementChild; m.dataset.key = key; m.worldHtml = html;
        if (n) n.replaceWith(m); n = m;
      }
      old.delete(key);
      const want = prev ? prev.nextElementSibling : el.firstElementChild;
      if (want !== n) el.insertBefore(n, want);
      prev = n;
    }
    old.forEach(n => n.remove());
  }
  function focusKey(el) {
    if (!el || !root.contains(el) || el === root) return null;
    if (el.id) return "#" + CSS.escape(el.id);
    const attrs = [...el.attributes].filter(a => a.name.startsWith("data-") && !["data-key", "data-theme"].includes(a.name)).map(a => `[${a.name}="${CSS.escape(a.value)}"]`).join("");
    return attrs ? el.tagName.toLowerCase() + attrs : null;
  }
  const column = () => state.overlay === "network" ? "network" : state.queueOpen ? "queue" : state.selection && state.level !== "world" ? "inspector" : null;
  function worldTransform() {
    const c = state.camera;
    if (!column()) return `translate(${c.x}px,${c.y}px) scale(${c.s})`;
    const xs = D.plants.map(p => p.x), ys = D.plants.map(p => p.y);
    const x0 = Math.min(...xs) - 160, x1 = Math.max(...xs) + 170, y0 = Math.min(...ys) - 118, y1 = Math.max(...ys) + 118;
    const s = Math.min(1, (COL.left - 40) / (x1 - x0), (800 - 110) / (y1 - y0));
    return `translate(${Math.round(20 - x0 * s)}px,${Math.round(Math.max(0, 110 - y0 * s))}px) scale(${s})`;
  }
  function siteHtml(p) {
    const rows = rowsOf(p.id), w = worst(rows), fresh = rows.some(r => state.fresh.includes(r.id));
    return `<section class="world-site${state.focus === p.id ? " world-focused" : ""}" data-world-site="${p.id}" data-trouble="${w ? w.kind : "none"}" style="left:${p.x}px;top:${p.y}px">
      <button class="world-plant-drawing iso" data-plant="${p.id}" tabindex="-1" aria-hidden="true">${art.thumb[p.id].svg}</button>
      <div class="world-plant-label"><button class="world-plant-name" data-plant="${p.id}" aria-label="Open ${esc(p.name)}, ${rows.length} waiting"><span class="world-site-id">${number(p.id)}</span><strong>${esc(p.name)}</strong><kbd class="world-alt-hint">Alt ${+number(p.id)}</kbd></button>${chip(p.id)}${fresh ? `<span class="world-new">new</span>` : ""}</div>
      <div class="world-site-detail">${esc(p.env)} ·${stateMark(p.binding)}<span>${esc(freshness(p))}</span></div>
      <div class="world-condition" data-kind="${w ? w.kind : "none"}">${w ? `<b>${KIND[w.kind].glyph}</b>${esc(w.title)}` : esc(p.condition)}</div>
    </section>`;
  }
  function canvas() {
    patch(q("#worldSites"), D.plants.map(p => ({ key: "site-" + p.id, html: siteHtml(p) })));
    q("#worldRoutes").style.display = state.belts ? "" : "none";
    q("#worldPorts").style.display = state.belts ? "" : "none";
    q("#worldViewport").hidden = state.level !== "world";
  }
  function edges(p) {
    const W = STAGE.w / 2, H = STAGE.h / 2;
    const list = D.plants.filter(o => o.id !== p.id).map(o => {
      const dx = o.x - p.x, dy = o.y - p.y;
      const side = Math.abs(dx) / W > Math.abs(dy) / H ? (dx > 0 ? "right" : "left") : (dy > 0 ? "bottom" : "top");
      const t = Math.min(W / Math.abs(dx || 1e-6), H / Math.abs(dy || 1e-6));
      const contracts = D.contracts.filter(c => (c.from === p.id && c.to === o.id) || (c.to === p.id && c.from === o.id));
      return { o, side, x: Math.round(W + dx * t), y: Math.round(H + dy * t), contracts };
    });
    for (const e of list) {
      if (e.side === "left" || e.side === "right") e.y = Math.max(84, Math.min(STAGE.h - 40 - 22 * e.contracts.length, e.y));
      else e.x = Math.max(e.side === "top" ? 720 : 360, Math.min(STAGE.w - 220, e.x));
    }
    for (const side of ["left", "right", "top", "bottom"]) {
      const same = list.filter(e => e.side === side).sort((a, b) => side === "left" || side === "right" ? a.y - b.y : a.x - b.x);
      for (let i = 1; i < same.length; i++) {
        if (side === "left" || side === "right") same[i].y = Math.max(same[i].y, same[i - 1].y + 34 + 24 * same[i - 1].contracts.length);
        else same[i].x = Math.max(same[i].x, same[i - 1].x + 250);
      }
    }
    return list;
  }
  function stageHtml(p) {
    const A = art.stage[p.id];
    const cut = state.level === "cutaway" ? art.cut[p.id + "/" + state.selection] : null;
    const title = `<div class="world-stage-title"><span class="world-site-id">${number(p.id)}</span><strong>${esc(p.name)}</strong>${chip(p.id)}<span class="world-stage-meta">${esc(p.env)} · ${esc(p.revision)} ·${stateMark(p.binding)}<span>${esc(freshness(p))}</span></span></div>`;
    if (cut) {
      const key = bindingOf(p.id, state.selection), b = state.bindings[key];
      const reading = b.runtime ? `<b>${esc(b.value)} declared ${esc(b.unit)}</b><small>${esc(b.channel)} · ${esc(b.locator)}</small>` : `<b>${esc(miniValue(b))} · ${esc(b.state)}</b><small>${esc(b.channel)} · ${esc(b.locator)}</small>`;
      const runtime = b.runtime ? `<div class="world-gauge-row"><span class="world-state-mark" data-evidence-state="unknown"><span class="world-bezel">?</span></span><span><b>occupancy not reported</b><small>${esc(b.runtime)}</small></span></div>` : "";
      return `<div class="world-stage-art iso" data-cutaway="${esc(state.selection)}">${cut.svg}</div>${title}<div class="world-cut-gauge"><span class="world-eyebrow">Cutaway · ${esc(state.selection)} · bound evidence</span><div class="world-gauge-row">${stateMark(key)}<span>${reading}</span></div>${runtime}</div>`;
    }
    const terr = A.territories.map(t => `<span class="world-terr-label" data-terr-state="${t.state}" style="left:${Math.round(t.pt[0])}px;top:${Math.round(t.pt[1])}px">${esc(t.label)}</span>`).join("");
    const machines = p.machines.map(n => {
      const pt = A.anchors[entity(p, n)];
      if (!pt) return "";
      const key = bindingOf(p.id, n), b = state.bindings[key];
      return `<button class="world-machine" data-machine="${esc(n)}" data-machine-plant="${p.id}" data-binding="${key}"${state.selection === n ? ' aria-current="true"' : ""} style="left:${Math.round(pt[0])}px;top:${Math.round(pt[1])}px"><span>${esc(n)}</span><span class="world-mini" data-evidence-state="${b.state}">${esc(miniValue(b))}</span>${b.runtime ? `<span class="world-mini" data-evidence-state="unknown">occupancy ?</span>` : ""}</button>`;
    }).join("");
    const list = edges(p);
    const lines = list.flatMap(e => e.contracts.map(c => {
      const local = c.from === p.id ? c.fromEntity : c.toEntity, pt = A.anchors[entity(p, local)];
      return pt ? `<path data-kind="${c.kind}" data-line="${c.id}" data-x="${Math.round(pt[0])}" data-y="${Math.round(pt[1])}" d=""/>` : "";
    })).join("");
    const edgeHtml = list.map(e => {
      const pos = e.side === "left" ? `left:10px;top:${e.y}px` : e.side === "right" ? `right:10px;top:${e.y}px` : e.side === "top" ? `top:10px;left:${e.x}px` : `bottom:10px;left:${e.x}px`;
      const contracts = e.contracts.map(c => {
        const local = c.from === p.id ? c.fromEntity : c.toEntity, far = c.from === p.id ? c.toEntity : c.fromEntity;
        return `<button class="world-edge-contract" data-contract="${c.id}" data-kind="${c.kind}">${esc(c.label)} · ${esc(local)} ${c.from === p.id ? "→" : "←"} ${esc(far)}</button>`;
      }).join("");
      return `<div class="world-edge" data-side="${e.side}" style="${pos}"><button class="world-edge-plant" data-plant="${e.o.id}"><span class="world-site-id">${number(e.o.id)}</span>${esc(e.o.name)} ↗${chip(e.o.id, true)}<kbd class="world-alt-hint">Alt ${+number(e.o.id)}</kbd></button>${contracts}</div>`;
    }).join("");
    return `<div class="world-stage-art iso">${A.svg}</div><svg class="world-connectors" viewBox="0 0 ${STAGE.w} ${STAGE.h}" aria-hidden="true">${lines}</svg>${terr}${machines}${title}${edgeHtml}`;
  }
  function stage() {
    const el = q("#worldStage");
    el.hidden = state.level === "world";
    if (el.hidden) return;
    el.style.left = (column() ? Math.round(20 + Math.max(0, (COL.left - 40 - STAGE.w) / 2)) : Math.round((innerWidth - STAGE.w) / 2)) + "px";
    const html = stageHtml(plants[state.focus]);
    if (html !== stageCache) { el.innerHTML = html; stageCache = html; declutter(el); }
  }
  const overlaps = (a, b) => a.left < b.right + 4 && b.left < a.right + 4 && a.top < b.bottom + 3 && b.top < a.bottom + 3;
  // Edge labels are measured once, then moved to the nearest free offset by rect arithmetic, so no layout runs per candidate.
  const OFFSETS = (() => {
    const out = [];
    for (let dx = -480; dx <= 480; dx += 20) for (let dy = -260; dy <= 260; dy += 13) out.push([dx, dy]);
    return out;
  })();
  const shifted = (r, dx, dy) => ({ left: r.left + dx, right: r.right + dx, top: r.top + dy, bottom: r.bottom + dy });
  function placeEdges(el, placed) {
    const box = el.getBoundingClientRect(), pad = 8;
    const inside = r => r.left >= box.left + pad && r.right <= box.right - pad && r.top >= box.top + pad && r.bottom <= box.bottom - pad;
    for (const e of el.querySelectorAll(".world-edge")) {
      e.style.translate = "";
      const r0 = e.getBoundingClientRect(), along = e.dataset.side === "left" || e.dataset.side === "right";
      const cost = ([dx, dy]) => along ? Math.abs(dx) * 4 + Math.abs(dy) : Math.abs(dx) + Math.abs(dy) * 4;
      const best = OFFSETS.filter(([dx, dy]) => { const r = shifted(r0, dx, dy); return inside(r) && !placed.some(p => overlaps(p, r)); }).sort((a, b) => cost(a) - cost(b))[0] || [0, 0];
      e.style.translate = `${best[0]}px ${best[1]}px`;
      placed.push(shifted(r0, best[0], best[1]));
    }
    const k = box.width / STAGE.w;
    for (const path of el.querySelectorAll(".world-connectors path")) {
      const b = el.querySelector(`.world-edge-contract[data-contract="${path.dataset.line}"]`);
      if (!b) continue;
      const r = b.getBoundingClientRect(), x0 = +path.dataset.x, y0 = +path.dataset.y;
      const cx = Math.max(r.left, Math.min(r.right, box.left + x0 * k)), cy = Math.max(r.top, Math.min(r.bottom, box.top + y0 * k));
      path.setAttribute("d", `M${x0} ${y0}L${Math.round((cx - box.left) / k)} ${Math.round((cy - box.top) / k)}`);
    }
  }
  function declutter(el) {
    const placed = [...el.querySelectorAll(".world-stage-title,.world-cut-gauge")].map(e => e.getBoundingClientRect());
    placeEdges(el, placed);
    const labels = [...el.querySelectorAll(".world-machine")].sort((a, b) => parseFloat(b.style.top) - parseFloat(a.style.top));
    for (const m of labels) {
      for (let lift = 0, i = 0; i < 12; i++, lift += 12) {
        m.style.setProperty("--lift", lift + "px");
        const r = m.getBoundingClientRect();
        if (!placed.some(p => overlaps(p, r))) break;
      }
      placed.push(m.getBoundingClientRect());
    }
    for (const t of el.querySelectorAll(".world-terr-label")) {
      const top = parseFloat(t.style.top);
      for (let i = 0; i < 8; i++) {
        const r = t.getBoundingClientRect();
        if (!placed.some(p => overlaps(p, r))) break;
        t.style.top = (parseFloat(t.style.top) + 16) + "px";
        if (i === 7) t.style.top = top + "px";
      }
      placed.push(t.getBoundingClientRect());
    }
  }
  function queueRow(r, i) {
    const fresh = state.fresh.includes(r.id);
    return `<button class="world-queue-row" data-queue="${r.id}" data-queue-plant="${r.plant}">
      <span class="world-row-meta"><b>${esc(plants[r.plant].name)}</b><span class="world-kind" data-kind="${r.kind}">${KIND[r.kind].glyph} ${esc(r.kind)}</span>${fresh ? `<span class="world-new">new</span>` : ""}<span class="world-estimate">${esc(r.estimate)}</span>${i < 4 ? `<kbd>⇧ ${i + 1}</kbd>` : ""}</span>
      <strong>${esc(r.title)}</strong><span>${esc(r.why)}</span><small>Waiting costs: ${esc(r.cost.toLowerCase())}. Safe until: ${esc(r.safe.toLowerCase())}.</small>
    </button>`;
  }
  function queue() {
    const el = q("#worldQueue");
    el.hidden = column() !== "queue";
    q("#worldQueueCount").textContent = state.queue.length;
    q("#worldQueueTitleCount").textContent = state.queue.length;
    q("#worldQueueFresh").hidden = !state.fresh.length;
    q("#worldQueueFresh").textContent = state.fresh.length + " new";
    q("[data-action=queue]").setAttribute("aria-pressed", state.queueOpen);
    patch(q("#worldQueueRows"), state.queue.map((r, i) => ({ key: "row-" + r.id, html: queueRow(r, i) })));
    const rows = q("#worldQueueRows"), below = el.hidden ? 0 : state.fresh.filter(id => { const r = rows.querySelector(`[data-queue="${id}"]`); return r && r.getBoundingClientRect().bottom > rows.getBoundingClientRect().bottom + 2; }).length;
    q("#worldQueueBelow").hidden = !below;
    q("#worldQueueBelow").textContent = below + " new below ↓";
  }
  function inspector() {
    const el = q("#worldInspector");
    q("#worldStack").hidden = el.hidden = column() !== "inspector";
    if (el.hidden) return;
    const p = plants[state.focus], key = bindingOf(p.id, state.selection), b = state.bindings[key], hasCut = !!art.cut[p.id + "/" + state.selection];
    el.innerHTML = `<header><span class="world-eyebrow">${esc(p.name)} / ${esc(state.selection)}</span><button data-close="inspector" aria-label="Close inspector">×</button></header>
      <h2>${esc(state.selection)}</h2><p class="world-identity">${esc(p.env)} · ${esc(p.revision)} · read only</p>
      <div class="world-inspect-tabs"><span>${state.level === "cutaway" ? "Cutaway" : "Machine"}</span>${hasCut ? `<button data-action="cutaway">${state.level === "cutaway" ? "Close cutaway" : "Open cutaway"} <kbd>X</kbd></button>` : `<small>no declared internals</small>`}</div>
      <div class="world-binding-reading">${state.selection === "verifier" ? `<strong>${esc(b.value)}</strong> declared cell<small>occupancy not reported</small>` : stateMark(key, true)}</div>
      <h3>Evidence binding</h3><dl><dt>Channel</dt><dd>${esc(b.channel)}</dd><dt>Value</dt><dd>${esc(b.value ?? "not reported")} · ${esc(b.state)}</dd><dt>Source</dt><dd>${esc(b.sourceClass)}</dd><dt>Locator</dt><dd>${esc(b.locator)}</dd><dt>Revision</dt><dd>${esc(b.revision)}</dd><dt>Age</dt><dd>${b.age ?? "unknown"}${b.age === null ? "" : " s"} · ${esc(b.policy)}</dd></dl>
      ${b.runtime ? `<p class="world-warning">${esc(b.runtime)}</p>` : ""}
      <button class="world-action" data-provenance="${key}">Inspect provenance <kbd>Alt + hover</kbd></button>
      <h3>Identity</h3><p>${esc(p.worktree)}<br>${esc(p.branch)} · ${esc(p.env)}</p>
      ${state.focus === "acquit" ? `<h3>Agent evidence</h3>${D.bots.map(b => `<div class="world-bot"><span class="world-bot-chip"><b>${esc(b.principal)}</b>${esc(b.session)}<span data-evidence="${b.evidence}">${esc(b.evidence)}</span></span><small>${esc(b.note)}</small></div>`).join("")}` : ""}
      <footer>Catalog geometry is designed. Fasteners and seams are non-semantic. Runtime is never inferred from a shell.</footer>`;
  }
  function stageAnchor(name) {
    const p = plants[state.focus], pt = art.stage[p.id].anchors[entity(p, name)];
    const left = parseFloat(q("#worldStage").style.left) || 0;
    return pt ? [left + pt[0], STAGE.top + pt[1]] : null;
  }
  function card() {
    const el = q("#worldCard");
    el.hidden = !(state.question || state.contract) || state.level === "world";
    if (el.hidden) { cardCache = ""; return; }
    const p = plants[state.focus];
    let html;
    if (state.contract) {
      const c = D.contracts.find(c => c.id === state.contract), far = state.focus === c.from ? c.to : c.from;
      html = `<header><span class="world-eyebrow">Declared contract / ${esc(c.kind)}</span><button data-close="card" aria-label="Close contract">×</button></header><h2>${esc(c.label)}</h2><p>${esc(c.detail)}</p><p class="world-warning">${esc(c.evidence)}</p><p class="world-contract-ends">${esc(plants[c.from].name)} / ${esc(c.fromEntity)}<br>→ ${esc(plants[c.to].name)} / ${esc(c.toEntity)}</p><button class="world-primary" data-far-end="${c.id}">Jump to other end · ${esc(plants[far].name)} ↗</button><small>Back restores this endpoint.</small>`;
    } else {
      const r = state.queue.find(r => r.id === state.question);
      html = `<header><span class="world-eyebrow">${esc(plants[r.plant].name)} / ${esc(r.entity)}</span><button data-close="card" aria-label="Close question">×</button></header>${r.departure ? `<aside id="worldDeparture"><span class="world-eyebrow">Where you left it</span><p>${esc(r.departure)}</p></aside>` : ""}<span class="world-kind" data-kind="${r.kind}">${KIND[r.kind].glyph} ${esc(r.kind)}</span><h2>${esc(r.title)}</h2><p>${esc(r.why)}. ${esc(r.cost)}. ${esc(r.safe)}.</p>${r.id === "p-2307" ? `<div class="world-readback"><b>readback · Acan</b><p>Claude Code keeps a failed payout pending.<br>Droid removes it from the effects queue.</p><span class="world-warning">w3 is not locked. No agent has been assigned.</span></div>` : ""}<small>${esc(r.estimate)} estimated review · inspection only</small>`;
    }
    if (html !== cardCache) { el.innerHTML = html; cardCache = html; }
  }
  function provenance(id, anchor) {
    state.overlay = "provenance"; state.provenance = id;
    const b = state.bindings[id], el = q("#worldProvenance"), box = anchor?.getBoundingClientRect();
    el.innerHTML = `<header><span class="world-eyebrow">Binding / ${esc(b.sourceClass)}</span><button data-close="overlay" aria-label="Close provenance">×</button></header><h2>${esc(b.entity)}</h2><p id="worldProvenanceText">${esc(bindText(b))}</p><dl><dt>Channel</dt><dd>${esc(b.channel)}</dd><dt>State</dt><dd>${esc(b.state)}</dd><dt>Freshness</dt><dd>${b.age === null ? "unknown" : b.age + " s"} · ${esc(b.policy)}</dd></dl>`;
    el.style.left = Math.max(24, Math.min(innerWidth - 510, box ? box.left - 200 : 700)) + "px";
    el.style.top = Math.max(120, Math.min(innerHeight - 360, box ? box.bottom + 12 : 380)) + "px";
    layers();
  }
  function pill() {
    const k = state.kvarm, n = k.pending.length, held = n ? n + " write" + (n > 1 ? "s" : "") + " held" : "no writes held";
    const dot = s => `<span class="world-connection-dot" data-kv="${s}"></span>`;
    if (k.state === "ok") return dot("ok") + "Kvarm · " + D.origin.replace("https://", "") + " · 3 repos · fresh 8 s";
    if (k.checked) return dot("stale") + "Kvarm checked · " + held + " · Enter sends";
    if (k.state === "offline") return dot("offline") + "Kvarm offline · " + held + " · " + kvAge() + " s";
    return dot("stale") + "Kvarm stale · 2 missed polls · " + kvAge() + " s";
  }
  function connection() {
    const k = state.kvarm;
    if (k.state === "offline") {
      k.contact = performance.now();
      operation({ op: "check_heads_and_leases", bytes: 384, reason: "Reconnect without replaying pending writes" });
      if (k.pending.length) { k.state = "stale"; k.checked = true; announce("Checked heads and leases. Press Enter to send the held writes."); }
      else { k.state = "ok"; k.checked = false; announce("Reconnected. Heads and leases checked. No writes were held."); }
    } else {
      k.state = "offline"; k.checked = false;
      if (!k.pending.length && !k.sent) k.pending = structuredClone(D.pending);
      announce("Kvarm is offline. Remote fields are stale. Local inspection still works.");
    }
    state.overlay = "network"; paint();
    if (k.checked) q("#worldNetwork [data-action=confirm]").focus();
  }
  function confirmWrites() {
    const k = state.kvarm;
    if (!k.checked) return;
    const sent = k.pending.length;
    k.pending.forEach(operation); k.pending = []; k.sent = true;
    k.state = "ok"; k.checked = false; paint();
    announce(sent + " held write" + (sent === 1 ? "" : "s") + " sent to the fixture origin.");
  }
  function networkPanel() {
    const el = q("#worldNetwork");
    el.hidden = column() !== "network";
    if (el.hidden) return;
    const k = state.kvarm;
    q("#worldNetworkBody").innerHTML = `<span class="world-tag">${esc(k.state)} · scripted fixture</span><h2>Kvarm connection</h2><p class="world-origin">${esc(D.origin)}</p><p>Your own origin · 3 repos · opt-in declared egress.<br>No browser network requests are made by this prototype.</p>
      ${k.checked ? `<p class="world-warning">checked heads and leases</p><button class="world-primary" data-action="confirm">Send ${k.pending.length} held write${k.pending.length === 1 ? "" : "s"} <kbd>Enter</kbd></button><button class="world-action" data-action="connection">Go offline</button>` : `<button class="world-primary" data-action="connection">${k.state === "offline" ? "Reconnect" : "Go offline"}</button>`}
      ${k.state !== "ok" ? `<div class="world-remote-warning"><b>Remote fields are stale</b><p>Train depth keeps its last authenticated value with age (${kvAge()} s). Local bindings keep their own freshness.</p></div>` : ""}
      <h3>Held writes · ${k.pending.length}</h3>${k.pending.map(p => `<div class="world-network-row" data-pending-write><b>${esc(p.op)}</b><span>${esc(p.path)}</span><small>${esc(p.reason)} · ${p.bytes} B held locally</small></div>`).join("") || "<p>None. Only an action creates a held write.</p>"}
      <h3>Network log · ${state.egressBytes} B</h3>${state.network.map(p => `<div class="world-network-row"><b>${esc(p.op)}<span>${p.bytes} B</span></b><span>${esc(p.destination)}</span><small>${esc(p.reason)}</small></div>`).join("") || "<p>No outbound operations.</p>"}`;
  }
  function bookmark() {
    const here = JSON.stringify(current()), same = state.bookmarks.findIndex(b => JSON.stringify(b.view) === here);
    if (same >= 0) { state.overlay = "bookmarks"; paint(); announce("This place is already marked as Shift " + (same + 1) + "."); return; }
    if (state.bookmarks.length >= 4) { state.overlay = "bookmarks"; paint(); announce("Four places are already marked. Nothing new was saved."); return; }
    state.bookmarks.push({ label: state.focus ? plants[state.focus].name + " / " + (state.selection || state.level) : "My world", view: current() });
    state.overlay = "bookmarks"; paint(); announce("Place marked. Camera, selection, mode and Floor scope saved.");
  }
  function recall(i) { const b = state.bookmarks[i]; if (b) navigate(structuredClone(b.view), "Recalled " + b.label); }
  function layers() {
    q("#worldPalette").hidden = state.overlay !== "search";
    q("#worldStates").hidden = state.overlay !== "states";
    q("#worldProvenance").hidden = state.overlay !== "provenance";
    q("#worldBookmarks").hidden = state.overlay !== "bookmarks";
    if (state.overlay === "states") q("#worldStatesList").innerHTML = D.plants.map(p => `<button data-provenance="${p.binding}">${stateMark(p.binding, true)}<p>${esc(state.bindings[p.binding].meaning)}</p><small>${esc(state.bindings[p.binding].locator).replaceAll("/", "/<wbr>")}</small></button>`).join("");
    q("#worldBookmarksBody").innerHTML = state.bookmarks.length ? state.bookmarks.map((b, i) => `<button class="world-action" data-recall="${i}">${esc(b.label)}<kbd>⇧ ${i + 1}</kbd></button>`).join("") : "<p>Mark a place with M. Nothing moves until you recall it.</p>";
    q("#worldBookmarksFull").hidden = state.bookmarks.length < 4;
  }
  function minimap() {
    q("#worldMinimap").hidden = !!column();
    all("[data-map]").forEach(b => { b.dataset.kind = worst(rowsOf(b.dataset.map))?.kind || "none"; b.toggleAttribute("aria-current", state.focus === b.dataset.map); });
    const v = q("#worldMapView");
    if (state.level === "world") {
      const c = state.camera;
      const w = Math.min(160, 160 / c.s), h = Math.min(90, 90 / c.s);
      v.style.cssText = `left:${Math.max(0, Math.min(160 - w, -c.x / c.s / 10))}px;top:${Math.max(0, Math.min(90 - h, -c.y / c.s / 10))}px;width:${w}px;height:${h}px`;
    } else {
      const p = plants[state.focus];
      v.style.cssText = `left:${p.x / 10 - 16}px;top:${p.y / 10 - 12}px;width:32px;height:24px`;
    }
  }
  function paint() {
    const key = focusKey(document.activeElement);
    refreshRemote();
    root.dataset.level = state.level; root.dataset.mode = state.mode; root.dataset.column = column() || "none";
    q("#worldViewport").style.transform = worldTransform();
    q("#worldBreadcrumb").textContent = state.focus ? plants[state.focus].name : "My world";
    q("#worldScope").textContent = state.focus ? plants[state.focus].env + " · " + plants[state.focus].revision + " · " + state.mode : scopeLine();
    q("#worldLevel").textContent = state.level;
    q("#worldBack").disabled = state.history.i === 0;
    q("#worldForward").disabled = state.history.i >= state.history.n - 1;
    q("#worldFitPlant").disabled = !state.focus;
    q("#worldKvarm").innerHTML = pill();
    q("#worldKvarm").dataset.kv = state.kvarm.state;
    q("#worldEgress").textContent = "↗ " + state.egressBytes.toLocaleString() + " B · declared egress";
    q("#worldBeltToggle").setAttribute("aria-pressed", state.belts);
    all("[data-world-mode]").forEach(b => b.setAttribute("aria-pressed", b.dataset.worldMode === state.mode));
    canvas(); stage(); queue(); inspector(); networkPanel(); card(); layers(); minimap();
    q("#worldBookmarksCount").textContent = state.bookmarks.length || "";
    all(".iso").forEach(e => e.dataset.theme = state.theme);
    save();
    if (key && !root.contains(document.activeElement)) root.querySelector(key)?.focus({ preventScroll: true });
  }
  function save() {
    entries[state.history.i] = current();
    history.replaceState({ world: true, index: state.history.i, view: current() }, "");
  }
  function navigate(view, message) {
    save();
    Object.assign(state, view);
    state.overlay = null; state.queueOpen = false;
    entries.splice(state.history.i + 1);
    state.history.i++;
    state.history.n = state.history.i + 1;
    entries.push(current());
    history.pushState({ world: true, index: state.history.i, view: current() }, "");
    setMode(state.mode);
    paint(); announce(message || "Place restored. No edit gesture is armed.");
    q("#worldCanvas").focus({ preventScroll: true });
  }
  const worldView = () => ({ focus: null, selection: null, level: "world", question: null, contract: null, mode: "live", floor: "All plants", camera: { x: 0, y: 0, s: 1 } });
  const plantView = (id, selection = null, level = "plant") => ({ focus: id, selection, level, question: null, contract: null, mode: id === state.focus ? state.mode : "live", floor: id });
  function goPlant(id, selection = null, level = "plant") { navigate(plantView(id, selection, level), "Opened " + plants[id].name + (selection ? " / " + selection : "") + ". Local evidence only."); }
  function openRow(id) {
    const row = state.queue.find(r => r.id === id);
    if (!row) return;
    state.fresh = state.fresh.filter(f => f !== id);
    const target = plantView(row.plant, row.entity, "machine");
    target.question = row.id;
    navigate(target, "Opened " + row.title + " in " + plants[row.plant].name);
  }
  function openQueue() { state.queueOpen = !state.queueOpen; if (state.overlay === "network") state.overlay = null; paint(); }
  function results(query) {
    const term = query.trim().toLowerCase(), items = [];
    if (!term) {
      entries.slice().reverse().slice(0, 3).forEach(v => items.push({ group: "Recent", label: v.focus ? plants[v.focus].name : "My world", detail: v.selection || v.level, view: v }));
      D.plants.forEach(p => items.push({ group: "Plants", label: number(p.id) + " " + p.name, detail: p.env + " · Alt " + +number(p.id), plant: p.id }));
      state.bookmarks.forEach((b, i) => items.push({ group: "Bookmarks", label: b.label, detail: "Saved place", bookmark: i }));
      state.queue.forEach(r => items.push({ group: "Waiting", label: r.title, detail: plants[r.plant].name + " · " + r.kind, row: r.id }));
      return items;
    }
    D.plants.forEach(p => {
      if (p.name.toLowerCase().includes(term)) items.push({ group: "Plants", label: p.name, detail: p.env + " · " + p.revision, plant: p.id });
      p.machines.filter(m => m.toLowerCase().includes(term)).forEach(m => items.push({ group: "Machines", label: m, detail: p.name + " · " + p.env, plant: p.id, machine: m }));
    });
    D.contracts.filter(c => (c.label + " " + plants[c.from].name + " " + plants[c.to].name + " contract").toLowerCase().includes(term)).forEach(c => items.push({ group: "Contracts", label: c.label, detail: plants[c.from].name + " → " + plants[c.to].name, contract: c.id }));
    state.queue.filter(r => (r.title + " " + plants[r.plant].name).toLowerCase().includes(term)).forEach(r => items.push({ group: "Waiting", label: r.title, detail: plants[r.plant].name + " · " + r.kind, row: r.id }));
    const commands = [{ label: state.kvarm.state === "offline" ? "Reconnect Kvarm" : "Go offline", action: "connection", detail: "Kvarm · " + D.origin }, { label: "Light appearance", action: "light", detail: "This window only" }, { label: "Dark appearance", action: "dark", detail: "This window only" }, { label: "System appearance", action: "system", detail: "Follow the operating system" }];
    commands.filter(c => c.label.toLowerCase().includes(term)).forEach(c => items.push({ group: "Commands", ...c }));
    return items;
  }
  function setActive(i) {
    searchIndex = Math.max(0, Math.min(searchResults.length - 1, i));
    all("[data-result]").forEach((b, j) => b.classList.toggle("world-result-active", j === searchIndex));
    all("[data-result]")[searchIndex]?.scrollIntoView({ block: "nearest" });
  }
  function paintSearch() {
    searchResults = results(q("#worldSearch").value); searchIndex = 0;
    q("#worldResults").innerHTML = searchResults.map((r, i) => `<button data-result="${i}" class="${i === 0 ? "world-result-active" : ""}"><span class="world-eyebrow">${esc(r.group)}</span><strong>${esc(r.label)}</strong><small>${esc(r.detail)}</small><span>↗</span></button>`).join("") || "<p>No matching place.</p>";
  }
  function openSearch() {
    opener = document.activeElement;
    if (state.overlay !== "search") underSearch = state.overlay;
    state.overlay = "search"; state.queueOpen = false; paint();
    q("#worldSearch").value = ""; paintSearch(); q("#worldSearch").focus();
  }
  function closeSearch() {
    state.overlay = underSearch; underSearch = null; paint();
    (opener && root.contains(opener) ? opener : q("#worldCanvas")).focus({ preventScroll: true });
  }
  function chooseResult(i) {
    const r = searchResults[i];
    if (!r) return;
    if (r.row) openRow(r.row);
    else if (r.bookmark !== undefined) recall(r.bookmark);
    else if (r.view) navigate(structuredClone(r.view));
    else if (r.contract) openContract(r.contract);
    else if (r.plant) goPlant(r.plant, r.machine || null, r.machine ? "machine" : "plant");
    else if (r.action === "connection") connection();
    else { appearance(r.action); state.overlay = null; paint(); }
  }
  function cutaway() {
    if (!state.focus) return;
    const p = plants[state.focus];
    if (!state.selection) state.selection = p.machines.find(n => art.cut[p.id + "/" + n]);
    if (!art.cut[p.id + "/" + state.selection]) { announce(state.selection + " has no declared internals to open."); return; }
    state.level = state.level === "cutaway" ? "machine" : "cutaway"; paint();
    announce(state.level === "cutaway" ? "Cutaway open. Declared shell, bound evidence." : "Cutaway closed.");
  }
  function openContract(id, endpoint) {
    const c = D.contracts.find(c => c.id === id);
    const focus = endpoint || ([c.from, c.to].includes(state.focus) ? state.focus : c.from);
    const name = focus === c.from ? c.fromEntity : c.toEntity;
    if (state.focus !== focus || state.level === "world") {
      const view = plantView(focus, name, "machine"); view.contract = id;
      navigate(view, "Selected " + c.label + " endpoint in " + plants[focus].name); return;
    }
    state.selection = name; state.level = "machine"; state.question = null; state.contract = id; paint();
  }
  const hot = (plant, on) => { const s = q(`[data-world-site="${plant}"]`); if (s) s.classList.toggle("world-hot", on); };
  root.addEventListener("click", e => {
    const b = e.target.closest("button");
    if (!b) return;
    if (b.dataset.plant) goPlant(b.dataset.plant);
    else if (b.dataset.worldEndpoint) openContract(b.dataset.worldEndpoint, b.dataset.endpointPlant);
    else if (b.dataset.map) goPlant(b.dataset.map);
    else if (b.dataset.machine) goPlant(b.dataset.machinePlant, b.dataset.machine, "machine");
    else if (b.dataset.queue) openRow(b.dataset.queue);
    else if (b.dataset.result !== undefined) chooseResult(+b.dataset.result);
    else if (b.dataset.provenance) provenance(b.dataset.provenance, b);
    else if (b.dataset.recall !== undefined) recall(+b.dataset.recall);
    else if (b.dataset.contract) openContract(b.dataset.contract);
    else if (b.dataset.farEnd) {
      const c = D.contracts.find(c => c.id === b.dataset.farEnd), far = state.focus === c.from ? c.to : c.from;
      const view = plantView(far, far === c.to ? c.toEntity : c.fromEntity, "machine");
      view.contract = c.id; navigate(view, "Far end of " + c.label + " in " + plants[far].name);
    } else if (b.dataset.worldMode) { state.mode = b.dataset.worldMode; setMode(state.mode); paint(); }
    else if (b.dataset.close) {
      if (b.dataset.close === "overlay") { if (state.overlay === "search") { closeSearch(); return; } state.overlay = null; }
      if (b.dataset.close === "queue") state.queueOpen = false;
      if (b.dataset.close === "card") { state.question = null; state.contract = null; }
      if (b.dataset.close === "inspector") { state.selection = null; state.level = "plant"; state.question = null; state.contract = null; }
      paint();
    } else if (b.id === "worldQueueBelow") {
      q(`[data-queue="${state.fresh[0]}"]`)?.scrollIntoView({ block: "nearest" }); queue();
    } else {
      const a = b.dataset.action;
      if (a === "search") openSearch();
      if (a === "queue") openQueue();
      if (a === "back") { save(); history.back(); }
      if (a === "forward") { save(); history.forward(); }
      if (a === "world") navigate(worldView());
      if (a === "fit" && state.focus) goPlant(state.focus);
      if (a === "belts") { state.belts = !state.belts; paint(); }
      if (a === "states") { state.overlay = "states"; paint(); }
      if (a === "network") { state.overlay = state.overlay === "network" ? null : "network"; paint(); }
      if (a === "connection") connection();
      if (a === "confirm") confirmWrites();
      if (a === "mark") bookmark();
      if (a === "bookmarks") { state.overlay = "bookmarks"; paint(); }
      if (a === "cutaway") cutaway();
    }
  });
  root.addEventListener("pointerover", e => {
    const row = e.target.closest("[data-queue-plant]");
    if (row) hot(row.dataset.queuePlant, true);
    const part = e.target.closest("[data-binding]");
    if (alt && part) provenance(part.dataset.binding, part);
  });
  root.addEventListener("pointerout", e => { const row = e.target.closest("[data-queue-plant]"); if (row && !row.contains(e.relatedTarget)) hot(row.dataset.queuePlant, false); });
  root.addEventListener("focusin", e => { const row = e.target.closest("[data-queue-plant]"); if (row) hot(row.dataset.queuePlant, true); });
  root.addEventListener("focusout", e => { const row = e.target.closest("[data-queue-plant]"); if (row) hot(row.dataset.queuePlant, false); });
  q("#worldAppearance").addEventListener("change", e => appearance(e.target.value));
  q("#worldSearch").addEventListener("input", paintSearch);
  addEventListener("popstate", e => {
    if (!conceptActive("world") || !e.state?.world) return;
    Object.assign(state, e.state.view);
    state.history.i = e.state.index; state.overlay = null; state.queueOpen = false;
    setMode(state.mode); paint(); announce("Returned to the saved camera, selection and mode.");
  });
  addEventListener("keydown", e => {
    if (!conceptActive("world")) return;
    const eat = () => { e.preventDefault(); e.stopImmediatePropagation(); };
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") { eat(); state.overlay === "search" ? closeSearch() : openSearch(); return; }
    if (e.altKey && ["ArrowLeft", "ArrowRight"].includes(e.key) && !typing(e)) { eat(); save(); e.key === "ArrowLeft" ? history.back() : history.forward(); return; }
    if (e.altKey && !e.ctrlKey && !e.metaKey && /^Digit[1-5]$/.test(e.code) && !typing(e)) { eat(); goPlant(D.plants[+e.code.slice(5) - 1].id); return; }
    if (e.key === "Alt") { alt = true; root.classList.add("world-alt"); return; }
    if (state.overlay === "search") {
      if (e.key === "Escape") { eat(); closeSearch(); return; }
      if (e.key === "Tab") {
        eat();
        const items = [q("#worldSearch"), ...all("[data-result]")];
        const i = (items.indexOf(document.activeElement) + (e.shiftKey ? -1 : 1) + items.length) % items.length;
        items[i].focus();
        if (items[i].dataset.result !== undefined) setActive(+items[i].dataset.result);
        return;
      }
      if (["Enter", "ArrowDown", "ArrowUp"].includes(e.key)) {
        eat();
        const focused = document.activeElement?.dataset?.result;
        if (e.key === "Enter") chooseResult(focused !== undefined ? +focused : searchIndex);
        else setActive(searchIndex + (e.key === "ArrowDown" ? 1 : -1));
        return;
      }
    }
    if (e.key === "Escape") {
      eat();
      if (typing(e)) { e.target.blur(); return; }
      if (state.overlay) { state.overlay = null; paint(); return; }
      if (state.queueOpen) { state.queueOpen = false; paint(); return; }
      if (state.question || state.contract) { state.question = null; state.contract = null; paint(); return; }
      if (state.level === "cutaway") { state.level = "machine"; paint(); return; }
      if (state.selection) { state.selection = null; state.level = "plant"; paint(); return; }
      return;
    }
    if (e.key === "F8" && !e.ctrlKey && !e.metaKey && !e.altKey) { eat(); appearance(state.theme === "light" ? "dark" : "light"); paint(); announce("Appearance changed in this window. Your view has not moved."); return; }
    if (!bare(e)) return;
    if (e.key === "Enter" && state.overlay === "network" && state.kvarm.checked && !e.target.closest?.("button,input,select,a")) { eat(); confirmWrites(); return; }
    const key = e.key.toLowerCase(), digit = Number(e.code.replace("Digit", ""));
    if (e.shiftKey && digit >= 1 && digit <= 4 && (state.overlay === "bookmarks" || state.queueOpen)) { eat(); state.overlay === "bookmarks" ? recall(digit - 1) : openRow(state.queue[digit - 1]?.id); return; }
    if (key === "u") { eat(); openQueue(); }
    if (key === "i") { eat(); const i = state.queue.findIndex(r => r.id === state.question); openRow(state.queue[(i + 1) % state.queue.length].id); }
    if (key === "v") { eat(); state.belts = !state.belts; paint(); }
    if (key === "p") { eat(); if (e.shiftKey) navigate(worldView()); else if (state.focus) goPlant(state.focus); }
    if (key === "m") { eat(); bookmark(); }
    if (key === "x") { eat(); cutaway(); }
    if (!e.shiftKey && ["1", "2", "3", "4"].includes(e.key) && state.focus) { eat(); state.mode = ["live", "blueprint", "simulate", "replay"][+e.key - 1]; setMode(state.mode); paint(); }
  }, { capture: true });
  addEventListener("keyup", e => { if (e.key === "Alt") { alt = false; root.classList.remove("world-alt"); } });
  addEventListener("blur", () => { alt = false; root.classList.remove("world-alt"); });
  function zoomTarget(e) {
    const hit = document.elementFromPoint(e.clientX, e.clientY);
    if (state.level === "world") return hit?.closest("[data-world-site]")?.dataset.worldSite || null;
    const m = hit?.closest("[data-machine]");
    if (m) return m.dataset.machine;
    const p = plants[state.focus], near = p.machines.map(n => [n, stageAnchor(n)]).filter(([, a]) => a).map(([n, a]) => [n, Math.hypot(a[0] - e.clientX, a[1] - e.clientY)]).sort((a, b) => a[1] - b[1])[0];
    return near && near[1] < 120 ? near[0] : null;
  }
  root.addEventListener("wheel", e => {
    if (e.target.closest(".world-panel")) return;
    e.preventDefault();
    if (e.timeStamp - wheel.last > 300) { wheel.locked = false; wheel.sum = 0; }
    wheel.last = e.timeStamp;
    if (wheel.locked) return;
    if (Math.sign(e.deltaY) !== Math.sign(wheel.sum)) wheel.sum = 0;
    wheel.sum += e.deltaY;
    const inward = wheel.sum < 0;
    if (Math.abs(wheel.sum) < (inward ? 110 : 160)) return;
    wheel.sum = 0;
    const before = state.level;
    if (inward && state.level === "world") { const id = zoomTarget(e); if (id) goPlant(id); }
    else if (inward && state.level === "plant") { const name = zoomTarget(e); if (name) goPlant(state.focus, name, "machine"); }
    else if (!inward && state.level === "cutaway") { state.level = "machine"; paint(); }
    else if (!inward && state.level === "machine") { state.selection = null; state.question = null; state.contract = null; state.level = "plant"; paint(); }
    else if (!inward && state.level === "plant") navigate(worldView());
    if (state.level !== before) wheel.locked = true;
  }, { passive: false });
  const canvasEl = q("#worldCanvas");
  canvasEl.addEventListener("pointerdown", e => {
    if (state.level !== "world" || column() || e.target.closest("button,[data-binding]")) return;
    drag = { x: e.clientX, y: e.clientY, camera: { ...state.camera } };
    canvasEl.setPointerCapture(e.pointerId);
  });
  canvasEl.addEventListener("pointermove", e => {
    if (!drag) return;
    state.camera.x = drag.camera.x + e.clientX - drag.x; state.camera.y = drag.camera.y + e.clientY - drag.y;
    q("#worldViewport").style.transform = worldTransform();
  });
  canvasEl.addEventListener("pointerup", () => { if (drag) { drag = null; paint(); } });
  matchMedia("(prefers-color-scheme: light)").addEventListener("change", () => { if (state.appearance === "system") { appearance("system"); paint(); } });
  q("#worldRoutes").innerHTML = D.contracts.map(c => `<g data-route="${c.id}" class="world-route" data-kind="${c.kind}"><path class="world-route-under" d="${c.path}"/><path class="world-route-line" d="${c.path}"/><text x="${c.labelAt[0]}" y="${c.labelAt[1]}" text-anchor="middle">${esc(c.label)}</text></g>`).join("");
  q("#worldPorts").innerHTML = D.contracts.map(c => {
    const n = c.path.match(/-?\d+(?:\.\d+)?/g).map(Number);
    return [[c.from, n[0], n[1]], [c.to, n.at(-2), n.at(-1)]].map(([id, x, y]) => `<button class="world-contract-port" data-world-endpoint="${c.id}" data-endpoint-plant="${id}" style="left:${x}px;top:${y}px" title="${esc(plants[id].name)} / ${esc(id === c.from ? c.fromEntity : c.toEntity)} · ${esc(c.label)}" aria-label="Inspect ${esc(c.label)} endpoint in ${esc(plants[id].name)}"></button>`).join("");
  }).join("");
  q("#worldMapSites").innerHTML = D.plants.map(p => `<button style="left:${p.x / 10}px;top:${p.y / 10}px" data-map="${p.id}" title="${esc(p.name)}">${esc(p.id === "format" ? "format" : p.id === "site" ? "site" : p.name)}</button>`).join("");
  appearance(state.appearance); paint();
  setInterval(() => { if (state.kvarm.state !== "ok") paint(); }, 5000);
  setTimeout(() => {
    state.queue.push(D.arrival); state.fresh.push(D.arrival.id); state.arrival = true;
    paint();
    announce("acan-site needs an artifact check. Added to Waiting on you. Your view has not moved.");
  }, 3000);
}
export { worldArt };
export default {
  id: "world",
  label: "World",
  hint: "U waiting · I next · Ctrl K jump · Alt 1–5 plants · Alt ← / → history · V contracts · P fit plant · Shift P world · M mark · X cutaway · F8 appearance",
  script: `${worldMount.toString()}\nfunction worldInit(params) { worldMount(${JSON.stringify(WORLD)}, ${JSON.stringify(worldArt)}); }`,
  html: `
<main id="worldApp">
  <svg class="world-defs" width="0" height="0" aria-hidden="true"><defs>
    <pattern id="worldFogHatch" width="6" height="6" patternUnits="userSpaceOnUse"><rect width="6" height="6" fill="var(--w-shell)"/><path d="M-1 1L1 -1M0 6L6 0M5 7L7 5" stroke="var(--w-line2)" stroke-width=".6"/></pattern>
    <pattern id="worldSimGrid" width="5" height="5" patternUnits="userSpaceOnUse"><rect width="5" height="5" fill="var(--w-shell)"/><path d="M0 .2H5M.2 0V5" stroke="var(--w-blue)" stroke-width=".5" opacity=".75"/></pattern>
  </defs></svg>
  <header class="world-topbar">
    <div class="world-brand"><span class="world-logo">◎</span><b>Acan</b><span class="world-divider"></span><button data-action="world">My world</button><span class="world-slash">/</span><span id="worldBreadcrumb">My world</span><small id="worldScope"></small></div>
    <button class="world-jump" data-action="search"><span>⌕</span> Jump to a place, machine, or decision <kbd>Ctrl K</kbd></button>
    <div class="world-window-tools"><label class="world-appearance-label">Appearance<select id="worldAppearance" aria-label="Appearance"><option value="system">System</option><option value="light">Light</option><option value="dark">Dark</option></select></label><button id="worldEgress" data-action="network"></button></div>
  </header>
  <nav class="world-toolbar" aria-label="World navigation"><button id="worldBack" data-action="back" title="Back · Alt Left" aria-label="Back">←</button><button id="worldForward" data-action="forward" title="Forward · Alt Right" aria-label="Forward">→</button><span></span><button data-action="world">Fit world <kbd>⇧ P</kbd></button><button id="worldFitPlant" data-action="fit">Fit plant <kbd>P</kbd></button><button id="worldBeltToggle" data-action="belts" aria-pressed="true">Contracts <kbd>V</kbd></button><span></span><button data-action="states">Evidence states <span class="world-legend">? ◷ 0 ⏻ SIM</span></button><button data-action="mark">Mark place <kbd>M</kbd></button><button data-action="bookmarks">Bookmarks <span id="worldBookmarksCount"></span></button></nav>
  <div id="worldCanvas" tabindex="-1" aria-label="Canvas of five stable plant sites"><div class="world-drafting-grid"></div><div id="worldViewport"><svg id="worldRoutes" viewBox="0 0 1600 900"></svg><div id="worldPorts"></div><div id="worldSites"></div></div></div>
  <section id="worldStage" aria-label="Focused plant" hidden></section>
  <div class="world-bottom-left"><button class="world-waiting-button" data-action="queue" aria-pressed="false"><span class="world-amber-dot"></span>Waiting on you <b id="worldQueueCount">5</b><span id="worldQueueFresh" class="world-new" hidden></span><kbd>U</kbd></button><button id="worldKvarm" data-action="network" title="Kvarm connection · opens the network log"></button></div>
  <aside id="worldMinimap"><header><span>Stable sites</span><span>N ↑</span></header><div id="worldMapSites"></div><div id="worldMapView"></div></aside>
  <footer class="world-footer"><span><b id="worldLevel">world</b> · read only · no activity animation</span><span class="world-footer-center"><span id="worldAnnouncement" role="status" aria-live="polite"></span><span class="world-footer-hints">U waiting · I next · Ctrl K jump · Alt 1–5 plants · Alt ← / → back / forward · F8 appearance</span></span><span class="world-fixture">Scripted evidence · no network</span></footer>
  <nav class="world-local-modes" aria-label="Focused plant mode"><span>Focused plant only</span><button data-world-mode="live">Live <kbd>1</kbd></button><button data-world-mode="blueprint">Blueprint <kbd>2</kbd></button><button data-world-mode="simulate">Simulate <kbd>3</kbd></button><button data-world-mode="replay">Replay <kbd>4</kbd></button></nav>
  <section id="worldQueue" class="world-panel world-column" hidden><header><div><span class="world-eyebrow">One human review allowance</span><h2>Waiting on you <span id="worldQueueTitleCount">5</span></h2></div><button data-close="queue" aria-label="Close Waiting on you">×</button></header><p class="world-queue-intro">Kvarm enforces publication rules. Recommended holds do not prevent self-claim.</p><div id="worldQueueRows"></div><button id="worldQueueBelow" class="world-queue-below" hidden></button></section>
  <div id="worldStack" hidden><section id="worldCard" class="world-panel" hidden></section><aside id="worldInspector" class="world-panel" hidden></aside></div>
  <section id="worldNetwork" class="world-panel world-column" tabindex="-1" hidden><header><span class="world-eyebrow">Declared egress / your connection</span><button data-close="overlay" aria-label="Close network log">×</button></header><div id="worldNetworkBody"></div></section>
  <section id="worldPalette" class="world-panel" role="dialog" aria-modal="true" aria-label="Global jump" hidden><header><span class="world-eyebrow">Jump anywhere / all plants</span><button data-close="overlay" aria-label="Close global jump">×</button></header><div class="world-search-field"><span>⌕</span><input id="worldSearch" placeholder="Plant, machine, contract, decision, or command" autocomplete="off"></div><div id="worldResults"></div><footer>↑ ↓ choose · Tab moves within · Enter jump · Esc dismisses without moving</footer></section>
  <section id="worldStates" class="world-panel" hidden><header><div><span class="world-eyebrow">Evidence, not activity</span><h2>Five states. Five different meanings.</h2></div><button data-close="overlay" aria-label="Close evidence states">×</button></header><p>Missing is not zero. A stopped process is not missing telemetry.</p><div id="worldStatesList"></div><footer>Each mark opens the exact binding that draws it. Failure and fog may coexist.</footer></section>
  <section id="worldProvenance" class="world-panel" hidden></section>
  <section id="worldBookmarks" class="world-panel" hidden><header><div><span class="world-eyebrow">Places, not tabs</span><h2>Marked places</h2></div><button data-close="overlay" aria-label="Close marked places">×</button></header><div id="worldBookmarksBody"></div><p id="worldBookmarksFull" class="world-warning" hidden>Four places is the limit. M saves nothing new until a place is free.</p><footer>Shift 1 to 4 recalls a place while this card is open.</footer></section>
</main>`,
  css: `
#worldApp{display:none}
body[data-world=active]>:not(#worldApp){display:none!important}
body[data-world=active]{background:#101112}
body[data-world=active] *{animation:none!important;transition:none!important}
#worldApp{--w-paper:#111314;--w-grid:#202324;--w-major:#2a2d2d;--w-ink:#e0e1dd;--w-dim:#a3a9a9;--w-faint:#868d8e;--w-line:#343b3c;--w-line2:#505a5b;--w-panel:#191d1e;--w-top:#202627;--w-accent:#c9a45c;--w-brass-fill:#463c29;--w-ok:#54b7a4;--w-wait:#e1b15b;--w-fail:#ef826e;--w-fail-fill:#3a2420;--w-blue:#89b6d7;--w-shell:#161a1b;--w-shadow:0 12px 40px #0005;position:fixed;inset:0;background:var(--w-paper);color:var(--w-ink);font:14px/1.5 var(--sans);isolation:isolate;overflow:hidden}
#worldApp{--ok:var(--w-ok);--strain:var(--w-wait);--wait:var(--w-wait);--fail:var(--w-fail);--idle:var(--w-faint);--ghost:var(--w-blue);--text:var(--w-ink);--dim:var(--w-dim);--faint:var(--w-faint);--glass:var(--w-panel);--line:var(--w-line);--line2:var(--w-line2)}
body[data-world=active] #worldApp{display:block}
#worldApp[data-theme=light]{--w-paper:#F4F1EA;--w-grid:#E5E0D5;--w-major:#D6CFBF;--w-ink:#24282D;--w-dim:#50575E;--w-faint:#60656B;--w-line:#D4CFC3;--w-line2:#BDB5A5;--w-panel:#FCFAF5;--w-top:#F8F5ED;--w-accent:#795A27;--w-brass-fill:#E1D1AA;--w-ok:#17685D;--w-wait:#805400;--w-fail:#AC392B;--w-fail-fill:#F6E1DC;--w-blue:#28658B;--w-shell:#EEEBE3;--w-shadow:0 8px 24px #29251F14,0 1px 3px #29251F0D}
#worldApp[data-theme=light][data-mode=blueprint] :is(.world-focused,#worldStage){--w-mode-paper:#EDF3F8}
#worldApp[data-theme=light][data-mode=simulate] :is(.world-focused,#worldStage){--w-mode-paper:#F0F4EE}
#worldApp[data-theme=light][data-mode=replay] :is(.world-focused,#worldStage){--w-mode-paper:#F5ECDC}
#worldApp[data-theme=dark][data-mode=blueprint] :is(.world-focused,#worldStage){--w-mode-paper:#0F2136}
#worldApp[data-theme=dark][data-mode=simulate] :is(.world-focused,#worldStage){--w-mode-paper:#141B17}
#worldApp[data-theme=dark][data-mode=replay] :is(.world-focused,#worldStage){--w-mode-paper:#1B160F}
#worldApp [hidden]{display:none!important}
#worldApp .world-defs{position:absolute;width:0;height:0}
#worldApp button{color:inherit;line-height:inherit}
#worldApp button:hover{color:var(--w-accent)}
#worldApp button:focus-visible,#worldApp input:focus-visible,#worldApp select:focus-visible,#worldApp [tabindex]:focus-visible{outline:2px solid var(--w-accent);outline-offset:3px}
#worldApp button:disabled{opacity:.35;cursor:default}
#worldApp kbd{color:var(--w-dim);border-color:var(--w-line2);font-size:11px;white-space:nowrap}
#worldApp h2{font-size:20px;line-height:1.3;letter-spacing:-.035em;font-weight:550;margin:8px 0 10px}
#worldApp h3{font:11px var(--mono);text-transform:uppercase;letter-spacing:.1em;color:var(--w-dim);margin:22px 0 8px}
#worldApp p{margin:8px 0 14px;color:var(--w-dim)}
#worldApp small{font:11px/1.5 var(--mono);color:var(--w-dim)}
#worldApp .world-eyebrow{font:11px/1.5 var(--mono);letter-spacing:.1em;text-transform:uppercase;color:var(--w-accent)}
#worldApp .world-topbar{position:absolute;z-index:25;left:0;right:0;top:0;height:72px;display:flex;align-items:center;justify-content:space-between;padding:0 28px;border-bottom:1px solid var(--w-line);background:var(--w-paper);gap:24px}
#worldApp .world-brand{display:flex;align-items:center;gap:12px;white-space:nowrap;position:relative;min-width:420px;font-size:13px}
#worldApp .world-brand>b{font-size:15px;letter-spacing:-.03em}
#worldApp .world-logo{font-size:28px;color:var(--w-accent);line-height:1}
#worldApp .world-divider{width:1px;height:20px;background:var(--w-line);margin:0 3px}
#worldApp .world-slash{color:var(--w-faint)}
#worldApp #worldBreadcrumb{font-weight:550}
#worldApp #worldScope{position:absolute;left:130px;top:27px;font:11px var(--mono);color:var(--w-dim)}
#worldApp[data-level=world] .world-slash,#worldApp[data-level=world] #worldBreadcrumb{display:none}
#worldApp .world-jump{width:350px;flex:none;display:flex;align-items:center;gap:10px;padding:9px 12px;border:1px solid var(--w-line);border-radius:7px;color:var(--w-dim);font-size:12px;background:var(--w-panel)}
#worldApp .world-jump>span{font-size:21px;line-height:16px}
#worldApp .world-jump kbd{margin-left:auto}
#worldApp .world-window-tools{display:flex;align-items:center;gap:20px}
#worldApp .world-appearance-label{display:flex;gap:7px;align-items:center;color:var(--w-dim);font-size:12px}
#worldApp select{background:var(--w-panel);color:var(--w-ink);border:1px solid var(--w-line);border-radius:5px;padding:5px 4px;font:12px var(--sans)}
#worldApp #worldEgress{font:11px var(--mono);color:var(--w-dim)}
#worldApp .world-toolbar{position:absolute;right:24px;top:86px;z-index:8;display:flex;align-items:center;gap:16px;font-size:12px;white-space:nowrap}
#worldApp .world-toolbar>span{width:1px;background:var(--w-line);height:16px}
#worldApp .world-toolbar button{display:flex;gap:7px;align-items:center}
#worldApp .world-legend{font:11px var(--mono);color:var(--w-dim)}
#worldApp #worldBack,#worldApp #worldForward{font-size:18px}
#worldApp #worldCanvas{position:absolute;inset:73px 0 40px;outline:none;overflow:hidden;touch-action:none}
#worldApp .world-drafting-grid{position:absolute;inset:0;background-image:linear-gradient(var(--w-major) 1px,transparent 1px),linear-gradient(90deg,var(--w-major) 1px,transparent 1px),linear-gradient(var(--w-grid) 1px,transparent 1px),linear-gradient(90deg,var(--w-grid) 1px,transparent 1px);background-size:128px 128px,128px 128px,32px 32px,32px 32px;opacity:.45;mask-image:radial-gradient(ellipse at center,#0008,transparent 78%)}
#worldApp #worldViewport{position:absolute;left:0;top:-73px;width:1600px;height:900px;transform-origin:0 0}
#worldApp #worldRoutes{position:absolute;inset:0;width:1600px;height:900px;overflow:visible;pointer-events:none}
#worldApp #worldPorts{position:absolute;inset:0;pointer-events:none;z-index:6}
#worldApp .world-route-under{stroke:var(--w-paper);stroke-width:7;fill:none}
#worldApp .world-route-line{stroke:var(--w-line2);stroke-width:1.5;fill:none}
#worldApp [data-kind=runtime] :is(.world-route-line),#worldApp .world-connectors [data-kind=runtime]{stroke-dasharray:7 4}
#worldApp [data-kind=egress] .world-route-line,#worldApp .world-connectors [data-kind=egress]{stroke:var(--w-accent);stroke-dasharray:7 3}
#worldApp [data-kind=package] .world-route-line,#worldApp .world-connectors [data-kind=package]{stroke-dasharray:2 4}
#worldApp [data-kind=artifact] .world-route-line,#worldApp .world-connectors [data-kind=artifact]{stroke-dasharray:10 3 2 3}
#worldApp .world-route text{font:11px var(--mono);fill:var(--w-dim);paint-order:stroke;stroke:var(--w-paper);stroke-width:6;stroke-linejoin:round}
#worldApp .world-contract-port{position:absolute;width:9px;height:9px;border:1px solid var(--w-accent);background:var(--w-paper);border-radius:50%;transform:translate(-50%,-50%);pointer-events:auto}
#worldApp .world-contract-port::after{content:"";position:absolute;inset:-7px}
#worldApp .world-site{position:absolute;width:320px;height:236px;transform:translate(-50%,-50%)}
#worldApp .world-focused::before,#worldApp .world-hot::before{content:"";position:absolute;inset:-10px -12px;background:var(--w-mode-paper,transparent);border:1px solid var(--w-line);border-radius:5px;pointer-events:none}
#worldApp .world-hot::before{border:1.5px solid var(--w-accent)}
#worldApp .world-plant-drawing{position:absolute;left:20px;top:-1px;width:280px;height:162px;background:transparent}
#worldApp .world-plant-drawing>svg{display:block;width:100%;height:100%;overflow:visible}
#worldApp .world-plant-drawing .iso-face-text,#worldApp .terr .iso-face-text,#worldApp .terr .seal{display:none}
#worldApp .world-plant-label{position:absolute;left:5px;top:161px;display:flex;align-items:center;gap:10px;white-space:nowrap}
#worldApp .world-plant-name{display:flex;align-items:baseline;gap:8px;text-align:left}
#worldApp .world-site-id{font:11px var(--mono);color:var(--w-accent)}
#worldApp .world-plant-name strong{font-size:19px;font-weight:500;letter-spacing:-.025em}
#worldApp .world-alt-hint{display:none}
#worldApp.world-alt .world-alt-hint{display:inline-block}
#worldApp .world-chip{display:inline-flex;align-items:center;gap:5px;border:1px solid var(--w-line2);border-radius:4px;padding:1px 7px;font:11px/1.6 var(--mono);color:var(--w-dim);white-space:nowrap}
#worldApp .world-chip>b{font-weight:600}
#worldApp .world-chip>span{color:var(--w-faint);margin-left:2px}
#worldApp .world-chip[data-kind="blocks publication"]{border-color:var(--w-fail);background:var(--w-fail-fill);color:var(--w-fail)}
#worldApp .world-chip[data-kind=question]{border-color:var(--w-wait);color:var(--w-wait)}
#worldApp .world-chip[data-kind="recommended hold"]{border-style:dashed;color:var(--w-ink)}
#worldApp .world-chip[data-kind=none]{border-color:transparent;color:var(--w-faint);padding-left:0}
#worldApp .world-new{display:inline-block;border-radius:3px;padding:0 5px;background:var(--w-accent);color:var(--w-paper);font:600 11px/1.6 var(--mono)}
#worldApp .world-site-detail{position:absolute;left:27px;top:194px;display:flex;align-items:center;gap:7px;font:11px var(--mono);color:var(--w-dim);white-space:nowrap}
#worldApp .world-condition{position:absolute;left:27px;top:220px;display:flex;align-items:center;gap:7px;font-size:13px;white-space:nowrap;color:var(--w-ink)}
#worldApp .world-condition>b{font:600 12px var(--mono)}
#worldApp .world-condition[data-kind="blocks publication"]>b{color:var(--w-fail)}
#worldApp .world-condition[data-kind=question]>b{color:var(--w-wait)}
#worldApp .world-condition[data-kind="recommended hold"]>b{color:var(--w-dim)}
#worldApp .world-condition[data-kind=none]{color:var(--w-dim)}
#worldApp .world-state-mark{display:inline-flex;gap:8px;align-items:center;font:12px var(--mono);color:var(--w-dim)}
#worldApp .world-state-mark small{display:block;margin-top:3px}
#worldApp .world-bezel{border:1.5px solid var(--w-dim);min-width:34px;height:28px;padding:0 4px;box-sizing:border-box;display:inline-flex;align-items:center;justify-content:center;border-radius:14px;font:13px var(--mono);background:var(--w-shell);color:var(--w-ink)}
#worldApp :is(.world-site-detail,.world-stage-meta) .world-bezel{min-width:28px;height:20px;font-size:11px;border-width:1px}
#worldApp [data-evidence-state=unknown] .world-bezel{background:repeating-linear-gradient(135deg,transparent 0 3px,var(--w-line2) 3px 4px);border-style:dotted;color:var(--w-dim)}
#worldApp [data-evidence-state=stale] .world-bezel{border-style:dashed;color:var(--w-faint);border-color:var(--w-faint)}
#worldApp [data-evidence-state=zero] .world-bezel{border-color:var(--w-dim);color:var(--w-ink);box-shadow:inset 0 -2px var(--w-line2)}
#worldApp [data-evidence-state=stopped] .world-bezel{border-radius:4px;background:var(--w-line);border-color:var(--w-dim)}
#worldApp [data-evidence-state=simulated] .world-bezel{border-radius:3px;border-color:var(--w-blue);color:var(--w-blue);font-size:11px;background-image:linear-gradient(var(--w-line) 1px,transparent 1px),linear-gradient(90deg,var(--w-line) 1px,transparent 1px);background-size:6px 6px}
#worldApp [data-evidence-state=valid] .world-bezel{border-color:var(--w-accent)}
#worldApp #worldStage{position:absolute;top:128px;width:1100px;height:620px;z-index:6;border:1px solid var(--w-line);border-radius:6px;background:var(--w-mode-paper,var(--w-paper))}
#worldApp[data-level=cutaway] #worldStage{border-color:var(--w-accent);border-style:dashed}
#worldApp .world-stage-art{position:absolute;inset:0;background:transparent}
#worldApp .world-stage-art>svg{display:block;width:1100px;height:620px;overflow:visible}
#worldApp .world-stage-art:not([data-cutaway]) .iso-face-text{display:none}
#worldApp .world-connectors{position:absolute;inset:0;width:1100px;height:620px;pointer-events:none}
#worldApp .world-connectors path{fill:none;stroke:var(--w-line2);stroke-width:1.2}
#worldApp .world-stage-title{position:absolute;left:18px;top:14px;display:flex;align-items:center;gap:10px;white-space:nowrap;z-index:4}
#worldApp .world-stage-title strong{font-size:19px;font-weight:500;letter-spacing:-.025em}
#worldApp .world-stage-meta{display:flex;align-items:center;gap:7px;font:11px var(--mono);color:var(--w-dim);margin-left:6px}
#worldApp .world-machine{--lift:0px;position:absolute;z-index:3;transform:translate(-50%,calc(-100% - 14px - var(--lift)));display:flex;gap:6px;align-items:center;padding:3px 7px;border:1px solid var(--w-line2);border-radius:4px;background:var(--w-panel);font:11px var(--mono);white-space:nowrap}
#worldApp .world-machine::after{content:"";position:absolute;left:50%;top:100%;width:1px;height:calc(14px + var(--lift));background:var(--w-line2)}
#worldApp .world-machine[aria-current]{border-color:var(--w-accent);color:var(--w-accent);box-shadow:0 0 0 1px var(--w-accent)}
#worldApp .world-machine[aria-current]::before{content:"";position:absolute;left:50%;top:calc(100% + 14px + var(--lift));width:30px;height:18px;border:1.5px solid var(--w-accent);border-radius:50%;transform:translate(-50%,-50%);pointer-events:none}
#worldApp .world-mini{font:11px/1.45 var(--mono);padding:0 5px;border:1px solid var(--w-line2);border-radius:8px;color:var(--w-dim);background:var(--w-shell)}
#worldApp .world-mini[data-evidence-state=unknown]{border-style:dotted}
#worldApp .world-mini[data-evidence-state=stale]{border-style:dashed;color:var(--w-faint)}
#worldApp .world-mini[data-evidence-state=zero]{color:var(--w-ink)}
#worldApp .world-mini[data-evidence-state=stopped]{border-radius:2px;background:var(--w-line)}
#worldApp .world-mini[data-evidence-state=simulated]{border-radius:2px;border-color:var(--w-blue);color:var(--w-blue)}
#worldApp .world-mini[data-evidence-state=valid]{border-color:var(--w-accent);color:var(--w-ink)}
#worldApp .world-terr-label{position:absolute;z-index:2;font:11px var(--mono);letter-spacing:.06em;color:var(--w-dim);white-space:nowrap;pointer-events:none;transform:translateY(-50%)}
#worldApp .world-terr-label[data-terr-state=unmatched],#worldApp .world-terr-label[data-terr-state=verifying]{color:var(--w-wait)}
#worldApp .world-edge{position:absolute;z-index:4;display:flex;flex-direction:column;gap:3px;align-items:flex-start;max-width:330px}
#worldApp .world-edge[data-side=right]{align-items:flex-end}
#worldApp .world-edge:is([data-side=top],[data-side=bottom]){transform:translateX(-50%);align-items:center}
#worldApp .world-edge-plant{white-space:nowrap;display:flex;align-items:center;gap:7px;font-size:13px;color:var(--w-dim);padding:2px 0}
#worldApp .world-edge-contract{font:11px var(--mono);color:var(--w-dim);padding:2px 7px;border:1px solid var(--w-line);border-radius:3px;background:var(--w-panel);white-space:nowrap}
#worldApp .world-edge-contract[data-kind=egress]{border-color:var(--w-accent)}
#worldApp .world-cut-gauge{position:absolute;left:18px;bottom:16px;z-index:4;display:grid;gap:10px;padding:14px 16px;border:1px solid var(--w-line2);border-radius:6px;background:var(--w-panel);max-width:430px}
#worldApp .world-gauge-row{display:flex;align-items:center;gap:12px}
#worldApp .world-gauge-row>span:last-child{display:grid}
#worldApp .world-gauge-row b{font-weight:500;font-size:13px}
#worldApp .xl{font:11px var(--mono);color:var(--w-ink);background:var(--w-panel);border:1px solid var(--w-line);border-radius:3px}
#worldApp .leaders path{stroke:var(--w-line2)}
#worldApp .leaders circle{fill:var(--w-accent)}
#worldApp .world-bottom-left{position:absolute;left:28px;bottom:52px;display:flex;gap:16px;align-items:center;z-index:12}
#worldApp .world-waiting-button{display:flex;align-items:center;gap:10px;padding:12px 16px;border:1px solid var(--w-line2);border-radius:7px;background:var(--w-panel);box-shadow:var(--w-shadow);font-size:13px}
#worldApp .world-waiting-button[aria-pressed=true]{border-color:var(--w-accent)}
#worldApp .world-waiting-button b{border-radius:3px;padding:0 5px;background:var(--w-brass-fill);color:var(--w-accent);font:11px/1.6 var(--mono)}
#worldApp .world-amber-dot{width:6px;height:6px;border-radius:50%;background:var(--w-wait)}
#worldApp #worldKvarm{font:11px var(--mono);text-align:left;display:flex;align-items:center;gap:7px;padding:6px 10px;border:1px solid var(--w-line);border-radius:14px;background:var(--w-panel);color:var(--w-dim)}
#worldApp #worldKvarm:not([data-kv=ok]){border-color:var(--w-wait);color:var(--w-wait)}
#worldApp .world-connection-dot{width:6px;height:6px;border-radius:50%;background:var(--w-ok)}
#worldApp .world-connection-dot:not([data-kv=ok]){background:transparent;border:1.5px solid var(--w-wait);box-sizing:border-box}
#worldApp #worldMinimap{position:absolute;bottom:52px;right:24px;width:184px;height:139px;padding:10px 12px;box-sizing:border-box;border:1px solid var(--w-line);border-radius:6px;background:var(--w-panel);z-index:10}
#worldApp #worldMinimap header{display:flex;justify-content:space-between;font:11px var(--mono);color:var(--w-dim)}
#worldApp #worldMapSites{position:relative;margin-top:4px;width:160px;height:90px}
#worldApp #worldMapSites button{position:absolute;font:11px var(--mono);transform:translate(-50%,-30%);z-index:2;color:var(--w-dim)}
#worldApp #worldMapSites button[aria-current]{color:var(--w-accent)}
#worldApp #worldMapSites button::before{content:"";display:block;width:5px;height:5px;border:1px solid var(--w-line2);margin:0 auto 1px;background:var(--w-shell)}
#worldApp #worldMapSites [data-kind="blocks publication"]::before{background:var(--w-fail);border-color:var(--w-fail)}
#worldApp #worldMapSites [data-kind=question]::before{background:var(--w-wait);border-color:var(--w-wait)}
#worldApp #worldMapSites [data-kind="recommended hold"]::before{border-color:var(--w-ink)}
#worldApp #worldMapView{position:absolute;margin:30px 12px 0 12px;pointer-events:none;top:0;left:0;border:1px solid var(--w-accent);opacity:.5}
#worldApp .world-footer{position:absolute;bottom:0;left:0;right:0;height:39px;display:flex;justify-content:space-between;align-items:center;gap:20px;padding:0 28px;border-top:1px solid var(--w-line);font:11px var(--mono);color:var(--w-dim);background:var(--w-paper);z-index:25;white-space:nowrap}
#worldApp .world-footer b{color:var(--w-accent);text-transform:uppercase;font-weight:500}
#worldApp .world-fixture{color:var(--w-faint)}
#worldApp #worldAnnouncement{color:var(--w-ink)}
#worldApp #worldAnnouncement:not(:empty)::before{content:"●";color:var(--w-accent);margin-right:8px}
#worldApp #worldAnnouncement:not(:empty)+.world-footer-hints{display:none}
#worldApp .world-local-modes{position:absolute;top:82px;left:28px;display:flex;align-items:center;gap:3px;padding:4px;border:1px solid var(--w-line);border-radius:6px;background:var(--w-panel);z-index:8}
#worldApp[data-level=world] .world-local-modes{display:none}
#worldApp .world-local-modes>span{font-size:11px;color:var(--w-dim);padding:0 9px}
#worldApp .world-local-modes button{padding:4px 9px;font-size:12px;border-radius:3px}
#worldApp .world-local-modes button[aria-pressed=true]{background:var(--w-brass-fill);color:var(--w-accent)}
#worldApp .world-panel{position:absolute;z-index:18;background:var(--w-panel);border:1px solid var(--w-line2);border-radius:9px;box-shadow:var(--w-shadow);padding:20px;box-sizing:border-box;color:var(--w-ink)}
#worldApp .world-panel>header{display:flex;align-items:flex-start;justify-content:space-between;gap:10px}
#worldApp .world-panel header>button{font-size:21px;line-height:18px;color:var(--w-dim);width:20px;height:24px;flex:none}
#worldApp .world-panel>header h2{margin-bottom:0}
#worldApp .world-panel>footer{padding-top:14px;margin-top:14px;border-top:1px solid var(--w-line);font:11px/1.6 var(--mono);color:var(--w-dim)}
#worldApp .world-panel dl{display:grid;grid-template-columns:72px 1fr;gap:6px 10px;font:11px/1.5 var(--mono);margin:12px 0}
#worldApp .world-panel dt{color:var(--w-faint)}
#worldApp .world-panel dd{margin:0;overflow-wrap:anywhere}
#worldApp .world-column{right:24px;top:128px;width:390px;max-height:724px;overflow:auto;z-index:20;font-size:12px}
#worldApp #worldQueue{padding:16px 18px;display:flex;flex-direction:column;overflow:hidden}
#worldApp #worldQueue>header,#worldApp .world-queue-intro{flex-shrink:0}
#worldApp #worldQueueRows{overflow:auto;min-height:0;border-top:1px solid var(--w-line)}
#worldApp #worldQueue h2{font-size:20px;margin-top:4px}
#worldApp #worldQueueTitleCount{font:13px var(--mono);color:var(--w-accent);margin-left:8px}
#worldApp .world-queue-intro{font-size:12px;margin:6px 0 8px}
#worldApp .world-queue-row{width:100%;text-align:left;border-top:1px solid var(--w-line);padding:8px 0;display:flex;flex-direction:column;gap:2px;line-height:1.3}
#worldApp .world-row-meta{display:flex;align-items:center;gap:8px;width:100%;font:11px var(--mono)}
#worldApp .world-row-meta>b{font-weight:500;color:var(--w-accent)}
#worldApp .world-row-meta kbd{font-size:11px}
#worldApp .world-estimate{margin-left:auto;color:var(--w-faint)}
#worldApp .world-kind{font:11px var(--mono);color:var(--w-dim);white-space:nowrap}
#worldApp .world-kind[data-kind="blocks publication"]{color:var(--w-fail)}
#worldApp .world-kind[data-kind=question]{color:var(--w-wait)}
#worldApp .world-queue-row>strong{font-size:14px;font-weight:550}
#worldApp .world-queue-row>span{font-size:12px;color:var(--w-dim)}
#worldApp .world-queue-row>small{font:12px/1.35 var(--sans);color:var(--w-dim)}
#worldApp .world-queue-below{flex:none;margin-top:6px;padding:6px;border:1px solid var(--w-accent);border-radius:4px;font:11px var(--mono);color:var(--w-accent)}
#worldApp #worldInspector h2{font-size:24px;margin:10px 0 2px}
#worldApp .world-identity{margin:0 0 12px}
#worldApp .world-inspect-tabs{display:flex;align-items:center;justify-content:space-between;border-top:1px solid var(--w-line);border-bottom:1px solid var(--w-line);padding:8px 0;font-size:12px}
#worldApp .world-inspect-tabs>span{color:var(--w-accent)}
#worldApp .world-binding-reading{padding:14px 0 0;font:12px var(--mono)}
#worldApp .world-binding-reading>strong{font-size:34px;color:var(--w-accent);font-weight:400;margin-right:6px}
#worldApp .world-binding-reading>small{display:block}
#worldApp .world-warning{color:var(--w-wait)!important;font-size:12px}
#worldApp .world-bot{display:grid;gap:3px;padding:8px 0;border-top:1px solid var(--w-line)}
#worldApp .world-bot-chip{display:inline-flex;gap:7px;align-items:center;font:11px var(--mono)}
#worldApp .world-bot-chip b{font-weight:500;color:var(--w-ink)}
#worldApp .world-bot-chip>span{border:1px solid var(--w-line2);border-radius:3px;padding:0 5px}
#worldApp .world-bot-chip>[data-evidence=authenticated]{border-color:var(--w-accent);color:var(--w-accent)}
#worldApp .world-action{display:flex;align-items:center;justify-content:space-between;gap:15px;padding:9px 12px;border:1px solid var(--w-line2);border-radius:5px;width:100%;font-size:12px;margin-top:8px}
#worldApp .world-primary{display:block;border-radius:5px;background:var(--w-brass-fill);border:1px solid var(--w-accent);color:var(--w-ink);padding:10px 13px;font-size:12px;width:100%;margin:14px 0 10px;text-align:left}
#worldApp #worldStack{position:absolute;right:24px;top:128px;width:390px;max-height:724px;overflow:auto;z-index:20;display:flex;flex-direction:column;gap:10px;border-radius:9px}
#worldApp #worldStack>.world-panel{position:relative;flex:none;box-shadow:none}
#worldApp #worldInspector{font-size:12px}
#worldApp #worldCard{font-size:12px;border-color:var(--w-accent)}
#worldApp #worldCard h2{font-size:21px}
#worldApp #worldCard>.world-kind{display:inline-block;margin-top:8px}
#worldApp .world-contract-ends{font:11px/1.6 var(--mono);color:var(--w-ink)}
#worldApp #worldDeparture{border:1px dashed var(--w-line2);border-radius:5px;padding:10px 12px;margin:10px 0 4px}
#worldApp #worldDeparture p{margin:4px 0 0}
#worldApp .world-tag{display:inline-block;border:1px solid var(--w-line2);border-radius:3px;padding:2px 6px;font:11px var(--mono);color:var(--w-wait);margin:9px 0}
#worldApp .world-readback{border-top:1px solid var(--w-line);border-bottom:1px solid var(--w-line);padding:10px 0;margin:12px 0}
#worldApp .world-readback b{font:11px var(--mono);color:var(--w-accent)}
#worldApp #worldPalette{top:150px;left:50%;transform:translateX(-50%);width:620px;padding:18px 0 12px;z-index:35}
#worldApp #worldPalette>header,#worldApp #worldPalette>footer{margin:0 22px}
#worldApp .world-search-field{display:flex;gap:14px;align-items:center;margin:14px 22px;padding:9px 0 15px;border-bottom:1px solid var(--w-line)}
#worldApp .world-search-field>span{font-size:29px;line-height:25px;color:var(--w-accent)}
#worldApp .world-search-field input{background:none;border:none;color:var(--w-ink);font:18px var(--sans);width:100%;outline:none}
#worldApp .world-search-field input::placeholder{color:var(--w-faint);font-size:16px}
#worldApp #worldResults{max-height:430px;overflow:auto;padding:0 10px}
#worldApp #worldResults>button{display:grid;width:100%;grid-template-columns:1fr 35px;text-align:left;padding:8px 12px;border-radius:5px;gap:2px}
#worldApp #worldResults>button strong{grid-column:1;font-weight:500}
#worldApp #worldResults>button small{grid-column:1}
#worldApp #worldResults>button>span:last-child{grid-column:2;grid-row:2;color:var(--w-accent)}
#worldApp .world-result-active{background:var(--w-brass-fill)}
#worldApp #worldStates{width:1040px;top:200px;left:50%;transform:translateX(-50%);z-index:24}
#worldApp #worldStates>p{font-size:13px}
#worldApp #worldStatesList{display:grid;grid-template-columns:repeat(5,1fr);margin-top:22px}
#worldApp #worldStatesList>button{text-align:left;padding:15px;border-left:1px solid var(--w-line)}
#worldApp #worldStatesList>button:first-child{border-left:0;padding-left:0}
#worldApp #worldStatesList .world-state-mark{align-items:flex-start;flex-direction:column;gap:13px}
#worldApp #worldStatesList .world-bezel{min-width:58px;height:49px;font-size:21px;border-radius:25px}
#worldApp #worldStatesList [data-evidence-state=simulated] .world-bezel{font-size:14px;border-radius:3px}
#worldApp #worldStatesList [data-evidence-state=stopped] .world-bezel{border-radius:4px}
#worldApp #worldStatesList p{font-size:12px;min-height:54px;margin:16px 0}
#worldApp #worldStatesList small{overflow-wrap:anywhere}
#worldApp #worldProvenance{width:480px;z-index:40;font-size:13px}
#worldApp #worldProvenanceText{line-height:1.9}
#worldApp #worldNetwork #worldNetworkBody>.world-tag{margin-top:12px}
#worldApp .world-origin{font:13px var(--mono);color:var(--w-ink)}
#worldApp .world-remote-warning{border:1px dashed var(--w-wait);border-radius:5px;padding:11px;margin:14px 0}
#worldApp .world-remote-warning>b{color:var(--w-wait);font-size:12px}
#worldApp .world-remote-warning p{margin:4px 0 0;font-size:12px}
#worldApp #worldNetwork h3{margin:16px 0 7px}
#worldApp .world-network-row{display:flex;flex-direction:column;gap:3px;border-top:1px solid var(--w-line);padding:8px 0;font:11px var(--mono)}
#worldApp .world-network-row>b{font-weight:500;display:flex;justify-content:space-between}
#worldApp .world-network-row>span{color:var(--w-dim)}
#worldApp .world-network-row small{font:12px/1.4 var(--sans)}
#worldApp #worldBookmarks{left:50%;transform:translateX(-50%);top:240px;width:470px;z-index:28}
#worldApp #worldBookmarksBody{display:grid;gap:4px;margin-top:16px}
#worldApp .iso{--anatomy-shade-0:#121617;--anatomy-shade-1:#1a2021;--anatomy-shade-2:#242c2d;--anatomy-shade-3:#303839;--anatomy-top:#353d3e;--anatomy-hi:#bdc5c5;--anatomy-mid:#7e8989;--anatomy-lo:#4b5657;--anatomy-faint:#384344;--anatomy-weight:.8px}
#worldApp[data-theme=light] .iso{--anatomy-paper:#F2EEE4;--anatomy-face:#F8F5ED;--anatomy-top:#FCFAF5;--anatomy-shade-0:#C5C0B4;--anatomy-shade-1:#D5D0C5;--anatomy-shade-2:#E4E0D6;--anatomy-shade-3:#EEEBE3;--anatomy-lit-top:#FCFAF5;--anatomy-lit-shade:#E4E0D6;--anatomy-hi:#343A40;--anatomy-mid:#59616A;--anatomy-lo:#929187;--anatomy-faint:#CBC7BD;--anatomy-lit:#1B1F24;--anatomy-halo:#7B7465;--anatomy-halo-opacity:.12;--anatomy-dot:#795A27;--anatomy-red:#AC392B;--anatomy-green:#17685D;--anatomy-blue:#28658B;--anatomy-weight:1px}
#worldApp .iso path{vector-effect:non-scaling-stroke}
#worldApp .iso-face-text{fill:var(--w-ink)}
#worldApp .terr-fill{fill:transparent}
#worldApp .terr[data-state=housing] .terr-fill{fill:var(--w-shell)}
#worldApp .terr[data-state=housing] .iso-line{stroke:var(--w-line2)}
#worldApp .terr[data-state=unmatched] .iso-line{stroke:var(--w-wait);stroke-dasharray:5 3}
#worldApp .ent :is([data-state=fog],[data-state=stale],[data-state=stopped],[data-state=sim]) .lamp,#worldApp .ent:is([data-state=fog],[data-state=stale],[data-state=stopped],[data-state=sim],[data-state=zero]) .lamp{display:none}
#worldApp .ent[data-state=fog]{filter:none;opacity:1}
#worldApp .ent[data-state=fog] .iso-top{fill:url(#worldFogHatch)}
#worldApp .ent[data-state=fog] .iso-edge{stroke-dasharray:2 2}
#worldApp .ent[data-state=fog] .iso-dots{opacity:.5}
#worldApp .ent[data-state=stale]{opacity:.55}
#worldApp .ent[data-state=stopped] :is(.iso-top,.iso-shade){fill:var(--anatomy-shade-0)}
#worldApp .ent[data-state=stopped]{opacity:.8}
#worldApp .ent[data-state=sim] .iso-top{fill:url(#worldSimGrid)}
#worldApp .ent[data-state=sim] .iso-line{stroke:var(--w-blue)}
@media(max-width:1200px){#worldApp .world-topbar{gap:12px;padding:0 18px}#worldApp .world-brand{min-width:280px}#worldApp .world-jump{width:290px}#worldApp .world-window-tools{gap:9px}#worldApp .world-appearance-label{font-size:0}}
`
};
