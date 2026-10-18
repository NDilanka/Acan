import { FIRST } from "./acquit.mjs";

export const FR_HTML = `
<style id="frHide"></style>
<aside class="fr" id="fr" aria-label="Set up">
  <div class="fr-h"><div><b>Set up acquit</b><small>${FIRST.folder} · ${FIRST.rev}</small></div><button class="fr-skip" data-fr="skip-all">Skip setup</button></div>
  <ol class="fr-steps" id="frSteps"><li data-p="read"><i>1</i>Read</li><li data-p="place"><i>2</i>Place</li><li data-p="connect"><i>3</i>Connect</li></ol>
  <div class="fr-body" id="frBody"></div>
  <div class="fr-ft" id="frFt"></div>
</aside>`;

export const FR_CSS = `
.fr{display:none}
body.first .fr{display:flex;position:fixed;left:16px;top:62px;bottom:16px;width:400px;flex-direction:column;border-radius:16px;background:var(--glass-hi);backdrop-filter:blur(20px) saturate(1.2);-webkit-backdrop-filter:blur(20px) saturate(1.2);border:1px solid var(--line2);box-shadow:var(--shadow);z-index:7;overflow:hidden}
body.first .tc,body.first .bl,body.first .br .readout,body.first .br .live,body.first .br .motion,body.first .br .sep,body.first .hint,body.first .terr,body.first .pins,body.first .threads,body.first .floor,body.first .watermark{display:none!important}
body.first .chip:not(.frv){opacity:0!important}
body.first .chip.frv{opacity:1;translate:0 0}
body.first .chip.frv .rd{display:inline}
body.first .chip.prop::after{content:none}
.chip[data-state=idle] .mono{background:#4a4a4a;color:#d0d0d0}
.fr-h{display:flex;align-items:flex-start;gap:10px;padding:14px 14px 8px}
.fr-h b{display:block;font-size:15px;font-weight:600;color:#f0f0f0}
.fr-h small{font:10.5px var(--mono);color:var(--dim)}
.fr-skip{margin-left:auto;height:24px;padding:0 8px;border-radius:7px;font-size:11.5px;color:var(--dim)}
.fr-skip:hover{background:rgba(255,255,255,.06);color:#fff}
.fr-steps{display:flex;gap:4px;margin:0;padding:0 14px 10px;list-style:none;border-bottom:1px solid var(--line)}
.fr-steps li{flex:1;display:flex;align-items:center;gap:6px;font:10.5px var(--mono);letter-spacing:.1em;text-transform:uppercase;color:var(--faint)}
.fr-steps li i{display:grid;place-items:center;width:16px;height:16px;border-radius:50%;border:1px solid var(--line2);font-style:normal;font-size:9.5px;letter-spacing:0}
.fr-steps li[aria-current=step]{color:#f0f0f0}.fr-steps li[aria-current=step] i{background:#e8e8e8;color:#0a0a0a;border-color:#e8e8e8}
.fr-steps li[data-done] {color:var(--dim)}.fr-steps li[data-done] i{color:var(--ok);border-color:rgba(53,183,162,.5)}
.fr-body{flex:1;overflow:auto;padding:12px 14px;display:flex;flex-direction:column;gap:12px;scrollbar-width:thin;scrollbar-color:#333 transparent}
.fr-body>*{flex:none}
.fr-lede{margin:0;font-size:12.5px;color:#cfcfcf;line-height:1.5}
.fr-lede b{font-weight:500;color:#f2f2f2}
.fr-rd{display:grid;grid-template-columns:72px 1fr auto;gap:2px 10px;align-items:baseline;padding:6px 0;border-top:1px solid var(--line);opacity:.35}
.fr-rd:first-of-type{border-top:0}
.fr-rd[data-done=true]{opacity:1}
.fr-rd .s{font:9.5px var(--mono);letter-spacing:.08em;text-transform:uppercase;color:var(--dim)}
.fr-rd b{font-weight:500;font-size:12px;color:#e2e2e2}
.fr-rd small{grid-column:2/-1;font:10.5px var(--mono);color:var(--dim);overflow-wrap:anywhere}
.fr-rd em{font:normal 10.5px var(--mono);color:var(--dim)}
.fr-rd em[data-v=fog]{color:#9a9a9a;border:1px dashed #5a5a5a;border-radius:4px;padding:0 4px}
.fr-rd em[data-v=wait]{color:var(--wait)}.fr-rd em[data-v=live]{color:var(--ok)}
.fr-sum summary,.fr-yaml summary{display:flex;align-items:center;gap:8px;cursor:pointer;list-style:none;font:500 10px var(--mono);letter-spacing:.14em;text-transform:uppercase;color:var(--dim)}
.fr-sum summary::-webkit-details-marker,.fr-yaml summary::-webkit-details-marker{display:none}
.fr-sum summary::before,.fr-yaml summary::before{content:"›";font-size:12px;letter-spacing:0;transition:none}
.fr-sum[open] summary::before,.fr-yaml[open] summary::before{content:"⌄"}
.fr-sum summary i,.fr-yaml summary i{font-style:normal;margin-left:auto;letter-spacing:.02em;text-transform:none;color:var(--faint)}
.fr-sum[open] summary{margin-bottom:6px}
.fr-sec h5{margin:0 0 8px;display:flex;align-items:center;gap:8px;font:500 10px var(--mono);letter-spacing:.14em;text-transform:uppercase;color:var(--dim)}
.fr-sec h5 i{font-style:normal;margin-left:auto;letter-spacing:.02em;text-transform:none;color:var(--faint)}
.fr-card,.fr-ag{display:block;width:100%;text-align:left;padding:8px 10px;margin-top:6px;border-radius:10px;border:1px solid var(--line);background:rgba(255,255,255,.015);color:var(--text)}
.fr-card:hover,.fr-ag:hover{border-color:var(--line2)}
.fr-card[aria-current=true],.fr-ag[aria-current=true]{border-color:rgba(255,255,255,.32);background:rgba(255,255,255,.04)}
.fr-ch{display:flex;align-items:center;gap:8px}
.fr-ch .mono{display:grid;place-items:center;min-width:22px;height:20px;padding:0 3px;border-radius:5px;font:700 9.5px var(--mono);background:transparent;color:var(--ghost);border:1px dashed var(--ghost)}
.fr-card[data-pick] .fr-ch .mono,.fr-ag .fr-ch .mono{background:#4a4a4a;color:#e0e0e0;border:0}
.fr-card[data-fx=ignore] .fr-ch .mono,.fr-card[data-fx=part] .fr-ch .mono{background:none;border:1px solid var(--line2);color:var(--faint)}
.fr-ag[data-st=seen] .fr-ch .mono{background:#e8e8e8;color:#0a0a0a}
.fr-ch b{flex:1;min-width:0;font-weight:500;font-size:12.5px;color:#ececec;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.fr-conf{width:46px;height:4px;border-radius:2px;background:#262626;overflow:hidden}.fr-conf i{display:block;height:100%;background:#cfcfcf}
.fr-ch em{font:normal 10.5px var(--mono);color:var(--dim);min-width:26px;text-align:right}
.fr-ask{margin:8px 0 2px;font-size:12.5px;color:#e6e6e6}
.fr-why{display:block;font:10.5px var(--mono);color:var(--dim);overflow-wrap:anywhere}
.fr-yn{display:flex;flex-direction:column;gap:4px;margin-top:8px}
.fr-yn button{display:flex;align-items:center;gap:8px;height:28px;padding:0 8px;border-radius:7px;border:1px solid var(--line2);font-size:12px;color:#d8d8d8;text-align:left}
.fr-yn button:first-child{background:rgba(255,255,255,.06);color:#fff}
.fr-yn button:hover{background:rgba(255,255,255,.09)}
.fr-done{display:block;margin-top:4px;padding-left:30px;font:10.5px var(--mono);color:var(--dim)}
.fr-done b{font-weight:400;color:#cfcfcf}
.fr-yaml pre{margin:6px 0 0;padding:8px 10px;border-radius:8px;background:rgba(0,0,0,.35);border:1px solid var(--line);font:10.5px/1.55 var(--mono);color:#a8a8a8;white-space:pre;overflow-x:auto;scrollbar-width:thin}
.fr-yaml pre span{display:block}
.fr-yaml pre .c{color:#5d5d5d}
.fr-yaml pre .n{color:#f2f2f2;background:rgba(255,255,255,.06);margin:0 -10px;padding:0 10px}
.fr-file{margin-top:8px;font:10.5px var(--mono);color:#cfcfcf}
.fr-diff{margin:4px 0 0;padding:6px 8px;border-radius:8px;background:rgba(0,0,0,.35);border:1px solid var(--line);font:10.5px/1.55 var(--mono);color:#b9dccb;white-space:pre-wrap;overflow-wrap:anywhere}
.fr-ag small{display:block;margin-top:2px;font:10.5px var(--mono);color:var(--dim)}
.fr-ag .note{margin-top:8px;font-size:11.5px}
.fr-st{font:normal 10.5px var(--mono);color:var(--dim);display:flex;align-items:center;gap:6px}
.fr-ag[data-st=written] .fr-st{color:var(--wait)}.fr-ag[data-st=seen] .fr-st{color:var(--ok)}
.fr-ag[data-st=written] .fr-st::before{content:"";width:6px;height:6px;border-radius:50%;background:var(--wait);animation:pulse 1.6s infinite}
.fr-acts{display:flex;gap:6px;margin-top:8px}
.fr-acts button{flex:1;display:flex;align-items:center;justify-content:center;gap:6px;height:28px;border-radius:7px;border:1px solid var(--line2);font-size:12px;color:#d8d8d8}
.fr-acts button:first-child{background:rgba(255,255,255,.07);color:#fff}
.fr-acts button:hover{background:rgba(255,255,255,.1)}
.fr-ft{display:flex;flex-direction:column;gap:8px;padding:10px 14px 14px;border-top:1px solid var(--line)}
.fr-keys{display:flex;flex-wrap:wrap;gap:4px 12px;font-size:11px;color:var(--faint)}
.fr-keys kbd{margin-right:3px}
.fr-go{display:flex;align-items:center;justify-content:center;gap:8px;height:34px;border-radius:9px;background:#e8e8e8;color:#0a0a0a;font-size:12.5px;font-weight:500}
.fr-go:hover{background:#fff}
.fr-go kbd{color:#555;border-color:rgba(0,0,0,.2)}
.fr-go small{font:10.5px var(--mono);color:#555}
`;

export const FR_SCRIPT = `
const FRD = ${JSON.stringify(FIRST)};
const FR = { phase: null, read: 0, items: [], i: 0, undo: [], agents: [], ai: 0, lit: false, saved: null, timers: [], prevYaml: null };
const frOn = () => document.body.classList.contains("first");
const frName = (it) => it.name || ENT[it.id].name;
const frMono = (it) => it.mono || ENT[it.id].mono;
const frFx = (it) => (it.pick ? it.fx[it.pick === "yes" ? 0 : 1] : null);
const DRAWS = new Set(["machine", "dock"]);
const slug = (s) => s.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");
const undecided = () => FR.items.filter((it) => !it.pick);

function frDrawn() {
  const out = new Set();
  for (const r of FRD.readers.slice(0, FR.read)) for (const id of r.finds) out.add(id);
  for (const it of FR.items) if (DRAWS.has(frFx(it))) { out.add(it.id); for (const w of it.with || []) out.add(w); }
  for (const a of FR.agents) if (a.st === "seen" && a.bot) out.add(a.bot);
  return out;
}
function frGhosts() {
  const it = FR.items[FR.i];
  if (FR.phase !== "place" || !it || it.pick || !ENT[it.id] || !DRAWS.has(it.fx[0])) return new Set();
  return new Set([it.id, ...(it.with || [])]);
}
function frApply() {
  const drawn = frDrawn(), ghosts = frGhosts(), focus = FR.phase === "place" ? FR.items[FR.i] : null;
  let changed = false;
  for (const e of SCENE.entities) {
    const st = ghosts.has(e.id) ? "ghost" : drawn.has(e.id) ? (FR.lit ? FR.saved[e.id] : "idle") : e.state;
    if (e.state !== st) { e.state = st; changed = true; }
  }
  const hide = SCENE.entities.filter((e) => !drawn.has(e.id) && !ghosts.has(e.id)).map((e) => '.ent[data-id="' + e.id + '"],.chip[data-id="' + e.id + '"]');
  for (const b of SCENE.belts) if (b.ghost || !drawn.has(b.from) || !drawn.has(b.to)) hide.push('.belt[data-from="' + b.from + '"][data-to="' + b.to + '"]');
  $("#frHide").textContent = hide.length ? hide.join(",") + "{display:none!important}" : "";
  for (const c of document.querySelectorAll(".chip")) {
    const e = ENT[c.dataset.id], on = drawn.has(e.id) || ghosts.has(e.id);
    c.classList.toggle("frv", on && (ghosts.has(e.id) || ASSETS[e.kind].family !== "fitting"));
    if (!on) continue;
    c.dataset.state = e.state;
    const it = FR.items.find((x) => x.id === e.id);
    c.querySelector(".rd").textContent = ghosts.has(e.id) ? "unplaced · " + (it ? it.conf.toFixed(2) : "with paypal") : FR.lit ? e.reading : "";
  }
  const api = document.querySelector('.chip[data-id="api"] .rd');
  if (api && focus && focus.merge && !focus.pick) api.textContent = "lane 1 on :4320?";
  else if (api && FR.items.some((it) => frFx(it) === "lane") && !FR.lit) api.textContent = "2 lanes";
  if (changed) { dirtyScene = true; animate(); }
}

function frYaml() {
  const L = [["c", "# .acan/system.yaml · drafted from " + FRD.folder + " @ " + FRD.rev], ["c", "# yours to edit · Acan re-reads it on save"], ["", "machines:"]];
  const ids = [...frDrawn()].filter((id) => FRD.match[id]);
  const parts = FR.items.filter((it) => frFx(it) === "part").map(frName), lane = FR.items.some((it) => frFx(it) === "lane");
  for (const id of ids) {
    const e = ENT[id];
    L.push(["", "  " + slug(e.name) + ": " + ASSETS[e.kind].label.toLowerCase()]);
    L.push(["", "    match: " + FRD.match[id]]);
    if (id === "api" && lane) L.push(["", "    lanes: { stride: 10 }  # :4310, :4320"]);
    if (id === "api" && parts.length) L.push(["", "    parts: [" + parts.map(slug).join(", ") + "]"]);
  }
  for (const it of FR.items) if (frFx(it) === "new") L.push(["", "  " + slug(frName(it)) + ": unknown"], ["", "    match: " + it.rule.split(" · ")[0]]);
  const ign = FR.items.filter((it) => frFx(it) === "ignore");
  if (ign.length) { L.push(["", "ignore:"]); for (const it of ign) L.push(["", "  - " + frName(it) + (FRD.match[it.id] ? "  # " + FRD.match[it.id] : "")]); }
  const left = undecided();
  if (left.length) { L.push(["", "tray:  # unplaced, you decide later"]); for (const it of left) L.push(["", "  - " + slug(frName(it)) + "  # " + it.conf.toFixed(2)]); }
  return L;
}
function yamlHtml() {
  const L = frYaml(), prev = FR.prevYaml;
  FR.prevYaml = new Set(L.map((l) => l[1]));
  return { n: L.length, html: L.map(([c, t]) => '<span class="' + (c || (prev && !prev.has(t) ? "n" : "")) + '">' + t.replace(/</g, "&lt;") + "</span>").join("") };
}

function readerRows() {
  return FRD.readers.map((r, i) => {
    const done = i < FR.read, k = r.fog ? "fog" : r.waiting ? (FR.lit ? "live" : "wait") : "";
    const n = r.fog ? "fogged" : r.waiting ? (FR.lit ? "receiving" : "waiting") : r.files ? r.files + (r.files > 1 ? " files" : " file") : "none";
    return '<div class="fr-rd" data-done="' + done + '"><span class="s">' + r.src + "</span><b>" + r.label + '</b><em data-v="' + k + '">' + (done ? n : "…") + "</em><small>" + r.detail + "</small></div>";
  }).join("");
}
function cardHtml(it, i) {
  const cur = i === FR.i, fx = frFx(it);
  let h = '<div class="fr-card" data-i="' + i + '" aria-current="' + cur + '"' + (it.pick ? ' data-pick="' + it.pick + '" data-fx="' + fx + '"' : "") + ">";
  h += '<div class="fr-ch"><span class="mono">' + frMono(it) + "</span><b>" + frName(it) + '</b><span class="fr-conf"><i style="width:' + it.conf * 100 + '%"></i></span><em>' + it.conf.toFixed(2) + "</em></div>";
  if (it.pick) h += '<span class="fr-done">' + (it.pick === "yes" ? "Agreed · " : "Changed · ") + "<b>" + (it.pick === "yes" ? it.yes : it.no) + "</b>" + (cur ? " · Ctrl Z undoes" : "") + "</span>";
  else if (cur) h += '<p class="fr-ask">' + it.ask + '</p><small class="fr-why">Why: ' + it.rule + (ENT[it.id] && DRAWS.has(it.fx[0]) ? "" : " · not drawn until you place it") + '</small><div class="fr-yn"><button data-fr="yes"><kbd>Y</kbd>' + it.yes + '</button><button data-fr="no"><kbd>N</kbd>' + it.no + "</button></div>";
  return h + "</div>";
}
const ST_WORD = { found: "found", written: "waiting for a span", seen: "live", quiet: "written · no session yet", skipped: "skipped" };
function agentHtml(a, i) {
  const cur = i === FR.ai;
  let h = '<div class="fr-ag" data-i="' + i + '" data-st="' + a.st + '" aria-current="' + cur + '"><div class="fr-ch"><span class="mono">' + (a.id === "codex" ? "CX" : a.name[0]) + "</span><b>" + a.name + '</b><span class="fr-st">' + (a.st === "seen" ? "live · first span " + a.at : ST_WORD[a.st]) + "</span></div><small>Found " + a.found + "</small>";
  if (cur || a.st === "written") {
    for (const w of a.writes) h += '<div class="fr-file">' + (a.st === "found" ? "Adds to " : "Added to ") + w.to + (w.via ? " <i>by running " + w.via + "</i>" : "") + '</div><pre class="fr-diff">' + w.lines.map((l) => "+ " + l).join("\\n") + "</pre>";
    h += '<div class="note">' + a.note + "</div>";
    h += "<small>Hooks: " + a.hooks + " · backfills " + a.backfill + " past sessions from transcripts</small>";
  }
  if (a.st === "quiet") h += "<small>Events start the next time you run claude in " + FRD.folder + ".</small>";
  if (cur && a.st === "found") h += '<div class="fr-acts"><button data-fr="write"><kbd>Enter</kbd>Write it</button><button data-fr="copy"><kbd>C</kbd>Copy lines</button><button data-fr="skip"><kbd>S</kbd>Skip</button></div>';
  return h + "</div>";
}
function frRender() {
  const order = ["read", "place", "connect"];
  for (const li of document.querySelectorAll("#frSteps li")) { li.setAttribute("aria-current", li.dataset.p === FR.phase ? "step" : "false"); li.toggleAttribute("data-done", order.indexOf(li.dataset.p) < order.indexOf(FR.phase)); }
  const body = $("#frBody"), keep = body.scrollTop;
  const placed = Object.keys(FRD.auto).length, fogged = FRD.readers.filter((r) => r.fog).length;
  const sum = '<details class="fr-sum"><summary>Read ' + FRD.readers.length + " sources<i>" + placed + " placed · " + fogged + " fogged · 0 bytes out</i></summary>" + readerRows() + "</details>";
  let h = "", ft = "";
  if (FR.phase === "read") {
    h = '<p class="fr-lede">Reading the repo on this machine. <b>Nothing leaves it.</b> The pill at the top right counts every outbound connection.</p><div>' + readerRows() + "</div>";
    if (FR.read >= FRD.readers.length) h += '<p class="fr-lede">Placed <b>' + placed + " machines</b> from config. <b>" + FR.items.length + " guesses</b> need you.</p>";
    ft = '<div class="fr-keys"><span><kbd>Enter</kbd> skip ahead</span></div>';
  }
  if (FR.phase === "place") {
    const left = undecided().length, y = yamlHtml();
    h = sum + '<p class="fr-lede">Confident matches are on the plant in grey. They light up when spans arrive. Check the rest. <b>Nothing is written until you continue.</b></p>';
    h += '<div class="fr-sec"><h5>Unplaced tray<i>' + (FR.items.length - left) + " of " + FR.items.length + " checked</i></h5>" + FR.items.map(cardHtml).join("") + "</div>";
    h += '<details class="fr-yaml" open><summary>.acan/system.yaml<i>' + y.n + " lines · draft</i></summary><pre>" + y.html + "</pre></details>";
    ft = '<div class="fr-keys"><span><kbd>J</kbd><kbd>K</kbd> move</span><span><kbd>Y</kbd> agree</span><span><kbd>N</kbd> the other</span><span><kbd>Ctrl Z</kbd> undo</span></div>';
    ft += '<button class="fr-go" data-fr="write-acan">Write .acan/' + (left ? " <small>" + left + " stay in the tray</small><kbd>Shift Enter</kbd>" : " <kbd>Enter</kbd>") + "</button>";
  }
  if (FR.phase === "connect") {
    const open = FR.agents.filter((a) => a.st === "found").length;
    h = sum + '<p class="fr-lede">Wrote <b>.acan/system.yaml</b> and <b>layout.yaml</b>. Now point your agents at Acan. It adds these lines and nothing else, and keeps a backup next to each file. Message content export stays off.</p>';
    h += '<div class="fr-sec"><h5>Agents on this machine<i>' + (FR.agents.length - open) + " of " + FR.agents.length + " done</i></h5>" + FR.agents.map(agentHtml).join("") + "</div>";
    ft = '<div class="fr-keys"><span><kbd>J</kbd><kbd>K</kbd> move</span><span><kbd>Enter</kbd> write</span><span><kbd>C</kbd> copy</span><span><kbd>S</kbd> skip</span></div>';
    ft += '<button class="fr-go" data-fr="done">Open the plant ' + (open ? "<kbd>Shift Enter</kbd>" : "<kbd>Enter</kbd>") + "</button>";
  }
  body.innerHTML = h; body.scrollTop = keep; $("#frFt").innerHTML = ft;
  const cur = body.querySelector('[aria-current="true"]');
  if (cur) { const r = cur.getBoundingClientRect(), b = body.getBoundingClientRect(); if (r.top < b.top || r.bottom > b.bottom) cur.scrollIntoView({ block: "nearest" }); }
}

function frFit() {
  const r = stage.getBoundingClientRect(), L = 432, s = Math.min((r.width - L - 24) / W, r.height / H) * 1.02;
  goal.s = s; goal.x = L + (r.width - L - W * s) / 2; goal.y = (r.height - H * s) / 2 + 10; animate();
}
const later = (fn, ms) => FR.timers.push(setTimeout(fn, ms));
function frStart() {
  if (frOn()) return;
  if (document.body.dataset.mode !== "live") setMode("live");
  select(null); palClose(); toggleQueue(false); toggleFloor(false);
  FR.saved = Object.fromEntries(SCENE.entities.map((e) => [e.id, e.state]));
  Object.assign(FR, { phase: "read", read: 0, i: 0, ai: 0, undo: [], lit: false, prevYaml: null, items: FRD.tray.map((t) => ({ ...t, pick: null })), agents: FRD.agents.map((a) => ({ ...a, st: "found", at: null })) });
  document.body.classList.add("first");
  frApply(); frRender(); frFit();
  const tick = setInterval(() => {
    FR.read++; frApply(); frRender();
    if (FR.read >= FRD.readers.length) { clearInterval(tick); later(() => frPhase("place"), 700); }
  }, 260);
  FR.timers.push(tick);
}
function frPhase(p) {
  if (FR.phase === "read" && p === "place") { for (const t of FR.timers) { clearTimeout(t); clearInterval(t); } FR.timers = []; FR.read = FRD.readers.length; }
  FR.phase = p; frApply(); frRender();
}
function frMove(d) {
  const list = FR.phase === "place" ? FR.items : FR.agents, key = FR.phase === "place" ? "i" : "ai";
  FR[key] = (FR[key] + d + list.length) % list.length; frApply(); frRender();
}
function frNextOpen(from) {
  for (let s = 1; s <= FR.items.length; s++) { const j = (from + s) % FR.items.length; if (!FR.items[j].pick) return j; }
  return from;
}
function frPick(pick) {
  const it = FR.items[FR.i]; if (!it || it.pick) return;
  FR.undo.push(FR.i); it.pick = pick; FR.i = frNextOpen(FR.i); frApply(); frRender();
}
function frUndo() {
  if (FR.phase !== "place") return toast("Each settings file has a backup next to it. Restore from there.");
  const i = FR.undo.pop(); if (i == null) return toast("Nothing to undo.");
  const it = FR.items[i]; toast("Undid " + frName(it) + ": " + (it.pick === "yes" ? it.yes : it.no).toLowerCase() + ".");
  it.pick = null; FR.i = i; frApply(); frRender();
}
function frWriteAcan() {
  const left = undecided().length;
  frPhase("connect");
  toast("Wrote .acan/system.yaml (" + frYaml().length + " lines) and layout.yaml." + (left ? " " + left + " stay in the tray." : ""));
}
function frClock(sec) { const d = new Date(Date.now() + sec * 1000); return d.toTimeString().slice(0, 8); }
function frAgent(act) {
  const a = FR.agents[FR.ai]; if (!a || a.st !== "found") return;
  const n = a.writes.reduce((k, w) => k + w.lines.length, 0), files = a.writes.map((w) => w.to).join(" and ");
  if (act === "copy") { copyText(a.writes.map((w) => (w.via ? "# or run: " + w.via + "\\n" : "") + "# " + w.to + "\\n" + w.lines.join("\\n")).join("\\n\\n")); return toast("Copied " + n + " lines for " + files + ". Acan watches for the first signal."); }
  if (act === "skip") a.st = "skipped";
  if (act === "write") {
    a.st = a.first == null ? "quiet" : "written";
    handoff("settings", { writes: a.writes }, "Wrote " + n + " lines to " + files + ". Backup saved next to each.");
    if (a.first != null) later(() => frSeen(a), a.first * 2000);
  }
  const next = FR.agents.findIndex((x) => x.st === "found");
  if (next >= 0) FR.ai = next;
  frRender();
}
function frSeen(a) {
  if (!frOn()) return;
  a.st = "seen"; a.at = frClock(0);
  const first = !FR.lit; FR.lit = true;
  frApply(); frRender();
  const bad = [...frDrawn()].filter((id) => FR.saved[id] === "fail").map((id) => ENT[id].name);
  toast(first ? "First span from " + a.name + ". The plant is live" + (bad.length ? " and " + bad.join(", ") + " is failing." : ".") + " Backfilled " + a.backfill + " sessions." : a.name + " is live. Backfilled " + a.backfill + " sessions.");
}
function frExit(msg) {
  for (const t of FR.timers) { clearTimeout(t); clearInterval(t); } FR.timers = [];
  document.body.classList.remove("first");
  for (const e of SCENE.entities) e.state = FR.saved[e.id];
  $("#frHide").textContent = "";
  for (const c of document.querySelectorAll(".chip")) { const e = ENT[c.dataset.id]; c.classList.remove("frv"); c.dataset.state = e.state; c.querySelector(".rd").textContent = e.reading; }
  FR.phase = null; dirtyScene = true; animate(); fit();
  if (msg) toast(msg);
}
function frDone() {
  const live = FR.agents.filter((a) => ["seen", "written", "quiet"].includes(a.st)).length, left = undecided().length;
  frExit("Set up. " + live + " agents connected" + (left ? ", " + left + " guesses wait under tray: in system.yaml." : ".") + " This prototype now shows the plant later in the day.");
}
function frClickEnt(id) {
  const i = FR.items.findIndex((it) => it.id === id || (it.with || []).includes(id));
  if (FR.phase === "place" && i >= 0) { FR.i = i; frApply(); frRender(); }
}

$("#fr").addEventListener("click", (ev) => {
  const b = ev.target.closest("[data-fr]");
  if (b) {
    const d = b.dataset.fr;
    if (d === "yes" || d === "no") return frPick(d);
    if (d === "write-acan") return frWriteAcan();
    if (d === "done") return frDone();
    if (d === "skip-all") return frExit(FR.phase === "connect" ? "Setup stopped. .acan/ is written. Agents not connected yet." : "Setup skipped. Nothing was written.");
    return frAgent(d);
  }
  const card = ev.target.closest("[data-i]");
  if (card && !ev.target.closest("pre,summary")) { FR[FR.phase === "place" ? "i" : "ai"] = +card.dataset.i; frApply(); frRender(); }
});
addEventListener("keydown", (ev) => {
  if (!frOn() || ev.key === "Escape" || ev.key === "Alt" || typing(ev)) return;
  const k = ev.key.toLowerCase(), mod = ev.ctrlKey || ev.metaKey;
  if (!mod && !ev.altKey && ["f", "q", "e", "t", "l"].includes(k)) return;
  ev.stopImmediatePropagation();
  if (mod) {
    if (k === "z") { ev.preventDefault(); frUndo(); }
    if (k === "k") { ev.preventDefault(); toast("Go to opens once setup is done. Skip setup at the top of the panel."); }
    return;
  }
  const handled = () => ev.preventDefault();
  if (FR.phase === "read") { if (k === "enter") { handled(); frPhase("place"); } return; }
  if (k === "j" || k === "arrowdown") { handled(); return frMove(1); }
  if (k === "k" || k === "arrowup") { handled(); return frMove(-1); }
  if (FR.phase === "place") {
    if (k === "y") frPick("yes");
    else if (k === "n") frPick("no");
    else if (k === "enter" && (ev.shiftKey || !undecided().length)) frWriteAcan();
    else if (k === "enter") frPick("yes");
    else return;
    return handled();
  }
  if (FR.phase === "connect") {
    const open = FR.agents.some((a) => a.st === "found");
    if (k === "enter" && (ev.shiftKey || !open)) frDone();
    else if (k === "enter") frAgent("write");
    else if (k === "c") frAgent("copy");
    else if (k === "s") frAgent("skip");
    else return;
    return handled();
  }
}, true);
function frInit(params) { if (params.get("first")) frStart(); }
`;
