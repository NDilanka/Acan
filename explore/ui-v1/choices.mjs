import { CHOICES, COST } from "./acquit.mjs";

export const CH_HTML = `
<div class="queue" id="queue"></div>`;

export const CH_CSS = `
.pins{position:absolute;inset:0;pointer-events:none}
body.choices .chip.pinned{transform:translate(-50%,calc(-100% - 50px))}
body.choices .chip.pinned::before{height:22px}
body:not(.choices) .pins{display:none}
.cpin{position:absolute;pointer-events:auto;display:flex;flex-direction:column;align-items:center;transform:translate(-50%,-100%);transform-origin:50% 100%;cursor:pointer;transition:opacity .3s}
.cpin .hd{width:var(--d);height:var(--d);border-radius:50%;background:#e8e8e8;box-shadow:0 0 0 2px rgba(10,10,10,.85),0 2px 6px rgba(0,0,0,.6);transition:transform .25s var(--ease),background .2s}
.cpin.low .hd{background:#121212;border:1.5px solid #e8e8e8}
.cpin .nd{width:1.4px;height:26px;background:linear-gradient(#d8d8d8,rgba(216,216,216,.15))}
.cpin .ft{width:5px;height:2.5px;border-radius:50%;background:rgba(255,255,255,.5);margin-top:-1px}
.cpin[data-cost=low]{--d:10px}.cpin[data-cost=medium]{--d:13px}.cpin[data-cost=high]{--d:17px}
.cpin:hover .hd{transform:scale(1.25)}
.cpin.cur .hd{box-shadow:0 0 0 2px rgba(10,10,10,.85),0 0 0 4px var(--strain),0 0 14px rgba(225,164,60,.6);animation:pinPulse 1.4s ease-in-out infinite}
@keyframes pinPulse{50%{box-shadow:0 0 0 2px rgba(10,10,10,.85),0 0 0 6px rgba(225,164,60,.35),0 0 18px rgba(225,164,60,.3)}}
.cpin[data-status=accepted] .hd{background:var(--ok);border:0}
.cpin[data-status=replaced] .hd{background:#cfe6fb;border:0}
.cpin[data-status=rejected] .hd{background:#121212;border:1.5px solid var(--fail)}
.cpin[data-status=later] .hd{border-style:dashed}
.cpin:not([data-status=open]):not([data-status=later]):not(.cur){opacity:.55}
.cpin:not([data-status=open]):not([data-status=later]) .nd{height:12px}
.has-sel .cpin:not(.cur):not(.mine){opacity:.25}

body.queue-open .bl{min-width:420px}
.queue{display:none}
body.queue-open .queue{display:block}
@keyframes qOpen{from{opacity:0;translate:0 6px}}
body.queue-open .bl .wait{border-bottom-left-radius:0;border-bottom-right-radius:0}
.qsec{padding:6px 2px 2px;font:500 9.5px var(--mono);letter-spacing:.14em;text-transform:uppercase;color:var(--faint);display:flex}
.qsec i{margin-left:auto;font-style:normal;letter-spacing:.02em;text-transform:none}
.qrow{display:grid;grid-template-columns:14px 1fr auto;align-items:center;gap:8px;width:100%;padding:5px 8px;border-radius:7px;font-size:12px;color:#d6d6d6;text-align:left}
.qrow:hover{background:rgba(255,255,255,.05)}
.qrow .ic{width:9px;height:9px;border-radius:50%;background:#e8e8e8;justify-self:center}
.qrow .ic.low{background:transparent;border:1.5px solid #e8e8e8}
.qrow .ic.ghost{border-radius:2px;background:transparent;border:1px dashed var(--ghost)}
.qrow .ic.blocked{border-radius:2px;background:var(--strain)}
.qrow>span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.qrow small{font:10px var(--mono);color:var(--dim);display:flex;align-items:center;gap:6px}
.qrow .rb{width:34px;height:3px;border-radius:2px;background:#262626;overflow:hidden}.qrow .rb i{display:block;height:100%;background:var(--strain)}
.qrow[data-status]:not([data-status=open]):not([data-status=later]){opacity:.4}
.qrow[data-status]:not([data-status=open]):not([data-status=later]) span{text-decoration:line-through;text-decoration-color:rgba(255,255,255,.3)}
.bl .qwalk{margin:6px 2px 4px;height:30px;border-radius:8px;background:#e8e8e8;color:#0a0a0a;font-size:12px;font-weight:500;display:flex;align-items:center;justify-content:center;gap:8px;width:calc(100% - 4px)}
.qwalk kbd{color:#555;border-color:#aaa}

.rv-steps{display:flex;align-items:center;gap:4px}
.rv-steps i{flex:1;height:4px;border-radius:2px;background:#2a2a2a;cursor:pointer}
.rv-steps i[data-s=accepted]{background:var(--ok)}.rv-steps i[data-s=replaced]{background:#cfe6fb}.rv-steps i[data-s=rejected]{background:var(--fail)}
.rv-steps i[data-s=later]{background:repeating-linear-gradient(90deg,#8a8a8a 0 3px,transparent 3px 5px)}
.rv-steps i.cur{outline:1.5px solid var(--strain);outline-offset:2px}
.rv-steps span{flex:none;margin-left:6px;font:10px var(--mono);color:var(--faint)}
.rv-q{font-size:16px;line-height:1.35;color:#f2f2f2;font-weight:500;letter-spacing:-.005em}
.rv-pick{padding:10px 12px;border-radius:10px;border:1px solid var(--line2);background:rgba(255,255,255,.03)}
.rv-pick small{display:block;font:10px var(--mono);color:var(--dim);margin-bottom:3px}
.rv-pick p{margin:0;font-size:13px;color:#e8e8e8}
.rv-pick[data-status=accepted]{border-color:rgba(53,183,162,.45)}
.rv-pick[data-status=rejected],.rv-pick[data-status=replaced]{opacity:.55}
.rv-pick[data-status=rejected] p,.rv-pick[data-status=replaced] p{text-decoration:line-through}
.rv-risk{display:grid;grid-template-columns:1fr 1fr 1fr;gap:10px;padding:10px 12px;border-radius:10px;background:rgba(0,0,0,.25);border:1px solid var(--line)}
.rv-m{display:flex;flex-direction:column;gap:4px;min-width:0}
.rv-m small{font:9.5px var(--mono);color:var(--faint);letter-spacing:.06em;text-transform:uppercase}
.rv-m b{font:400 18px var(--mono);color:#f0f0f0}
.rv-m.risk b{color:#f1d5a2}
.rv-m code{font:9.5px var(--mono);color:var(--dim);white-space:nowrap}
.pips{display:flex;gap:3px}.pips i{flex:1;height:4px;border-radius:2px;background:#2a2a2a}.pips i.on{background:#d6d6d6}
.alt{display:grid;grid-template-columns:1fr auto;align-items:center;gap:10px;padding:8px 0;border-top:1px solid var(--line);font-size:12px;color:#d8d8d8}
.alt:first-of-type{border-top:0}
.alt small{display:block;font:10px var(--mono);color:var(--faint);margin-top:2px}
.alt button{height:26px;padding:0 10px;border-radius:7px;border:1px solid var(--line2);font-size:11.5px;color:#ddd;white-space:nowrap}
.alt button:hover{background:rgba(255,255,255,.06)}
.alt.picked{color:#cfe6fb}.alt.picked button{border-color:#cfe6fb;color:#cfe6fb}
.rv-why{display:none;flex-direction:column;gap:6px;padding:10px;border-radius:10px;border:1px solid rgba(232,96,74,.35);background:rgba(232,96,74,.05)}
.rv-why.on{display:flex}
.rv-why label{font-size:12px;color:#f0c2b8}
.rv-why input{height:30px;border-radius:7px;border:1px solid var(--line2);background:rgba(0,0,0,.35);color:#eee;padding:0 9px;font:12px var(--sans);outline:none}
.rv-why input:focus{border-color:rgba(232,96,74,.6)}
.rv-why .row{display:flex;gap:6px;justify-content:flex-end}
.rv-why .row button{height:26px;padding:0 10px;border-radius:7px;border:1px solid var(--line2);font-size:11.5px}
.rv-why .row button.send{background:var(--fail);border-color:var(--fail);color:#140805}
.rv-done{font-size:12px;color:#cfcfcf;padding:9px 11px;border-radius:9px;border:1px solid var(--line2);background:rgba(255,255,255,.03)}
.rv-done b{color:#f0f0f0;font-weight:500}
.yaml .del{color:#e6a598;display:block}.yaml .del::before{content:"- ";color:#a0574a}
#ibody{flex:1 1 auto;min-height:0}
.rv-acts{position:sticky;bottom:-14px;margin:auto -14px -14px;padding:10px 14px 14px;display:flex;gap:6px;background:linear-gradient(rgba(26,26,26,0),rgba(22,22,22,.96) 30%);}
.rv-acts button{flex:1;height:34px;border-radius:9px;border:1px solid var(--line2);font-size:12.5px;color:#e6e6e6;display:flex;align-items:center;justify-content:center;gap:7px;background:rgba(255,255,255,.03)}
.rv-acts button:hover{background:rgba(255,255,255,.07)}
.rv-acts button.rvnav{flex:none;width:34px}
.rv-acts button.pri{background:#e8e8e8;color:#0a0a0a;border-color:#e8e8e8}
.rv-acts button.pri kbd{color:#666;border-color:#aaa}
.rv-acts button.no{color:#f0b2a6;border-color:rgba(232,96,74,.35)}
.rv-sum{display:grid;grid-template-columns:repeat(4,1fr);gap:6px}
.rv-sum div{display:flex;flex-direction:column;gap:2px;padding:8px;border-radius:9px;border:1px solid var(--line);font:10px var(--mono);color:var(--faint)}
.rv-sum b{font:400 20px var(--mono);color:#eee}
.rv-sum [data-s=accepted] b{color:var(--ok)}.rv-sum [data-s=rejected] b{color:var(--fail)}.rv-sum [data-s=replaced] b{color:#cfe6fb}
`;

export const CH_SCRIPT = `
const COST = ${JSON.stringify(COST)};
const CH = ${JSON.stringify(CHOICES)}.map((c) => ({ ...c, risk: +((1 - c.conf) * COST[c.cost]).toFixed(2), status: "open" })).sort((a, b) => b.risk - a.risk);
const COST_PIPS = { low: 1, medium: 2, high: 3 };
let review = null;

function choicesAfterRender() {
  let box = $("#pins");
  if (!box) {
    $("#chips").insertAdjacentHTML("beforebegin", '<div class="pins" id="pins">' + CH.map((c) => '<button class="cpin' + (c.conf < 0.75 ? " low" : "") + '" data-c="' + c.id + '" data-cost="' + c.cost + '" data-status="open" data-tip="' + c.question + " · risk " + c.risk.toFixed(2) + '"><b class="hd"></b><i class="nd"></i><i class="ft"></i></button>').join("") + "</div>");
    box = $("#pins");
    box.addEventListener("click", (ev) => { const p = ev.target.closest(".cpin"); if (p) openReview(p.dataset.c); });
    box.addEventListener("pointermove", (ev) => { const p = ev.target.closest(".cpin"); const t = $("#tip"); if (!p) { t.classList.remove("on"); return; } t.textContent = p.dataset.tip; t.style.left = ev.clientX + "px"; t.style.top = ev.clientY + "px"; t.classList.add("on"); });
    box.addEventListener("pointerleave", () => $("#tip").classList.remove("on"));
  }
  const seen = {};
  for (const p of box.children) {
    const c = CH.find((x) => x.id === p.dataset.c), a = anchors[c.on]; if (!a) continue;
    const n = (seen[c.on] = (seen[c.on] || 0) + 1) - 1;
    p.style.left = a[0] + n * 14 + "px"; p.style.top = a[1] + 4 + "px";
    p.dataset.status = c.status;
    p.classList.toggle("cur", !!review && CH[review.i] === c);
    p.classList.toggle("mine", !review && !!selected && c.on === selected);
  }
  for (const ch of document.querySelectorAll(".chip")) ch.classList.toggle("pinned", !!seen[ch.dataset.id]);
}
function pinScale(sc) { for (const p of document.querySelectorAll(".cpin")) p.style.scale = sc; }

function queueHtml() {
  const open = CH.filter((c) => c.status === "open" || c.status === "later").length;
  let h = '<div><div class="qsec">Choices<i>risk = (1 − confidence) × reversal cost</i></div>';
  h += CH.map((c) => '<button class="qrow" data-choice="' + c.id + '" data-status="' + c.status + '"><b class="ic' + (c.conf < 0.75 ? " low" : "") + '"></b><span>' + c.question + "</span><small>" + ENT[c.on].name + '<span class="rb"><i style="width:' + c.risk * 250 + '%"></i></span>' + c.risk.toFixed(2) + "</small></button>").join("");
  const g = GHOST.breaker && GHOST.breaker.stage === "proposed";
  if (g) h += '<div class="qsec">Ghost to approve</div><button class="qrow" data-go="breaker"><b class="ic ghost"></b><span>paypal breaker, proposed by Droid</span><small>blueprint</small></button>';
  h += '<div class="qsec">Agent blocked</div><button class="qrow" data-sess="codex-paypal"><b class="ic blocked"></b><span>Codex wants to run npm test</span><small>its terminal</small></button>';
  if (open) h += '<button class="qwalk" data-walk="1">Walk ' + open + " choice" + (open === 1 ? "" : "s") + ", riskiest first <kbd>W</kbd></button>";
  return h + "</div>";
}
function toggleQueue(on = !document.body.classList.contains("queue-open")) {
  if (on) $("#queue").innerHTML = queueHtml();
  document.body.classList.toggle("queue-open", on);
}
function refreshQueue() { if (document.body.classList.contains("queue-open")) $("#queue").innerHTML = queueHtml(); }
$(".bl").insertBefore($("#queue"), $(".bl .agent"));
$(".bl .wait").onclick = () => toggleQueue();
$("#queue").addEventListener("click", (ev) => {
  const c = ev.target.closest("[data-choice]"), w = ev.target.closest("[data-walk]"), g = ev.target.closest("[data-go]"), s = ev.target.closest("[data-sess]");
  if (c) openReview(c.dataset.choice); else if (w) walk(); else if (g) select(g.dataset.go); else if (s) openSession(s.dataset.sess);
});

function walk(from = -1) {
  const i = nextOpen(from, true);
  if (i < 0) return toast("Nothing waiting. Every choice has been reviewed.");
  openReview(CH[i].id);
}
function nextOpen(i, includeLater) {
  const n = CH.length, order = [...Array(n).keys()].map((d) => (i + 1 + d) % n).filter((j) => j !== i);
  const j = order.find((j) => CH[j].status === "open");
  if (j !== undefined) return j;
  if (includeLater) { const l = order.find((j) => CH[j].status === "later"); if (l !== undefined) return l; }
  return -1;
}
function flyTo(id) {
  const r = stage.getBoundingClientRect(), s = Math.min(r.width / W, r.height / H) * 1.02 * 1.6, [ax, ay] = anchors[id];
  goal.s = s; goal.x = (innerWidth - 420) / 2 - ax * s; goal.y = innerHeight * 0.52 - ay * s; animate();
}
function openReview(id) {
  const i = CH.findIndex((c) => c.id === id), c = CH[i];
  session = null; for (const r of document.querySelectorAll("[data-session]")) r.classList.remove("on");
  if (explode.id) { explode.goal = 0; animate(); }
  review = { i, why: false }; selected = c.on;
  document.body.classList.remove("queue-open");
  $("#insp").classList.add("open"); document.body.classList.add("has-insp");
  paintReview(); flyTo(c.on);
  paintSelection(); floorAfterRender(); choicesAfterRender();
}
function paintReview() {
  const c = CH[review.i];
  $("#ih").dataset.state = c.status === "open" || c.status === "later" ? (c.conf < 0.75 ? "strain" : "ok") : c.status === "rejected" ? "fail" : "ok";
  $("#iMono").textContent = review.i + 1 + "/" + CH.length; $("#iName").textContent = "Choice on " + ENT[c.on].name;
  $("#iKind").textContent = "by " + c.by + " · " + c.when + " · choices/" + c.id + ".yaml";
  $("#ibody").innerHTML = reviewHtml(c); $("#ibody").scrollTop = 0;
}
function reviewHtml(c) {
  const done = CH.filter((x) => x.status !== "open" && x.status !== "later").length;
  let h = '<div class="rv-steps">' + CH.map((x, j) => '<i data-step="' + j + '" data-s="' + x.status + '" class="' + (j === review.i ? "cur" : "") + '" title="' + x.question + '"></i>').join("") + "<span>" + done + " of " + CH.length + " reviewed</span></div>";
  h += '<div class="rv-q">' + c.question + "</div>";
  h += '<div class="rv-pick" data-status="' + c.status + '"><small>' + c.by + " chose · " + c.when + "</small><p>" + c.text + "</p></div>";
  h += '<div class="rv-risk"><div class="rv-m"><small>confidence</small><b>' + c.conf.toFixed(2) + '</b><div class="bar"><i style="width:' + c.conf * 100 + '%"></i></div></div>';
  h += '<div class="rv-m"><small>reversal cost</small><b>' + c.cost + '</b><div class="pips">' + [1, 2, 3].map((n) => '<i class="' + (n <= COST_PIPS[c.cost] ? "on" : "") + '"></i>').join("") + "</div></div>";
  h += '<div class="rv-m risk"><small>risk</small><b>' + c.risk.toFixed(2) + "</b><code>(1 − " + c.conf.toFixed(2) + ") × " + COST[c.cost].toFixed(1) + "</code></div></div>";
  if (c.status !== "open" && c.status !== "later") h += decidedHtml(c);
  h += '<div class="rv-why' + (review.why ? " on" : "") + '" id="why"><label>Why not? ' + (c.session ? c.by + " gets this over MCP." : "Saved on the choice for the next agent in " + (c.territory || "this area") + ".") + '</label><input id="whyIn" data-suggest="' + (c.alts[0] ? "Prefer: " + c.alts[0].text.toLowerCase() : "") + '" placeholder="' + (c.alts[0] ? "Prefer: " + c.alts[0].text.toLowerCase() + "   Tab uses this" : "Reason, or Enter for none") + '"><div class="row"><button data-rv="cancel">Cancel</button><button class="send" data-rv="send">Reject and send <kbd>↵</kbd></button></div></div>';
  h += '<div class="sec"><h5>Evidence<i>opens in your editor</i></h5><div class="tags">' + c.evidence.map((e) => "<button data-ev>" + e + "</button>").join("") + "</div></div>";
  h += '<div class="sec"><h5>Alternatives it weighed<i>' + c.alts.length + "</i></h5>" + c.alts.map((a, j) => '<div class="alt' + (c.picked === j ? " picked" : "") + '"><div>' + a.text + "<small>" + a.note + '</small></div><button data-alt="' + j + '">' + (c.picked === j ? "Your pick" : "Pick this <kbd>Shift " + (j + 1) + "</kbd>") + "</button></div>").join("") + "</div>";
  const s = c.session && SESSIONS.find((x) => x.id === c.session);
  h += '<div class="sec"><h5>Where it came from</h5>' + (s ? '<button class="lnk" data-sess="' + s.id + '">' + DRONE_ICON_JS + "<span><b>" + s.agent + "</b> · " + s.goal + "</span><small>record_choice</small></button>" : '<div class="note">' + c.by + " recorded this over MCP. That session has ended, so nobody is waiting on the answer.</div>") + (c.territory ? '<div class="tmeta" style="margin-top:6px">territory ' + c.territory + " · " + TERR[c.territory].state + (TERR[c.territory].stale ? " (stale)" : "") + "</div>" : "") + "</div>";
  h += '<div class="rv-acts"><button class="rvnav" data-rv="prev" title="Previous (←)">←</button><button class="rvnav" data-rv="next" title="Next (→)">→</button><button data-rv="later">Later <kbd>D</kbd></button><button class="no" data-rv="reject">Reject <kbd>N</kbd></button><button class="pri" data-rv="accept">Accept <kbd>Y</kbd></button></div>';
  return h;
}
function decidedHtml(c) {
  const D = (t) => '<span class="del">' + t + "</span>", A = (t) => '<span class="add">' + t + "</span>", C = (t) => '<span class="ctx">' + t + "</span>";
  let y = '<span class="f">.acan/choices/' + c.id + ".yaml</span>" + C("id: " + c.id) + C("on: " + c.on) + D("status: open") + A("status: " + c.status) + A("reviewed_by: you");
  if (c.status === "replaced") y += D("choice: " + c.text) + A("choice: " + c.alts[c.picked].text) + D("confidence: " + c.conf.toFixed(2)) + A("confidence: 1.00");
  if (c.status === "rejected") y += A("reason: " + JSON.stringify(c.why || "no reason given"));
  const note = { accepted: "Accepted. It stays pinned, filled in, and drops out of the queue.", replaced: "Replaced with your pick at confidence 1.00. " + (c.session ? c.by + " is told to rework it." : "The next agent here inherits it."), rejected: "Rejected. " + (c.consequence || "") }[c.status];
  return '<div class="rv-done"><b>' + note + '</b></div><div class="yaml">' + y + "</div>";
}

const CHU = [];
function decide(kind, extra) {
  const c = CH[review.i], was = c.status;
  if (kind !== "later" ? was === "open" || was === "later" : was === "open") {
    const st = c.territory && SCENE.territories.find((x) => x.id === c.territory);
    CHU.push({ i: review.i, c: { status: c.status, picked: c.picked, why: c.why, consequence: c.consequence }, t: c.territory && TERR[c.territory] ? TERR[c.territory].state : null, st: st ? { state: st.state, label: st.label } : null, counted: kind !== "later" });
  }
  if (kind === "later") { if (was !== "open" && was !== "later") return advance(); if (was === "open") c.status = "later"; toast("Kept open. It comes back after the rest of the walk."); return advance(); }
  if (was !== "open" && was !== "later") return advance();
  c.status = kind;
  if (kind === "replaced") c.picked = extra;
  if (kind === "rejected") {
    c.why = extra;
    const t = c.territory && TERR[c.territory];
    if (t && t.state === "passed") {
      t.state = "claimed"; const st = SCENE.territories.find((x) => x.id === c.territory); st.state = "claimed"; st.label = c.territory.toUpperCase() + " · REOPENED"; dirtyScene = true;
      c.consequence = c.territory + " loses its seal and reopens. " + c.by + " is asked to rework it.";
    } else c.consequence = c.session ? c.by + " gets the reason and the choice reopens." : "Saved for the next agent in " + (c.territory || ENT[c.on].name) + ".";
  }
  waiting(-1); review.why = false; paintReview(); choicesAfterRender(); refreshQueue();
  const msg = { accepted: "Accepted · wrote choices/" + c.id + ".yaml", replaced: "Your pick replaces it · choices/" + c.id + ".yaml", rejected: "Rejected · " + c.consequence }[kind];
  toast(msg + " · → for the next one, Ctrl Z undoes"); animate();
}
function chUndo() {
  const u = CHU.pop(); if (!u) return toast("Nothing to undo in the walk.");
  const c = CH[u.i]; Object.assign(c, u.c);
  if (u.t) TERR[c.territory].state = u.t;
  if (u.st) { Object.assign(SCENE.territories.find((x) => x.id === c.territory), u.st); dirtyScene = true; }
  if (u.counted) waiting(1);
  refreshQueue(); openReview(c.id); animate();
  toast("Undid it. choices/" + c.id + ".yaml is back to " + c.status + ".");
}
function advance() {
  const j = nextOpen(review.i, true);
  if (j < 0) return summary();
  openReview(CH[j].id);
}
function summary() {
  const n = (s) => CH.filter((c) => c.status === s).length;
  review = { i: -1 }; selected = null;
  $("#ih").dataset.state = "ok"; $("#iMono").textContent = "✓"; $("#iName").textContent = "Walk finished"; $("#iKind").textContent = CH.length + " choices · riskiest first";
  $("#ibody").innerHTML = '<div class="rv-sum">' + ["accepted", "replaced", "rejected", "later"].map((s) => '<div data-s="' + s + '"><b>' + n(s) + "</b>" + s + "</div>").join("") + "</div>"
    + '<div class="note">Every decision is a line in .acan/choices/. Agents read them over MCP before they touch these machines again. ' + (n("later") ? n("later") + " kept for later stay pinned and count in Waiting on you." : "") + "</div>"
    + '<div class="acts">' + (n("later") ? '<button data-walk="1">Walk the later ones</button>' : "") + '<button data-rv="close">Back to the plant</button></div>';
  paintSelection(); floorAfterRender(); choicesAfterRender(); fit();
}
$("#ibody").addEventListener("click", (ev) => {
  if (!review) return;
  const b = ev.target.closest("[data-rv],[data-alt],[data-step],[data-ev],[data-walk],[data-choice]"); if (!b) return;
  if (b.dataset.choice) return openReview(b.dataset.choice);
  if (b.dataset.walk) return walk();
  if (b.dataset.ev !== undefined) return handoff("editor", ["code", "--goto", b.textContent], "Opened " + b.textContent + " in your editor.");
  if (b.dataset.step) return openReview(CH[+b.dataset.step].id);
  if (b.dataset.alt) return decide("replaced", +b.dataset.alt);
  const a = b.dataset.rv;
  if (a === "accept") decide("accepted");
  if (a === "later") decide("later");
  if (a === "prev") openReview(CH[(review.i - 1 + CH.length) % CH.length].id);
  if (a === "next") openReview(CH[(review.i + 1) % CH.length].id);
  if (a === "reject") askWhy();
  if (a === "cancel") { review.why = false; $("#why").classList.remove("on"); }
  if (a === "send") decide("rejected", $("#whyIn").value.trim() || "no reason given");
  if (a === "close") select(null);
});
$("#ibody").addEventListener("click", (ev) => {
  if (review) return;
  const p = ev.target.closest("[data-choice]"); if (p) return openReview(p.dataset.choice);
  if (ev.target.closest("[data-walk]")) { const c = CH.find((x) => x.on === selected && (x.status === "open" || x.status === "later")); c ? openReview(c.id) : walk(); }
});
function askWhy() { const c = CH[review.i]; if (c.status !== "open" && c.status !== "later") return; review.why = true; $("#why").classList.add("on"); $("#whyIn").focus(); $("#why").scrollIntoView({ block: "nearest", behavior: "smooth" }); }
addEventListener("keydown", (ev) => {
  if (ev.target.id === "whyIn") {
    if (ev.key === "Enter") { decide("rejected", ev.target.value.trim() || "no reason given"); ev.preventDefault(); }
    if (ev.key === "Tab" && !ev.target.value && ev.target.dataset.suggest) { ev.target.value = ev.target.dataset.suggest; ev.preventDefault(); ev.stopImmediatePropagation(); }
    if (ev.key === "Escape") { review.why = false; $("#why").classList.remove("on"); ev.target.blur(); ev.stopImmediatePropagation(); }
    return;
  }
  if ((ev.ctrlKey || ev.metaKey) && ev.key.toLowerCase() === "z" && review && !typing(ev)) { ev.preventDefault(); chUndo(); return; }
  if (!bare(ev)) return;
  const key = ev.key.toLowerCase();
  if (key === "w" && ev.shiftKey) { toggleQueue(); return; }
  if (key === "w") { walk(review && review.i >= 0 ? review.i - 1 : -1); return; }
  if (!review || review.i < 0) return;
  const alt = ev.shiftKey && /^Digit[1-9]$/.test(ev.code) ? +ev.code.slice(5) - 1 : -1;
  if (alt >= 0) { if (CH[review.i].alts[alt]) decide("replaced", alt); ev.preventDefault(); return; }
  if (key === "y") decide("accepted");
  if (key === "n") { askWhy(); ev.preventDefault(); }
  if (key === "d") decide("later");
  if (ev.key === "ArrowRight") { const c = CH[review.i]; c.status === "open" || c.status === "later" ? openReview(CH[(review.i + 1) % CH.length].id) : advance(); }
  if (ev.key === "ArrowLeft") openReview(CH[(review.i - 1 + CH.length) % CH.length].id);
});
function chInit(params) {
  if (params.get("queue")) toggleQueue(true);
  if (params.get("review")) { const id = params.get("review"); id === "1" ? walk() : openReview(id); }
}
`;
