import { SESSIONS, TERRITORIES, TSTATES, CI } from "./acquit.mjs";

const CRANE_ICON = `<svg viewBox="0 0 22 16" width="22" height="15" fill="none" stroke="currentColor" stroke-width=".9"><path d="M3 15V3M19 15V3M2 3h18M2 5h18"/><rect x="9" y="2" width="4" height="4" fill="#1a1a1a"/><path d="M11 6v4"/><rect x="9" y="10" width="4" height="3.4" fill="#2c2c2c"/></svg>`;

const at = (t) => (((t + 60) / 60) * 100).toFixed(2) + "%";
const mark = ([t, kind, label]) => `<i class="mk" data-k="${kind}" style="left:${at(t)}" data-tip="${label}"></i>`;
const seg = ([t0, t1, kind, label]) => `<b class="sg" data-k="${kind}" style="left:${at(t0)};width:${(((t1 - t0) / 60) * 100).toFixed(2)}%" data-tip="${label}"></b>`;
const YOU = {
  lane: [[-22, -20, "wait1", "1 waiting · choice from Droid"], [-20, -14, "wait2", "2 waiting · Droid proposed the breaker"], [-14, -9, "wait3", "3 waiting · second choice"], [-9, 0, "wait4", "4 waiting · Codex blocked"]],
  marks: [],
};
const lane = (who, sub, data, attrs = "") => `<div class="lane"${attrs}><div class="who">${who}<small>${sub}</small></div><div class="track">${data.lane.map(seg).join("")}${data.marks.map(mark).join("")}</div></div>`;

export function floorHtml(DRONE_ICON) {
  const ticks = [-60, -50, -40, -30, -20, -10, 0].map((t) => `<span style="left:${at(t)}">${t ? t + "m" : "now"}</span>`).join("");
  const lanes = SESSIONS.map((s) => lane(`${DRONE_ICON}<strong>${s.agent}</strong>`, s.goal, s, ` data-session="${s.id}" data-state="${s.state}"`)).join("")
    + lane(`${CRANE_ICON}<strong>CI</strong>`, "crane · " + CI.runner, { lane: CI.runs.map((r) => [r.t0, r.t1, "ci-" + r.status, "#" + r.n + " " + r.status + " · " + r.what]), marks: [] }, ' data-ci="1"')
    + lane(`<span class="me">ME</span><strong>You</strong>`, "the queue in front of the inspection station", YOU, ' data-you="1"')
    + `<div class="lane off"><div class="who">${DRONE_ICON}<strong>Claude Code</strong><small>connected over MCP · idle</small></div><div class="track"></div></div>`;
  return `
<div class="fl floor" id="floor">
  <div class="fh"><b>Floor</b><span>last 60 min</span>
    <span class="legend"><i data-k="scout"></i>scout <i data-k="build"></i>build <i data-k="verify"></i>verify <i data-k="wait"></i>blocked <i class="mk" data-k="choice"></i>choice <i class="mk" data-k="seal"></i>seal <i class="mk" data-k="stale"></i>stale</span>
    <span class="fkeys"><kbd>J</kbd><kbd>K</kbd> lanes</span><button id="floorX" title="Close (G)">✕</button></div>
  <div class="lanes"><div class="axis"><div></div><div class="ticks">${ticks}</div></div>${lanes}</div>
</div>
<div class="tip" id="tip"></div>`;
}

export const FLOOR_CSS = `
.agent.blocked{color:#f1d5a2}.agent.blocked svg{color:var(--strain)}.agent.blocked span{color:#c9a466}
.agent.on{background:rgba(255,255,255,.07);color:var(--text)}
.bl .tl{display:flex;align-items:center;gap:8px;height:28px;padding:0 8px;border-radius:8px;color:var(--dim);font-size:11.5px;border-top:1px solid var(--line);margin-top:2px}
.bl .tl:hover{color:var(--text);background:rgba(255,255,255,.04)}.bl .tl kbd{margin-left:auto}
.terr-fill{fill:rgba(255,255,255,.016)}
.terr .iso-line{stroke:#5c5c5c}
.terr .iso-face-text{fill:#6c6c6c;letter-spacing:.08em}
.terr[data-state=verifying] .terr-fill,.terr[data-state=building] .terr-fill{fill:rgba(225,164,60,.045)}
.terr[data-state=verifying] .iso-line,.terr[data-state=building] .iso-line{stroke:rgba(225,164,60,.55)}
.terr[data-state=verifying] .iso-face-text,.terr[data-state=building] .iso-face-text{fill:#b98d47}
.terr[data-state=passed] .terr-fill{fill:rgba(201,164,102,.05)}
.terr[data-state=passed] .iso-line{stroke:#a88a55}.terr[data-state=passed] .iso-face-text{fill:#a88a55}
.terr[data-state=failed] .iso-line{stroke:var(--fail)}
.terr[data-stale] .iso-line{stroke-dasharray:7 3}
.terr[data-stale] .iso-face-text{fill:#c9a466}
.seal .iso-top{fill:#c9a466}.seal .iso-shade{fill:#6e5530}.seal .iso-line{stroke:#e2c48c;stroke-dasharray:none}
.terr[data-stale] .seal{opacity:.55}
body:not(.territories) .terr{display:none}
.has-sel .terr:not(.on){opacity:.35}
.has-sel .ent.touch{opacity:1}
.chip.touch{opacity:1;translate:0 0}
.terr.on .iso-line{stroke-width:1.2}
.threads{position:absolute;left:0;top:0;pointer-events:none;overflow:visible}
.threads path{fill:none;stroke:rgba(255,255,255,.35);stroke-width:.8;stroke-dasharray:2 3;animation:thread 1.2s linear infinite}
.threads circle{fill:#fff}
.threads text{font:9px var(--mono);fill:#9a9a9a}
@keyframes thread{to{stroke-dashoffset:-10}}
.ent[data-id=bot-cx] .beam{stroke-dasharray:.6 2.6;opacity:.6}

.floor{left:296px;right:16px;bottom:70px;flex-direction:column;align-items:stretch;gap:4px;padding:8px 10px 10px;opacity:0;translate:0 10px;pointer-events:none}
body.floor-open .floor{opacity:1;translate:0 0;pointer-events:auto}
.has-insp .floor{right:420px}
.fh{display:flex;align-items:baseline;gap:10px;padding:0 2px 2px}
.fh b{font-weight:600;font-size:12.5px;color:#eee}.fh>span{font:10px var(--mono);color:var(--dim);white-space:nowrap}
.fh .legend{margin-left:auto;display:flex;align-items:center;gap:5px;font:10px var(--mono);color:var(--faint)}
.fh .legend i{position:relative;display:inline-block;width:12px;height:6px;border-radius:2px;margin-left:6px}
.fh .legend i.mk{width:8px;height:8px;top:0;left:0;transform:none}
.fh button{width:22px;height:22px;border-radius:6px;color:var(--dim)}
.fh .fkeys{font:10px var(--mono);color:var(--faint);white-space:nowrap}.fh .fkeys kbd{margin-right:2px}
.lane.kf{background:rgba(255,255,255,.05);box-shadow:inset 2px 0 0 #e8e8e8}
.lanes{display:flex;flex-direction:column;gap:2px}
.lane,.axis{display:grid;grid-template-columns:200px 1fr;align-items:center;gap:10px}
.lane{height:28px;border-radius:7px;cursor:pointer;padding-right:6px}
.lane:hover,.lane.on{background:rgba(255,255,255,.04)}
.lane.off{cursor:default;opacity:.55}
.who{display:flex;align-items:center;gap:6px;min-width:0;padding-left:4px;font-size:12px;color:var(--text);white-space:nowrap}
.who strong{font-weight:500}
.who small{font:10px var(--mono);color:var(--faint);overflow:hidden;text-overflow:ellipsis}
.who svg{flex:none;color:#bbb}
.lane[data-state=blocked] .who svg{color:var(--strain)}
.who .me{display:grid;place-items:center;width:22px;height:15px;border-radius:4px;background:var(--strain);color:#0a0a0a;font:700 8.5px var(--mono)}
.track{position:relative;height:100%;background:repeating-linear-gradient(90deg,transparent 0 calc(100%/6 - 1px),rgba(255,255,255,.05) calc(100%/6 - 1px) calc(100%/6))}
.track::after{content:"";position:absolute;right:0;top:3px;bottom:3px;width:1px;background:rgba(255,255,255,.4)}
.ticks{position:relative;height:14px}
.ticks span{position:absolute;transform:translateX(-50%);font:9.5px var(--mono);color:var(--faint)}
.ticks span:last-child{transform:translateX(-100%)}
.sg{position:absolute;top:10px;height:8px;border-radius:2px}
[data-k=scout]{background:repeating-linear-gradient(90deg,#8a8a8a 0 2px,transparent 2px 4px)}
[data-k=build]{background:#cfcfcf}
[data-k=verify]{background:repeating-linear-gradient(135deg,#cfcfcf 0 3px,#6a6a6a 3px 5px)}
[data-k=wait]{background:var(--strain)}
[data-k^=wait]{background:var(--strain)}
.sg[data-k=wait1]{opacity:.35}.sg[data-k=wait2]{opacity:.55}.sg[data-k=wait3]{opacity:.75}
.sg[data-k=wait1],.sg[data-k=wait2],.sg[data-k=wait3],.sg[data-k=wait4]{top:auto;bottom:6px}
.sg[data-k=wait1]{height:4px}.sg[data-k=wait2]{height:8px}.sg[data-k=wait3]{height:12px}.sg[data-k=wait4]{height:16px}
.mk{position:absolute;top:50%;width:9px;height:9px;transform:translate(-50%,-50%);border-radius:50%;z-index:1}
.mk[data-k=claim]{border-radius:1px;width:2px;height:18px;background:#e8e8e8}
.mk[data-k=choice]{background:#e8e8e8;box-shadow:0 0 0 2px #121212}
.mk[data-k=choice-low]{background:#121212;border:1.5px solid #e8e8e8}
.mk[data-k=pass]{background:var(--ok);width:7px;height:7px;box-shadow:0 0 0 2px #121212}
.mk[data-k=fail]{background:var(--fail);width:7px;height:7px;box-shadow:0 0 0 2px #121212}
.mk[data-k=seal]{background:radial-gradient(circle at 40% 35%,#f0d9a8,#a88a55 70%);width:11px;height:11px;box-shadow:0 0 0 2px #121212}
.mk[data-k=stale]{background:transparent;border:1.5px solid #c9a466;border-radius:2px;transform:translate(-50%,-50%) rotate(45deg);width:7px;height:7px}
.mk[data-k=wait]{background:var(--strain);width:3px;height:16px;border-radius:1px}
.lane[data-ci] .who svg{color:#cfe6fb}
.sg[data-k^=ci-]{top:9px;height:10px;border-radius:2px}
[data-k=ci-passed]{background:var(--ok)}
[data-k=ci-failed]{background:var(--fail)}
[data-k=ci-running]{background:repeating-linear-gradient(135deg,#cfe6fb 0 3px,#56718a 3px 6px);background-size:8.5px 100%;animation:ciRun 1s linear infinite}
@keyframes ciRun{to{background-position:8.5px 0}}
.runs{display:flex;flex-direction:column}
.run{display:grid;grid-template-columns:12px 44px 1fr auto;align-items:center;gap:8px;padding:6px 0;border-top:1px solid var(--line);font-size:12px;color:#d8d8d8;text-align:left;width:100%}
.run:first-child{border-top:0}
.run i{width:8px;height:8px;border-radius:50%;background:var(--ok)}
.run[data-s=failed] i{background:var(--fail)}
.run[data-s=running] i{background:#cfe6fb;animation:lampPulse 1.1s ease-in-out infinite}
.run b{font:500 11px var(--mono);color:#eee}
.run small{font:10px var(--mono);color:var(--faint)}
button.run:hover{background:rgba(255,255,255,.03)}
.checks .run-now i{color:#cfe6fb;animation:lampPulse 1.1s ease-in-out infinite}
.lnk{display:flex;align-items:center;gap:8px;width:100%;padding:7px 8px;border-radius:8px;border:1px solid var(--line2);font-size:12px;color:#ddd;text-align:left}
.lnk:hover{background:rgba(255,255,255,.04)}
.lnk small{margin-left:auto;font:10px var(--mono);color:var(--faint)}
.tip{position:fixed;z-index:12;pointer-events:none;padding:4px 8px;border-radius:6px;background:#1c1c1c;border:1px solid var(--line2);font:10.5px var(--mono);color:#ddd;white-space:nowrap;opacity:0;transform:translate(-50%,calc(-100% - 10px));transition:opacity .12s}
.tip.on{opacity:1}

.sstate{display:flex;flex-direction:column;gap:4px;padding:10px;border-radius:10px;border:1px solid var(--line2);background:rgba(255,255,255,.025)}
.sstate .st{font:600 10px var(--mono);letter-spacing:.12em;text-transform:uppercase;color:#cfcfcf;display:flex;gap:8px}
.sstate .st i{font-style:normal;margin-left:auto;color:var(--faint);letter-spacing:.02em;font-weight:400}
.sstate p{margin:0;font-size:12.5px;color:#e6e6e6}
.sstate small{font:10px var(--mono);color:var(--dim)}
.sstate[data-state=blocked]{border-color:rgba(225,164,60,.45);background:rgba(225,164,60,.06)}
.sstate[data-state=blocked] .st{color:#e7bd73}
.sstate[data-state=verifying] .st{color:#d9b06a}
.sstate .row{display:flex;gap:6px;margin-top:4px}
.sstate .row button{flex:1;height:28px;border-radius:7px;border:1px solid var(--line2);font-size:12px}
.tsteps{display:grid;grid-template-columns:repeat(5,1fr);gap:4px}
.tsteps div{display:flex;flex-direction:column;gap:5px;font-size:10px;color:var(--faint)}
.tsteps i{height:3px;border-radius:2px;background:#262626}
.tsteps .done{color:var(--dim)}.tsteps .done i{background:#8a8a8a}
.tsteps .cur{color:#f0d9a8}.tsteps .cur i{background:#c9a466}
.tgoal{font-size:12.5px;color:#e6e6e6;margin:8px 0 4px}
.tmeta{font:10px var(--mono);color:var(--faint)}
.tags{display:flex;flex-wrap:wrap;gap:4px;margin-top:6px}
.tags button{height:22px;padding:0 7px;border-radius:6px;border:1px solid var(--line2);font:10.5px var(--mono);color:#cfcfcf}
.tags button:hover{border-color:#777;color:#fff}
.warn{margin-top:8px;padding:7px 9px;border-radius:8px;font-size:11.5px;color:#e7cf9f;background:rgba(201,164,102,.07);border:1px solid rgba(201,164,102,.25)}
.budl{margin-top:8px;display:flex;flex-direction:column;gap:3px;font:10.5px var(--mono);color:#bdbdbd}
.budl span::before{content:"🔒 ";opacity:.6}
.file{display:grid;grid-template-columns:1fr auto;align-items:baseline;gap:4px 10px;padding:5px 0;border-top:1px solid var(--line);font-size:11.5px}
.file:first-of-type{border-top:0}
.file code{font:10.5px var(--mono);color:#d6d6d6;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.file button{font:10.5px var(--mono);color:var(--dim)}.file button:hover{color:#fff}
.file .fb{grid-column:1/-1;height:3px;border-radius:2px;background:#232323;overflow:hidden}.file .fb i{display:block;height:100%;background:#9a9a9a}
.pin{display:flex;align-items:center;gap:8px;padding:6px 0;border-top:1px solid var(--line);font-size:12px;color:#d8d8d8}
.pin:first-of-type{border-top:0}
button.pin{width:100%;text-align:left}button.pin:hover{color:#fff}
.pin i{flex:none;width:10px;height:10px;border-radius:50%;background:#e8e8e8}.pin i.low{background:transparent;border:1.5px solid #e8e8e8}
.pin small{margin-left:auto;font:10px var(--mono);color:var(--faint);white-space:nowrap}
.via{display:grid;grid-template-columns:86px 12px 1fr;gap:2px 8px;padding:4px 0;font-size:11.5px;color:#d6d6d6}
.via .n{font:10px var(--mono);color:var(--dim);text-transform:uppercase;letter-spacing:.06em}
.via .ok{color:var(--ok)}.via .no{color:var(--faint)}.via small{font:10px var(--mono);color:var(--faint)}
.spend{display:grid;grid-template-columns:repeat(4,1fr);gap:6px}
.spend div{display:flex;flex-direction:column;gap:2px;font:10px var(--mono);color:var(--faint)}
.spend b{font:15px var(--mono);font-weight:400;color:#eee}
.passive{padding:9px 10px;border-radius:9px;border:1px dashed var(--line2);font-size:12px;color:var(--dim)}
.passive button{margin-top:6px;height:26px;padding:0 10px;border-radius:7px;border:1px solid var(--line2);color:#ddd;font-size:12px}
`;

export const FLOOR_SCRIPT = `
const SESSIONS = ${JSON.stringify(SESSIONS)};
const TERR = ${JSON.stringify(TERRITORIES)};
const TSTATES = ${JSON.stringify(TSTATES)};
const BOT_SESSION = Object.fromEntries(SESSIONS.filter((s) => s.bot).map((s) => [s.bot, s.id]));
const STATE_LABEL = { verifying: "Verifying", building: "Building", blocked: "Blocked on you", done: "Done", working: "Working" };
const CI = ${JSON.stringify(CI)};
let session = null;

const ago = (t) => (t ? -t + "m ago" : "now");
const runRow = (r) => '<button class="run" data-s="' + r.status + '" data-go="crane"><i></i><b>#' + r.n + "</b><span>" + r.what + "</span><small>" + (r.status === "running" ? "running " + (r.t1 - r.t0) + "m" : r.status + " · " + ago(r.t1)) + "</small></button>";
function ciHtml(e) {
  const s = SESSIONS.find((x) => x.id === CI.session), checks = GHOST.radar.checks, now = checks.findIndex((c) => !c.ok);
  let h = '<div class="fig" id="fig"><div class="iso" id="figPlant"></div><div class="read">' + e.reading + "<small>" + e.sub + '</small></div><div class="tools"><button data-do="runlog">Open run log <kbd>R</kbd></button><button data-do="rerun">Re-run</button></div></div>';
  h += '<div class="sec"><h5>On the hook<i>radar/otel · run #418</i></h5><div class="checks">' + checks.map((c, i) => '<div class="' + (c.ok ? "ok" : i === now ? "run-now" : "") + '"><i>' + (c.ok ? "✓" : i === now ? "●" : "·") + "</i>" + c.text + (c.locked ? "<small>🔒 K16</small>" : "") + "</div>").join("") + "</div></div>";
  h += '<div class="sec"><h5>Runs<i>' + CI.runner + '</i></h5><div class="runs">' + CI.runs.slice().reverse().map(runRow).join("") + "</div></div>";
  h += '<div class="sec"><h5>Called by</h5><button class="lnk" data-sess="' + s.id + '">' + DRONE_ICON_JS + "<span><b>" + s.agent + "</b> · " + s.goal + "</span><small>run_checks</small></button></div>";
  h += '<div class="note">The crane is CI. It lifts a branch through the checks in .orrery/checks/, which agents can read but not edit (K16). Two red runs in a row on one territory wake the canvas.</div>';
  return h;
}

function sessionHtml(s) {
  let h = '<div class="sstate" data-state="' + s.state + '"><span class="st">' + STATE_LABEL[s.state] + "<i>" + s.since + "</i></span><p>" + s.now + "</p>";
  if (s.state === "blocked") h += "<small>The prompt lives in Codex's own terminal. Orrery shows that it waits but can't answer for it.</small><div class=\\"row\\"><button data-do='terminal'>Open its terminal</button><button>Snooze 10 min</button></div>";
  h += "</div>";
  const t = s.territory && TERR[s.territory];
  if (t) {
    const idx = TSTATES.indexOf(t.state);
    h += '<div class="sec"><h5>Territory<i>' + s.territory + "</i></h5><div class=\\"tsteps\\">" + TSTATES.map((st, i) => '<div class="' + (i < idx || (st === "passed" && t.state === "passed") ? "done" : i === idx ? "cur" : "") + '"><i></i>' + st + "</div>").join("") + "</div>";
    h += '<div class="tgoal">' + t.goal + '</div><div class="tmeta">' + t.file + (t.depends.length ? " · depends on " + t.depends.join(", ") : "") + "</div>";
    h += '<div class="tags">' + t.entities.filter((id) => ENT[id]).map((id) => '<button data-go="' + id + '">' + ENT[id].name + "</button>").join("") + "</div>";
    if (t.stale) h += '<div class="warn">Stale. ' + t.stale + ". Droid was notified and the seal needs a re-check.</div>";
    if (s.territory === "radar-otel" && GHOST.radar && GHOST.radar.checks) h += '<div class="checks" style="margin-top:8px">' + GHOST.radar.checks.map((c) => '<div class="' + (c.ok ? "ok" : "") + '"><i>' + (c.ok ? "✓" : "·") + "</i>" + c.text + (c.locked ? "<small>🔒 K16</small>" : "") + "</div>").join("") + "</div>";
    h += '<div class="budl">' + t.budgets.map((b) => "<span>" + b + "</span>").join("") + "</div></div>";
  } else if (s.perm === "passive") {
    h += '<div class="passive">' + s.agent + " is observed passively, through its own OTel export and hooks. It shows up where it works, but it can't claim a territory, run checks or record choices.<br><button>Connect " + s.agent + " over MCP</button></div>";
  }
  const max = Math.max(...s.files.map((f) => f[2]));
  h += '<div class="sec"><h5>Where it works<i>files mapped to machines</i></h5>' + s.files.map(([f, id, n]) => '<div class="file"><code>' + f + '</code><button data-go="' + id + '">' + ENT[id].name + " · " + n + '</button><div class="fb"><i style="width:' + (n / max) * 100 + '%"></i></div></div>').join("") + "</div>";
  const runs = s.territory ? CI.runs.filter((r) => r.what.startsWith(s.territory)) : [];
  if (runs.length) h += '<div class="sec"><h5>Crane runs<i>CI on ' + s.territory + '</i></h5><div class="runs">' + runs.map(runRow).join("") + "</div></div>";
  const mine = CH.filter((c) => c.session === s.id);
  if (mine.length) h += '<div class="sec"><h5>Choices it recorded<i>pinned on machines</i></h5>' + mine.map((c) => '<button class="pin" data-choice="' + c.id + '"><i class="' + (c.conf < 0.75 ? "low" : "") + '"></i>' + c.text + "<small>" + c.conf.toFixed(2) + " · " + ENT[c.on].name + "</small></button>").join("") + "</div>";
  h += '<div class="sec"><h5>How Orrery sees it<i>' + s.perm + "</i></h5>" + s.via.map(([n, ok, d]) => '<div class="via"><span class="n">' + n + '</span><span class="' + (ok ? "ok" : "no") + '">' + (ok ? "●" : "○") + "</span><span>" + (d || (ok ? "on" : "off")) + "</span></div>").join("") + "</div>";
  h += '<div class="sec"><h5>Spend<i>this session</i></h5><div class="spend">' + s.spend.map(([k2, v]) => "<div><b>" + v + "</b>" + k2 + "</div>").join("") + "</div></div>";
  h += '<div class="acts"><button data-do="hand">Hand to agent <kbd>H</kbd></button><button data-do="log">Open session log</button>' + (s.bot ? '<button data-act="follow">Follow the drone</button>' : "") + "</div>";
  return h;
}
function openSession(sid) {
  const s = SESSIONS.find((x) => x.id === sid);
  session = sid; review = null; selected = s.bot && ENT[s.bot] ? s.bot : null;
  $("#insp").classList.add("open"); document.body.classList.add("has-insp");
  $("#ih").dataset.state = s.state === "blocked" ? "wait" : s.state === "done" ? "idle" : "ok";
  $("#iMono").textContent = s.agent === "Codex" ? "CX" : s.agent[0]; $("#iName").textContent = s.agent;
  $("#iKind").textContent = s.goal + " · " + s.model;
  $("#ibody").innerHTML = sessionHtml(s); $("#ibody").scrollTop = 0;
  const ids = touched(s), pts = (s.bot && anchors[s.bot] ? [s.bot] : ids).map((id) => anchors[id]).filter(Boolean);
  if (pts.length) { const ax = pts.reduce((m, p) => m + p[0], 0) / pts.length, ay = pts.reduce((m, p) => m + p[1], 0) / pts.length; goal.x += (innerWidth - 420) / 2 - (goal.x + ax * goal.s); goal.y += innerHeight / 2 - 40 - (goal.y + ay * goal.s); animate(); }
  for (const r of document.querySelectorAll("[data-session]")) r.classList.toggle("on", r.dataset.session === sid);
  paintSelection(); floorAfterRender(); choicesAfterRender();
}
function touched(s) { return [...new Set([...s.files.map((f) => f[1]), ...(s.territory ? TERR[s.territory].entities : [])])].filter((id) => ENT[id]); }
function floorAfterRender() {
  const s = session && SESSIONS.find((x) => x.id === session), ids = s ? touched(s) : [];
  if (s && !s.bot) stage.classList.add("has-sel");
  for (const g of plant.querySelectorAll(".ent")) g.classList.toggle("touch", ids.includes(g.dataset.id));
  for (const c of document.querySelectorAll(".chip")) c.classList.toggle("touch", ids.includes(c.dataset.id));
  for (const t of plant.querySelectorAll(".terr")) t.classList.toggle("on", !!s && t.dataset.id === s.territory);
  let th = $("#threads");
  if (!th) { world.insertAdjacentHTML("beforeend", '<svg class="threads" id="threads" width="' + W + '" height="' + H + '"></svg>'); th = $("#threads"); }
  const from = s && s.bot && anchors[s.bot];
  if (!from) { th.innerHTML = ""; return; }
  const f = [from[0], from[1] + 6];
  th.innerHTML = s.files.filter((x, i, a) => a.findIndex((y) => y[1] === x[1]) === i && anchors[x[1]]).map(([, id]) => { const p = anchors[id], mx = (f[0] + p[0]) / 2, my = Math.min(f[1], p[1]) - 30; return '<path d="M' + f[0].toFixed(1) + " " + f[1].toFixed(1) + "Q" + mx.toFixed(1) + " " + my.toFixed(1) + " " + p[0].toFixed(1) + " " + p[1].toFixed(1) + '"/><circle cx="' + p[0].toFixed(1) + '" cy="' + p[1].toFixed(1) + '" r="1.6"/>'; }).join("");
}
$("#ibody").addEventListener("click", (ev) => {
  const go = ev.target.closest("[data-go]"); if (go) { session = null; select(go.dataset.go); return; }
  const ss = ev.target.closest("[data-sess]"); if (ss) { openSession(ss.dataset.sess); return; }
  const f = ev.target.closest("[data-act=follow]"); if (f && session) { const s = SESSIONS.find((x) => x.id === session); toast("Following " + s.agent + ". The camera keeps its drone in view."); }
});
for (const b of document.querySelectorAll(".bl [data-session]")) b.onclick = () => openSession(b.dataset.session);
for (const l of document.querySelectorAll(".lane[data-session]")) l.onclick = () => openSession(l.dataset.session);
document.querySelector(".lane[data-you]").onclick = () => select("you");
document.querySelector(".lane[data-ci]").onclick = () => select("crane");
function toggleFloor(on = !document.body.classList.contains("floor-open")) { document.body.classList.toggle("floor-open", on); $("#tlBtn").setAttribute("aria-pressed", on); }
$("#tlBtn").onclick = () => toggleFloor();
$("#floorX").onclick = () => toggleFloor(false);
function floorStep(d) {
  const lanes = [...document.querySelectorAll("#floor .lane[data-session],#floor .lane[data-ci],#floor .lane[data-you]")];
  const cur = lanes.findIndex((l) => l.classList.contains("kf")), j = ((cur < 0 ? (d > 0 ? -1 : 0) : cur) + d + lanes.length) % lanes.length;
  lanes.forEach((l, i) => l.classList.toggle("kf", i === j));
  lanes[j].click();
}
addEventListener("keydown", (ev) => {
  if (!bare(ev)) return;
  const k = ev.key.toLowerCase();
  if (k === "g") toggleFloor();
  if (document.body.classList.contains("floor-open") && (k === "j" || k === "k")) { ev.preventDefault(); floorStep(k === "j" ? 1 : -1); }
});
const tip = $("#tip");
$("#floor").addEventListener("pointermove", (ev) => {
  const m = ev.target.closest(".mk,.sg");
  if (!m) { tip.classList.remove("on"); return; }
  tip.textContent = m.dataset.tip; tip.style.left = ev.clientX + "px"; tip.style.top = ev.clientY + "px"; tip.classList.add("on");
});
$("#floor").addEventListener("pointerleave", () => tip.classList.remove("on"));
function floorInit(params) {
  if (params.get("floor")) toggleFloor(true);
  if (params.get("session")) openSession(params.get("session"));
}
`;
