import { BENCH, BENCH_WITNESSES, ATTEMPTS, RECORDED, EVENTS, EXTENSION, KVARM_OPS } from "./bench-data.mjs";

function benchMount(D) {
  if (!conceptActive("bench")) return;
  const { bench, witnesses, attempts, recorded, events, extension, ops } = D;
  const AT = Object.fromEntries(attempts.map((a) => [a.id, a]));
  const WT = Object.fromEntries(witnesses.map((w) => [w.id, w]));
  const WIDS = witnesses.map((w) => w.id);
  const state = window.benchState = {
    phase: "framing", view: null, focus: null, question: bench.question,
    attempts: Object.fromEntries(attempts.map((a) => [a.id, { status: "queued", replayed: 0, results: Object.fromEntries(WIDS.map((w) => [w, "waiting"])), seen: {}, tokens: 0, minutes: 0, budget: { tokens: bench.budget.tokens, minutes: bench.budget.minutes }, extended: false }])),
    pick: { draft: null, chosen: null, reason: "" },
    kvarm: { ops: [], bytes: 0 },
    order: { rev: bench.order.rev, design: null },
    clash: { c1: null },
  };
  const layer = $("#benchLayer"), card = $("#benchCard"), svg = $("#benchDrawing"), pins = $("#benchPins"), plates = $("#benchPlates"), plate = $("#benchOrder");
  const esc = (v) => String(v).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
  const clamp = (n, a, b) => Math.max(a, Math.min(b, n));
  const kfmt = (n) => (n >= 1000 ? Math.round(n / 1000) + "k" : String(n));
  const bytes = (n) => n.toLocaleString("en-US") + " B";
  const path = (pts) => pts.map((p, i) => (i ? "L" : "M") + p.map((v) => v.toFixed(1)).join(" ")).join("");
  const toScreen = (p) => { const r = world.getBoundingClientRect(), s = r.width / W; return [r.left + p[0] * s, r.top + p[1] * s]; };
  const point = (x, y, z = 0) => toScreen(k.iso([x, y, z], P));
  const anchor = (id) => toScreen(anchors[id]);
  const inView = ([x, y]) => x > 16 && x < innerWidth - 16 && y > 120 && y < innerHeight - 60;
  const live = () => document.body.dataset.mode === "live";
  const where = (w) => {
    if (w.entity !== "sqlite") return anchor(w.entity);
    const e = ENT.sqlite, s = e.size || 1;
    return point(e.x + 21 * s, e.y - 4 * s, 26 * s);
  };
  const polarity = (w) => (w.polarity === "must-not" ? "must not" : "must keep");
  const replayedCount = () => attempts.reduce((n, a) => n + state.attempts[a.id].replayed, 0);
  const stopped = (s) => ["held", "disproved", "spent"].includes(s.status);
  const word = { waiting: "waiting", holds: "holds", breaks: "breaks", unreplayed: "not replayed" };
  const statusText = (id) => {
    const s = state.attempts[id];
    if (state.pick.chosen === id) return "picked by you";
    if (s.status === "queued") return "not funded";
    if (s.status === "disproved") return "disproved by " + WIDS.find((w) => s.results[w] === "breaks");
    if (s.status === "held") return "held " + s.replayed + " of " + WIDS.length;
    if (s.status === "spent") return "budget spent · " + s.replayed + " of " + WIDS.length;
    return s.replayed ? s.replayed + " of " + WIDS.length + " replayed" : "no evidence yet";
  };
  const statusKind = (id) => (state.pick.chosen === id ? "picked" : state.attempts[id].status);
  const blocker = (id) => {
    const s = state.attempts[id];
    if (s.status === "disproved") return "Disproved. It cannot be picked.";
    if (s.status === "spent") return WIDS.filter((w) => s.results[w] === "unreplayed").join(", ") + " not replayed. Extend it or pick another.";
    if (s.status !== "held") return "Still replaying. Pick when every witness is replayed.";
    return "";
  };
  const rejectReason = (id) => {
    const s = state.attempts[id];
    if (s.status === "disproved") { const w = WIDS.find((w) => s.results[w] === "breaks"); return "disproved by " + w + " · " + s.seen[w]; }
    if (s.status === "spent") return WIDS.filter((w) => s.results[w] === "unreplayed").join(", ") + " not replayed · budget spent";
    return "held " + s.replayed + " of " + WIDS.length + " · not picked";
  };
  const evidenceLine = (id) => "Held " + WIDS.join(", ") + " on sealed replay in " + bench.id + ".";

  ASSETS.benchZone = {
    family: "floor", label: "Bench", maps: "sealed replay of attempts, not live", w: 10, d: 10, top: 0,
    build(c, { zone }) {
      const ring = [[zone.x0, zone.y0], [zone.x1, zone.y0], [zone.x1, zone.y1], [zone.x0, zone.y1], [zone.x0, zone.y0]];
      const fill = ring.map((p, i) => (i ? "L" : "M") + c.at([p[0], p[1], 0]).map((v) => v.toFixed(1)).join(" ")).join("") + "Z";
      c.floor('<path class="bench-zone-fill" d="' + fill + '"/>' + c.L(c.k.onTop(ring, 0.2, c.P), { tone: "mid", dashed: true }) + c.text(zone.x0 + 3, zone.y1 + 7, 0, "BENCH · SEALED REPLAY · NOT LIVE", { size: 4.2, tone: "mid" }));
    },
  };
  ASSETS.benchRig = {
    family: "floor", label: "Bench rig", maps: "one attempt replayed against the witnesses", w: 64, d: 48, top: 16,
    build(c, { x, y, approach, tag, lamps }) {
      c.plinth(x, y, 64, 48, 3);
      c.floor(c.L(c.k.lineOnTop([x - 28, y - 3], [x + 17, y - 3], 3.1, c.P), { tone: "mid" }));
      c.put([x + 23, y - 3, 8], c.box(x + 18, y - 11, 10, 16, 3, 11, { tone: "mid" }, 2, 0.4));
      if (approach !== "retry") ASSETS.breaker.build(c, { x: x - 15, y: y - 3 });
      if (approach === "split") c.put([x - 4, y - 3, 6], c.box(x - 5.5, y - 9, 3, 12, 3, 8, { tone: "hi" }, 2, 0.3));
      ASSETS.retry.build(c, { x: x + 7, y: y - 3 });
      WIDS.forEach((w, i) => { const lx = x - 14 + i * 12, ly = y + 15; c.put([lx, ly, 4], '<g class="bench-lamp" data-r="' + lamps[w] + '">' + c.lamp(lx, ly, 3) + "</g>"); });
      c.floor(c.text(x - 29, y + 20, 3, tag, { size: 5.5, tone: "hi" }));
    },
  };
  const zone = bench.zone;
  const zoneEnt = { id: "bench-zone", kind: "benchZone", x: (zone.x0 + zone.x1) / 2, y: (zone.y0 + zone.y1) / 2, state: "bench", name: "bench", mono: "BN", reading: "sealed replay", sub: "not live", params: { zone } };
  SCENE.entities.push(zoneEnt); ENT[zoneEnt.id] = zoneEnt;
  const rigs = attempts.map((a) => ({ id: "bench-" + a.id, kind: "benchRig", x: a.x, y: a.y, state: "bench", name: a.label, mono: a.tag, reading: "sealed replay", sub: a.agent, params: { approach: a.approach, tag: a.tag, lamps: state.attempts[a.id].results } }));
  const syncRigs = () => {
    for (const r of rigs) {
      const id = r.id.slice(6), s = state.attempts[id];
      r.state = state.pick.chosen === id ? "benchPicked" : s.status === "disproved" ? "benchOut" : "bench";
      r.params.lamps = s.results;
      const on = state.phase !== "framing";
      if (on && !ENT[r.id]) { SCENE.entities.push(r); ENT[r.id] = r; }
    }
    renderPlant();
  };

  const territory = SCENE.territories.find((t) => t.id === bench.order.territory);
  Object.assign(territory, { state: "claimed", fog: false, label: "PAYPAL-PAYOUTS · CLAIMED · CLAUDE CODE" });
  if (TERR[bench.order.territory]) Object.assign(TERR[bench.order.territory], { state: "claimed", owner: bench.order.claimedBy, fog: null });
  if (FOGD.paypal) { Object.keys(FOGD.paypal.s).forEach((key) => (FOGD.paypal.s[key] = 1)); FOGD.paypal.why = ""; }

  const agentRow = $(".bl .agent.off");
  const agentText = [...agentRow.childNodes].find((n) => n.nodeType === Node.TEXT_NODE && n.textContent.trim());
  $(".wait").insertAdjacentHTML("beforeend", '<small id="benchWaiting"></small>');
  const egress = $(".egress");

  const timers = [];
  const later = (ms, fn) => timers.push(setTimeout(() => { if (conceptActive("bench")) fn(); }, ms));
  const log = (list) => {
    for (const op of list) { state.kvarm.ops.push(op.op); state.kvarm.bytes += op.bytes; }
    egress.innerHTML = '<span class="dot"></span>kvarm · ' + bytes(state.kvarm.bytes) + " out";
    egress.title = "Outbound to " + bench.kvarm.origin + ": " + state.kvarm.ops.join(", ");
  };
  const opRows = () => {
    const all = [...ops.fund, ...ops.results, ...ops.pick].filter((op) => state.kvarm.ops.includes(op.op));
    return all;
  };

  const announce = (text) => { toast(text); $("#benchLive").textContent = text; };

  const frame = (instant) => {
    const pts = [];
    for (const id of ["sqlite", "breaker", "retry", "paypal", "models"]) {
      const e = ENT[id], a = ASSETS[e.kind], s = e.size || 1;
      for (const sx of [-1, 1]) for (const sy of [-1, 1]) pts.push(k.iso([e.x + (sx * a.w * s) / 2, e.y + (sy * a.d * s) / 2, 0], P));
      pts.push(anchors[id]);
    }
    for (const [x, y] of [[zone.x0, zone.y0], [zone.x1, zone.y0], [zone.x1, zone.y1], [zone.x0, zone.y1]]) pts.push(k.iso([x, y, 0], P));
    const xs = pts.map((p) => p[0]), ys = pts.map((p) => p[1]);
    const L = 520, R = innerWidth - 300, T = 230, B = innerHeight - 110;
    const fitS = Math.min(innerWidth / W, innerHeight / H) * 1.02;
    const s = Math.min(fitS, (R - L) / (Math.max(...xs) - Math.min(...xs)), (B - T) / (Math.max(...ys) - Math.min(...ys)));
    goal.s = s;
    goal.x = L + (R - L - s * (Math.max(...xs) - Math.min(...xs))) / 2 - s * Math.min(...xs);
    goal.y = T + (B - T - s * (Math.max(...ys) - Math.min(...ys))) / 2 - s * Math.min(...ys);
    if (instant) { view.x = goal.x; view.y = goal.y; view.s = goal.s; apply(); } else animate();
  };
  const framed = () => {
    const [sx, sy] = where(WT.w3), [rx, ry] = anchor("paypal"), [zx, zy] = point(zone.x1, zone.y1);
    return sx > 500 && rx < innerWidth - 40 && zy < innerHeight - 60 && sy > 160 && zx < innerWidth;
  };

  const cardOpen = () => !card.hidden;
  const open = (focusId) => {
    if (!live()) return;
    select(null);
    state.view = state.phase === "framing" ? "brief" : "results";
    if (!framed()) frame(false);
    card.hidden = false;
    render();
    const target = focusId && card.querySelector(focusId) || card.querySelector("[data-bench-fund],[data-bench-record]:not(:disabled)") || card.querySelector("[data-bench-close]");
    target.focus({ preventScroll: true });
  };
  const close = () => {
    if (!cardOpen()) return;
    card.hidden = true; state.view = null; state.pick.draft = null;
    paint();
    $("#benchZoneLabel").focus({ preventScroll: true });
  };

  const fund = () => {
    if (state.phase !== "framing" || !live()) return;
    const q = card.querySelector("#benchQuestion");
    if (q) state.question = q.value.trim() || bench.question;
    state.phase = "running";
    for (const a of attempts) state.attempts[a.id].status = "running";
    log(ops.fund);
    syncRigs();
    state.view = "results";
    render();
    card.querySelector("[data-bench-close]").focus({ preventScroll: true });
    announce("Bench funded. Three attempts replay against w1, w2 and w3. Results land on the bench.");
    for (const ev of events) later(ev.at, () => land(ev));
  };
  const land = (ev) => {
    const s = state.attempts[ev.attempt];
    s.tokens = ev.tokens; s.minutes = ev.minute;
    if (ev.witness) { s.results = { ...s.results, [ev.witness]: ev.result }; s.seen[ev.witness] = ev.seen; }
    if (ev.spent) s.results = Object.fromEntries(WIDS.map((w) => [w, s.results[w] === "waiting" ? "unreplayed" : s.results[w]]));
    s.replayed = WIDS.filter((w) => ["holds", "breaks"].includes(s.results[w])).length;
    s.status = ev.stop || "running";
    const a = AT[ev.attempt];
    if (ev.stop === "disproved") announce(a.label + " is disproved. " + ev.witness + " breaks on replay: " + ev.seen + ".");
    if (ev.stop === "held") announce(a.label + " held " + s.replayed + " of " + WIDS.length + ". Not disproved.");
    if (ev.stop === "spent") announce(a.label + " spent its " + kfmt(s.budget.tokens) + " tokens before " + WIDS.filter((w) => s.results[w] === "unreplayed").join(", ") + ". Extend with U, or pick another.");
    if (attempts.every((x) => stopped(state.attempts[x.id]))) {
      if (state.phase === "running" && !state.kvarm.ops.includes("rank_arena")) log(ops.results);
      state.phase = "results";
    }
    syncRigs();
    if (cardOpen()) render(); else paint();
  };
  const extend = () => {
    const id = attempts.map((a) => a.id).find((id) => state.attempts[id].status === "spent" && !state.attempts[id].extended);
    if (!id) return toast(state.phase === "framing" ? "Nothing to extend. Fund the bench first." : "No attempt is out of budget.");
    if (state.pick.chosen) return toast(bench.id + " already has a pick. Extending would not change it.");
    const s = state.attempts[id];
    s.extended = true;
    s.budget = { tokens: s.budget.tokens + bench.budget.extend.tokens, minutes: s.budget.minutes + bench.budget.extend.minutes };
    s.results = Object.fromEntries(WIDS.map((w) => [w, s.results[w] === "unreplayed" ? "waiting" : s.results[w]]));
    s.status = "running"; state.phase = "running";
    if (state.pick.draft === id) state.pick.draft = null;
    syncRigs();
    if (cardOpen()) render(); else paint();
    announce("Extended " + AT[id].label + " by " + kfmt(bench.budget.extend.tokens) + " tokens and " + bench.budget.extend.minutes + " min. Replaying " + WIDS.filter((w) => s.results[w] === "waiting").join(", ") + ".");
    later(extension.at, () => land(extension));
  };
  const draft = (id) => {
    if (!AT[id] || !live()) return;
    if (state.pick.chosen) return toast(bench.id + " already has a pick. Kvarm keeps one winner per arena.");
    const why = blocker(id);
    if (why) return toast(AT[id].label + ": " + why);
    if (!cardOpen()) open();
    state.pick.draft = id;
    render();
    card.querySelector("[data-bench-record]").focus({ preventScroll: true });
  };
  const record = () => {
    const id = state.pick.draft;
    if (!id || state.pick.chosen || blocker(id)) return;
    const field = card.querySelector("#benchReason");
    state.pick.reason = (field && field.value.trim()) || evidenceLine(id);
    state.pick.chosen = id; state.pick.draft = null;
    log(ops.pick);
    state.order = { rev: bench.order.next, design: id };
    state.clash.c1 = AT[id].approach === "split" ? "split" : "left";
    state.phase = "recorded";
    syncRigs();
    render();
    plate.classList.remove("bench-ink"); void plate.offsetWidth; plate.classList.add("bench-ink");
    plate.onanimationend = () => plate.classList.remove("bench-ink");
    card.querySelector("[data-bench-close]").focus({ preventScroll: true });
    announce("Pick recorded. " + bench.order.id + " is now " + bench.order.next + " with " + AT[id].label + ". " + bench.order.claimedBy + " builds from " + AT[id].workspace + ".");
  };
  const focusWitness = (w) => {
    state.focus = w;
    if (cardOpen()) render(); else paint();
  };
  const nextWitness = () => {
    const i = WIDS.indexOf(state.focus);
    const w = WIDS[(i + 1) % WIDS.length];
    if (!cardOpen()) open();
    focusWitness(w);
    const b = card.querySelector('[data-bench-witness="' + w + '"]');
    if (b) b.focus({ preventScroll: true });
  };

  const witnessChip = (w) => '<span class="bench-pol" data-polarity="' + w.polarity + '">' + esc(w.id) + " · " + polarity(w) + " · " + esc(w.clock) + "</span>";
  const head = (title, sub, icon) => '<header class="bench-head"><span class="bench-icon">' + icon + '</span><div><h2>' + esc(title) + "</h2><small>" + sub + '</small></div><button data-bench-close aria-label="Close the bench">× <kbd>Esc</kbd></button></header>';
  const briefHtml = () => {
    let h = head("Bench · Retry meets breaker", "experiment before build · " + esc(bench.sheet), "◇");
    h += '<label class="bench-label" for="benchQuestion">Question</label><textarea id="benchQuestion" rows="1" spellcheck="false">' + esc(state.question) + "</textarea>";
    h += '<section class="bench-sec bench-wrong"><h3>Wrong if <small>locked · agents cannot edit it (K16)</small></h3><p>' + esc(bench.wrongIf) + "</p>";
    h += witnesses.map((w) => '<button class="bench-clause" data-bench-witness="' + w.id + '" aria-pressed="' + (state.focus === w.id) + '">' + witnessChip(w) + "<span>" + esc(w.entity) + "</span><em>" + esc(bench.clauses[w.id]) + "</em></button>").join("");
    h += '<small class="bench-note">' + esc(bench.notProof) + "</small></section>";
    h += '<section class="bench-sec"><h3>On the bench <small>' + attempts.length + " attempts · one approach each</small></h3>";
    h += attempts.map((a) => '<div class="bench-arow" title="' + esc(a.says) + '"><b>' + a.tag + "</b><span>" + esc(a.label) + "</span><i>" + esc(a.agent) + " · " + esc(a.diff) + "</i></div>").join("");
    h += '<small class="bench-note">' + esc(bench.cannotRun.agent) + " cannot run an attempt. " + esc(bench.cannotRun.why) + "</small></section>";
    h += '<section class="bench-sec"><h3>Budget <small>an attempt stops when it runs out</small></h3><div class="bench-budget"><span><b>' + bench.budget.minutes + " min</b>per attempt</span><span><b>" + kfmt(bench.budget.tokens) + "</b>tokens per attempt</span><span><b>" + bench.budget.review + " min</b>your review, estimate</span></div><small class=\"bench-note\">" + esc(bench.environment) + "</small></section>";
    h += '<footer class="bench-foot"><button class="bench-primary" data-bench-fund>Fund the bench <kbd>Enter</kbd></button><small>Kvarm keeps the arena. Orrery keeps the criterion, budget and replay.</small></footer>';
    return h;
  };
  const resultsHtml = () => {
    const total = attempts.length * WIDS.length;
    const sub = (state.phase === "recorded" ? "picked" : state.phase === "results" ? "results in" : "funded · replaying") + " · arena " + bench.kvarm.arena + " · " + replayedCount() + " of " + total + " replayed";
    let h = head(bench.id + " · Retry meets breaker", sub, state.phase === "recorded" ? "✓" : "◇");
    if (!state.pick.chosen && !state.pick.draft) h += '<p class="bench-q">' + esc(state.question) + "</p>";
    h += '<p class="bench-wrongline"><b>Wrong if</b> ' + esc(bench.wrongIf.replace(/^An attempt is wrong if /, "")) + " <small>locked</small></p>";
    h += '<table class="bench-grid"><thead><tr><th></th>' + attempts.map((a) => '<th data-attempt="' + a.id + '" data-s="' + statusKind(a.id) + '"><b>' + a.tag + " · " + esc(a.short) + "</b><small>" + esc(a.agent) + "</small></th>").join("") + "</tr></thead><tbody>";
    for (const w of witnesses) {
      const f = state.focus === w.id;
      h += '<tr class="bench-wrow" data-w="' + w.id + '" aria-selected="' + f + '"><th><button data-bench-witness="' + w.id + '" aria-pressed="' + f + '">' + witnessChip(w) + "</button></th>" + attempts.map((a) => '<td data-r="' + state.attempts[a.id].results[w.id] + '">' + word[state.attempts[a.id].results[w.id]] + "</td>").join("") + "</tr>";
      if (f && !state.pick.chosen && !state.pick.draft) h += '<tr class="bench-trace"><th><small>recorded</small><span>' + esc(recorded[w.id]) + "</span></th>" + attempts.map((a) => { const s = state.attempts[a.id]; return '<td data-r="' + s.results[w.id] + '">' + esc(s.seen[w.id] || (s.results[w.id] === "unreplayed" ? "budget spent before this moment" : "no evidence yet")) + "</td>"; }).join("") + "</tr>";
    }
    h += "</tbody><tfoot>";
    h += '<tr class="bench-status"><th>status</th>' + attempts.map((a) => '<td data-s="' + statusKind(a.id) + '">' + statusText(a.id) + "</td>").join("") + "</tr>";
    h += '<tr class="bench-spend"><th>spend</th>' + attempts.map((a) => { const s = state.attempts[a.id], pct = Math.min(100, (s.tokens / s.budget.tokens) * 100), base = Math.min(100, (bench.budget.tokens / s.budget.tokens) * 100); return '<td><div class="bench-bar" data-full="' + (s.tokens >= s.budget.tokens) + '"><i style="width:' + pct.toFixed(1) + '%"></i>' + (s.extended ? '<u style="left:' + base.toFixed(1) + '%"></u>' : "") + "</div><small>" + kfmt(s.tokens) + " / " + kfmt(s.budget.tokens) + " tok · " + s.minutes + " / " + s.budget.minutes + " min</small></td>"; }).join("") + "</tr>";
    h += '<tr class="bench-diff"><th>diff</th>' + attempts.map((a) => "<td>" + (state.attempts[a.id].status === "queued" ? "" : esc(a.diff)) + "</td>").join("") + "</tr>";
    if (!state.pick.chosen && !state.pick.draft) h += '<tr class="bench-act"><th></th>' + attempts.map((a) => {
      const s = state.attempts[a.id];
      if (s.status === "spent" && !s.extended) return '<td><button data-bench-extend="' + a.id + '"><kbd>U</kbd> Extend +' + kfmt(bench.budget.extend.tokens) + "</button></td>";
      if (s.status === "disproved") return '<td><span class="bench-cant">cannot be picked</span></td>';
      return '<td><button data-bench-pick="' + a.id + '" aria-pressed="' + (state.pick.draft === a.id) + '"' + (s.status === "held" ? "" : " disabled") + "><kbd>⇧" + a.key + "</kbd> Pick</button></td>";
    }).join("") + "</tr>";
    h += "</tfoot></table>";
    if (state.pick.draft) {
      const a = AT[state.pick.draft];
      h += '<section class="bench-sec bench-pick"><h3>Pick ' + a.tag + " · " + esc(a.label) + ' <small>only you pick · agents never grade</small></h3>';
      h += '<label class="bench-label" for="benchReason">Your reason <small>optional · blank sends the evidence line</small></label><textarea id="benchReason" rows="1" placeholder="' + esc(evidenceLine(a.id)) + '"></textarea>';
      h += '<div class="bench-rejects">' + attempts.filter((x) => x.id !== a.id).map((x) => "<span><b>" + x.tag + "</b> " + esc(rejectReason(x.id)) + "</span>").join("") + "</div>";
      h += '<footer class="bench-foot"><button class="bench-primary" data-bench-record>Record the pick <kbd>Enter</kbd></button><small>Writes ' + esc(bench.kvarm.choice) + " and three Kvarm ops as you.</small></footer></section>";
    }
    if (state.pick.chosen) {
      const a = AT[state.pick.chosen];
      h += '<section class="bench-sec bench-done"><h3>Picked ' + a.tag + " · " + esc(a.label) + " <small>by you · " + esc(bench.kvarm.choice) + "</small></h3><p>" + esc(bench.order.id) + " is now " + bench.order.next + ". " + esc(bench.order.claimedBy) + " builds from " + esc(a.workspace) + ". Clash c1 is settled by " + bench.id + ".</p><small class=\"bench-note\">Reason · " + esc(state.pick.reason) + " · Final selection confirmed by you; the other workspaces stay open in Kvarm until you abandon them.</small></section>";
    }
    const rows = opRows();
    h += '<section class="bench-kvarm"><h3>Kvarm <small>' + esc(bench.kvarm.origin) + " · " + bytes(state.kvarm.bytes) + " out</small></h3>" + rows.map((op) => '<div class="bench-op"><b>' + op.op + "</b><span>" + bytes(op.bytes) + "</span><small>" + esc(op.note ? op.note : op.reason) + "</small></div>").join("") + "</section>";
    return h;
  };
  const render = () => {
    if (!cardOpen()) return paint();
    const focused = document.activeElement && card.contains(document.activeElement) ? document.activeElement : null;
    const key = focused && ["data-bench-witness", "data-bench-pick", "data-bench-extend", "data-bench-record", "data-bench-close", "data-bench-fund", "id"].map((a) => focused.getAttribute(a) && "[" + a + '="' + focused.getAttribute(a) + '"]').find(Boolean);
    const reason = card.querySelector("#benchReason");
    const keepReason = reason ? reason.value : "";
    const q = card.querySelector("#benchQuestion");
    if (q) state.question = q.value;
    card.dataset.view = state.view = state.phase === "framing" ? "brief" : "results";
    card.innerHTML = state.view === "brief" ? briefHtml() : resultsHtml();
    const r2 = card.querySelector("#benchReason"); if (r2) r2.value = keepReason;
    if (key) { const again = card.querySelector(key); if (again) again.focus({ preventScroll: true }); }
    paint();
  };

  function paint() {
    if (!conceptActive("bench")) return;
    const kinds = attempts.map((a) => statusKind(a.id));
    document.body.classList.toggle("bench-open", cardOpen());
    document.body.classList.toggle("bench-retry-picked", state.clash.c1 === "left");
    $("#benchWaiting").textContent = state.phase === "framing" ? "bench · frame" : state.phase === "running" ? "bench · replaying" : state.phase === "results" ? "bench · your pick" : "";
    agentText.textContent = state.pick.chosen ? " builds from " + AT[state.pick.chosen].workspace : state.phase === "framing" ? " claimed payout order" : " " + attempts.filter((a) => a.agent === "Claude Code").length + " attempts on the bench";
    agentRow.classList.remove("off");
    agentRow.querySelector("span").textContent = state.phase === "framing" ? "before code" : "bench";
    let drawing = "";
    const reserved = [...document.querySelectorAll(".chip,.fl,.cbar,.nav,#benchCard")].filter((e) => !e.hidden && e.getClientRects().length && +getComputedStyle(e).opacity > 0.01).map((e) => e.getBoundingClientRect());
    const overlaps = (r) => reserved.some((q) => r.left < q.right + 4 && r.right > q.left - 4 && r.top < q.bottom + 4 && r.bottom > q.top - 4);

    const zl = $("#benchZoneLabel"), [zx, zy] = point(zone.x0 - 6, zone.y1 + 6, 0);
    zl.hidden = cardOpen() || !inView([zx, zy]);
    zl.innerHTML = state.phase === "framing" ? '<b>Bench</b><span>empty · frame an experiment</span><kbd>P</kbd>' : "<b>" + bench.id + "</b><span>" + replayedCount() + " of " + attempts.length * WIDS.length + " replayed · " + (state.phase === "recorded" ? "picked" : state.phase === "results" ? "your pick" : "replaying") + "</span><kbd>P</kbd>";
    zl.dataset.phase = state.phase;
    zl.style.left = zx + "px"; zl.style.top = zy + "px";
    reserved.push(zl.getBoundingClientRect());

    const placedPlates = [];
    for (const a of attempts) {
      const el = plates.querySelector('[data-bench-plate="' + a.id + '"]');
      const ent = ENT["bench-" + a.id];
      el.hidden = !ent || !anchors[ent.id];
      if (el.hidden) continue;
      const [x, y] = anchor(ent.id), s = state.attempts[a.id];
      el.dataset.s = statusKind(a.id);
      el.innerHTML = "<b>" + a.tag + " · " + esc(a.label) + "</b><span>" + esc(a.agent) + '</span><em>' + statusText(a.id) + '</em><span class="bench-lamps">' + WIDS.map((w) => '<i data-r="' + s.results[w] + '" title="' + w + " " + word[s.results[w]] + '">' + w + "</i>").join("") + "</span>";
      el.hidden = false;
      const pw = el.offsetWidth, ph = el.offsetHeight;
      const left = clamp(x - pw / 2, 8, innerWidth - pw - 8);
      let top = clamp(y - ph - 20, 130, innerHeight - ph - 70);
      const box = () => ({ left, top, right: left + pw, bottom: top + ph });
      while (placedPlates.some((q) => { const r = box(); return r.left < q.right + 4 && r.right > q.left - 4 && r.top < q.bottom + 4 && r.bottom > q.top - 4; })) top += 14;
      placedPlates.push(box());
      el.style.left = left + "px"; el.style.top = top + "px";
      el.hidden = !inView([x, y]);
      if (!el.hidden) drawing += '<path class="bench-tick" d="' + path([[x, y - 4], [left + pw / 2, top + ph + 2]]) + '"/>';
      reserved.push(el.getBoundingClientRect());
    }

    const c1 = $("#benchClashPin"), [bx, by] = anchor("breaker");
    c1.style.left = bx + "px"; c1.style.top = by + "px";
    c1.hidden = !inView([bx, by]);
    c1.dataset.settled = !!state.clash.c1;
    c1.querySelector(".bench-pin-head").textContent = state.clash.c1 ? "✓" : "◇";
    c1.querySelector(".bench-pin-label").innerHTML = state.clash.c1 ? (state.clash.c1 === "split" ? "split at the port" : "retry wins") + " · " + bench.id : state.phase === "framing" ? "Retry meets breaker · bench it <kbd>P</kbd>" : "Retry meets breaker · on " + bench.id;
    reserved.push(c1.getBoundingClientRect(), c1.querySelector(".bench-pin-label").getBoundingClientRect());

    for (const w of witnesses) {
      const pin = pins.querySelector('[data-bench-pin="' + w.id + '"]'), [x, y] = where(w);
      pin.style.left = x + "px"; pin.style.top = y + "px";
      pin.hidden = !inView([x, y]);
      pin.setAttribute("aria-pressed", state.focus === w.id);
      const label = pin.querySelector(".bench-pin-label");
      label.innerHTML = "<b>" + w.id + " · " + polarity(w) + " · " + w.clock + "</b>" + (state.phase === "framing" ? "" : '<span class="bench-cells">' + attempts.map((a) => '<i data-r="' + state.attempts[a.id].results[w.id] + '">' + a.tag + "</i>").join("") + "</span>");
      pin.dataset.side = "right";
      if (overlaps(label.getBoundingClientRect())) pin.dataset.side = "left";
      reserved.push(pin.querySelector(".bench-pin-head").getBoundingClientRect(), label.getBoundingClientRect());
    }

    const [px, py] = anchor(bench.order.on);
    plate.hidden = !inView([px, py]);
    if (!plate.hidden) {
      const a = state.order.design && AT[state.order.design];
      plate.innerHTML = '<span class="bench-flag">CLAIMED · ' + esc(bench.order.claimedBy) + " · " + state.order.rev + "</span><p>" + esc(bench.order.sentence) + '</p><div class="bench-checks">' + WIDS.map((id) => "<span>✓ " + id + "</span>").join("") + "<em>" + esc(bench.order.budget) + "</em></div>" + (a ? '<div class="bench-design" data-set="true"><b>design</b> ' + esc(a.label) + " · from " + bench.id + " · builds from " + esc(a.workspace) + "</div>" : '<div class="bench-design"><b>design</b> open · ' + (state.phase === "framing" ? "c1 unsettled" : bench.id + " " + (state.phase === "running" ? "replaying" : "waits for your pick")) + "</div>");
      const width = plate.offsetWidth, height = plate.offsetHeight;
      const left = clamp(px + 60, 500, innerWidth - width - 20), top = clamp(py + 70, 150, innerHeight - height - 72);
      plate.style.left = left + "px"; plate.style.top = top + "px";
      drawing += '<path class="bench-leader" d="' + path([[px, py], [clamp(px, left, left + width), top]]) + '"/>';
    }

    if (cardOpen()) {
      const r = card.getBoundingClientRect(), target = state.phase === "framing" ? [bx, by] : [zx, zy];
      if (inView(target)) drawing += '<path class="bench-leader" d="' + path([[r.right, clamp(target[1], r.top + 24, r.bottom - 24)], target]) + '"/>';
    }

    if (state.clash.c1 === "split") {
      const p = point(126, -20, 7);
      drawing += '<g class="bench-ghost"><rect x="' + (p[0] - 74) + '" y="' + (p[1] + 18) + '" width="148" height="40" rx="6"/><text x="' + p[0] + '" y="' + (p[1] + 34) + '">paypal.result v1</text><text class="bench-ghost-sub" x="' + p[0] + '" y="' + (p[1] + 50) + '">ghost · from ' + bench.id + "</text></g>";
    }
    if (state.clash.c1 === "left") {
      const [rx, ry] = anchor("retry");
      drawing += '<g class="bench-ghost"><text x="' + rx + '" y="' + (ry + 40) + '">retry x2 · jitter · ghost</text><text class="bench-ghost-sub" x="' + bx + '" y="' + (by + 36) + '">breaker not built</text></g>';
    }
    svg.innerHTML = drawing;
    $("#benchStatus").textContent = kinds.join(" ");
  }

  pins.innerHTML = witnesses.map((w) => '<button class="bench-pin bench-wpin" data-bench-pin="' + w.id + '" data-polarity="' + w.polarity + '" aria-label="Witness ' + w.id + '"><span class="bench-pin-head"></span><span class="bench-pin-stem"></span><span class="bench-pin-label"></span></button>').join("") + '<button class="bench-pin bench-cpin" id="benchClashPin" aria-label="Clash c1, retry meets breaker"><span class="bench-pin-head">◇</span><span class="bench-pin-stem"></span><span class="bench-pin-label"></span></button>';
  plates.innerHTML = '<button id="benchZoneLabel" class="bench-zone-label" aria-label="Open the bench"></button>' + attempts.map((a) => '<button class="bench-plate" data-bench-plate="' + a.id + '" hidden></button>').join("");

  layer.addEventListener("click", (ev) => {
    if (!conceptActive("bench")) return;
    const t = ev.target;
    if (t.closest("[data-bench-close]")) return close();
    if (t.closest("[data-bench-fund]")) return fund();
    if (t.closest("[data-bench-record]")) return record();
    const pick = t.closest("[data-bench-pick]"); if (pick) return draft(pick.dataset.benchPick);
    if (t.closest("[data-bench-extend]")) return extend();
    const w = t.closest("[data-bench-witness]"); if (w) return focusWitness(state.focus === w.dataset.benchWitness ? null : w.dataset.benchWitness);
    const pin = t.closest("[data-bench-pin]"); if (pin) { if (!cardOpen()) open(); return focusWitness(pin.dataset.benchPin); }
    const pl = t.closest("[data-bench-plate]"); if (pl) return cardOpen() ? null : open();
    if (t.closest("#benchZoneLabel,#benchClashPin")) return cardOpen() ? close() : open();
  });

  window.benchKeydown = (ev) => {
    if (!conceptActive("bench")) return;
    const consume = () => { ev.preventDefault(); ev.stopImmediatePropagation(); };
    const inCard = ev.target && ev.target.closest && ev.target.closest("#benchLayer");
    if (typing(ev)) {
      if (!inCard) return;
      if (ev.key === "Escape") { consume(); ev.target.blur(); card.focus({ preventScroll: true }); return; }
      if (ev.key === "Enter" && !ev.shiftKey) { consume(); if (ev.target.id === "benchQuestion") fund(); else record(); }
      return;
    }
    if (ev.key === "Escape") {
      if ($("#pal").classList.contains("open") || $("#layers").classList.contains("open")) return;
      if (state.pick.draft) { consume(); state.pick.draft = null; render(); card.querySelector("[data-bench-close]").focus({ preventScroll: true }); return; }
      if (cardOpen()) { consume(); close(); }
      return;
    }
    if (!bare(ev)) { if (cardOpen() && (ev.ctrlKey || ev.metaKey)) consume(); return; }
    if (cardOpen() && ev.key === "Tab") {
      const items = [...card.querySelectorAll("button:not(:disabled),textarea")];
      const i = items.indexOf(document.activeElement);
      consume(); items[(i + (ev.shiftKey ? -1 : 1) + items.length) % items.length].focus({ preventScroll: true }); return;
    }
    if (ev.key === "Enter") {
      if (inCard && ev.target.tagName === "BUTTON") { ev.stopImmediatePropagation(); return; }
      if (cardOpen()) { consume(); if (state.view === "brief") fund(); else if (state.pick.draft) record(); }
      return;
    }
    if (ev.key === " " && inCard && ev.target.tagName === "BUTTON") { ev.stopImmediatePropagation(); return; }
    const key = ev.key.toLowerCase();
    if (ev.shiftKey && /^Digit[1-9]$/.test(ev.code)) {
      if (!cardOpen() || state.view !== "results") return;
      const a = attempts.find((a) => a.key === ev.code.slice(5));
      consume(); if (a) draft(a.id); return;
    }
    if (key === "p" && !ev.shiftKey) { consume(); if (cardOpen()) close(); else open(); return; }
    if (key === "i" && !ev.shiftKey) { consume(); nextWitness(); return; }
    if (key === "u" && !ev.shiftKey) { consume(); extend(); return; }
    if (cardOpen() && (/^[a-z0-9]$/.test(key) || ev.key.startsWith("Arrow") || ev.key === " ")) consume();
  };

  const observer = new MutationObserver(paint);
  observer.observe(world, { attributes: true, attributeFilter: ["style"] });
  observer.observe(plant, { childList: true });
  new MutationObserver(() => { if (!live() && cardOpen()) close(); paint(); }).observe(document.body, { attributes: true, attributeFilter: ["data-mode"] });
  addEventListener("resize", () => { frame(true); paint(); });
  layer.hidden = false;
  renderPlant();
  frame(true);
  paint();
}

export default {
  id: "bench",
  label: "The bench",
  hint: "P bench · Enter fund · I next witness · U extend budget · Shift 1-3 pick · Esc close",
  css: `
body:not([data-concept=bench]) #benchLayer{display:none}
body[data-concept=bench] *:not(.bench-ink),body[data-concept=bench] *::before,body[data-concept=bench] *::after{transition:none!important;animation:none!important}
body[data-concept=bench] .pins{display:none!important}
body[data-concept=bench] .hint{opacity:1!important;z-index:45;background:#101110;border:1px solid var(--line);padding:5px 12px;border-radius:6px;top:62px}
body[data-concept=bench] .chip[data-id=breaker]{display:none}
body[data-concept=bench]:not([data-mode=live]) #benchLayer{display:none}
body[data-concept=bench]:not([data-mode=live]) .ent[data-kind=benchRig],body[data-concept=bench]:not([data-mode=live]) .ent[data-kind=benchZone]{display:none}
body[data-concept=bench] .ent[data-kind=benchRig],body[data-concept=bench] .ent[data-kind=benchZone]{pointer-events:none}
body[data-concept=bench] .ent[data-kind=benchRig] .iso-top{fill:url(#benchGraph)}
body[data-concept=bench] .ent[data-kind=benchRig] .iso-line{stroke:#7f9a8a}
body[data-concept=bench] .ent[data-kind=benchRig] .iso-face-text{fill:#b9cfc0}
body[data-concept=bench] .ent[data-kind=benchRig][data-state=benchOut]{opacity:.5}
body[data-concept=bench] .ent[data-kind=benchRig][data-state=benchPicked] .iso-line{stroke:#c9a45c}
body[data-concept=bench] .bench-lamp .iso-top{fill:#1c1f1d}
body[data-concept=bench] .bench-lamp[data-r=holds] .iso-top{fill:var(--ok)}
body[data-concept=bench] .bench-lamp[data-r=breaks] .iso-top{fill:var(--fail)}
body[data-concept=bench] .bench-lamp[data-r=unreplayed] .iso-top{fill:url(#benchUnknown)}
body[data-concept=bench] .bench-zone-fill{fill:#8fb8a00a}
body[data-concept=bench].bench-retry-picked .ent[data-id=breaker]{opacity:.35}
body[data-concept=bench] .wait b{margin-left:0}
body[data-concept=bench] #benchWaiting{font:10px var(--mono);margin-left:auto;color:#d4b67f;white-space:nowrap}
body[data-concept=bench] .toast{left:50%;top:128px;bottom:auto;transform:translateX(-50%);max-width:560px;z-index:46}
#benchLayer{position:fixed;inset:0;pointer-events:none;z-index:8}
#benchLayer [hidden]{display:none!important}
#benchLayer button:focus-visible,#benchLayer textarea:focus-visible{outline:2px solid #c9a45c;outline-offset:2px}
#benchDrawing{position:absolute;inset:0;width:100%;height:100%;overflow:visible}
.bench-leader{stroke:#c9a45c80;stroke-width:1;fill:none}
.bench-tick{stroke:#8fb8a050;stroke-width:1;fill:none;stroke-dasharray:2 3}
.bench-ghost rect{fill:#111b22;stroke:var(--ghost);stroke-dasharray:4 3}
.bench-ghost text{fill:#c0dced;font:10px var(--mono);text-anchor:middle;paint-order:stroke;stroke:#0d0f10;stroke-width:4px}
.bench-ghost .bench-ghost-sub{fill:#bba67b;font-size:9px}
.bench-pin{position:absolute;display:flex;flex-direction:column;align-items:center;transform:translate(-50%,-100%);pointer-events:auto;min-width:20px}
.bench-pin-head{display:grid;place-items:center;width:14px;height:14px;border-radius:50%;box-shadow:0 0 0 3px #0e0e0e;font:600 13px var(--mono)}
.bench-pin-stem{width:1px;height:22px}
.bench-wpin[data-polarity=must-not] .bench-pin-head{background:var(--fail)}
.bench-wpin[data-polarity=must-not] .bench-pin-stem{background:var(--fail)}
.bench-wpin[data-polarity=must-keep] .bench-pin-head{background:var(--ok)}
.bench-wpin[data-polarity=must-keep] .bench-pin-stem{background:var(--ok)}
.bench-pin-label{position:absolute;top:-5px;left:calc(50% + 12px);display:flex;flex-direction:column;gap:4px;font:10px var(--mono);white-space:nowrap;color:#ddd;padding:4px 6px;background:#121212f2;border:1px solid var(--line2);border-radius:5px;text-align:left}
.bench-pin[data-side=left] .bench-pin-label{left:auto;right:calc(50% + 12px)}
.bench-wpin[data-polarity=must-not] .bench-pin-label b{color:#f09a89;font-weight:500;letter-spacing:.04em;text-transform:uppercase}
.bench-wpin[data-polarity=must-keep] .bench-pin-label b{color:#7fd3c3;font-weight:500;letter-spacing:.04em;text-transform:uppercase}
.bench-wpin[aria-pressed=true] .bench-pin-label{border-color:#c9a45c;background:#19160f}
.bench-cells{display:flex;gap:3px}
.bench-cells i,.bench-lamps i{font-style:normal;font:9px var(--mono);padding:1px 4px;border-radius:3px;border:1px solid #3a3a3a;color:#8a8a8a}
.bench-cells i[data-r=holds],.bench-lamps i[data-r=holds]{border-color:#35b7a270;color:#8fdccd;background:#35b7a214}
.bench-cells i[data-r=breaks],.bench-lamps i[data-r=breaks]{border-color:#e8604a90;color:#f6b2a5;background:#e8604a22}
.bench-cells i[data-r=unreplayed],.bench-lamps i[data-r=unreplayed]{border-style:dashed;color:#d4b67f;border-color:#c9a45c70;background:repeating-linear-gradient(135deg,#c9a45c18 0 2px,transparent 2px 5px)}
.bench-cpin .bench-pin-head{width:24px;height:24px;background:#c9a45c;color:#18140d;font-size:17px}
.bench-cpin .bench-pin-stem{height:22px;background:#c9a45c}
.bench-cpin .bench-pin-label{left:auto;right:calc(50% + 16px);top:0;color:#d4b67f;border-color:#c9a45c50;background:#15130f;flex-direction:row;align-items:center;gap:6px}
.bench-cpin .bench-pin-label kbd{color:#c9a45c;border-color:#c9a45c60}
.bench-cpin[data-settled=true] .bench-pin-head{width:18px;height:18px;font-size:11px;background:#17211e;border:1px solid var(--ok);color:var(--ok)}
.bench-cpin[data-settled=true] .bench-pin-label{color:#a4d3c7;border-color:#35b7a250;background:#101d19}
.bench-zone-label{position:absolute;transform:translate(-50%,-130%);pointer-events:auto;display:flex;align-items:center;gap:8px;padding:5px 8px;border:1px dashed #8fb8a070;border-radius:6px;background:#0f1311f0;font:10px var(--mono);color:#aac2b4;white-space:nowrap}
.bench-zone-label b{color:#d7e6dc;font-weight:500;letter-spacing:.06em;text-transform:uppercase}
.bench-zone-label kbd{color:#aac2b4}
.bench-plate{position:absolute;pointer-events:auto;display:grid;grid-template-columns:auto auto;gap:2px 10px;align-items:baseline;padding:6px 8px;border:1px dashed #8fb8a060;border-radius:6px;background:#0f1311f2;text-align:left;font:10px var(--mono);color:#aac2b4;white-space:nowrap}
.bench-plate b{grid-column:1/-1;font:500 11px var(--sans);color:#e2ece5}
.bench-plate span{color:#8fa597}
.bench-plate em{font-style:normal;justify-self:end;color:#cfd8d2}
.bench-plate .bench-lamps{grid-column:1/-1;display:flex;gap:3px}
.bench-plate[data-s=held] em{color:var(--ok)}
.bench-plate[data-s=disproved]{border-color:#e8604a70;border-style:solid}
.bench-plate[data-s=disproved] em{color:var(--fail)}
.bench-plate[data-s=disproved] b{text-decoration:line-through;text-decoration-color:#e8604a;color:#b9b0ad}
.bench-plate[data-s=spent]{border-color:#c9a45c80}
.bench-plate[data-s=spent] em{color:#e1b86a}
.bench-plate[data-s=picked]{border:1px solid #c9a45c;background:#19160ff2}
.bench-plate[data-s=picked] em{color:#e6cb96}
.bench-order{position:absolute;width:300px;padding:10px 12px;border:1px solid #c9a45c4a;border-radius:9px;background:rgba(23,21,16,.95);box-shadow:var(--shadow);pointer-events:auto}
.bench-flag{font:10px var(--mono);color:#d8b776;letter-spacing:.04em}
.bench-order p{font-size:11.5px;line-height:1.45;color:#d7d1c5;margin:6px 0 8px}
.bench-checks{display:flex;align-items:center;gap:9px;font:10px var(--mono);color:var(--ok)}
.bench-checks em{margin-left:auto;font-style:normal;font-size:9px;color:#b5afa4}
.bench-design{margin-top:8px;padding-top:7px;border-top:1px solid #c9a45c25;font:10px/1.5 var(--mono);color:#9c927f}
.bench-design b{font-weight:500;color:#c9a45c;margin-right:4px}
.bench-design[data-set=true]{color:#e6cb96}
@keyframes bench-ink{from{clip-path:inset(100% 0 0)}to{clip-path:inset(0)}}
.bench-ink{animation:bench-ink .5s ease-out!important}
.bench-card{position:absolute;left:24px;top:108px;width:466px;max-height:calc(100vh - 300px);overflow:auto;padding:12px 16px;border:1px solid #c9a45c60;border-radius:13px;background:#161715;box-shadow:var(--shadow);pointer-events:auto;scrollbar-width:thin;scrollbar-color:#555 transparent}
.bench-card:focus{outline:none}
.bench-head{display:flex;align-items:center;gap:10px;margin-bottom:8px}
.bench-icon{display:grid;place-items:center;width:26px;height:26px;background:#c9a45c;color:#18140d;border-radius:7px;font:16px var(--mono);flex:none}
.bench-head h2{font-size:14px;font-weight:550;margin:0;color:#eee}
.bench-head small{display:block;font:10px var(--mono);margin-top:3px;color:#999}
.bench-head>button{display:flex;align-items:center;gap:7px;margin-left:auto;align-self:flex-start;padding:4px;color:#aaa;font-size:16px}
.bench-label{display:flex;gap:8px;align-items:baseline;font:10px var(--mono);letter-spacing:.07em;text-transform:uppercase;color:#999;margin:0 0 5px}
.bench-label small{letter-spacing:0;text-transform:none;color:#777}
.bench-card textarea{width:100%;resize:none;border-radius:7px;background:#101010;border:1px solid #3d3d3d;color:#eee;padding:8px 10px;font:13px/1.45 var(--sans);outline:none;display:block}
.bench-card textarea:focus{border-color:#c9a45c;box-shadow:0 0 0 2px #c9a45c22}
.bench-card textarea::placeholder{color:#777}
.bench-sec{margin-top:9px}
.bench-sec h3,.bench-kvarm h3{display:flex;align-items:baseline;gap:8px;font:500 10px var(--mono);letter-spacing:.1em;text-transform:uppercase;color:#c9a45c;margin:0 0 5px}
.bench-sec h3 small,.bench-kvarm h3 small{letter-spacing:0;text-transform:none;color:#888;font-weight:400}
.bench-wrong{padding:8px 12px;border:1px solid #e8604a50;border-left:3px solid var(--fail);border-radius:7px;background:#e8604a0c}
.bench-wrong h3{color:#efa190}
.bench-wrong>p{margin:0 0 6px;font-size:13px;line-height:1.45;color:#f1d0c7}
.bench-clause{display:grid;grid-template-columns:150px 1fr;gap:1px 8px;width:100%;text-align:left;padding:4px 6px;margin:0 -6px;border-radius:5px;font-size:11.5px;color:#d8d0cd}
.bench-clause:hover,.bench-clause[aria-pressed=true]{background:#ffffff0a}
.bench-clause>span:not(.bench-pol){display:none}
.bench-clause em{font-style:normal}
.bench-pol{font:10px var(--mono);letter-spacing:.03em;white-space:nowrap}
.bench-pol[data-polarity=must-not]{color:#f09a89}
.bench-pol[data-polarity=must-keep]{color:#7fd3c3}
.bench-note{display:block;font:10px/1.45 var(--mono);color:#8e8e8e;margin-top:4px}
.bench-arow{display:grid;grid-template-columns:24px 1fr auto;gap:1px 8px;padding:4px 0;border-top:1px solid var(--line);font-size:12px;color:#ddd}
.bench-arow:first-of-type{border-top:0}
.bench-arow b{font:600 10px var(--mono);color:#aac2b4;padding-top:1px}
.bench-arow i{font-style:normal;font:10px var(--mono);color:#999}
.bench-budget{display:grid;grid-template-columns:repeat(3,1fr);gap:8px}
.bench-budget span{display:flex;flex-direction:column;gap:2px;padding:6px 8px;border:1px solid var(--line2);border-radius:6px;font:10px var(--mono);color:#8e8e8e}
.bench-budget b{font:15px var(--mono);color:#eee;font-weight:400}
.bench-foot{display:flex;flex-direction:column;gap:6px;margin-top:9px}
.bench-primary{align-self:flex-start;padding:7px 13px;border-radius:7px;background:#c9a45c;color:#14110b;font:500 12px var(--sans)}
.bench-primary kbd{color:#524222;border-color:#8c723d;margin-left:5px}
.bench-foot small{font:10px/1.5 var(--mono);color:#8e8e8e}
.bench-q{margin:0 0 4px;font-size:13px;line-height:1.45;color:#e6e6e6}
.bench-wrongline{margin:0 0 6px;font-size:11.5px;color:#d8c4be}
.bench-wrongline b{font:500 10px var(--mono);letter-spacing:.08em;text-transform:uppercase;color:#efa190;margin-right:4px}
.bench-wrongline small{font:9px var(--mono);color:#888;margin-left:4px}
.bench-grid{width:100%;border-collapse:collapse;table-layout:fixed;font:10px var(--mono)}
.bench-grid th,.bench-grid td{padding:4px 4px;text-align:left;vertical-align:top;border-top:1px solid var(--line);font-weight:400}
.bench-grid thead th:first-child{width:144px}
.bench-grid thead th{border-top:0;padding-bottom:6px}
.bench-grid thead th b{display:block;font:500 11px var(--sans);color:#e2ece5}
.bench-grid thead th small{color:#8a8a8a}
.bench-grid thead th[data-s=disproved] b{text-decoration:line-through;text-decoration-color:#e8604a;color:#a9a19e}
.bench-grid thead th[data-s=picked] b{color:#e6cb96}
.bench-grid tbody th button{display:block;width:100%;text-align:left;padding:2px 4px;margin:-2px -4px;border-radius:4px}
.bench-grid tbody th button:hover,.bench-grid tbody th button[aria-pressed=true]{background:#ffffff0c}
.bench-grid td[data-r=waiting]{color:#6f6f6f}
.bench-grid td[data-r=holds]{color:#8fdccd}
.bench-grid td[data-r=breaks]{color:#f6b2a5;background:#e8604a1a}
.bench-grid td[data-r=unreplayed]{color:#d4b67f;background:repeating-linear-gradient(135deg,#c9a45c14 0 2px,transparent 2px 6px)}
.bench-trace th,.bench-trace td{border-top:1px dashed #ffffff14;font-size:9.5px;line-height:1.4;color:#aaa;background:#ffffff05}
.bench-trace th small{display:block;color:#777;text-transform:uppercase;letter-spacing:.06em}
.bench-trace th span{color:#c9c9c9}
.bench-trace td[data-r=breaks]{color:#f6b2a5;background:#e8604a14}
.bench-trace td[data-r=unreplayed]{color:#d4b67f}
.bench-grid tfoot th{color:#777;text-transform:uppercase;letter-spacing:.06em;font-size:9px}
.bench-status td{color:#cfcfcf}
.bench-status td[data-s=held]{color:var(--ok)}
.bench-status td[data-s=disproved]{color:var(--fail)}
.bench-status td[data-s=spent]{color:#e1b86a}
.bench-status td[data-s=picked]{color:#e6cb96}
.bench-bar{position:relative;height:4px;border-radius:2px;background:#262626;margin:3px 0 4px}
.bench-bar i{position:absolute;left:0;top:0;bottom:0;border-radius:2px;background:#9a9a9a}
.bench-bar[data-full=true] i{background:#c9a45c}
.bench-bar u{position:absolute;top:-2px;bottom:-2px;width:1px;background:#e6cb96}
.bench-spend small{display:block;color:#8a8a8a;font-size:9px;line-height:1.4}
.bench-diff td{color:#9a9a9a;font-size:9.5px}
.bench-act td{border-top:0;padding-top:6px}
.bench-act button{display:flex;align-items:center;gap:6px;padding:5px 8px;border:1px solid var(--line2);border-radius:6px;font:500 11px var(--sans);color:#e5e5e5;background:#ffffff05;white-space:nowrap}
.bench-act button:hover{border-color:#aaa;background:#ffffff0c}
.bench-act button[aria-pressed=true]{border-color:#c9a45c;background:#c9a45c18}
.bench-act button:disabled{opacity:.35;cursor:default}
.bench-act button kbd{line-height:15px}
.bench-act [data-bench-extend]{border-color:#c9a45c70;color:#e6cb96}
.bench-cant{color:#8a6a64;font-size:9.5px}
.bench-picked{color:#e6cb96;font:500 11px var(--sans)}
.bench-pick{padding:8px 12px;border:1px solid #c9a45c60;border-radius:8px;background:#c9a45c0a}
.bench-rejects{display:flex;flex-direction:column;gap:3px;margin-top:8px;font:10px/1.45 var(--mono);color:#9a9a9a}
.bench-rejects b{color:#cfcfcf;font-weight:500}
.bench-done{padding:8px 12px;border:1px solid #35b7a250;border-radius:8px;background:#35b7a20a}
.bench-done h3{color:#8fdccd}
.bench-done p{margin:0;font-size:12px;line-height:1.5;color:#d6e3de}
.bench-kvarm{margin-top:9px;padding-top:6px;border-top:1px dashed #ffffff18}
.bench-kvarm h3{color:#999}
.bench-op{display:grid;grid-template-columns:auto 1fr;gap:0 10px;padding:2px 0;font:10px var(--mono);line-height:1.35}
.bench-op b{color:#cfcfcf;font-weight:500}
.bench-op span{justify-self:end;color:#999}
.bench-op small{grid-column:1/-1;color:#7d7d7d;font-size:9px}
`.replace(/^([.#][^{]+)\{/gm, (_, selectors) => selectors.split(",").map((s) => "body[data-concept=bench] " + s.trim()).join(",") + "{"),
  html: `<script>addEventListener("keydown", (ev) => { if (document.body.dataset.concept === "bench" && window.benchKeydown) window.benchKeydown(ev); }, { capture: true });</script><svg width="0" height="0" style="position:absolute" aria-hidden="true"><defs><pattern id="benchGraph" width="5" height="5" patternUnits="userSpaceOnUse"><rect width="5" height="5" fill="#141a17"/><path d="M0 0H5M0 0V5" stroke="#8fb8a0" stroke-opacity=".28" stroke-width=".45"/></pattern><pattern id="benchUnknown" width="2" height="2" patternUnits="userSpaceOnUse" patternTransform="rotate(45)"><rect width="2" height="2" fill="#1c1f1d"/><path d="M0 0V2" stroke="#c9a45c" stroke-width=".7"/></pattern></defs></svg><div id="benchLayer" hidden><svg id="benchDrawing" aria-hidden="true"></svg><div id="benchPins"></div><div id="benchPlates"></div><article class="bench-order" id="benchOrder" aria-label="Claimed payout order"></article><section class="bench-card" id="benchCard" tabindex="-1" aria-label="The bench" hidden></section><span id="benchStatus" hidden></span><span id="benchLive" class="bench-sr" role="status" aria-live="polite" style="position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0 0 0 0)"></span></div>`,
  script: `function benchInit(params) { (${benchMount.toString()})(${JSON.stringify({ bench: BENCH, witnesses: BENCH_WITNESSES, attempts: ATTEMPTS, recorded: RECORDED, events: EVENTS, extension: EXTENSION, ops: KVARM_OPS })}); }`,
};
