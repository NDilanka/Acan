import { REPLAY, SESSIONS, CI, TERRITORIES } from "./acquit.mjs";

const SKIP_MARKS = new Set(["fail", "pass"]);
const sessionEvents = SESSIONS.flatMap((s) => {
  const on = s.bot || TERRITORIES[s.territory].entities[0], start = Math.min(...s.lane.map((l) => l[0]));
  const out = start > -REPLAY.mins ? [{ t: start, kind: "agent", on, text: s.agent + " starts · " + s.goal }] : [];
  for (const [t, kind, text] of s.marks) if (!SKIP_MARKS.has(kind)) out.push({ t, kind: kind.startsWith("choice") ? "choice" : kind, on, text: s.agent + " · " + text });
  return out;
});
const ciEvents = CI.runs.map((r) => ({ t: r.status === "running" ? r.t0 : r.t1, kind: "ci", on: TERRITORIES[r.what.split(" ")[0]].entities[0], text: "CI #" + r.n + " " + r.status + " · " + r.what }));
const EVENTS = [...sessionEvents, ...ciEvents, ...REPLAY.changes.map((c) => ({ ...c, on: c.touches[0], change: true })), ...REPLAY.extra];

export const RP_HTML = `
<style id="rpHide"></style>
<div class="fl simdeck rpdeck" id="rpdeck">
  <div class="rp-grid">
    <button class="sd-play" id="rpPlay" title="Play (Space)"><svg viewBox="0 0 12 12" width="12" height="12"><path id="rpIcon" d="M3 1.5v9l7.5-4.5z" fill="currentColor"/></svg></button>
    <div class="sd-clock"><b id="rpTime">16:30:00</b><small>recorded</small></div>
    <svg class="sd-scrub" id="rpScrub" viewBox="0 0 300 34" preserveAspectRatio="none"></svg>
    <div class="rp-r"><button class="sd-chip" id="rpSpeed" title="Playback speed">60×</button><button class="sd-chip" id="rpNowBtn" title="Jump to now (End)">Now</button></div>
    <div class="rp-lanes" id="rpLanes"></div>
  </div>
  <div class="rp-ev" id="rpEv"></div>
  <div class="sd-res" id="rpRes"></div>
</div>`;

export const RP_CSS = `
body[data-mode=replay] .ent[data-kind=crane],body[data-mode=replay] .ent[data-kind=scaffold],body[data-mode=replay] .ent[data-kind=inspection],
body[data-mode=replay] .terr,body[data-mode=replay] .pins,body[data-mode=replay] .threads,body[data-mode=replay] .floor,body[data-mode=replay] .br,body[data-mode=replay] .bl,body[data-mode=replay] .hint{display:none!important}
body[data-mode=replay] .chip:not(.rp):not(.sel):not(.hover){opacity:0!important}
body[data-mode=replay] .chip.prop::after{content:none}
body[data-mode=replay] .chip.rp:not(.loud):not(.hover):not(.sel){opacity:0;translate:0 4px}
body[data-mode=replay] .chip.rp.loud,body[data-mode=replay] .chip.rp.hover,body[data-mode=replay] .chip.rp.sel,body[data-mode=replay].alt .chip.rp{opacity:1;translate:0 0}
body[data-mode=replay] .chip.rp .rd{display:inline}
.chip.rpfirst::after{content:"failed first";position:absolute;left:50%;bottom:calc(100% + 4px);transform:translateX(-50%);font:600 9px var(--mono);letter-spacing:.1em;color:var(--fail);text-transform:uppercase}
body[data-mode=replay] .belt .iso-line[data-flow]{stroke:var(--anatomy-hi);animation-duration:var(--fd,1600ms)}
body[data-mode=replay] .belt.warm .iso-line[data-flow]{stroke:var(--strain)}
body[data-mode=replay] .belt.hot .iso-line[data-flow]{stroke:var(--fail)}
body[data-mode=replay].still .belt .iso-line[data-flow],body[data-mode=replay].paused .belt .iso-line[data-flow]{animation-play-state:paused}

.rpdeck{display:none!important}
body[data-mode=replay] .rpdeck{display:flex!important}
.rpdeck .sd-play{background:#e3d3b8;color:#120e0a}
.rpdeck .sd-clock b{color:#f4ead8}
.rp-grid{display:grid;grid-template-columns:30px 62px 1fr auto;column-gap:10px;row-gap:3px;align-items:center}
.rp-r{display:flex;gap:6px}
.rpdeck .sd-scrub{width:100%;min-width:0}
.rpdeck .sd-scrub .area{fill:rgba(227,211,184,.1);stroke:rgba(227,211,184,.45)}
.rpdeck .sd-scrub .done{fill:rgba(227,211,184,.06)}
.rpdeck .sd-scrub .ph{stroke:#f4ead8}
.sd-scrub .ev{stroke:none}
.rp-lanes{grid-column:1/-1;display:grid;grid-template-columns:subgrid;row-gap:2px}
.rp-ln{grid-column:1/3;justify-self:end;align-self:center;font:9.5px/8px var(--mono);color:var(--dim)}
.rp-ln:hover{color:#eee}
.rp-tr{grid-column:3;display:block;width:100%;height:7px;cursor:ew-resize}
.rp-tr .bg{fill:rgba(255,255,255,.04)}
.rp-tr .strain{fill:var(--strain)}.rp-tr .fail{fill:var(--fail)}
.rp-tr .ph{stroke:#f4ead8;stroke-width:1;vector-effect:non-scaling-stroke}
[data-k=break]{--k:var(--fail)}[data-k=recover]{--k:#9fd3b0}[data-k=deploy]{--k:#d9b26a}[data-k=agent],[data-k=claim]{--k:#e3d3b8}[data-k=ci]{--k:#9cc7ee}
[data-k=choice],[data-k=wait]{--k:var(--wait)}[data-k=ghost]{--k:#cfc6b6}[data-k=seal]{--k:#c9a24e}[data-k=stale]{--k:#8a8a8a}
.sd-scrub .ev{fill:var(--k)}
.rp-ev{display:flex;align-items:center;gap:8px;min-height:24px;font-size:12px;color:#d6d6d6}
.rp-ev button.nv{flex:none;width:24px;height:22px;border-radius:6px;border:1px solid var(--line2);color:#cfcfcf;font:12px var(--mono)}
.rp-ev button.nv:hover{background:rgba(255,255,255,.05)}
.rp-ev b{font:500 11px var(--mono);color:#f4ead8;font-variant-numeric:tabular-nums}
.rp-ev i{flex:none;width:7px;height:7px;border-radius:50%;background:var(--k)}
.rp-ev .tx{flex:1;min-width:0;text-align:left;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.rp-ev .tx:hover{color:#fff}
.rp-ev kbd{flex:none}
.rpdeck .sd-res button{color:#e3d3b8}

.rp-now{display:flex;align-items:baseline;gap:10px}
.rp-now b{font:400 24px var(--mono);color:#f2f2f2;font-variant-numeric:tabular-nums}
.rp-now span{font-size:12px;color:var(--dim)}
.rp-now[data-state=strain] b{color:#f1d5a2}.rp-now[data-state=fail] b{color:#f0a796}
.rp-now em{margin-left:auto;font:normal 10.5px var(--mono);color:var(--faint)}
.rp-curve{display:block;width:100%;height:82px;overflow:visible;cursor:ew-resize}
.rp-curve .fb{fill:rgba(232,96,74,.1)}
.rp-curve path{fill:none;stroke:#e3d3b8;stroke-width:1.1;vector-effect:non-scaling-stroke}
.rp-curve .bud{stroke:#8a8a8a;stroke-dasharray:3 3;stroke-width:.8;vector-effect:non-scaling-stroke}
.rp-curve .ph{stroke:#f4ead8;stroke-width:1;vector-effect:non-scaling-stroke}
.rp-curve .ev{fill:var(--k)}
.rp-curve text{font:9px var(--mono);fill:var(--faint)}
.cause{padding:7px 0;border-top:1px solid var(--line);font-size:12px;color:#d6d6d6;line-height:1.45}
.cause:first-of-type{border-top:0;padding-top:2px}
.cause b{font-weight:500;color:#f2f2f2}
.chg{display:grid;grid-template-columns:auto 1fr auto;gap:2px 10px;align-items:baseline;padding:7px 0;border-top:1px solid var(--line);font-size:12px;color:#d6d6d6}
.chg>b{font:500 11px var(--mono);color:#f4ead8}
.chg small{grid-column:2/-1;font:10px var(--mono);color:var(--faint)}
.chg .tag{font:9.5px var(--mono);text-transform:uppercase;letter-spacing:.06em;color:var(--dim);border:1px solid var(--line2);border-radius:5px;padding:1px 5px}
.chg .tag.on{color:#f0a796;border-color:rgba(232,96,74,.45)}
.evrow{display:flex;gap:10px;width:100%;padding:5px 0;border-top:1px solid var(--line);font-size:12px;color:#d6d6d6;text-align:left}
.evrow:first-of-type{border-top:0}
.evrow b{flex:none;font:500 11px var(--mono);color:#f4ead8;font-variant-numeric:tabular-nums}
.evrow i{flex:none;width:7px;height:7px;margin-top:5px;border-radius:50%;background:var(--k)}
.evrow:hover{color:#fff}
`;

export const RP_SCRIPT = `
const RPD = ${JSON.stringify({ ...REPLAY, events: EVENTS })};
const RP = { t: 0, playing: false, speed: 60, timer: 0, saved: null, visited: false, frames: [], onset: {}, recover: {}, lanes: [], events: [], first: null, curve: null };
const RP_SECS = RPD.mins * 60;
const rpOn = () => document.body.dataset.mode === "replay";
const toSec = (m) => (m + RPD.mins) * 60;
const rpClock = (sec) => { const s = RPD.now - RP_SECS + Math.round(sec); return [s / 3600, (s / 60) % 60, s % 60].map((v) => String(Math.floor(v)).padStart(2, "0")).join(":"); };
const rpName = (id) => (ENT[id] ? ENT[id].name : id);
const fmtMs = (v) => (v >= 1000 ? (v / 1000).toFixed(1) + " s" : Math.round(v) + " ms");
function interp(keys, m, step) {
  let i = 0; while (i < keys.length - 2 && keys[i + 1][0] <= m) i++;
  const [a, va] = keys[i], [b, vb] = keys[i + 1];
  if (step || m <= a) return m < b ? va : vb;
  return va + (vb - va) * Math.min(1, (m - a) / (b - a));
}
function rpState(s, v, err) {
  if ((s.fail != null && v >= s.fail) || (s.errFail && err >= s.errFail)) return "fail";
  if ((s.strain != null && v >= s.strain) || (s.errBudget && err > s.errBudget)) return "strain";
  return "ok";
}
function rpReading(id, st) {
  const s = RPD.series[id], v = st.v;
  let r = s.fmt.replace("{i}", Math.round(v)).replace("{1}", v.toFixed(1)).replace("{2}", v.toFixed(2)).replace("{ms}", fmtMs(v));
  if (st.err > (s.errBudget || Infinity)) r += " · " + st.err.toFixed(0) + "% err";
  return r;
}
function rpBuild() {
  const r = rng(RPD.seed), n = RP_SECS / RPD.dt, prev = {};
  for (let i = 0; i <= n; i++) {
    const sec = i * RPD.dt, m = sec / 60 - RPD.mins, M = {};
    for (const [id, s] of Object.entries(RPD.series)) {
      let v = interp(s.keys, m, s.step);
      if (s.noise) v *= 1 + (r() - 0.5) * 2 * s.noise;
      const err = s.err ? interp(s.err, m) * (1 + (r() - 0.5) * 0.06) : 0, state = rpState(s, v, err);
      M[id] = { v, err, state };
      if (state === "fail" && prev[id] !== "fail") { if (!(id in RP.onset)) RP.onset[id] = sec; RP.events.push({ sec, kind: "break", on: id, text: rpName(id) + " starts failing" }); }
      if (prev[id] === "fail" && state !== "fail") { RP.recover[id] = sec; RP.events.push({ sec, kind: "recover", on: id, text: rpName(id) + " recovers" }); }
      prev[id] = state;
    }
    RP.frames.push({ sec, rps: interp(RPD.rps, m) * (1 + (r() - 0.5) * 0.1), M });
  }
  for (const id of Object.keys(RP.recover)) if (RP.frames[RP.frames.length - 1].M[id].state === "fail") delete RP.recover[id];
  RP.first = Object.entries(RP.onset).sort((a, b) => a[1] - b[1])[0]?.[0] || null;
  for (const e of RP.events) if (e.kind === "break" && e.on === RP.first && e.sec === RP.onset[RP.first]) e.text += " · first in the plant";
  for (const e of RPD.events) RP.events.push({ ...e, sec: toSec(e.t) });
  RP.events.sort((a, b) => a.sec - b.sec);
  const firstOff = (id) => RP.frames.findIndex((f) => f.M[id].state !== "ok");
  RP.lanes = Object.keys(RPD.series).filter((id) => firstOff(id) >= 0).sort((a, b) => firstOff(a) - firstOff(b));
}
const rpFrame = (t = RP.t) => RP.frames[Math.min(RP.frames.length - 1, Math.max(0, Math.round(t / RPD.dt)))];
function botAt(id, m) {
  const s = SESSIONS.find((x) => x.bot === id); if (!s) return null;
  const lane = [...s.lane].sort((a, b) => a[0] - b[0]);
  if (m < lane[0][0]) return null;
  const seg = lane.find((l) => m >= l[0] && m < l[1]) || lane[lane.length - 1];
  return { state: seg[2] === "wait" ? "wait" : "ok", text: seg[3] };
}
function hiddenAt(m) {
  const out = Object.entries(RPD.appear).filter(([, t]) => m < t).map(([id]) => id);
  for (const s of SESSIONS) if (s.bot && !botAt(s.bot, m)) out.push(s.bot);
  return out;
}

function rpApply() {
  if (!rpOn() || !RP.frames.length) return;
  const f = rpFrame(), m = RP.t / 60 - RPD.mins;
  let changed = false;
  const setState = (id, st) => { const e = ENT[id]; if (e && e.state !== st) { e.state = st; changed = true; } };
  for (const id of Object.keys(RPD.series)) setState(id, f.M[id].state);
  for (const s of SESSIONS) if (s.bot) { const b = botAt(s.bot, m); if (b) setState(s.bot, b.state); }
  const hide = hiddenAt(m);
  $("#rpHide").textContent = hide.map((id) => '.ent[data-id="' + id + '"],.chip[data-id="' + id + '"],.belt[data-to="' + id + '"]').join(",") + (hide.length ? "{display:none!important}" : "");
  if (changed) { dirtyScene = true; animate(); } else rpCanvas();
  $("#rpTime").textContent = rpClock(RP.t);
  drawRpScrub(); paintEv();
  if (selected && $("#rpNow")) paintRpNow(selected);
}
function rpCanvas() {
  if (!rpOn() || !RP.frames.length) return;
  const f = rpFrame(), m = RP.t / 60 - RPD.mins, base = RPD.rps[0][1];
  for (const c of document.querySelectorAll(".chip")) {
    const id = c.dataset.id, s = RPD.series[id], bot = botAt(id, m), live = s || bot || GHOST[id];
    c.classList.toggle("rp", !!live);
    if (!live) continue;
    if (s) { c.dataset.state = f.M[id].state; c.querySelector(".rd").textContent = rpReading(id, f.M[id]); }
    if (bot) { c.dataset.state = bot.state; c.querySelector(".rd").textContent = bot.text; }
    c.classList.toggle("loud", !["ok", "idle"].includes(c.dataset.state));
    c.classList.toggle("rpfirst", id === RP.first && RP.t >= RP.onset[id] && RP.t < (RP.recover[id] ?? Infinity));
  }
  for (const b of plant.querySelectorAll(".belt")) {
    const to = f.M[b.dataset.to];
    b.style.setProperty("--fd", Math.round(1600 / Math.max(0.5, f.rps / base)) + "ms");
    b.classList.toggle("hot", !!to && to.state === "fail");
    b.classList.toggle("warm", !!to && to.state === "strain");
  }
}
function rpAfterRender() { if (rpOn() && RP.frames.length) rpCanvas(); }

const X3 = (sec) => (sec / RP_SECS) * 300;
function laneSegs(id) {
  let out = "", on = null, kind = null;
  RP.frames.forEach((f, i) => {
    const st = f.M[id].state, last = i === RP.frames.length - 1;
    if (on !== null && (st !== kind || last)) { out += '<rect class="' + kind + '" x="' + X3(on).toFixed(1) + '" y="0" width="' + Math.max(1, X3(f.sec) - X3(on)).toFixed(1) + '" height="7"/>'; on = null; }
    if (st !== "ok" && on === null && !last) { on = f.sec; kind = st; }
  });
  return out;
}
function rpLanesHtml() {
  $("#rpLanes").innerHTML = RP.lanes.map((id) => '<button class="rp-ln" data-go="' + id + '">' + rpName(id) + '</button><svg class="rp-tr" data-seek="1" viewBox="0 0 300 7" preserveAspectRatio="none"><rect class="bg" x="0" y="0" width="300" height="7"/>' + laneSegs(id) + '<line class="ph" y1="-1" y2="8"/></svg>').join("");
}
function drawRpScrub() {
  const H3 = 34, top = 8, max = Math.max(...RP.frames.map((f) => f.rps)) * 1.1;
  if (!RP.area) RP.area = "M0 " + H3 + RP.frames.map((f) => "L" + X3(f.sec).toFixed(1) + " " + (H3 - (f.rps / max) * (H3 - top - 4)).toFixed(1)).join("") + "L300 " + H3 + "Z";
  const ticks = RP.events.map((e) => '<circle class="ev" data-k="' + e.kind + '" cx="' + X3(e.sec).toFixed(1) + '" cy="3" r="' + (e.kind === "break" || e.change ? 2.4 : 1.7) + '"/>').join("");
  const x = X3(RP.t).toFixed(1);
  $("#rpScrub").innerHTML = '<path class="area" d="' + RP.area + '"/><rect class="done" x="0" y="' + top + '" width="' + x + '" height="' + (H3 - top) + '"/>' + ticks + '<line class="ph" x1="' + x + '" x2="' + x + '" y1="-2" y2="' + (H3 + 2) + '"/>';
  for (const l of document.querySelectorAll("#rpLanes .ph")) { l.setAttribute("x1", x); l.setAttribute("x2", x); }
}
const evAt = () => [...RP.events].reverse().find((e) => e.sec <= RP.t + 0.5);
function paintEv() {
  const e = evAt();
  $("#rpEv").innerHTML = '<button class="nv" data-ev="-1" title="Previous event ([)">‹</button>' + (e ? '<b>' + rpClock(e.sec) + '</b><i data-k="' + e.kind + '"></i><button class="tx" data-go="' + e.on + '">' + e.text + "</button>" : '<span class="tx" style="color:var(--faint)">No events yet in this window</span>') + '<button class="nv" data-ev="1" title="Next event (])">›</button><kbd>[</kbd><kbd>]</kbd>';
}
function story() {
  const F = RP.first; if (!F) return { html: "<b>Nothing failed in this hour.</b>", what: ["Nothing failed in " + RPD.window + "."] };
  const t0 = RP.onset[F], fr = RP.frames, peak = (id) => Math.max(...fr.map((f) => f.M[id].v));
  const followers = Object.entries(RP.onset).filter(([id]) => id !== F).sort((a, b) => a[1] - b[1]);
  const before = RPD.changes.filter((c) => toSec(c.t) < t0 && toSec(c.t) >= t0 - 600);
  const touching = before.filter((c) => c.touches.includes(F));
  const ago = (s) => (s < 90 ? Math.round(s) + " s" : Math.round(s / 60) + " min");
  let h = '<b class="bad">' + rpName(F) + " starts failing at " + rpClock(t0) + ", first in the plant.</b> ";
  if (followers.length) h += followers.map(([id, s], i) => rpName(id) + (i ? " in " : " follows in ") + ago(s - t0)).join(", ") + ". ";
  const back = followers.filter(([id]) => RP.recover[id] != null);
  if (back.length) { const last = Math.max(...back.map(([id]) => RP.recover[id])); h += (back.length === 1 ? rpName(back[0][0]) + " recovers at " : back.length === 2 ? "Both recover by " : "They recover by ") + rpClock(last).slice(0, 5) + ". "; }
  if (before.length && !touching.length) h += before.length === 1 ? "The only change in the 10 min before, " + before[0].text + ", touched " + before[0].touches.map(rpName).join(" and ") + ", not " + rpName(F) + ". " : before.length + " changes in the 10 min before, none touching " + rpName(F) + ". ";
  else if (!before.length) h += "Nothing in the plant changed in the 10 min before. ";
  const s = RPD.series[F], now = fr[fr.length - 1].M[F];
  const lines = [
    "At " + rpClock(t0) + " " + rpName(F) + " started failing. It is first in the plant. Peak " + rpReading(F, { v: peak(F), err: 0 }) + ", budget " + s.budget + s.unit + ".",
    followers.length ? "Followers: " + followers.map(([id, t]) => rpName(id) + " +" + ago(t - t0) + (RP.recover[id] ? " (recovered " + rpClock(RP.recover[id]) + ")" : "") + (RPD.because[id] ? ". " + RPD.because[id] : "")).join(" ") : "",
    before.length ? "Changes in the 10 min before: " + before.map((c) => c.text + " (" + c.detail + "), touches " + c.touches.map(rpName).join(", ") + (c.touches.includes(F) ? "" : ", not " + rpName(F))).join("; ") + "." : "Changes in the 10 min before: none.",
    RPD.outside[F] && !touching.length ? "Likely outside the plant: " + RPD.outside[F] + " (guess, no plant change touches it)." : "",
    "Now (" + rpClock(RP_SECS) + "): " + rpName(F) + " " + rpReading(F, now) + (now.state === "fail" ? ", still failing." : "."),
  ].filter(Boolean);
  const nowTxt = now.state === "fail" ? rpName(F) + " still failing<br>" + rpReading(F, now) + " · budget " + s.budget + s.unit : "recovered";
  return { html: "<p>" + h + '<button data-goto="first">Go there</button> · <button data-brief="1">Copy brief</button> <kbd>C</kbd></p><small>' + nowTxt + "</small>", what: ["Recorded " + RPD.window + ".", ...lines] };
}
function copyBrief() { runAction("brief", selected && RPD.series[selected] ? { kind: "entity", id: selected } : { kind: "entity", id: RP.first }); }
function rpSeek(sec, stop = true) { RP.rew = 0; if (stop) rpPlay(false); RP.t = Math.max(0, Math.min(RP_SECS, sec)); rpApply(); }
function gotoFirst() { if (!RP.first) return; rpSeek(RP.onset[RP.first]); select(RP.first); }
function stepEv(d) {
  const e = d < 0 ? [...RP.events].reverse().find((x) => x.sec < RP.t - 0.5) : RP.events.find((x) => x.sec > RP.t + 0.5);
  if (e) rpSeek(e.sec);
}
function rpTick() { RP.t = Math.min(RP_SECS, RP.t + RP.speed * 0.2); if (RP.t >= RP_SECS) rpPlay(false); rpApply(); }
function rpPlay(on = !RP.playing) {
  if (on && RP.t >= RP_SECS) RP.t = 0;
  RP.playing = on; clearInterval(RP.timer);
  if (on) RP.timer = setInterval(rpTick, 200);
  document.body.classList.toggle("paused", !on);
  $("#rpIcon").setAttribute("d", on ? "M2.5 1.5h2.6v9H2.5zM6.9 1.5h2.6v9H6.9z" : "M3 1.5v9l7.5-4.5z");
}
function rewind(to) {
  const from = RP.t, t0 = performance.now(), dur = 1200, tok = (RP.rew = t0);
  const step = (now) => { if (RP.rew !== tok) return; const p = Math.min(1, (now - t0) / dur), e = 1 - Math.pow(1 - p, 3); RP.t = from + (to - from) * e; rpApply(); if (p < 1 && rpOn() && !RP.playing) requestAnimationFrame(step); };
  requestAnimationFrame(step);
}
function rpMode(m) {
  if (m === "replay" && !RP.saved) {
    if (!RP.frames.length) { rpBuild(); rpLanesHtml(); }
    RP.saved = Object.fromEntries(SCENE.entities.map((e) => [e.id, e.state]));
    if (selected) select(null);
    $("#rpRes").innerHTML = story().html;
    document.body.classList.add("paused");
    if (!RP.visited) { RP.visited = true; RP.t = RP_SECS; rpApply(); if (RP.first) rewind(RP.onset[RP.first] - 20); }
    else rpApply();
  } else if (m !== "replay" && RP.saved) {
    rpPlay(false); document.body.classList.remove("paused");
    for (const e of SCENE.entities) if (e.id in RP.saved) e.state = RP.saved[e.id];
    RP.saved = null; $("#rpHide").textContent = "";
    for (const c of document.querySelectorAll(".chip")) { const e = ENT[c.dataset.id]; if (!e) continue; c.classList.remove("rp", "rpfirst"); c.dataset.state = e.state; c.querySelector(".rd").textContent = e.reading; }
    for (const b of plant.querySelectorAll(".belt")) b.classList.remove("hot", "warm");
    if (selected) select(null);
    dirtyScene = true; animate();
  }
}

function rpInspector(e) {
  const id = e.id, s = RPD.series[id], t0 = RP.onset[id], here = RP.events.filter((x) => x.on === id);
  let h = '<div class="fig" id="fig"><div class="iso" id="figPlant"></div><div class="read" id="rpRead"></div><div class="tools"><button data-do="open">Open code <kbd>O</kbd></button><button data-do="hand">Hand to agent <kbd>H</kbd></button></div></div>';
  h += '<div class="sec"><h5 id="rpAt">At</h5><div id="rpNow"></div></div>';
  h += '<div class="sec"><h5>Over the hour<i>' + (s.metric || "reading") + (s.budget ? " · budget dashed" : "") + '</i></h5><svg class="rp-curve" id="rpCurve" viewBox="0 0 364 82" preserveAspectRatio="none"></svg></div>';
  if (t0 != null) {
    h += '<div class="sec"><h5>Before it broke<i>changes in the 10 min before</i></h5>';
    if (id === RP.first) {
      const before = RPD.changes.filter((c) => toSec(c.t) < t0 && toSec(c.t) >= t0 - 600);
      h += '<div class="cause"><b>First in the plant</b> at ' + rpClock(t0) + ". Nothing failed before it.</div>";
      h += before.map((c) => '<div class="chg"><b>' + rpClock(toSec(c.t)) + "</b><span>" + c.text + '</span><span class="tag' + (c.touches.includes(id) ? " on" : "") + '">' + (c.touches.includes(id) ? "on this path" : "not on this path") + "</span><small>" + c.detail + " · touches " + c.touches.map(rpName).join(", ") + "</small></div>").join("");
      if (!before.some((c) => c.touches.includes(id))) h += '<div class="note">Nothing that changed touches ' + e.name + "." + (RPD.outside[id] ? " The cause is likely outside the plant, at " + RPD.outside[id] + ". This is a guess from the change log." : "") + "</div>";
    } else {
      h += '<div class="cause"><b>Follows ' + rpName(RP.first) + "</b> by " + Math.round(t0 - RP.onset[RP.first]) + " s. " + (RPD.because[id] || "") + "</div>";
    }
    h += '<div class="cause">' + (RP.recover[id] != null ? "Recovers at <b>" + rpClock(RP.recover[id]) + "</b>, after " + Math.round((RP.recover[id] - t0) / 60) + " min." : "<b>Still failing</b> at " + rpClock(RP_SECS) + ".") + "</div></div>";
  }
  if (here.length) h += '<div class="sec"><h5>Events here<i>' + here.length + "</i></h5>" + here.map((x) => '<button class="evrow" data-seek-to="' + x.sec + '"><b>' + rpClock(x.sec) + '</b><i data-k="' + x.kind + '"></i><span>' + x.text + "</span></button>").join("") + "</div>";
  h += '<div class="acts"><button data-rp="brief">Copy brief <kbd>C</kbd></button><button data-rp="link">Copy link</button>' + (RP.first && SM[RP.first] && t0 != null ? '<button data-rp="sim">Rerun in Simulate</button>' : "") + "</div>";
  return h;
}
function drawRpCurve(id) {
  const s = RPD.series[id], svg = $("#rpCurve"); if (!svg) return;
  const vals = RP.frames.map((f) => f.M[id].v), top = Math.max(...vals, s.budget || 0) * 1.12 || 1, Y = (v) => 62 - (v / top) * 56, X = (sec) => (sec / RP_SECS) * 364;
  let bands = "", on = null;
  RP.frames.forEach((f, i) => { const bad = f.M[id].state === "fail", last = i === RP.frames.length - 1; if (bad && on === null) on = f.sec; if (on !== null && (!bad || last)) { bands += '<rect class="fb" x="' + X(on).toFixed(1) + '" y="0" width="' + (X(f.sec) - X(on)).toFixed(1) + '" height="62"/>'; on = null; } });
  const line = '<path d="' + RP.frames.map((f, i) => (i ? "L" : "M") + X(f.sec).toFixed(1) + " " + Y(f.M[id].v).toFixed(1)).join("") + '"/>';
  const bud = s.budget ? '<line class="bud" x1="0" x2="364" y1="' + Y(s.budget).toFixed(1) + '" y2="' + Y(s.budget).toFixed(1) + '"/>' : "";
  const ev = RP.events.filter((x) => x.on === id).map((x) => '<circle class="ev" data-k="' + x.kind + '" cx="' + X(x.sec).toFixed(1) + '" cy="66" r="2"/>').join("");
  svg.innerHTML = bands + bud + line + ev + '<line class="ph" id="rpCph" y1="0" y2="62"/><text x="0" y="78">' + rpClock(0).slice(0, 5) + '</text><text x="364" y="78" text-anchor="end">' + rpClock(RP_SECS).slice(0, 5) + "</text>";
}
function paintRpNow(id) {
  const s = RPD.series[id], box = $("#rpNow"); if (!s || !box) return;
  const st = rpFrame().M[id];
  $("#rpAt").innerHTML = "At " + rpClock(RP.t) + "<i>recorded · per-second rollups</i>";
  $("#rpRead").innerHTML = rpReading(id, st) + "<small>recorded at " + rpClock(RP.t) + "</small>";
  const val = s.unit === "ms" ? fmtMs(st.v) : s.unit === "%" ? st.v.toFixed(1) + "%" : rpReading(id, st);
  const word = { ok: s.budget ? "within budget" : "normal", strain: s.budget ? "over budget" : "under strain", fail: "failing" }[st.state];
  box.innerHTML = '<div class="rp-now" data-state="' + st.state + '"><b>' + val + "</b><span>" + (s.metric || "") + " · " + word + "</span><em>exact</em></div>" + (st.err > (s.errBudget || Infinity) ? '<div class="note">Errors ' + st.err.toFixed(1) + "% against a " + s.errBudget + "% budget.</div>" : "");
  const ph = $("#rpCph"), x = ((RP.t / RP_SECS) * 364).toFixed(1);
  if (ph) { ph.setAttribute("x1", x); ph.setAttribute("x2", x); }
  $("#ih").dataset.state = st.state;
}
function rpSelected(id) { if ($("#rpNow")) { drawRpCurve(id); paintRpNow(id); $("#iKind").textContent = "Recorded · " + RPD.window; } }

$("#rpPlay").onclick = () => rpPlay();
$("#rpSpeed").onclick = () => { const o = [10, 60, 300]; RP.speed = o[(o.indexOf(RP.speed) + 1) % o.length]; $("#rpSpeed").textContent = RP.speed + "×"; };
$("#rpNowBtn").onclick = () => rpSeek(RP_SECS);
$("#rpEv").addEventListener("click", (ev) => { const n = ev.target.closest("[data-ev]"), g = ev.target.closest("[data-go]"); if (n) stepEv(+n.dataset.ev); else if (g && ENT[g.dataset.go]) select(g.dataset.go); });
$("#rpRes").addEventListener("click", (ev) => { if (ev.target.closest("[data-goto]")) gotoFirst(); if (ev.target.closest("[data-brief]")) copyBrief(); });
$("#rpLanes").addEventListener("click", (ev) => { const g = ev.target.closest("[data-go]"); if (g) select(g.dataset.go); });
for (const sel of ["#rpScrub", "#rpLanes"]) {
  const host = $(sel);
  const seek = (ev) => { const r = $("#rpScrub").getBoundingClientRect(); rpSeek(((ev.clientX - r.left) / r.width) * RP_SECS); };
  host.addEventListener("pointerdown", (ev) => { if (!ev.target.closest(".rp-tr,#rpScrub")) return; seek(ev); host.setPointerCapture(ev.pointerId); host.onpointermove = seek; });
  host.addEventListener("pointerup", () => { host.onpointermove = null; });
}
$("#ibody").addEventListener("pointerdown", (ev) => { const c = ev.target.closest("#rpCurve"); if (!c || !rpOn()) return; const r = c.getBoundingClientRect(); rpSeek(((ev.clientX - r.left) / r.width) * RP_SECS); });
$("#ibody").addEventListener("click", (ev) => {
  if (!rpOn()) return;
  const row = ev.target.closest("[data-seek-to]"); if (row) return rpSeek(+row.dataset.seekTo);
  const b = ev.target.closest("[data-rp]"); if (!b) return;
  if (b.dataset.rp === "brief") copyBrief();
  if (b.dataset.rp === "link") { copyText("orrery://acquit/replay?t=" + rpClock(RP.t) + (selected ? "&sel=" + selected : "")); toast("Link to " + rpClock(RP.t) + " copied."); }
  if (b.dataset.rp === "sim") { SIMS.down.add(RP.first); setMode("simulate"); toast(rpName(RP.first) + " is down on yesterday's recorded traffic, with the breaker ghost in."); }
});
addEventListener("keydown", (ev) => {
  if (!rpOn() || !bare(ev)) return;
  if (ev.key === " ") { ev.preventDefault(); rpPlay(); }
  if (ev.key === "[") stepEv(-1);
  if (ev.key === "]") stepEv(1);
  if (ev.key === ",") rpSeek(RP.t - 60);
  if (ev.key === ".") rpSeek(RP.t + 60);
  if (ev.key === "ArrowLeft") rpSeek(RP.t - 30);
  if (ev.key === "ArrowRight") rpSeek(RP.t + 30);
  if (ev.key === "End") rpSeek(RP_SECS);
  if (ev.key === "Home") rpSeek(0);
});
function rpInit(params) {
  if (!rpOn()) return;
  if (params.get("t")) rpSeek(+params.get("t"));
  if (params.get("rsel")) select(params.get("rsel"));
}
`;
