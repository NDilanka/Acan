import { DECLARATIONS, CLASHES, WITNESSES, QUESTION, ORDER, KVARM } from "./core-data.mjs";

function clashMount(clashData) {
  if (!conceptActive("clash")) return;
  const { declarations, clashes, witnesses, order } = clashData;
  const blocking = clashes.filter((c) => c.severity === "blocking");
  const kvFlag = new URLSearchParams(window.location.hash.slice(1)).get("kv");
  const kv = { script: kvFlag === "offline" ? "offline" : "ok", connection: kvFlag === "offline" ? "offline" : "ok", log: [], held: [], bytes: 0, panel: false, decisions: {}, seq: 0, lastReturn: null };
  const state = window.clashState = { footprints: false, open: null, settled: { c1: null, c2: null }, kv };
  let kvMs = 0, kvInflight = [], kvStripHtmlCache = null, provKey = null;
  const undo = [];
  window.clashUndo = undo;
  let lastOpened = null;
  const postedOrder = { sentence: order.placeholder };
  const layer = $("#clashLayer"), card = $("#clashCard"), svg = $("#clashDrawing");
  const pins = $("#clashPins"), labels = $("#clashLabels"), plate = $("#clashOrder");
  const byId = Object.fromEntries(declarations.map((d) => [d.id, d]));
  const provenance = (d) => d.order ? "claimed · declared" : d.proposal ? "proposal · not claimed" : "history · not declared";
  const inferred = (c) => c.decls.some((id) => !byId[id].order && byId[id].writes.every((w) => w.source === "history"));
  const title = (c) => c.at.belt ? "Retry meets breaker" : "One table, two migrations";
  const result = (c) => (c.id === "c1" ? { split: "port v1", left: "waits for payout order", right: "breaker first", serialize: "after payout order" } : { serialize: "after radar-otel", owner: "owner Droid", merge: "joined migration territory" })[state.settled[c.id]];
  const color = (d) => d.agent === "Claude Code" ? "#c9a45c" : "#64bbae";
  const escape = (value) => String(value).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
  const clamp = (n, min, max) => Math.max(min, Math.min(max, n));
  const K = clashData.kv;
  const bytesFmt = (n) => n.toLocaleString("en-US") + " B";
  const kvRow = (r) => '<div class="clash-kv-row" data-state="' + r.state + '" title="' + escape(r.destination + " · " + r.reason) + '"><b>' + r.op + (r.count > 1 ? " ×" + r.count : "") + '</b><span>' + escape(r.reason) + '</span><i>' + (r.result || (r.state === "held" ? "held" : r.state === "sending" ? "sending" : bytesFmt(r.bytes))) + '</i></div>';
  const kvStripHtml = (max) => {
    const rows = [...kv.log, ...kv.held.map((h) => h.row)];
    return '<header><span>↗ ' + K.host + (kv.connection === "ok" ? "" : " · " + (kv.connection === "offline" ? "offline" : "checked")) + ' · ' + rows.length + (rows.length === 1 ? " op" : " ops") + '</span>' + (kv.held.length ? '<b data-tone="wait">' + kv.held.length + ' held</b>' : '<b>' + bytesFmt(kv.bytes) + '</b>') + '</header>' + rows.slice(-max).map(kvRow).join("");
  };
  const kvPill = () => {
    const el = $(".egress");
    if (!el) return;
    el.dataset.kv = kv.connection === "ok" ? "ok" : kv.connection === "offline" ? "offline" : "checked";
    el.setAttribute("role", "button"); el.tabIndex = 0;
    el.title = "Kvarm · " + K.origin + " · declared egress · U";
    const next = '<span class="dot"></span>' + (kv.connection === "ok" ? "local + kvarm · " + bytesFmt(kv.bytes) + " out" : "kvarm " + (kv.connection === "offline" ? "offline" : "checked") + " · " + kv.held.length + " held");
    if (el.innerHTML !== next) el.innerHTML = next;
  };
  const kvRender = () => {
    kvPill();
    const strip = $("#clashKv");
    if (strip) { const html = kvStripHtml(2); if (html !== kvStripHtmlCache) { kvStripHtmlCache = html; strip.innerHTML = html; } }
    if (kv.panel) kvPanelRender();
  };
  const kvAt = (ms, done) => kvInflight.push({ due: kvMs + ms, row: null, done });
  const kvTransmit = (row, done) => {
    kv.log.push(row);
    kv.bytes += row.bytes;
    row.state = "sending";
    kvInflight.push({ due: kvMs + K.ops[row.op].ms, row, done });
    kvRender();
  };
  const kvSend = (op, reason, done, force = false) => {
    const spec = K.ops[op];
    const row = { op, reason: reason || spec.reason || op, bytes: spec.bytes, destination: K.host, state: "held", count: 1 };
    if (kv.connection !== "ok" && !force) { kv.held.push({ row, done }); kvRender(); paint(); return; }
    kvTransmit(row, done);
  };
  const kvReconnect = () => {
    if (kv.connection !== "offline") return;
    kv.connection = "checking";
    kvSend("check_heads_and_leases", null, () => {
      kv.connection = "checked"; kvRender();
      toast("Checked heads and leases. Enter sends " + kv.held.length + " held writes.");
    }, true);
    kvPanelToggle(true);
  };
  const kvSendHeld = () => {
    if (kv.connection !== "checked") return;
    kv.connection = "ok";
    const n = kv.held.length;
    const next = () => { const h = kv.held.shift(); if (!h) { kvRender(); paint(); return; } kvTransmit(h.row, (row) => { if (h.done) h.done(row); next(); }); };
    next();
    toast("Sending " + n + " held writes to Kvarm, in order.");
  };
  const kvPanelRender = () => {
    const p = $("#clashKvPanel");
    p.hidden = !kv.panel;
    if (!kv.panel) return;
    const action = kv.connection === "offline" ? '<div class="clash-kv-status" data-tone="wait">Offline · Kvarm fields are stale · writes are held</div><button class="clash-kv-primary" data-clash-kv="reconnect">Reconnect <kbd>Enter</kbd></button>'
      : kv.connection === "checking" ? '<div class="clash-kv-status">checking heads and leases</div>'
      : kv.connection === "checked" ? '<div class="clash-kv-status" id="clashKvChecked">checked heads and leases · ' + escape(K.checked) + '</div><button class="clash-kv-primary" data-clash-kv="send">Send ' + kv.held.length + ' held ' + (kv.held.length === 1 ? "write" : "writes") + ' <kbd>Enter</kbd></button>' : "";
    const decisions = Object.values(kv.decisions).flat();
    p.innerHTML = '<header class="clash-card-head"><div><h2>' + K.host + '</h2><small>Kvarm · declared egress</small></div><button data-clash-panel-close aria-label="Close Kvarm">× <kbd>Esc</kbd></button></header><div class="clash-kv-body"><p class="clash-kv-note">Opt-in connection to your own origin, ' + K.origin + '. Scripted fixture. No browser network requests.</p>' + action +
      '<div class="clash-kv-section">Held writes · ' + kv.held.length + '</div>' + (kv.held.map((h) => kvRow(h.row)).join("") || '<div class="clash-kv-row"><b>0</b><span>nothing held locally</span><i></i></div>') +
      '<div class="clash-kv-section">Network log · ' + bytesFmt(kv.bytes) + '</div>' + (kv.log.map(kvRow).join("") || '<div class="clash-kv-row"><b>0 B</b><span>no outbound operations</span><i></i></div>') +
      '<div class="clash-kv-section">Decisions · ' + decisions.length + '</div>' + (decisions.map((d) => '<div class="clash-kv-row" data-state="' + d.state + '"><b>' + d.id + '</b><span>' + escape(d.label) + '</span><i>' + d.state + '</i></div>').join("") || '<div class="clash-kv-row"><b>0</b><span>no decisions recorded</span><i></i></div>') + '</div>';
    if (kv.panel) { const b = p.querySelector(".clash-kv-primary") || p.querySelector("[data-clash-panel-close]"); if (document.activeElement && !p.contains(document.activeElement)) b.focus({ preventScroll: true }); }
  };
  const kvPanelToggle = (openPanel = !kv.panel) => { kv.panel = openPanel; kvPanelRender(); };
  const decisionTop = (id) => { const list = kv.decisions[id]; return list && list[list.length - 1]; };
  const kvRecordDecision = (c, kind, label, supersedes) => {
    const id = "dec-" + c.id + "-" + ++kv.seq;
    const decision = { id, kind, label, state: kv.connection === "ok" ? "syncing" : "pending" };
    (kv.decisions[c.id] = kv.decisions[c.id] || []).push(decision);
    kvSend("record_decision", K.choices[c.id] + " · " + id + (supersedes ? " · supersedes " + supersedes : ""), () => { if (decision.state === "syncing" || decision.state === "pending") { decision.state = "synced"; paint(); } });
    return decision;
  };
  const kvSettle = (c, kind) => {
    const r = c.resolutions.find((r) => r.kind === kind);
    const map = (K.settle[c.id] || {})[kind] || {};
    const prior = decisionTop(c.id);
    if (prior) prior.state = "superseded";
    kvRecordDecision(c, kind, r.label, prior && prior.id);
    if (map.push) kvSend("git_push", map.push);
    for (const [decl, text] of map.intents || []) kvSend("set_intent", decl + " · " + text);
  };
  const kvUndo = (id) => {
    const top = decisionTop(id);
    if (top) top.state = "superseded";
    const c = blocking.find((c) => c.id === id);
    const previous = state.settled[id];
    kvRecordDecision(c, previous, previous ? "Restores " + previous : "Reopens the clash", top && top.id);
  };
  const forecastLine = (c) => {
    const f = K.forecast[c.id];
    if (!f) return "";
    const text = f.truncated ? "forecast truncated · " + f.detail : "forecast · " + f.text + " · " + f.detail;
    return '<div class="clash-forecast" data-clash-binding="forecast:' + c.id + '" data-truncated="' + !!f.truncated + '" title="Kvarm forecast reads file paths only. This clash is proven by the declarations.">' + escape(text) + ' · ' + f.ageS + ' s old</div>';
  };
  const binding = (key) => {
    const rev = K.lease.generation, sheet = order.sheet, remote = kv.connection === "ok" ? "valid" : "stale";
    const since = kv.lastReturn === null ? "age not reported" : Math.max(0, Math.round((kvMs - kv.lastReturn) / 1000)) + " s ago";
    if (key === "flag") return { entity: "Order " + order.id, channel: "order.claim", value: "CLAIMED · Claude Code · lease " + rev, state: remote, sourceClass: K.lease.evidence + " · Kvarm lease", locator: "kvarm://" + K.host + "/" + K.repo + "/tasks/" + K.lease.task, revision: rev + " · " + sheet, age: since, policy: "Kvarm fields stale after " + K.staleAfter, meaning: "Claimed by readback match and a live lease of the same generation" };
    if (key === "bot:bot-cc") return { entity: "Claude Code · " + K.lease.session, channel: "agent.principal", value: K.lease.session, state: remote, sourceClass: K.lease.evidence, locator: "kvarm token · principal " + K.lease.session, revision: rev + " · " + sheet, age: since, policy: "session handshake · lease " + rev, meaning: "Claimed " + K.lease.task + " with its own Kvarm token" };
    if (key.startsWith("forecast:")) {
      const id = key.slice(9), f = K.forecast[id];
      return { entity: "Forecast · " + id, channel: "kvarm.forecast", value: f.truncated ? "truncated · " + f.detail : f.text + " · " + f.detail, state: f.truncated ? "unknown" : remote, sourceClass: "observed · Kvarm forecast", locator: "kvarm://" + K.host + "/" + K.repo + "/forecasts/" + id, revision: sheet, age: f.ageS + " s ago", policy: "cached forecast · Kvarm fields stale after " + K.staleAfter, meaning: f.truncated ? "Forecast stopped early. The clash is proven by the declarations, not the forecast." : "Forecast reads file paths only. A semantic clash needs no path overlap." };
    }
    if (key.startsWith("pin:")) {
      const c = blocking.find((c) => c.id === key.slice(4));
      const d = decisionTop(c.id);
      return { entity: "Clash " + c.id + " · " + title(c), channel: "clash.decision", value: state.settled[c.id] ? state.settled[c.id] + (d ? " · decision " + d.state : "") : "open · before code", state: d && d.state === "pending" ? "stale" : remote, sourceClass: "declared · your settlement", locator: "kvarm://" + K.host + "/" + K.repo + "/" + K.choices[c.id], revision: (d ? d.id : "no decision") + " · " + sheet, age: since, policy: "decisions are append-only · Ctrl Z supersedes", meaning: "Settled before code. No file was edited." };
    }
    return null;
  };
  const provOpen = (key, anchorEl) => {
    const b = binding(key);
    if (!b) return;
    provKey = key;
    const el = $("#clashProv");
    el.innerHTML = '<header class="clash-card-head"><div><h2>' + escape(b.entity) + '</h2><small>Binding · ' + escape(b.sourceClass) + '</small></div><button data-clash-prov-close aria-label="Close provenance">× <kbd>Esc</kbd></button></header><div class="clash-kv-body"><dl class="clash-prov-dl"><dt>Channel</dt><dd>' + escape(b.channel) + '</dd><dt>Value · state</dt><dd data-evidence-state="' + b.state + '">' + escape(b.value + " · " + b.state) + '</dd><dt>Source</dt><dd>' + escape(b.sourceClass) + '</dd><dt>Locator</dt><dd>' + escape(b.locator) + '</dd><dt>Revision</dt><dd>' + escape(b.revision) + '</dd><dt>Freshness</dt><dd>' + escape(b.age + " · " + b.policy) + '</dd><dt>Meaning</dt><dd>' + escape(b.meaning) + '</dd></dl></div>';
    el.hidden = false;
    const r = anchorEl.getBoundingClientRect(), w = el.offsetWidth, h = el.offsetHeight;
    el.style.left = Math.max(20, Math.min(innerWidth - w - 20, r.left)) + "px";
    el.style.top = (r.bottom + 10 + h <= innerHeight - 50 ? r.bottom + 10 : Math.max(70, r.top - h - 10)) + "px";
  };
  const provClose = () => { provKey = null; $("#clashProv").hidden = true; };
  const point = (x, y, z = 0) => {
    const p = k.iso([x, y, z], P), r = world.getBoundingClientRect(), s = r.width / W;
    return [r.left + p[0] * s, r.top + p[1] * s];
  };
  const anchor = (id) => {
    const p = anchors[id], r = world.getBoundingClientRect(), s = r.width / W;
    return [r.left + p[0] * s, r.top + p[1] * s];
  };
  const location = (c) => {
    if (!c.at.layer) return anchor(c.at.entity);
    const e = ENT[c.at.entity], s = e.size || 1;
    return point(e.x + 21 * s, e.y - 4 * s, 26 * s);
  };
  const path = (pts) => pts.map((p, i) => (i ? "L" : "M") + p.map((v) => v.toFixed(1)).join(" ")).join("");
  const wire = (id, pad = 4) => {
    const e = ENT[id], a = ASSETS[e.kind], s = e.size || 1;
    const corners = [[-1, -1], [1, -1], [1, 1], [-1, 1]];
    const ring = (z) => corners.map(([x, y]) => point(e.x + x * (a.w * s / 2 + pad), e.y + y * (a.d * s / 2 + pad), z));
    const base = ring(2), top = ring(a.top * s + pad);
    return path([...base, base[0]]) + path([...top, top[0]]) + top.map((p, i) => path([p, base[i]])).join("");
  };
  const inView = ([x, y]) => x > 20 && x < innerWidth - 20 && y > 180 && y < innerHeight - 65;
  const focusPin = (id) => pins.querySelector('[data-clash-pin="' + id + '"]').focus({ preventScroll: true });
  const close = () => {
    const id = state.open;
    state.open = null; card.hidden = true;
    if (id) focusPin(id);
    paint();
  };
  const next = () => {
    const current = blocking.findIndex((c) => c.id === (state.open || lastOpened));
    const cycle = [...blocking.slice(current + 1), ...blocking.slice(0, current + 1)];
    open((cycle.find((c) => !state.settled[c.id]) || cycle[0]).id);
  };
  const settle = (kind) => {
    const c = blocking.find((c) => c.id === state.open);
    if (document.body.dataset.mode !== "live" || !c || !c.resolutions.some((r) => r.kind === kind)) return;
    if (state.settled[c.id] === kind) {
      close();
      toast("Settlement unchanged. No file was edited.");
      return;
    }
    undo.push({ id: c.id, previous: state.settled[c.id] });
    state.settled[c.id] = kind;
    kvSettle(c, kind);
    syncPlant();
    close();
    toast(kv.connection === "ok" ? "Settled before code. Decision logged to Kvarm. No file was edited." : "Settled before code. Decision pending · Kvarm offline. No file was edited.");
  };
  const undoSettlement = () => {
    if (document.body.dataset.mode !== "live") return;
    const last = undo.pop();
    if (!last) return toast("No settlement to undo.");
    const superseded = decisionTop(last.id);
    kvUndo(last.id);
    state.settled[last.id] = last.previous;
    syncPlant();
    open(last.id);
    toast("Undid the settlement. " + (superseded ? superseded.id + " superseded · Kvarm keeps both. " : "") + "No file was edited.");
  };
  const declarationHtml = (d, c) => {
    const write = d.writes.find((w) => c.at.layer ? w.part === c.at.layer : w.entity === (d.proposal || "retry"));
    const difference = write?.detail?.replace(/^adds /, "") || d.asserts.split(". ")[0];
    return '<details class="clash-declaration" data-unclaimed="' + !d.order + '" style="--agent:' + color(d) + '"><summary><b>' + escape(difference) + '</b><span>' + escape(d.agent) + '</span><small>' + escape(provenance(d)) + '</small></summary><div class="clash-declaration-body" tabindex="0"><header>' + escape(d.territory) + '</header><p>' + escape(d.asserts) + '</p><div class="clash-writes">' + d.writes.map((w) => '<div><span class="clash-source">' + escape(d.order && w.source === "declared" ? "declared · kvarm intent" : w.source) + '</span><b>' + escape(w.entity + " · " + w.part) + '</b><code>' + escape(w.path) + '</code>' + (w.detail ? '<small>' + escape(w.detail) + '</small>' : '') + '</div>').join("") + '</div></div></details>';
  };
  const open = (id) => {
    const c = blocking.find((c) => c.id === id);
    if (!c || document.body.dataset.mode !== "live") return;
    select(null);
    state.open = lastOpened = id;
    const settled = state.settled[id];
    const witness = witnesses.find((w) => w.id === c.breaks);
    const suggestion = c.resolutions.find((r) => r.kind === c.suggested);
    const why = c.at.layer ? suggestion.detail.split(". ")[0] + "." : suggestion.detail.split(". ").slice(1).join(". ");
    card.dataset.advisory = inferred(c);
    card.innerHTML = '<header class="clash-card-head"><span class="clash-icon">' + (settled ? "✓" : "◇") + '</span><div><h2>' + title(c) + '</h2><small>' + escape(c.at.belt ? "API to PayPal belt" : c.at.entity + " · " + c.at.layer) + (settled ? " · settled" : " · before code") + '</small></div><button data-clash-close aria-label="Close clash">× <kbd>Esc</kbd></button></header><div class="clash-consequence"><small>' + (inferred(c) ? "POSSIBLE CONSEQUENCE · HISTORY IS ADVISORY" : "CONSEQUENCE · BREAKS " + escape(c.breaks)) + '</small><p>' + escape(c.consequence) + '</p>' + (witness ? '<div class="clash-breaks"><b>Breaks ' + escape(witness.id) + ' · ' + escape(witness.context) + '</b><span>Required · ' + escape(witness.check) + '</span></div>' : '<div class="clash-history-note">History is not a declaration. This overlap does not block code.</div>') + forecastLine(c) + '</div><div class="clash-pair">' + c.decls.map((id) => declarationHtml(byId[id], c)).join("") + '</div><div class="clash-resolutions"><h3>Settle <small>before any file is edited</small></h3><p class="clash-why"><b>Why ·</b> ' + escape(why) + '</p>' + [...c.resolutions].sort((a, b) => Number(b.kind === c.suggested) - Number(a.kind === c.suggested)).map((r) => '<button data-clash-resolution="' + r.kind + '" data-suggested="' + (r.kind === c.suggested) + '" aria-pressed="' + (settled === r.kind) + '"><kbd>Shift ' + r.key + '</kbd><span><b>' + escape(r.label) + (r.kind === c.suggested ? ' <i>suggested</i>' : '') + (settled === r.kind ? ' <em>✓ settled</em>' : '') + '</b><small>' + escape(r.detail) + '</small></span></button>').join("") + '</div>';
    card.hidden = false;
    paint();
    card.querySelector("[data-clash-close]").focus({ preventScroll: true });
    const [x, y] = location(c), left = innerWidth - card.offsetWidth - 24;
    const hiddenX = x < 300 || x > left - 220, hiddenY = y < 250 || y > innerHeight - 260;
    if (hiddenX || hiddenY) {
      if (hiddenX) goal.x += (left + 240) / 2 - x;
      if (hiddenY) goal.y += innerHeight / 2 - y;
      animate();
    }
  };

  const bot = { id: "bot-cc", mono: "CC", kind: "bot", size: 1, x: ENT.paypal.x - 34, y: ENT.paypal.y + 34, state: "ok", params: { target: [ENT.paypal.x, ENT.paypal.y, 12] }, name: "Claude Code", reading: "claimed · paypal-payouts", sub: "readback matched every witness" };
  SCENE.entities.push(bot); ENT[bot.id] = bot;
  const agentRow = $(".bl .agent.off");
  const agentStatus = [...agentRow.childNodes].find((n) => n.nodeType === Node.TEXT_NODE && n.textContent.trim());
  agentRow.classList.remove("off");
  agentRow.querySelector("span").textContent = "before code";
  agentRow.onclick = () => { if (conceptActive("clash")) flyTo("bot-cc"); };
  $("#chips").insertAdjacentHTML("beforeend", '<div class="chip clash-agent-chip" data-id="bot-cc" data-state="ok" data-clash-binding="bot:bot-cc" tabindex="0"><span class="mono">CC</span>Claude Code<small class="clash-chip-sub">' + K.lease.session + ' · ' + K.lease.evidence + ' · lease ' + K.lease.generation + '</small><span class="rd">claimed · paypal-payouts</span></div>');
  const territory = SCENE.territories.find((t) => t.id === order.territory);
  Object.assign(territory, { state: "claimed", fog: false, label: "PAYPAL-PAYOUTS · CLAIMED · CLAUDE CODE" });
  const territoryLabels = Object.fromEntries(SCENE.territories.map((t) => [t.id, t.label]));
  ASSETS.clashPort = {
    family: "fitting", label: "Port contract", maps: "a declared result contract", w: 26, d: 12, top: 4,
    build(c, { x, y }) { c.put([x, y, 2], c.box(x - 13, y - 6, 26, 12, 0, 4, { tone: "hi" })); },
  };
  function syncPlant() {
    for (const t of SCENE.territories) t.label = state.settled.c2 === "merge" && ["paypal-payouts", "radar-otel"].includes(t.id) ? t.id === "paypal-payouts" ? "PAYPAL-PAYOUTS + RADAR-OTEL" : "" : territoryLabels[t.id];
    if (state.settled.c1 === "split" && !ENT["clash-port"]) {
      const port = { id: "clash-port", kind: "clashPort", x: 126, y: -20, state: "ghost", mono: "v1", name: "paypal.result v1", reading: "owner Claude Code", sub: "before code" };
      SCENE.entities.push(port); ENT[port.id] = port;
    } else if (state.settled.c1 !== "split" && ENT["clash-port"]) {
      SCENE.entities.splice(SCENE.entities.indexOf(ENT["clash-port"]), 1); delete ENT["clash-port"];
    }
    renderPlant();
  }
  Object.assign(TERR[order.territory], { state: "claimed", owner: "Claude Code", fog: null });
  if (FOGD.paypal) { Object.keys(FOGD.paypal.s).forEach((key) => FOGD.paypal.s[key] = 1); FOGD.paypal.why = ""; }
  plate.innerHTML = '<span class="clash-order-flag" data-clash-binding="flag" tabindex="0">CLAIMED · Claude Code · lease ' + K.lease.generation + '</span><p>' + escape(postedOrder.sentence) + '</p><div class="clash-checks">' + witnesses.map((w) => '<span title="' + escape(w.check) + '">✓ ' + w.id + '</span>').join("") + '<span class="clash-budget">' + escape(order.budgets[0]) + '</span></div><small>' + escape(order.sheet) + ' · locked checks</small><small>' + escape(order.cannotSee[0].agent) + ' cannot see orders. Passive session, no MCP.</small><div class="clash-kv" id="clashKv" data-clash-binding="flag"></div>';
  pins.innerHTML = blocking.map((c) => '<button class="clash-pin" data-clash-pin="' + c.id + '" data-clash-binding="pin:' + c.id + '" data-advisory="' + inferred(c) + '" aria-label="Open clash ' + c.id + '"><span class="clash-pin-head">◇</span><span class="clash-pin-stem"></span><span class="clash-pin-label"></span></button>').join("");
  $(".wait").insertAdjacentHTML("beforeend", '<small id="clashWaiting"></small>');
  $("#clashAdvisory").textContent = "advisory · " + clashes.find((c) => c.severity === "advisory").consequence;
  renderPlant();
  layoutChips();

  function paint() {
    if (!conceptActive("clash")) return;
    const count = blocking.filter((c) => !state.settled[c.id]).length;
    $("#clashCount").textContent = count ? count + (count === 1 ? " clash before code" : " clashes before code") : "0 clashes · " + blocking.length + " settled before code";
    $("#clashCount").dataset.done = !count;
    $("#clashWaiting").textContent = count + (count === 1 ? " clash" : " clashes");
    $("#clashFootprints").setAttribute("aria-pressed", state.footprints);
    $("#clashUndo").disabled = !undo.length;
    document.body.classList.toggle("clash-retry-wins", state.settled.c1 === "left");
    document.body.classList.toggle("clash-breaker-wins", state.settled.c1 === "right");
    document.body.classList.toggle("clash-open", !!state.open);
    agentStatus.textContent = state.settled.c1 === "right" ? " replanning" : " claimed payout order";
    let drawing = "";
    const effectRects = [];
    const badges = [];
    for (const c of blocking) {
      const pin = pins.querySelector('[data-clash-pin="' + c.id + '"]'), [x, y] = location(c);
      pin.style.left = x + "px"; pin.style.top = y + "px";
      pin.hidden = !inView([x, y]);
      pin.dataset.settled = !!state.settled[c.id];
      pin.querySelector(".clash-pin-head").textContent = state.settled[c.id] ? "✓" : "◇";
      const label = pin.querySelector(".clash-pin-label");
      const dec = decisionTop(c.id);
      label.textContent = state.settled[c.id] ? result(c) : title(c) + (inferred(c) ? " · history advisory" : "");
      if (dec && dec.state !== "synced") label.dataset.decision = dec.state === "syncing" ? "pending" : dec.state; else delete label.dataset.decision;
      if (state.settled[c.id] && state.settled[c.id] !== "split" && state.settled[c.id] !== "serialize") label.dataset.clashEffect = state.settled[c.id];
      else delete label.dataset.clashEffect;
      pin.setAttribute("aria-label", (state.settled[c.id] ? "Reopen settled clash " : "Open clash ") + c.id);
      if (c.at.layer) {
        const e = ENT.sqlite, s = e.size || 1;
        const pts = Array.from({ length: 33 }, (_, i) => point(e.x + Math.cos(i / 32 * Math.PI * 2) * 26 * s, e.y + Math.sin(i / 32 * Math.PI * 2) * 26 * s, 26 * s));
        drawing += '<path class="clash-layer-ring" data-advisory="' + inferred(c) + '" d="' + path(pts) + '"/>';
      }
    }
    const [px, py] = anchor(order.on);
    plate.hidden = !inView([px, py]) || !!state.open;
    if (!plate.hidden) {
      const width = plate.offsetWidth, height = plate.offsetHeight;
      const left = clamp(px + 72, 24, innerWidth - width - 24), top = clamp(py + 66, 170, innerHeight - height - 75);
      plate.style.left = left + "px"; plate.style.top = top + "px";
      drawing += '<path class="clash-card-leader" d="' + path([[px, py], [clamp(px, left, left + width), clamp(py, top, top + height)]]) + '"/>';
    }
    if (state.open) {
      const [x, y] = location(blocking.find((c) => c.id === state.open));
      const width = card.offsetWidth, height = card.offsetHeight;
      const cx = innerWidth - width - 24, cy = clamp(y - height / 2, 170, innerHeight - height - 74);
      card.style.left = cx + "px"; card.style.top = cy + "px";
      drawing += '<path class="clash-card-leader" d="' + path([[x, y - 22], [cx > x ? cx : cx + width, clamp(y - 22, cy + 20, cy + height - 20)]]) + '"/>';
    }
    drawing += '<path class="clash-advisory" d="' + wire("api", 7) + '"><title>' + escape(clashes.find((c) => c.severity === "advisory").consequence) + '</title></path>';
    if (state.footprints) {
      for (const d of declarations) {
        drawing += '<g class="clash-footprint" data-declaration="' + d.id + '" data-unclaimed="' + !d.order + '" stroke="' + color(d) + '" opacity="' + (d.order ? ".65" : ".3") + '">' + d.touches.map((id) => '<path d="' + wire(id, d.id === "d2" ? 8 : 3) + '"/>').join("") + d.belts.map((belt) => {
          const [from, to] = belt.split("->"), b = SCENE.belts.find((b) => b.from === from && b.to === to);
          return b ? '<path class="clash-belt-outline" d="' + path(b.pts.map(([x, y]) => point(x, y, 3))) + '"/>' : '';
        }).join("") + '</g>';
        for (const w of d.writes) badges.push({ at: anchor(w.entity), color: color(d), text: w.entity + " · " + d.agent + " · " + (d.order ? (w.source === "declared" ? "declared · kvarm intent" : w.source) : provenance(d)), title: d.territory + " · " + w.source + " · " + w.path + (w.detail ? " · " + w.detail : ""), unclaimed: !d.order });
      }
    }
    if (state.settled.c1 === "split") {
      const p = point(126, -20, 7), pr = plate.getBoundingClientRect();
      let left = p[0] - 85;
      if (!plate.hidden && left + 170 > pr.left - 8 && left < pr.right + 8 && p[1] + 64 > pr.top - 8 && p[1] + 20 < pr.bottom + 8) left = pr.left - 182;
      effectRects.push({ left, right: left + 170, top: p[1] + 20, bottom: p[1] + 64 });
      drawing += '<g class="clash-port" data-clash-effect="split"><rect x="' + left + '" y="' + (p[1] + 20) + '" width="170" height="44" rx="6"/><text x="' + (left + 85) + '" y="' + (p[1] + 37) + '">paypal.result v1</text><text class="clash-port-owner" x="' + (left + 85) + '" y="' + (p[1] + 54) + '">owner Claude Code</text>' + ["retry", "breaker"].map((id) => '<path marker-end="url(#clashArrow)" d="' + path([anchor(id), [left + 85, p[1] + 20]]) + '"/>').join("") + '</g>';
    }
    if (state.settled.c1 === "right") {
      const [x, y] = anchor("bot-cc");
      drawing += '<text class="clash-plant-note" data-clash-effect="replanning" x="' + x + '" y="' + (y + 28) + '">Claude Code · replanning</text>';
    }
    for (const c of blocking.filter((c) => state.settled[c.id] === "serialize")) {
      const [x, y] = location(c);
      const from = c.id === "c1" ? [x - 115, y + 42] : [x - 115, y + 27], to = c.id === "c1" ? [x - 3, y + 4] : [x + 75, y + 27];
      drawing += '<g class="clash-sequence" data-clash-effect="' + c.id + '-serialize"><path class="clash-after" marker-end="url(#clashArrow)" d="' + path([from, to]) + '"/><text class="clash-plant-note" x="' + (x - 20) + '" y="' + (y + 47) + '">' + (c.id === "c1" ? "payout order → breaker" : "traceparent → payout_state") + '</text><text class="clash-plant-note clash-pin-wait" x="' + (x - 20) + '" y="' + (y + 61) + '">after · pin waits for upstream revision</text></g>';
      effectRects.push({ left: x - 120, right: x + 90, top: y + 20, bottom: y + 70 });
    }
    if (state.settled.c2 === "merge") {
      const entities = [...new Set(SCENE.territories.filter((t) => ["paypal-payouts", "radar-otel"].includes(t.id)).flatMap((t) => t.entities))].map((id) => ENT[id]);
      const xs = entities.map((e) => e.x), ys = entities.map((e) => e.y);
      const x0 = Math.min(...xs) - 45, x1 = Math.max(...xs) + 45, y0 = Math.min(...ys) - 45, y1 = Math.max(...ys) + 45;
      drawing += '<path class="clash-merged" d="' + path([[x0,y0],[x1,y0],[x1,y1],[x0,y1],[x0,y0]].map(([x,y]) => point(x,y,1))) + '"/>';
    }
    labels.innerHTML = badges.filter((b) => inView(b.at)).map((b) => '<span class="clash-foot-label" data-unclaimed="' + b.unclaimed + '" style="--agent:' + b.color + '" title="' + escape(b.title || b.text) + '">' + escape(b.text) + '</span>').join("");
    const visibleBadges = badges.filter((b) => inView(b.at));
    const reserved = [...document.querySelectorAll(".chip,.clash-pin,.clash-pin-label,#clashOrder,#clashCard,.fl,.cbar,#clashToolbar")].filter((e) => !e.hidden && getComputedStyle(e).display !== "none" && +getComputedStyle(e).opacity > 0).map((e) => e.getBoundingClientRect());
    reserved.push(...effectRects);
    const overlaps = (r) => reserved.some((q) => r.left < q.right + 6 && r.right > q.left - 6 && r.top < q.bottom + 6 && r.bottom > q.top - 6);
    [...labels.children].forEach((el, i) => {
      const b = visibleBadges[i], [x, y] = b.at, w = el.offsetWidth, h = el.offsetHeight;
      const candidates = [];
      for (let ty = 188; ty < innerHeight - h - 75; ty += h + 10) {
        for (let tx = 24; tx < innerWidth - w - 24; tx += 80) candidates.push({ left: tx, top: ty, right: tx + w, bottom: ty + h });
      }
      candidates.sort((a, b) => Math.hypot(a.left + w / 2 - x, a.top - y) - Math.hypot(b.left + w / 2 - x, b.top - y));
      const spot = candidates.find((r) => !overlaps(r));
      if (!spot) { el.hidden = true; return; }
      const { left, top } = spot;
      el.style.left = left + "px"; el.style.top = top + "px";
      reserved.push({ left, top, right: left + w, bottom: top + h });
      drawing += '<path class="clash-label-leader" stroke="' + b.color + '" d="' + path([[x, y], [clamp(x, left, left + w), top > y ? top : top + h]]) + '"/>';
    });
    const writes = declarations.reduce((n, d) => n + d.writes.length, 0);
    $("#clashWriteCount").textContent = state.footprints ? writes + " writes · " + [...labels.children].filter((el) => !el.hidden).length + " shown" : "";
    svg.innerHTML = '<defs><marker id="clashArrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="5" markerHeight="5" orient="auto-start-reverse"><path d="M0 0L10 5L0 10" fill="#c9a45c"/></marker></defs>' + drawing;
    kvRender();
  }
  const toggleFootprints = () => { state.footprints = !state.footprints; paint(); };
  $("#clashFootprints").onclick = () => { if (conceptActive("clash")) toggleFootprints(); };
  $("#clashNext").onclick = () => { if (conceptActive("clash")) next(); };
  $("#clashUndo").onclick = () => { if (conceptActive("clash")) undoSettlement(); };
  pins.onclick = (ev) => { if (!conceptActive("clash")) return; const pin = ev.target.closest("[data-clash-pin]"); if (pin) open(pin.dataset.clashPin); };
  card.onclick = (ev) => {
    if (!conceptActive("clash")) return;
    if (ev.target.closest("[data-clash-close]")) close();
    const button = ev.target.closest("[data-clash-resolution]");
    if (button) settle(button.dataset.clashResolution);
  };
  window.clashKeydown = (ev) => {
    if (!conceptActive("clash")) return;
    if (typing(ev)) {
      if (ev.key === "Escape" && state.open) { ev.preventDefault(); ev.stopImmediatePropagation(); ev.target.blur(); }
      return;
    }
    const consume = () => { ev.preventDefault(); ev.stopImmediatePropagation(); };
    const key = ev.key.toLowerCase();
    if (bare(ev) && ev.key === "Escape" && provKey) { consume(); provClose(); return; }
    if (bare(ev) && ev.key === "Escape" && kv.panel) { consume(); kvPanelToggle(false); return; }
    if (bare(ev) && key === "u") { consume(); kvPanelToggle(); return; }
    if (kv.panel && bare(ev) && !["tab", "enter", "escape"].includes(key)) { consume(); return; }
    if (state.open && (ev.ctrlKey || ev.metaKey) && key !== "z") { consume(); return; }
    if (bare(ev) && ev.key === "Escape") {
      if ($("#pal").classList.contains("open")) { consume(); palClose(); return; }
      if ($("#layers").classList.contains("open")) { consume(); $("#layers").classList.remove("open"); return; }
    }
    if ((ev.ctrlKey || ev.metaKey) && !ev.altKey && !ev.shiftKey && ev.key.toLowerCase() === "z") {
      ev.preventDefault(); ev.stopImmediatePropagation(); undoSettlement(); return;
    }
    if (!bare(ev)) return;
    if (ev.key === "Tab" && state.open) {
      const items = [...card.querySelectorAll("button,summary,[tabindex]")].filter((e) => e.getClientRects().length);
      const i = items.indexOf(document.activeElement);
      consume(); items[(i + (ev.shiftKey ? -1 : 1) + items.length) % items.length].focus(); return;
    }
    if (ev.key === "Tab" && ev.target.closest("#clashLayer")) { ev.stopImmediatePropagation(); return; }
    if (state.open && ev.key === "Escape") { ev.preventDefault(); ev.stopImmediatePropagation(); close(); return; }
    if (state.open && ev.shiftKey) {
      const digit = ev.code.replace("Digit", ""), c = blocking.find((c) => c.id === state.open), r = c.resolutions.find((r) => r.key === digit);
      if (r) { ev.preventDefault(); ev.stopImmediatePropagation(); settle(r.kind); return; }
    }
    if (ev.key === "Enter" && ev.target.closest("[data-clash-pin]")) {
      ev.preventDefault(); ev.stopImmediatePropagation(); open(ev.target.closest("[data-clash-pin]").dataset.clashPin); return;
    }
    if (key === "v" || key === "i") {
      ev.preventDefault(); ev.stopImmediatePropagation();
      if (key === "v") toggleFootprints(); else next();
      return;
    }
    if (state.open && ev.target.closest(".clash-declaration-body") && ev.key.startsWith("Arrow")) { ev.stopImmediatePropagation(); return; }
    if (state.open && (/^[a-z1-4]$/.test(key) || ev.key.startsWith("Arrow") || ev.key === " ")) consume();
  };
  card.addEventListener("toggle", (ev) => {
    if (ev.target.open) for (const details of card.querySelectorAll("details")) if (details !== ev.target) details.open = false;
    paint();
  }, true);
  for (const r of K.reads) kv.log.push({ op: r.op, reason: r.reason + " · cached " + K.cachedAgeS + " s", bytes: r.bytes, destination: K.host, state: "sent", count: 1 });
  kv.bytes = K.reads.reduce((n, r) => n + r.bytes, 0);
  kv.lastReturn = 0;
  $("#clashKvPanel").onclick = (ev) => {
    if (!conceptActive("clash")) return;
    if (ev.target.closest("[data-clash-panel-close]")) kvPanelToggle(false);
    const act = ev.target.closest("[data-clash-kv]");
    if (act) act.dataset.clashKv === "reconnect" ? kvReconnect() : kvSendHeld();
  };
  $("#clashProv").onclick = (ev) => { if (ev.target.closest("[data-clash-prov-close]")) provClose(); };
  const clashEgress = $(".egress");
  if (clashEgress) {
    clashEgress.addEventListener("click", () => { if (conceptActive("clash")) kvPanelToggle(); });
    clashEgress.addEventListener("keydown", (ev) => { if (conceptActive("clash") && (ev.key === "Enter" || ev.key === " ")) { ev.preventDefault(); kvPanelToggle(); } });
  }
  document.addEventListener("pointerover", (ev) => {
    if (!conceptActive("clash") || !document.body.classList.contains("alt")) return;
    const t = ev.target.closest("[data-clash-binding]");
    if (t && !t.closest("#clashProv")) provOpen(t.dataset.clashBinding, t);
  });
  document.addEventListener("focusin", (ev) => {
    if (!conceptActive("clash") || !document.body.classList.contains("alt")) return;
    const t = ev.target.closest("[data-clash-binding]");
    if (t && !t.closest("#clashProv")) provOpen(t.dataset.clashBinding, t);
  });
  addEventListener("keyup", (ev) => { if (ev.key === "Alt") provClose(); });
  addEventListener("blur", () => provClose());
  window.clashProv = { open: provOpen, close: provClose };
  let kvLastFrame = null;
  const kvTick = (now) => {
    const delta = kvLastFrame === null ? 0 : Math.min(100, now - kvLastFrame);
    kvLastFrame = now;
    if (conceptActive("clash")) {
      kvMs += delta;
      const due = kvInflight.filter((f) => f.due <= kvMs);
      if (due.length) {
        kvInflight = kvInflight.filter((f) => f.due > kvMs);
        for (const f of due) { if (f.row) { f.row.state = "sent"; kv.lastReturn = kvMs; } if (f.done) f.done(f.row); }
        kvRender(); paint();
      }
    }
    requestAnimationFrame(kvTick);
  };
  requestAnimationFrame(kvTick);
  kvRender();
  const observer = new MutationObserver(paint);
  observer.observe(world, { attributes: true, attributeFilter: ["style"] });
  observer.observe(plant, { childList: true });
  new MutationObserver(() => { if (document.body.dataset.mode !== "live" && state.open) close(); }).observe(document.body, { attributes: true, attributeFilter: ["data-mode"] });
  addEventListener("resize", paint);
  layer.hidden = false;
  paint();
}

export default {
  id: "clash",
  label: "Clash on the belt",
  hint: "V footprints · I next clash · Shift 1-4 settle · Ctrl Z undo · U Kvarm · Alt hover provenance · Esc close",
  css: `
body:not([data-concept=clash]) #clashLayer{display:none}
body[data-concept=clash] *,body[data-concept=clash] *::before,body[data-concept=clash] *::after{transition:none!important;animation:none!important}
body[data-concept=clash] .pins{display:none!important}
body[data-concept=clash] .hint{opacity:1!important;z-index:45;background:#101110;border:1px solid var(--line);padding:5px 12px;border-radius:6px;top:62px}
body[data-concept=clash] .cbar{left:16px;transform:none;top:96px}
body[data-concept=clash] .wait b{margin-left:0}
body[data-concept=clash] #clashWaiting{font:10px var(--mono);margin-left:auto;color:#d4b67f;white-space:nowrap}
body[data-concept=clash]:not([data-mode=live]) #clashLayer{display:none}
body[data-concept=clash] .chip[data-id=bot-cc]{opacity:1;translate:0 0}
body[data-concept=clash] .chip[data-id=breaker]{display:none}
body[data-concept=clash] .ent[data-id=clash-port]{pointer-events:none}
body[data-concept=clash].clash-breaker-wins .ent[data-id=bot-cc] .drone-lamp circle{fill:var(--wait)}
body[data-concept=clash] .toast{left:16px;top:139px;bottom:auto;transform:none;max-width:360px}
#clashLayer{position:fixed;inset:0;pointer-events:none;z-index:8}
#clashLayer [hidden]{display:none!important}
#clashLayer button:focus-visible{outline:2px solid #c9a45c;outline-offset:4px}
#clashDrawing{position:absolute;inset:0;width:100%;height:100%;overflow:visible}
#clashToolbar{position:absolute;right:16px;top:96px;display:flex;align-items:center;gap:5px;padding:3px 7px 3px 13px;border:1px solid var(--line2);background:#121312;border-radius:9px;pointer-events:auto;z-index:2}
#clashCount{font:11px var(--mono);color:#dfbf84;margin-right:10px;white-space:nowrap}
#clashCount[data-done=true]{color:var(--ok)}
#clashToolbar button{padding:5px 8px;border-radius:5px;color:#aaa;font-size:11px;white-space:nowrap}
#clashToolbar button:hover,#clashToolbar button[aria-pressed=true]{background:#ffffff0c;color:#eee}
#clashToolbar button:disabled{opacity:.3;cursor:default}
#clashWriteCount{font:10px var(--mono);color:#aaa;white-space:nowrap}
#clashAdvisory{position:absolute;left:16px;bottom:235px;max-width:330px;font:10px/1.5 var(--mono);color:#929992;background:#101211e8;border:1px dashed #85858540;padding:7px 10px;border-radius:6px}
.clash-pin{position:absolute;display:flex;flex-direction:column;align-items:center;transform:translate(-50%,-100%);pointer-events:auto;min-width:34px}
.clash-pin-head{display:grid;place-items:center;width:26px;height:26px;border-radius:50%;background:#c9a45c;color:#18140d;box-shadow:0 0 0 4px #13110e;font:600 19px var(--mono)}
.clash-pin-stem{width:1px;height:25px;background:#c9a45c}
.clash-pin-label{position:absolute;top:0;left:calc(100% + 4px);font:10px var(--mono);white-space:nowrap;color:#d4b67f;padding:4px 6px;background:#15130f;border:1px solid #c9a45c40;border-radius:5px}
.clash-pin .clash-pin-label{left:auto;right:calc(100% + 4px)}
.clash-pin[data-advisory=true]:not([data-settled=true]) .clash-pin-head{background:#272a25;color:#b0b8a3;border:1px dashed #abb79a}
.clash-pin[data-advisory=true]:not([data-settled=true]) .clash-pin-label{border-style:dashed;color:#a9b49e;background:#131512}
.clash-pin[data-settled=true] .clash-pin-head{width:18px;height:18px;font-size:11px;background:#17211e;border:1px solid var(--ok);color:var(--ok);box-shadow:0 0 0 3px #101510}
.clash-pin[data-settled=true] .clash-pin-stem{height:15px;background:#35b7a270}
.clash-pin[data-settled=true] .clash-pin-label{color:#a4d3c7;border-color:#35b7a250;font-size:11px;padding:4px 7px;background:#101d19}
.clash-order{position:absolute;width:318px;padding:12px 14px;border:1px solid #c9a45c4a;border-radius:9px;background:rgba(23,21,16,.94);box-shadow:var(--shadow);pointer-events:auto}
.clash-order-flag{font:10px var(--mono);color:#d8b776;letter-spacing:.04em}
.clash-order p{font-size:12px;line-height:1.5;color:#d7d1c5;margin:7px 0 10px}
.clash-checks{display:flex;align-items:center;gap:10px;font:10px var(--mono);color:var(--ok)}
.clash-checks .clash-budget{margin-left:auto;font-size:9px;color:#b5afa4}
.clash-order>small{display:block;margin-top:7px;font:9px var(--mono);color:#8c8271}
.clash-card{position:absolute;width:560px;max-width:calc(100vw - 48px);padding:14px;border:1px solid #c9a45c60;border-radius:13px;background:#161715;box-shadow:var(--shadow);pointer-events:auto;max-height:calc(100vh - 244px);overflow:auto;scrollbar-width:thin;scrollbar-color:#555 transparent}
.clash-card-head{display:flex;align-items:center;gap:10px;margin-bottom:10px}
.clash-icon{display:grid;place-items:center;width:32px;height:32px;background:#c9a45c;color:#18140d;border-radius:8px;font:20px var(--mono)}
.clash-card h2{font-size:17px;font-weight:550;margin:0;color:#eee}
.clash-card-head small{display:block;font:10px var(--mono);margin-top:3px;color:#999}
.clash-card-head>button{display:flex;align-items:center;gap:7px;margin-left:auto;padding:5px;color:#aaa;font-size:18px}
.clash-pair{display:grid;gap:5px;margin:10px 0 14px}
.clash-declaration{min-width:0;border:1px solid var(--line2);border-left:2px solid var(--agent);border-radius:6px;background:#ffffff03}
.clash-declaration[data-unclaimed=true]{border-style:dashed}
.clash-declaration summary{display:flex;align-items:center;gap:8px;padding:8px;cursor:pointer;list-style:none;font-size:11px}
.clash-declaration summary::before{content:"›";color:var(--agent)}
.clash-declaration[open] summary::before{content:"⌄"}
.clash-declaration summary b{white-space:nowrap;overflow:hidden;text-overflow:ellipsis;max-width:220px;color:#ddd;font-weight:500}
.clash-declaration summary span{margin-left:auto;white-space:nowrap;color:var(--agent);font-size:10px}
.clash-declaration summary small{font:9px var(--mono);white-space:nowrap;color:#a5a99e}
.clash-declaration-body{padding:0 10px 10px;max-height:96px;overflow:auto;scrollbar-width:thin}
.clash-declaration header{font:10px var(--mono);color:#aaa}
.clash-declaration p{margin:7px 0;font-size:12px;line-height:1.45;color:#d0d0d0}
.clash-writes>div{border-top:1px solid var(--line);padding:5px 0 0;margin-top:5px}
.clash-writes b{display:block;font:10px var(--mono);color:#bbb;margin:0 0 3px}
.clash-writes code{display:block;font:9px/1.5 var(--mono);overflow-wrap:anywhere;color:#8e8e8e}
.clash-writes small{display:block;font-size:10px;color:#aaa;margin-top:3px}
.clash-source{float:right;font:9px var(--mono);color:var(--agent);padding:1px 4px;border:1px solid color-mix(in srgb,var(--agent) 30%,transparent);border-radius:3px}
.clash-consequence{padding:12px;border:1px solid #e8604a60;border-left:3px solid var(--fail);background:#e8604a10;border-radius:7px}
.clash-consequence>small{font:9px var(--mono);letter-spacing:.08em;color:#efa190}
.clash-consequence p{font-size:15px;line-height:1.5;color:#f1c2b6;margin:6px 0 0}
.clash-breaks{font:10px/1.5 var(--mono);margin-top:9px;color:#efb3a1;padding-top:8px;border-top:1px solid #e8604a30}
.clash-breaks b{font-weight:500}
.clash-breaks span{display:block;color:#c9b4a9;margin-top:2px}
.clash-card[data-advisory=true] .clash-consequence{border-color:#a9b49e45;background:#a9b49e07}
.clash-card[data-advisory=true] .clash-consequence>small,.clash-card[data-advisory=true] .clash-consequence p{color:#bdc6b1}
.clash-history-note{font-size:10px;color:#9ca392;margin-top:8px}
.clash-why{font-size:11px;line-height:1.5;color:#b9b4a7;margin:0 0 9px}
.clash-why b{font-weight:500;color:#dfbf84}
.clash-resolutions h3{display:flex;align-items:center;gap:10px;font-size:12px;font-weight:500;margin:0 0 7px}
.clash-resolutions h3 small{font:9px var(--mono);color:#888;font-weight:400}
.clash-resolutions button{display:flex;gap:10px;width:100%;text-align:left;padding:8px 10px;border:1px solid var(--line);border-radius:7px;margin-top:5px;background:#ffffff03}
.clash-resolutions button[data-suggested=true]{border-color:#c9a45c65;background:#c9a45c0b}
.clash-resolutions button[aria-pressed=true]{background:#19372c;border-color:#50bba0}
.clash-resolutions button:hover{background:#ffffff0a;border-color:#aaa}
.clash-resolutions button[aria-pressed=true]:hover{background:#19372c;border-color:#50bba0}
.clash-resolutions button>kbd{align-self:flex-start;flex:none;color:#aaa;margin-top:1px;line-height:17px}
.clash-resolutions b{display:block;font-size:12px;font-weight:500;color:#e5e5e5}
.clash-resolutions .clash-why b{display:inline;font-size:11px;color:#dfbf84}
.clash-card:has(.clash-declaration[open]) .clash-resolutions button small{display:none}
.clash-resolutions small{display:block;font-size:10.5px;line-height:1.45;color:#929292;margin-top:3px}
.clash-resolutions i,.clash-resolutions em{font:9px var(--mono);font-style:normal;color:#c9a45c;margin-left:7px}
.clash-resolutions em{color:var(--ok)}
.clash-footprint path{stroke-width:1;fill:none;stroke-dasharray:4 3;vector-effect:non-scaling-stroke}
.clash-footprint .clash-belt-outline{stroke-width:6;stroke-opacity:.35}
.clash-footprint[data-unclaimed=true] path{stroke-dasharray:2 7}
.clash-foot-label[data-unclaimed=true]{border-style:dashed;color:#9cae9e}
.clash-advisory{fill:none;stroke:#858585;stroke-width:1;opacity:.24;stroke-dasharray:3 6}
.clash-layer-ring{fill:none;stroke:#c9a45c;stroke-width:1;opacity:.6}
.clash-layer-ring[data-advisory=true]{stroke:#a3b49a;stroke-dasharray:3 5;opacity:.4}
.clash-merged{fill:#64bbae06;stroke:#64bbae;stroke-width:1.5;stroke-dasharray:8 4}
.clash-plant-note{font:10px var(--mono);fill:#d3bd8e;text-anchor:middle;paint-order:stroke;stroke:#101210;stroke-width:5px;stroke-linejoin:round}
.clash-card-leader{stroke:#c9a45c90;stroke-width:1;fill:none}
.clash-label-leader{stroke-width:.7;opacity:.4;fill:none}
.clash-foot-label{position:absolute;white-space:nowrap;font:9.5px var(--mono);color:var(--agent);padding:4px 7px;background:rgba(17,18,17,.96);border:1px solid color-mix(in srgb,var(--agent) 35%,transparent);border-radius:4px}
.clash-port rect{fill:#111b22;stroke:var(--ghost);stroke-dasharray:4 3}
.clash-port text{fill:#c0dced;font:10px var(--mono);text-anchor:middle}
.clash-port text.clash-port-owner{fill:#bba67b;font-size:9px}
.clash-port path,.clash-after{stroke:#c9a45c;stroke-width:1;fill:none}
.clash-kv{border-top:1px solid var(--line);margin-top:8px;padding-top:6px;font:9px/1.5 var(--mono);color:#8f897e}
.clash-kv header{display:flex;gap:6px;color:#9f9583;font:9px var(--mono)}
.clash-kv header b{margin-left:auto;font-weight:400;color:#c9a45c}
.clash-kv header b[data-tone=wait]{color:var(--wait)}
.clash-kv-row{display:grid;grid-template-columns:auto minmax(0,1fr) auto;gap:8px;white-space:nowrap}
.clash-kv-row b{font-weight:400;color:#c8bfae}
.clash-kv-row span{overflow:hidden;text-overflow:ellipsis;color:#7f796f}
.clash-kv-row i{font-style:normal;color:#9f9583}
.clash-kv-row[data-state=held] b,.clash-kv-row[data-state=held] i{color:var(--wait)}
.clash-kv-row[data-state=pending] i{color:var(--wait)}
.clash-kv-row[data-state=superseded] i{color:#9b917f;text-decoration:line-through}
.clash-forecast{margin-top:6px;padding-top:5px;border-top:1px dashed #444;font:9px var(--mono);color:#a39a86}
.clash-forecast[data-truncated=true]{color:var(--wait)}
.clash-kv-panel{position:absolute;right:16px;top:139px;width:380px;z-index:46;max-height:calc(100vh - 220px)}
.clash-prov{position:fixed;z-index:47;width:390px}
.clash-prov-dl{display:grid;grid-template-columns:86px 1fr;gap:4px 10px;margin:0;font:10px/1.5 var(--mono)}
.clash-prov-dl dt{color:#888}
.clash-prov-dl dd{margin:0;color:#ccc;overflow-wrap:anywhere}
.clash-kv-note{font-size:11px;line-height:1.5;color:#999;margin:0 0 10px}
.clash-kv-status{font:10px var(--mono);color:#c9a45c;min-height:22px;display:flex;align-items:center;gap:6px}
.clash-kv-status[data-tone=wait]{color:var(--wait)}
.clash-kv-primary{padding:8px 13px;border-radius:7px;background:#c9a45c;color:#14110b;font:500 12px var(--sans);margin:6px 0 10px}
.clash-kv-section{font:9px var(--mono);letter-spacing:.07em;text-transform:uppercase;color:#999;margin:10px 0 4px}
.clash-kv-panel .clash-kv-row{padding:3px 0;border-top:1px solid var(--line)}
.clash-kv-panel .clash-kv-row span{white-space:normal}
.clash-chip-sub{display:block;font:9px var(--mono);color:#9a9a9a;letter-spacing:0;text-transform:none}
.clash-pin[data-settled=true] .clash-pin-label[data-decision]::after{content:"\A decision " attr(data-decision);white-space:pre;display:block;font-size:8.5px;color:#8fbfae}
.clash-pin[data-settled=true] .clash-pin-label[data-decision=pending]::after{color:var(--wait)}
.clash-pin-wait{font-size:8.5px;fill:#a89a7d}
.egress{cursor:pointer}
.egress:hover{color:var(--text)}
.egress[data-kv=offline] .dot,.egress[data-kv=checked] .dot{background:var(--wait)}
.clash-prov [data-evidence-state=unknown]{color:#9a9a9a}
.clash-prov [data-evidence-state=stale]{color:#9b917f;text-decoration:underline dashed #9b917f80;text-underline-offset:3px}
`.replace(/^([.#][^{]+)\{/gm, (_, selectors) => selectors.split(",").map((s) => "body[data-concept=clash] " + s.trim()).join(",") + "{"),
  html: `<script>addEventListener("keydown", (ev) => { if (document.body.dataset.concept === "clash" && window.clashKeydown) window.clashKeydown(ev); }, { capture: true });</script><div id="clashLayer" hidden><svg id="clashDrawing" aria-hidden="true"></svg><div id="clashToolbar"><span id="clashCount" role="status"></span><button id="clashFootprints" aria-pressed="false">Footprints</button><span id="clashWriteCount"></span><button id="clashNext">Next clash</button><button id="clashUndo">Undo</button></div><div id="clashPins"></div><div id="clashLabels"></div><aside id="clashAdvisory"></aside><article class="clash-order" id="clashOrder" aria-label="Claimed payout order"></article><section class="clash-card" id="clashCard" aria-label="Settle clash" hidden></section><section class="clash-card clash-kv-panel" id="clashKvPanel" role="dialog" aria-label="Kvarm connection" hidden></section><section class="clash-card clash-prov" id="clashProv" role="dialog" aria-label="Provenance" hidden></section></div>`,
  script: `function clashInit(params) { (${clashMount.toString()})(${JSON.stringify({ declarations: DECLARATIONS, clashes: CLASHES, witnesses: [...WITNESSES, QUESTION.options[0].witness], order: ORDER, kv: { origin: KVARM.origin, host: KVARM.host, repo: KVARM.repo, staleAfter: KVARM.staleAfter, ...KVARM.clash } })}); }`,
};
