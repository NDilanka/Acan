export const PAL_HTML = `
<div class="pal" id="pal" role="dialog" aria-label="Go to">
  <div class="pal-box">
    <div class="pal-in"><input id="palIn" type="text" placeholder="Go to a machine, agent, choice or file" autocomplete="off" spellcheck="false"><kbd>Esc</kbd></div>
    <div class="pal-list" id="palList" role="listbox"></div>
    <div class="pal-ft"><span><kbd>↑</kbd><kbd>↓</kbd> move</span><span><kbd>Enter</kbd> go</span><span><kbd>Tab</kbd> next machine by severity</span></div>
  </div>
</div>`;

export const PAL_CSS = `
.pal{position:fixed;inset:0;z-index:20;display:none;align-items:flex-start;justify-content:center;padding-top:96px;background:rgba(0,0,0,.28)}
.pal.open{display:flex}
.pal-box{width:600px;max-width:calc(100vw - 32px);border-radius:14px;background:var(--glass-hi);backdrop-filter:blur(22px);-webkit-backdrop-filter:blur(22px);border:1px solid var(--line2);box-shadow:var(--shadow);overflow:hidden}
.pal-in{display:flex;align-items:center;gap:10px;padding:0 14px;border-bottom:1px solid var(--line)}
.pal-in input{flex:1;height:46px;background:none;border:0;outline:0;color:#f2f2f2;font:14px var(--sans)}
.pal-in input::placeholder{color:var(--faint)}
.pal-list{max-height:404px;overflow:auto;padding:6px;scrollbar-width:thin;scrollbar-color:#333 transparent}
.pal-h{padding:8px 8px 4px;font:9.5px var(--mono);letter-spacing:.12em;text-transform:uppercase;color:var(--faint)}
.pal-row{display:grid;grid-template-columns:28px 1fr auto;align-items:center;gap:10px;width:100%;min-height:38px;padding:4px 8px;border-radius:9px;text-align:left;color:#d6d6d6}
.pal-row[aria-selected=true]{background:rgba(255,255,255,.07);color:#fff}
.pal-row .ic{width:24px;height:20px;border-radius:5px;display:grid;place-items:center;font:600 9.5px var(--mono);background:#2a2a2a;color:#bbb}
.pal-row .ic[data-state=fail]{background:var(--fail);color:#1a0703}.pal-row .ic[data-state=strain]{background:var(--strain);color:#1d1404}.pal-row .ic[data-state=wait]{background:var(--wait);color:#1d1404}
.pal-row .ic[data-state=ghost]{background:none;border:1px dashed #7a8fa3;color:#a9bccd}.pal-row .ic[data-state=ok]{background:#cfe0d4;color:#0b100d}
.pal-row .tx{min-width:0;display:flex;flex-direction:column;gap:1px}
.pal-row .tx b{font-weight:500;font-size:13px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.pal-row .tx b mark{background:none;color:#fff;text-decoration:underline;text-underline-offset:2px}
.pal-row .tx small{font:10.5px var(--mono);color:var(--dim);white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.pal-row .kd{font:10px var(--mono);color:var(--faint);white-space:nowrap}
.pal-empty{padding:18px 10px;color:var(--dim);font-size:12.5px}
.pal-ft{display:flex;gap:16px;padding:8px 14px;border-top:1px solid var(--line);font-size:11px;color:var(--faint)}
.pal-ft kbd{margin-right:3px}
`;

export const PAL_SCRIPT = `
const PAL = { items: [], shown: [], i: 0 };
const KIND_LABEL = { machine: "Machines", agent: "Agents", choice: "Choices", territory: "Territories", file: "Files", mode: "Modes", action: "Actions" };
function palIndex() {
  const out = [], hidden = rpOn() ? hiddenAt(RP.t / 60 - RPD.mins) : [];
  for (const e of SCENE.entities) {
    if (["scaffold", "inspection"].includes(e.kind) || hidden.includes(e.id)) continue;
    const s = BOT_SESSION[e.id] && SESSIONS.find((x) => x.id === BOT_SESSION[e.id]);
    if (s) continue;
    out.push({ kind: "machine", label: e.name, sub: (GHOST[e.id] ? "ghost · " : "") + e.reading + " · " + ASSETS[e.kind].label.toLowerCase(), ic: e.mono, state: e.state, sev: SEVERITY[e.state] ?? 5, fitting: ASSETS[e.kind].family === "fitting", run: () => select(e.id) });
  }
  for (const s of SESSIONS) out.push({ kind: "agent", label: s.agent + " · " + s.goal, sub: STATE_LABEL[s.state].toLowerCase() + " · " + s.since + (s.territory ? " · " + s.territory : ""), ic: s.agent === "Codex" ? "CX" : s.agent[0], state: s.state === "blocked" ? "wait" : s.state === "done" ? "idle" : "ok", sev: s.state === "blocked" ? 2 : 6, run: () => openSession(s.id) });
  for (const c of CH) if (c.status === "open" || c.status === "later") out.push({ kind: "choice", label: c.question, sub: c.by + " · confidence " + c.conf.toFixed(2) + " · " + ENT[c.on].name, ic: "?", state: c.conf < 0.75 ? "wait" : "ok", sev: 1.5 + c.conf, run: () => { if (document.body.dataset.mode !== "live") setMode("live"); openReview(c.id); } });
  for (const [id, t] of Object.entries(TERR)) out.push({ kind: "territory", label: id, sub: t.goal + " · " + t.state, ic: "T", state: "idle", sev: 6, run: () => select(t.entities.find((x) => ENT[x] && !hidden.includes(x)) || t.entities[0]) });
  const files = new Map();
  for (const s of SESSIONS) for (const [f, id] of s.files) files.set(f, id);
  for (const d of Object.entries(INSPECT)) for (const x of d[1].drift) files.set(x.ref, d[0]);
  for (const c of CH) for (const r of c.evidence) if (r.includes("/")) files.set(r, c.on);
  for (const [f, id] of files) out.push({ kind: "file", label: f, sub: "opens in your editor · maps to " + ENT[id].name, ic: "{}", state: "idle", sev: 7, run: () => handoff("editor", ["code", "--goto", f], "Opened " + f + " in your editor.") });
  for (const [m, n] of [["live", 1], ["blueprint", 2], ["simulate", 3], ["replay", 4]]) out.push({ kind: "mode", label: m[0].toUpperCase() + m.slice(1), sub: "mode", key: String(n), ic: String(n), state: "idle", sev: 8, run: () => setMode(m) });
  const ctx = context();
  if (ctx) for (const id of ["open", "hand", "brief", "brief-safe"]) out.push({ kind: "action", label: ACTIONS[id].label + " · " + nameOf(ctx), sub: "on the current selection", key: ACTIONS[id].key, ic: "›", state: "idle", sev: 1.5, run: () => runAction(id, ctx) });
  out.push({ kind: "action", label: "Walk the choices", sub: "riskiest first", key: "W", ic: "›", state: "idle", sev: 8, run: () => { if (document.body.dataset.mode !== "live") setMode("live"); walk(); } });
  out.push({ kind: "action", label: "Floor timeline", sub: "agents, CI and you over the last hour", key: "G", ic: "›", state: "idle", sev: 8, run: () => toggleFloor() });
  out.push({ kind: "action", label: document.body.classList.contains("fogged") ? "Hide fog" : "Show fog", sub: "layer · what Orrery doesn't know yet, per machine", ic: "›", state: "idle", sev: 8, run: () => fogToggle() });
  out.push({ kind: "action", label: "Export a picture", sub: "PNG, SVG or frames · share-safe by default", ic: "›", state: "idle", sev: 8, run: () => exToggle(true) });
  out.push({ kind: "action", label: "Network log", sub: "every outbound connection · offline switch", ic: "›", state: "idle", sev: 8, run: () => netToggle(true) });
  out.push({ kind: "action", label: NET.off ? "Go online" : "Go offline", sub: "blocks all egress · loopback keeps working", ic: "›", state: "idle", sev: 8, run: () => setOffline(!NET.off) });
  out.push({ kind: "action", label: "Set up a repo", sub: "first run · read, place, connect agents", ic: "›", state: "idle", sev: 8, run: frStart });
  out.push({ kind: "action", label: "Fit the plant", sub: "view", key: "F", ic: "›", state: "idle", sev: 8, run: fit });
  return out;
}
function score(q, text) {
  const t = text.toLowerCase();
  const at = t.indexOf(q);
  if (at >= 0) return { s: 100 - at + (at === 0 || /[\\s·/._-]/.test(t[at - 1]) ? 40 : 0), hit: [at, at + q.length] };
  let j = 0, gaps = 0, last = -1;
  for (let i = 0; i < t.length && j < q.length; i++) if (t[i] === q[j]) { if (last >= 0) gaps += i - last - 1; last = i; j++; }
  return j === q.length ? { s: 40 - Math.min(39, gaps), hit: null } : null;
}
function palRender() {
  const q = $("#palIn").value.trim().toLowerCase();
  let rows;
  if (!q) {
    const needs = PAL.items.filter((x) => (x.kind === "machine" && x.sev <= 2 && !x.fitting) || (x.kind === "agent" && x.sev === 2) || x.kind === "choice").sort((a, b) => a.sev - b.sev).slice(0, 7);
    rows = [...needs.map((x) => ({ ...x, group: "Needs you" })), ...PAL.items.filter((x) => x.kind === "action" && x.sev < 2).map((x) => ({ ...x, group: "Actions" }))];
  } else {
    rows = PAL.items.map((x) => { const m = score(q, x.label) || (score(q, x.sub) && { s: score(q, x.sub).s - 30, hit: null }); return m && { ...x, s: m.s - x.sev, hit: m.hit, group: KIND_LABEL[x.kind] }; }).filter(Boolean).sort((a, b) => b.s - a.s).slice(0, 12);
  }
  PAL.shown = rows; PAL.i = Math.min(PAL.i, Math.max(0, rows.length - 1));
  let h = "", g = null;
  rows.forEach((x, i) => {
    if (x.group !== g && !q) { g = x.group; h += '<div class="pal-h">' + g + "</div>"; }
    const lb = x.hit ? x.label.slice(0, x.hit[0]) + "<mark>" + x.label.slice(x.hit[0], x.hit[1]) + "</mark>" + x.label.slice(x.hit[1]) : x.label;
    h += '<button class="pal-row" role="option" data-i="' + i + '" aria-selected="' + (i === PAL.i) + '"><span class="ic" data-state="' + x.state + '">' + x.ic + '</span><span class="tx"><b>' + lb + "</b><small>" + (q ? x.kind + " · " : "") + x.sub + '</small></span><span class="kd">' + (x.key ? "<kbd>" + x.key + "</kbd>" : "") + "</span></button>";
  });
  $("#palList").innerHTML = h || '<div class="pal-empty">Nothing matches "' + q.replace(/[<>&]/g, "") + '". Try a machine name, an agent, a file or a mode.</div>';
  const cur = $('#palList [aria-selected="true"]'); if (cur) cur.scrollIntoView({ block: "nearest" });
}
function palOpen() { PAL.items = palIndex(); PAL.i = 0; $("#palIn").value = ""; $("#pal").classList.add("open"); palRender(); $("#palIn").focus(); }
function palClose() { $("#pal").classList.remove("open"); $("#palIn").blur(); }
function palToggle() { $("#pal").classList.contains("open") ? palClose() : palOpen(); }
function palGo(i = PAL.i) { const x = PAL.shown[i]; if (!x) return; palClose(); x.run(); }
$("#palIn").addEventListener("input", () => { PAL.i = 0; palRender(); });
$("#palIn").addEventListener("keydown", (ev) => {
  if (ev.key === "ArrowDown") { PAL.i = Math.min(PAL.shown.length - 1, PAL.i + 1); palRender(); ev.preventDefault(); }
  if (ev.key === "ArrowUp") { PAL.i = Math.max(0, PAL.i - 1); palRender(); ev.preventDefault(); }
  if (ev.key === "Enter") { palGo(); ev.preventDefault(); }
  if (ev.key === "Escape") palClose();
  if (!((ev.ctrlKey || ev.metaKey) && ev.key.toLowerCase() === "k")) ev.stopPropagation();
});
$("#palList").addEventListener("click", (ev) => { const b = ev.target.closest("[data-i]"); if (b) palGo(+b.dataset.i); });
$("#pal").addEventListener("pointerdown", (ev) => { if (ev.target.id === "pal") palClose(); });
`;
