import { GATE } from "./gate-data.mjs";

function gateMount(data) {
  if (!conceptActive("gate")) return;
  const classes = data.classes, rank = (i) => classes.findIndex((c) => c.id === i.cls);
  const byId = Object.fromEntries(data.items.map((i) => [i.id, i]));
  const state = window.gateState = {
    queueOpen: true, capacity: 40, allowanceOpen: false,
    away: false, backAt: null, returned: false, awayChooser: false,
    open: null, briefOpen: false, held: {}, logOpen: false,
  };
  const layer = $("#gateLayer"), svg = $("#gateDrawing"), station = $("#gateStation"), card = $("#gateCard"), brief = $("#gateBrief");
  const escape = (v) => String(v).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
  const clamp = (n, min, max) => Math.max(min, Math.min(max, n));
  const point = (x, y, z = 0) => {
    const p = k.iso([x, y, z], P), r = world.getBoundingClientRect(), s = r.width / W;
    return [r.left + p[0] * s, r.top + p[1] * s];
  };
  const anchor = (id) => {
    const p = anchors[id], r = world.getBoundingClientRect(), s = r.width / W;
    return [r.left + p[0] * s, r.top + p[1] * s];
  };
  const path = (pts) => pts.map((p, i) => (i ? "L" : "M") + p.map((v) => v.toFixed(1)).join(" ")).join("");
  const est = (i) => i.est[0] + "–" + i.est[1] + " min";
  const bytesFmt = (n) => n.toLocaleString("en-US") + " B";
  const kvHost = data.kvarm.origin.replace(/^https?:\/\//, ""), kvBytes = data.kvarm.ops.reduce((n, o) => n + o.bytes, 0);

  // Ranked lists for the current moment (departure while away, now otherwise).
  const flag = (i) => state.away ? i.before : i.after;
  const present = () => data.items.filter((i) => flag(i) === true).sort((a, b) => rank(a) - rank(b) || data.items.indexOf(a) - data.items.indexOf(b));
  const forecast = () => data.items.filter((i) => typeof flag(i) === "string").sort((a, b) => rank(a) - rank(b) || data.items.indexOf(a) - data.items.indexOf(b));
  const demand = () => {
    const all = [...present(), ...forecast()];
    return { low: all.reduce((n, i) => n + i.est[0], 0), high: all.reduce((n, i) => n + i.est[1], 0), all };
  };
  const slips = () => {
    let cum = 0, out = [];
    for (const i of demand().all) { if (cum >= state.capacity) out.push(i.title); cum += i.est[0]; }
    return out;
  };

  const gaugeSvg = () => {
    const { low, high } = demand(), cap = state.capacity, w = 340, x0 = 8;
    const X = (v) => x0 + (w - 16) * v / Math.max(high, cap, 20) * 0.92;
    let s = '<svg class="gate-gauge" viewBox="0 0 ' + w + ' 34">';
    s += '<line class="gg-rail" x1="' + x0 + '" y1="20" x2="' + (w - 8) + '" y2="20"/>';
    s += '<rect class="gg-sure" x="' + x0 + '" y="16" width="' + Math.max(2, X(low) - x0) + '" height="8" rx="2"/>';
    s += '<rect class="gg-range" x="' + X(low) + '" y="16" width="' + Math.max(2, X(high) - X(low)) + '" height="8" rx="2"/>';
    if (high > cap) s += '<rect class="gg-over" x="' + X(cap) + '" y="16" width="' + (X(high) - X(cap)) + '" height="8" rx="2"/>';
    s += '<line class="gg-cap" x1="' + X(cap) + '" y1="8" x2="' + X(cap) + '" y2="30"/>';
    s += '<text x="' + X(cap) + '" y="6" text-anchor="middle">' + cap + '</text>';
    s += "</svg>";
    return s;
  };

  const rowHtml = (i, isForecast) => {
    const held = state.held[i.id];
    const tag = held ? '<b class="gt gt-hold">advised hold</b>' : "";
    return '<button class="gate-row' + (isForecast ? " gate-forecast" : "") + (state.open === i.id ? " gate-cur" : "") + '" data-gate-item="' + i.id + '" title="' + classes[rank(i)].label + '">'
      + '<span class="gate-cls" data-cls="' + i.cls + '"></span>'
      + '<span class="gate-row-main"><b>' + escape(i.title) + '</b>'
      + '<small>' + escape(i.agent) + " · " + est(i) + " · " + i.basis + (isForecast ? " · expected " + escape(flag(i)) : "") + (i.unforecast ? " · not forecast" : "") + '</small></span>'
      + tag + '</button>';
  };

  const stationHtml = () => {
    const { low, high } = demand(), over = high - state.capacity, sl = slips();
    let h = '<header class="gate-station-head"><span class="gate-ic">⏛</span><div><h2>Inspection gate</h2><small>' + (state.away ? "away · back " + escape(state.backAt.label) : "you · review station") + ' · advisory</small></div></header>';
    h += '<div class="gate-demand"><div class="gate-demand-line"><b>' + low + "–" + high + " min</b><span class='gate-needed'>next " + data.windowMin + " min</span>"
      + '<button class="gate-cap" data-gate-allowance>allowance <b>' + state.capacity + ' min</b> <kbd>M</kbd></button></div>'
      + (state.allowanceOpen
        ? '<div class="gate-choices" role="group" aria-label="Review allowance">' + data.presets.map((p, i) => '<button data-gate-cap="' + p + '" aria-pressed="' + (state.capacity === p) + '"><kbd>⇧' + (i + 1) + '</kbd>' + p + ' min</button>').join("") + '</div>'
        : gaugeSvg())
      + '<small class="gate-demand-note">' + (over > 0
        ? "Over allowance at the high estimate by " + over + " min. Slips first: " + (sl.length ? escape(sl.slice(0, 2).join(" · ")) : escape(demand().all.at(-1).title)) + ". Everything stays listed."
        : "Fits the allowance at every estimate.") + '</small></div>';
    if (state.awayChooser && !state.away) {
      h += '<div class="gate-sec">Step away · back at</div><div class="gate-choices" role="group" aria-label="Back at">'
        + data.backAt.map((b, i) => '<button data-gate-back="' + b.min + '"><kbd>⇧' + (i + 1) + '</kbd>' + b.label + '</button>').join("") + '</div>'
        + '<small class="gate-away-note">The mock replays your recorded half hour out, 16:00 to 16:30.</small>';
      return h;
    }
    const kv = present().filter((i) => i.kvarm), rest = present().filter((i) => !i.kvarm);
    if (kv.length) h += '<div class="gate-sec">Blocks publication · Kvarm</div>' + kv.map((i) => rowHtml(i)).join("");
    h += '<div class="gate-sec">Advisory queue · ranked by cost of waiting</div>' + rest.map((i) => rowHtml(i)).join("");
    const fc = forecast();
    if (fc.length) h += '<div class="gate-sec">Expected, not here yet</div>' + fc.map((i) => rowHtml(i, true)).join("");
    h += '<div class="gate-kvlog" id="gateKv" tabindex="-1" aria-label="Kvarm network log"' + (state.logOpen ? " data-open" : "") + '><header><span>↗ ' + kvHost + ' · ' + data.kvarm.ops.length + ' ops</span><b>' + bytesFmt(kvBytes) + '</b></header>'
      + data.kvarm.ops.map((o) => '<div class="gate-kvlog-row" title="' + escape(kvHost + " · " + o.reason) + '"><b>' + o.op + '</b><span>' + escape(o.reason) + '</span><i>' + bytesFmt(o.bytes) + '</i></div>').join("") + '</div>';
    if (state.away) {
      h += '<div class="gate-away-note">Replaying 16:00, the moment you left. Acan holds nothing back while you are gone.</div>'
        + '<div class="gate-foot"><button class="gate-go" data-gate-return>Return now <kbd>P</kbd></button></div>';
    } else {
      h += '<div class="gate-foot"><button class="gate-go" data-gate-away>Step away <kbd>P</kbd></button>'
        + (state.returned ? '<button class="gate-go gate-go-dim" data-gate-brief>Return brief</button>' : "") + '</div>';
    }
    return h;
  };

  const openItem = (id) => {
    if (document.body.dataset.mode !== "live") return;
    select(null);
    const i = byId[id];
    if (!i || flag(i) !== true && typeof flag(i) !== "string") return;
    state.open = id;
    const held = state.held[id];
    card.innerHTML = '<header class="gate-card-head"><span class="gate-kind">' + i.kind + '</span><div><h2>' + escape(i.title) + '</h2><small>'
      + escape(i.agent) + (i.territory ? " · " + escape(i.territory) : "") + (i.at != null ? " · arrived 16:" + String(30 + i.at).padStart(2, "0") : flag(i) === true ? " · already waiting at 16:00" : " · not here yet") + '</small></div>'
      + '<button data-gate-close aria-label="Close">× <kbd>Esc</kbd></button></header>'
      + '<div class="gate-est"><b>' + est(i) + '</b><span class="gate-basis" data-basis="' + i.basis + '">' + i.basis + '</span><small>' + escape(i.basisNote) + '</small></div>'
      + '<div class="gate-cost" data-cls="' + i.cls + '"><small>' + classes[rank(i)].label.toUpperCase() + ' · COST OF WAITING</small><p>' + escape(i.cost) + '</p></div>'
      + (i.kvarm ? '<div class="gate-kv"><b>Blocks publication · Kvarm ' + i.kvarm.change + '</b><span>' + escape(i.kvarm.rule) + '</span><small>Nothing publishes until you review. The plant keeps running.</small></div>' : "")
      + '<dl class="gate-facts">'
      + '<dt>Safe to wait</dt><dd>' + escape(i.safe) + '</dd>'
      + '<dt>Meanwhile</dt><dd>' + escape(i.meanwhile) + '</dd>'
      + '<dt>If you advise a hold</dt><dd>' + escape(i.advice) + '</dd></dl>'
      + '<div class="gate-acts">'
      + (i.review ? '<button class="gate-pri" data-gate-review>Open review <kbd>↵</kbd></button>' : '<button class="gate-pri" data-gate-review>Open on the plant <kbd>↵</kbd></button>')
      + '<button data-gate-hold aria-pressed="' + !!held + '">' + (held ? "Hold advised · withdraw" : "Advise hold") + ' <kbd>V</kbd></button>'
      + '</div><small class="gate-note">Advisory. ' + escape(i.agent) + ' decides whether to park; Acan never stops an agent.</small>';
    card.hidden = false;
    paint();
    card.querySelector("[data-gate-close]").focus({ preventScroll: true });
  };
  const closeItem = () => { state.open = null; card.hidden = true; paint(); };
  const nextItem = () => {
    const list = present();
    if (!list.length) return;
    const i = list.findIndex((x) => x.id === state.open);
    openItem(list[(i + 1) % list.length].id);
  };
  const review = (i) => {
    closeItem();
    if (!i.review) { select(i.on); return; }
    if (i.review.kind === "choice") openReview(i.review.id);
    else if (i.review.kind === "session") openSession(i.review.id);
    else select(i.review.id);
  };
  const adviseHold = () => {
    const i = byId[state.open];
    if (!i) return;
    state.held[i.id] = !state.held[i.id];
    openItem(i.id);
    toast(state.held[i.id] ? "Advised a hold on “" + i.title + "”. " + i.advice : "Hold advice withdrawn. " + i.meanwhile);
  };

  // Departure: the plant at 16:00. Return: the base fixture is the truth, so restore it.
  const terr = SCENE.territories.find((t) => t.id === data.departure.territory.id);
  const savedTerr = { state: terr.state, label: terr.label };
  const savedYou = { sub: ENT.you.sub };
  const agentRowText = (sid) => {
    const row = document.querySelector('.bl .agent[data-session="' + sid + '"]');
    return { row, text: [...row.childNodes].find((n) => n.nodeType === Node.TEXT_NODE && n.textContent.trim()), span: row.querySelector("span") };
  };
  const savedAgents = Object.fromEntries(Object.keys(data.departure.agents).map((sid) => { const a = agentRowText(sid); return [sid, [a.text.textContent, a.span.textContent]]; }));
  const setAway = (on) => {
    state.away = on;
    document.body.classList.toggle("gate-depart", on);
    if (on) {
      Object.assign(terr, { state: data.departure.territory.state, label: data.departure.territory.label });
      ENT.you.sub = "3 choices · none blocked";
      for (const [sid, [t, m]] of Object.entries(data.departure.agents)) { const a = agentRowText(sid); a.text.textContent = " " + t; a.span.textContent = m; }
    } else {
      Object.assign(terr, savedTerr);
      ENT.you.sub = savedYou.sub;
      for (const [sid, [t, m]] of Object.entries(savedAgents)) { const a = agentRowText(sid); a.text.textContent = t; a.span.textContent = m; }
    }
    paint();
    renderPlant();
  };
  const stepAway = (min) => {
    state.backAt = data.backAt.find((b) => b.min === min);
    state.awayChooser = false;
    closeItem();
    setAway(true);
    toast("Away. Back " + state.backAt.label + ". Acan holds nothing back; Droid keeps building.");
  };
  const returnNow = () => {
    select(null);
    setAway(false);
    state.returned = true;
    state.briefOpen = true;
    brief.hidden = false;
    paintBrief();
    paint();
    toast("Back at 16:30 · 4 reviews arrived, gate-sessions went stale · nothing was held back.");
  };

  const paintBrief = () => {
    brief.innerHTML = '<header class="gate-card-head"><span class="gate-ic">↩</span><div><h2>While you were out</h2><small>16:00–16:30 · recorded · ranked by cost of waiting</small></div>'
      + '<button data-gate-brief-close aria-label="Close brief">× <kbd>Esc</kbd></button></header>'
      + '<p class="gate-held-nothing">' + escape(data.heldNothing) + '</p>'
      + data.brief.map((r, i) => {
        const it = byId[r.items[0]];
        return '<div class="gate-brow" data-cls="' + r.cls + '"><div class="gate-brow-main"><small>' + classes.find((c) => c.id === r.cls).label.toUpperCase() + (r.unforecast ? " · NOT FORECAST" : "") + '</small>'
          + '<b>' + escape(r.title) + '</b><p>' + escape(r.what) + '</p>'
          + '<div class="gate-srcs">' + r.sources.map((s) => '<span data-ev="' + s[2] + '" title="' + escape(s[2]) + '">' + escape(s[0]) + " · " + escape(s[1]) + '</span>').join("") + '</div></div>'
          + '<div class="gate-brow-acts"><button data-gate-seek="' + i + '">Replay 16:' + String(30 + r.t).padStart(2, "0") + '</button>'
          + (it && it.review ? '<button data-gate-breview="' + it.id + '">Open review</button>' : "") + '</div></div>';
      }).join("")
      + '<p class="gate-unchanged">' + escape(data.unchanged) + '</p>';
  };
  const seekBrief = (i) => {
    const r = data.brief[i];
    setMode("replay");
    rpSeek((data.windowMin + r.t) * 60);
    if (r.sel) select(r.sel);
  };

  function paint() {
    if (!conceptActive("gate")) return;
    setWaitingCount(present().length);
    station.hidden = !state.queueOpen;
    if (state.queueOpen) {
      station.style.maxHeight = Math.max(200, $(".bl").getBoundingClientRect().top - 12 - station.offsetTop) + "px";
      station.innerHTML = stationHtml();
    }
    let drawing = "";
    const [sx, sy] = anchor("you");
    if (state.queueOpen) {
      const r = station.getBoundingClientRect();
      drawing += '<path class="gate-leader" d="' + path([[sx, sy], [clamp(sx, r.left, r.right), clamp(sy, r.top + 24, r.bottom - 24)]]) + '"/>';
    }
    if (state.open && byId[state.open]) {
      const i = byId[state.open], [x, y] = anchor(i.on);
      const width = card.offsetWidth, height = card.offsetHeight;
      const cx = Math.min(innerWidth - width - 24, $("#nav").getBoundingClientRect().left - 16 - width), cy = clamp(y - height / 2, 170, innerHeight - height - 74);
      card.style.left = cx + "px"; card.style.top = cy + "px";
      if (x > 0 && x < innerWidth && y > 0 && y < innerHeight) drawing += '<path class="gate-leader" d="' + path([[x, y - 20], [cx > x ? cx : cx + width, clamp(y - 20, cy + 20, cy + height - 20)]]) + '"/>';
    }
    if (state.away) {
      const tx = Math.max(sx - 62, 400);
      drawing += '<g class="gate-away-tag"><rect x="' + tx + '" y="' + (sy - 66) + '" width="124" height="34" rx="6"/><text x="' + (tx + 62) + '" y="' + (sy - 52) + '">16:00 · away</text><text class="gate-away-sub" x="' + (tx + 62) + '" y="' + (sy - 39) + '">back ' + escape(state.backAt.label) + '</text></g>';
    }
    svg.innerHTML = drawing;
  }

  // Events.
  station.onclick = (ev) => {
    if (!conceptActive("gate")) return;
    const row = ev.target.closest("[data-gate-item]"), cap = ev.target.closest("[data-gate-cap]"), back = ev.target.closest("[data-gate-back]");
    if (row) return openItem(row.dataset.gateItem);
    if (cap) { state.capacity = +cap.dataset.gateCap; state.allowanceOpen = false; paint(); return; }
    if (back) return stepAway(+back.dataset.gateBack);
    if (ev.target.closest("[data-gate-allowance]")) { state.allowanceOpen = !state.allowanceOpen; state.awayChooser = false; paint(); return; }
    if (ev.target.closest("[data-gate-away]")) { state.awayChooser = true; state.allowanceOpen = false; paint(); return; }
    if (ev.target.closest("[data-gate-return]")) returnNow();
    if (ev.target.closest("[data-gate-brief]")) { state.briefOpen = true; brief.hidden = false; paintBrief(); paint(); }
  };
  card.onclick = (ev) => {
    if (!conceptActive("gate")) return;
    if (ev.target.closest("[data-gate-close]")) return closeItem();
    if (ev.target.closest("[data-gate-hold]")) return adviseHold();
    if (ev.target.closest("[data-gate-review]")) review(byId[state.open]);
  };
  brief.onclick = (ev) => {
    if (!conceptActive("gate")) return;
    if (ev.target.closest("[data-gate-brief-close]")) { state.briefOpen = false; brief.hidden = true; paint(); return; }
    const s = ev.target.closest("[data-gate-seek]"), r = ev.target.closest("[data-gate-breview]");
    if (s) seekBrief(+s.dataset.gateSeek);
    if (r) { state.briefOpen = false; brief.hidden = true; openItem(r.dataset.gateBreview); }
  };
  window.gateKeydown = (ev) => {
    if (!conceptActive("gate")) return;
    if (typing(ev)) { if (ev.key === "Escape") ev.target.blur(); return; }
    const consume = () => { ev.preventDefault(); ev.stopImmediatePropagation(); };
    const key = ev.key.toLowerCase();
    if (state.open && (ev.ctrlKey || ev.metaKey)) { ev.preventDefault(); ev.stopImmediatePropagation(); return; }
    if (!bare(ev)) return;
    if (ev.shiftKey && /^Digit[1-4]$/.test(ev.code)) {
      const d = +ev.code.slice(5) - 1;
      if (state.allowanceOpen) { consume(); state.capacity = data.presets[d]; state.allowanceOpen = false; paint(); toast("Allowance set to " + data.presets[d] + " min per hour. Forecast unchanged."); return; }
      if (state.awayChooser && !state.away) { consume(); stepAway(data.backAt[d].min); return; }
      if (state.open || state.briefOpen) { consume(); return; }
      return;
    }
    if (key === "escape") {
      if (state.open) { consume(); closeItem(); return; }
      if (state.allowanceOpen || state.awayChooser) { consume(); state.allowanceOpen = state.awayChooser = false; paint(); return; }
      if (state.briefOpen) { consume(); state.briefOpen = false; brief.hidden = true; paint(); return; }
      return;
    }
    if (key === "u") { consume(); state.queueOpen = !state.queueOpen; paint(); return; }
    if (key === "i") { consume(); nextItem(); return; }
    if (key === "m") { if (document.body.dataset.mode !== "live") return; consume(); state.allowanceOpen = !state.allowanceOpen; state.awayChooser = false; paint(); return; }
    if (key === "p") {
      if (document.body.dataset.mode !== "live") return;
      consume();
      if (state.away) returnNow();
      else { state.awayChooser = !state.awayChooser; state.allowanceOpen = false; paint(); }
      return;
    }
    if (key === "v") { if (state.open) { consume(); adviseHold(); } return; }
    if (key === "enter" && state.open) { consume(); review(byId[state.open]); return; }
    if (state.open && (/^[a-z0-9]$/.test(key) || ev.key.startsWith("Arrow") || ev.key === " ")) consume();
  };
  const egress = $(".egress");
  egress.dataset.kv = "ok";
  egress.title = "Kvarm · " + data.kvarm.origin + " · declared egress";
  egress.innerHTML = '<span class="dot"></span>local + kvarm · ' + bytesFmt(kvBytes) + " out";
  egress.addEventListener("click", () => {
    if (!conceptActive("gate")) return;
    $("#net").classList.remove("open");
    state.queueOpen = true; state.logOpen = !state.logOpen; paint();
    const log = $("#gateKv");
    log.scrollIntoView({ block: "nearest" }); log.focus({ preventScroll: true });
  });
  const observer = new MutationObserver(paint);
  observer.observe(world, { attributes: true, attributeFilter: ["style"] });
  observer.observe(plant, { childList: true });
  new MutationObserver(() => { if (document.body.dataset.mode !== "live" && state.open) closeItem(); }).observe(document.body, { attributes: true, attributeFilter: ["data-mode"] });
  addEventListener("resize", paint);
  layer.hidden = false;
  paint();
}

export default {
  id: "gate",
  label: "Inspection gate",
  hint: "U queue · I next review · M allowance · P step away · V advise hold · Shift 1-4 pick · Esc close",
  css: `
body:not([data-concept=gate]) #gateLayer{display:none}
body[data-concept=gate] *,body[data-concept=gate] *::before,body[data-concept=gate] *::after{transition:none!important;animation:none!important}
body[data-concept=gate] .hint{opacity:1!important;z-index:45;background:#101110;border:1px solid var(--line);padding:5px 12px;border-radius:6px;top:62px}
body[data-concept=gate]:not([data-mode=live]) #gateLayer{display:none}
body[data-concept=gate].gate-depart .ent[data-id=breaker],body[data-concept=gate].gate-depart .ent[data-id=bot-cx],body[data-concept=gate].gate-depart .ent[data-id=crane],
body[data-concept=gate].gate-depart .chip[data-id=breaker],body[data-concept=gate].gate-depart .chip[data-id=bot-cx],body[data-concept=gate].gate-depart .chip[data-id=crane],
body[data-concept=gate].gate-depart .cpin[data-c=traceparent-queue-rows],body[data-concept=gate].gate-depart .cpin[data-c=wrap-databasesync]{display:none}
#gateLayer{position:fixed;inset:0;pointer-events:none;z-index:8}
#gateLayer [hidden]{display:none!important}
#gateLayer button:focus-visible,#gateLayer summary:focus-visible{outline:2px solid #c9a45c;outline-offset:3px}
#gateDrawing{position:absolute;inset:0;width:100%;height:100%;overflow:visible}
.gate-leader{stroke:#c9a45c90;stroke-width:1;fill:none}
.gate-away-tag rect{fill:#17150f;stroke:#c9a45c60}
.gate-away-tag text{fill:#dfbf84;font:10px var(--mono);text-anchor:middle}
.gate-away-tag .gate-away-sub{fill:#a5936f;font-size:9px}
.gate-station{position:absolute;left:16px;top:68px;width:400px;padding:10px 14px;border:1px solid var(--line2);border-radius:12px;background:var(--glass-hi);backdrop-filter:blur(18px);-webkit-backdrop-filter:blur(18px);box-shadow:var(--shadow);pointer-events:auto;overflow:auto;scrollbar-width:thin;scrollbar-color:#555 transparent}
.gate-station-head{display:flex;align-items:center;gap:10px;margin-bottom:6px}
.gate-ic{display:grid;place-items:center;width:30px;height:30px;background:#c9a45c;color:#18140d;border-radius:8px;font-size:16px;flex:none}
.gate-station h2,.gate-card h2,.gate-brief h2{font-size:15px;font-weight:550;margin:0;color:#eee}
.gate-station-head small,.gate-card-head small{display:block;font:10px var(--mono);margin-top:2px;color:#999}
.gate-demand{border:1px solid var(--line);border-radius:8px;padding:8px 10px 6px;background:#ffffff03}
.gate-demand-line{display:flex;align-items:center;gap:8px;font-size:11px;color:#cfcfcf}
.gate-demand-line>b{font:600 13px var(--mono);color:#f0e6d2;white-space:nowrap}
.gate-needed{white-space:nowrap;overflow:hidden;text-overflow:ellipsis;min-width:0}
.gate-demand-line{flex-wrap:nowrap}
.gate-demand-line>span,.gate-demand-line{font-size:11px}
.gate-demand-line{overflow:hidden}
.gate-demand-line{font-variant-numeric:tabular-nums}
.gate-cap{margin-left:auto;font:10px var(--mono);color:#dfbf84;padding:3px 6px;border:1px solid #c9a45c40;border-radius:5px;white-space:nowrap}
.gate-cap:hover{background:#c9a45c14}
.gate-cap b{color:#f0e6d2}
.gate-gauge{width:100%;height:30px;margin-top:2px}
.gg-rail{stroke:#3a3a3a}
.gg-sure{fill:#c9a45c55}
.gg-range{fill:repeating-linear-gradient(45deg,#c9a45c30 0 3px,transparent 3px 6px)}
.gg-over{fill:repeating-linear-gradient(45deg,rgba(232,96,74,.5) 0 3px,rgba(232,96,74,.12) 3px 6px)}
.gg-cap{stroke:#eee;stroke-width:1.4}
.gate-gauge text{fill:#e6e6e6;font:9px var(--mono)}
.gate-demand-note{display:block;font:10px/1.45 var(--mono);color:#a89e8a;margin-top:2px}
.gate-sec{padding:6px 2px 3px;font:500 9.5px var(--mono);letter-spacing:.12em;text-transform:uppercase;color:var(--faint)}
.gate-row{display:flex;align-items:flex-start;gap:8px;width:100%;text-align:left;padding:2px 8px;border-radius:7px;font-size:12px;color:#d6d6d6;border:1px solid transparent}
.gate-row:hover{background:#ffffff08}
.gate-row.gate-cur{border-color:#c9a45c50;background:#c9a45c0a}
.gate-row.gate-forecast{border-style:none;opacity:.75}
.gate-row.gate-forecast b{font-weight:400;font-style:italic}
.gate-cls{width:9px;height:9px;margin-top:4px;border-radius:50%;flex:none;background:#8a8a8a}
.gate-cls[data-cls=harm]{background:var(--fail)}
.gate-cls[data-cls=idle]{background:var(--strain)}
.gate-cls[data-cls=reversal]{background:#c9a45c}
.gate-cls[data-cls=steady]{background:#9fb0c9}
.gate-cls[data-cls=publish]{background:#b39ddb}
.gate-cls[data-cls=none]{background:#565656}
.gate-row-main{min-width:0;flex:1;display:flex;flex-direction:column}
.gate-row-main b{font-weight:500;line-height:1.35;color:#e5e5e5;overflow-wrap:anywhere}
.gate-row-main small{font:9.5px/1.4 var(--mono);color:var(--dim);overflow-wrap:anywhere}
.gt{flex:none;margin-top:1px;font:9px var(--mono);font-weight:400;padding:2px 5px;border-radius:4px;white-space:nowrap}
.gt-hold{color:#dfbf84;border:1px solid #c9a45c55}
.gate-kvlog{margin-top:8px;border-top:1px solid var(--line);padding:6px 0 0;font:10px/1.55 var(--mono);color:#8f897e;outline:none}
.gate-kvlog:focus-visible{outline:1px solid #c9a45c;outline-offset:2px}
.gate-kvlog header{display:flex;gap:6px;color:#9f9583;font:10px var(--mono)}
.gate-kvlog header b{margin-left:auto;font-weight:400;color:#c9a45c}
.gate-kvlog-row{display:grid;grid-template-columns:auto minmax(0,1fr) auto;gap:8px;white-space:nowrap}
.gate-kvlog-row b{font-weight:400;color:#c8bfae;white-space:nowrap}
.gate-kvlog-row span{overflow:hidden;text-overflow:ellipsis;color:#7f796f}
.gate-kvlog[data-open] .gate-kvlog-row span{white-space:normal}
.gate-kvlog-row i{font-style:normal;color:#9f9583;white-space:nowrap}
.gate-foot{display:flex;gap:6px;margin-top:8px}
.gate-go{flex:1;height:30px;border-radius:8px;border:1px solid #c9a45c55;background:#c9a45c0d;color:#ecd9ae;font-size:12px}
.gate-go:hover{background:#c9a45c1c}
.gate-go-dim{border-color:var(--line2);background:#ffffff05;color:#ddd}
.gate-choices{display:flex;gap:6px;margin-top:6px;flex-wrap:wrap}
.gate-choices button{display:flex;align-items:center;gap:7px;height:28px;padding:0 9px;border-radius:7px;border:1px solid var(--line2);font-size:11px;color:#ddd}
.gate-choices button:hover{background:#ffffff0a}
.gate-choices button[aria-pressed=true]{border-color:#c9a45c;background:#c9a45c14;color:#f0e6d2}
.gate-away-note{display:block;margin-top:8px;font:10px/1.5 var(--mono);color:#a89e8a}
.gate-card{position:absolute;width:460px;max-width:calc(100vw - 48px);padding:13px 15px;border:1px solid #c9a45c60;border-radius:13px;background:#161715;box-shadow:var(--shadow);pointer-events:auto}
.gate-card-head{display:flex;align-items:center;gap:10px;margin-bottom:10px}
.gate-kind{display:grid;place-items:center;min-width:32px;height:32px;padding:0 6px;background:#c9a45c;color:#18140d;border-radius:8px;font:600 11px var(--mono)}
.gate-card-head>button{display:flex;align-items:center;gap:7px;margin-left:auto;padding:5px;color:#aaa;font-size:18px}
.gate-est{display:flex;align-items:baseline;gap:9px;padding:9px 11px;border:1px solid var(--line);border-radius:8px;background:#ffffff03}
.gate-est b{font:400 20px var(--mono);color:#f0e6d2}
.gate-basis{font:9.5px var(--mono);padding:2px 6px;border-radius:4px;border:1px solid var(--line2);color:#cfcfcf;text-transform:uppercase;letter-spacing:.06em}
.gate-basis[data-basis=guess]{border-style:dashed;color:#c9b8a0}
.gate-est small{font:10px var(--mono);color:var(--dim)}
.gate-cost{margin-top:9px;padding:10px 12px;border:1px solid #e8604a45;border-left:3px solid var(--fail);background:#e8604a0c;border-radius:7px}
.gate-cost[data-cls=idle]{border-color:#e1a43c45;border-left-color:var(--strain);background:#e1a43c0a}
.gate-cost[data-cls=reversal]{border-color:#c9a45c45;border-left-color:#c9a45c;background:#c9a45c0a}
.gate-cost[data-cls=steady],.gate-cost[data-cls=publish],.gate-cost[data-cls=none]{border-color:#85858540;border-left-color:#8a8a8a;background:#85858508}
.gate-cost small{font:9px var(--mono);letter-spacing:.08em;color:#c9b8a6}
.gate-cost p{font-size:13px;line-height:1.45;color:#e8dcc9;margin:5px 0 0}
.gate-kv{margin-top:9px;padding:9px 11px;border:1px solid #b39ddb55;border-radius:7px;background:#b39ddb0c;font-size:11.5px;color:#ded3ee}
.gate-kv b{display:block;font:600 10.5px var(--mono);color:#cfbde8}
.gate-kv span{display:block;margin-top:3px}
.gate-kv small{display:block;margin-top:4px;font:9.5px var(--mono);color:#9c8fae}
.gate-facts{margin:10px 0 0;display:grid;grid-template-columns:96px 1fr;gap:5px 10px}
.gate-facts dt{font:9.5px var(--mono);text-transform:uppercase;letter-spacing:.08em;color:var(--faint);padding-top:1px}
.gate-facts dd{margin:0;font-size:11.5px;line-height:1.45;color:#cfcfcf}
.gate-acts{display:flex;gap:6px;margin-top:11px}
.gate-acts button{flex:1;height:30px;border-radius:8px;border:1px solid var(--line2);font-size:12px;color:#ddd;background:#ffffff03}
.gate-acts button:hover{background:#ffffff09}
.gate-acts .gate-pri{background:#e8e8e8;color:#0a0a0a;border-color:#e8e8e8}
.gate-acts .gate-pri kbd{color:#666;border-color:#aaa}
.gate-acts [aria-pressed=true]{border-color:#c9a45c;color:#ecd9ae;background:#c9a45c12}
.gate-note{display:block;margin-top:8px;font:9.5px var(--mono);color:var(--faint)}
.gate-brief{position:absolute;right:112px;top:150px;width:540px;max-width:calc(100vw - 560px);padding:11px 15px;border:1px solid var(--line2);border-radius:13px;background:var(--glass-hi);backdrop-filter:blur(18px);-webkit-backdrop-filter:blur(18px);box-shadow:var(--shadow);pointer-events:auto;max-height:calc(100vh - 210px);overflow:auto;scrollbar-width:thin;scrollbar-color:#555 transparent}
.gate-held-nothing{font:10px/1.45 var(--mono);color:#b9ad93;border:1px dashed #c9a45c45;border-radius:7px;padding:5px 9px;margin:0 0 6px}
.gate-brow{display:flex;gap:10px;padding:6px 4px;border-top:1px solid var(--line)}
.gate-brow-main{min-width:0;flex:1}
.gate-brow-main>small{font:9px var(--mono);letter-spacing:.08em;color:#c9b8a6}
.gate-brow[data-cls=none] .gate-brow-main>small{color:var(--faint)}
.gate-brow-main b{display:block;font-size:13px;font-weight:550;color:#eee;margin:2px 0}
.gate-brow-main p{margin:0;font-size:11px;line-height:1.4;color:#bdbdbd}
.gate-srcs{display:flex;gap:5px;flex-wrap:wrap;margin-top:4px}
.gate-srcs span{font:9px var(--mono);color:#a8b0a4;padding:2px 6px;border:1px solid var(--line2);border-radius:4px}
.gate-srcs span[data-ev=observed]{color:#a4d3c7;border-color:#35b7a240}
.gate-srcs span[data-ev=authenticated]{color:#cfbde8;border-color:#b39ddb40}
.gate-brow-acts{display:flex;flex-direction:column;gap:5px;flex:none;align-self:center}
.gate-brow-acts button{height:26px;padding:0 10px;border-radius:7px;border:1px solid var(--line2);font:10.5px var(--mono);color:#ddd;white-space:nowrap}
.gate-brow-acts button:hover{background:#ffffff0a;border-color:#c9a45c70;color:#f0e6d2}
.gate-unchanged{font:10px/1.45 var(--mono);color:var(--dim);margin:6px 0 0;border-top:1px solid var(--line);padding-top:6px}
`.replace(/^([.#][^{]+)\{/gm, (_, selectors) => selectors.split(",").map((s) => "body[data-concept=gate] " + s.trim()).join(",") + "{"),
  html: `<script>addEventListener("keydown", (ev) => { if (document.body.dataset.concept === "gate" && window.gateKeydown) window.gateKeydown(ev); }, { capture: true });</script><div id="gateLayer" hidden><svg id="gateDrawing" aria-hidden="true"></svg><div class="gate-station" id="gateStation" role="region" aria-label="Inspection gate queue"></div><section class="gate-card" id="gateCard" aria-label="Review item" hidden></section><section class="gate-brief" id="gateBrief" aria-label="Return brief" hidden></section></div>`,
  script: `function gateInit(params) { (${gateMount.toString()})(${JSON.stringify(GATE)}); }`,
};
