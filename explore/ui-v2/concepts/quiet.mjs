import { ORDER, WITNESSES, QUESTION, READBACKS, SHEET, KVARM } from "./core-data.mjs";

export default {
  id: "quiet",
  label: "Quiet orders",
  hint: "M pin · P post · I answer · Alt words",
  css: `
body[data-concept=quiet] *{transition:none!important}
body[data-concept=quiet] .stage *,body[data-concept=quiet] .bl *{animation:none!important}
body[data-concept=quiet] .hint{display:none}
body[data-concept=quiet] .quiet-ui{display:block}
body[data-concept=quiet] .quiet-ui [hidden]{display:none!important}
.quiet-ui{display:none}
body[data-concept=quiet] .quiet-hint{position:fixed;left:20px;top:130px;font:11px var(--mono);color:#aaa;z-index:8}
body[data-concept=quiet] .quiet-gate{margin:4px 0 8px;color:var(--wait);font:10px/1.5 var(--mono)}
body[data-concept=quiet] .quiet-guide{position:fixed;left:20px;top:130px;z-index:8;width:288px;padding:14px;border:1px solid var(--line);border-radius:12px;background:var(--glass);box-shadow:var(--shadow)}
body[data-concept=quiet] .quiet-eyebrow{font:10px var(--mono);letter-spacing:.12em;text-transform:uppercase;color:#c9a45c}
body[data-concept=quiet] .quiet-guide h2{margin:6px 0;font-size:17px;font-weight:500;color:#eee}
body[data-concept=quiet] .quiet-guide p{margin:0 0 12px;color:#999;font-size:12px}
body[data-concept=quiet] .quiet-guide button{display:flex;align-items:center;gap:8px;width:100%;padding:8px 0;text-align:left;border-top:1px solid var(--line);font-size:12px}
body[data-concept=quiet] .quiet-guide button small{margin-left:auto;color:#888;font:10px var(--mono)}
body[data-concept=quiet] .quiet-guide .quiet-dot{width:6px;height:6px;border-radius:50%;background:var(--fail)}
body[data-concept=quiet] .quiet-guide [data-quiet-go=w2] .quiet-dot{background:var(--ok)}
body[data-concept=quiet] .quiet-guide footer{margin-top:8px;font:10px/1.8 var(--mono);color:#888}
body[data-concept=quiet] .quiet-mark{position:fixed;z-index:9;border:1px solid #c9a45c;border-radius:7px;background:#19160f;color:#e6cb96;padding:5px 8px;font:10px var(--mono);white-space:nowrap}
body[data-concept=quiet] .quiet-card{position:fixed;z-index:12;width:470px;max-width:calc(100vw - 40px);border:1px solid #665333;border-radius:14px;background:rgba(22,22,21,.97);box-shadow:var(--shadow);overflow:auto;scrollbar-width:thin;scrollbar-color:#55462f transparent}
body[data-concept=quiet] .quiet-card header{display:flex;gap:10px;align-items:center;padding:12px 16px;border-bottom:1px solid var(--line)}
body[data-concept=quiet] .quiet-card header[data-quiet-plate]{background:#c9a45c13}
body[data-concept=quiet] .quiet-card h3{font-size:15px;font-weight:500;color:#eee;margin:3px 0 0}
body[data-concept=quiet] .quiet-close{margin-left:auto;align-self:flex-start;color:#999;width:26px;height:26px;border-radius:5px}
body[data-concept=quiet] .quiet-close:hover{background:#333;color:#fff}
body[data-concept=quiet] .quiet-body{padding:12px 16px}
body[data-concept=quiet] .quiet-body label,body[data-concept=quiet] .quiet-section{display:block;font:10px var(--mono);letter-spacing:.07em;text-transform:uppercase;color:#999;margin-bottom:8px}
body[data-concept=quiet] .quiet-body input,body[data-concept=quiet] .quiet-body textarea{width:100%;border-radius:7px;background:#111;border:1px solid #444;color:#eee;padding:10px;font:12px/1.5 var(--sans);outline:none}
body[data-concept=quiet] .quiet-body input:focus,body[data-concept=quiet] .quiet-body textarea:focus{border-color:#c9a45c;box-shadow:0 0 0 2px #c9a45c22}
body[data-concept=quiet] .quiet-body textarea{resize:vertical;min-height:70px}
body[data-concept=quiet] #quiet-sentence{resize:none;min-height:58px;overflow:hidden;white-space:pre-wrap;overflow-wrap:anywhere;display:block;margin-bottom:12px}
body[data-concept=quiet] .quiet-sentence{margin:0 0 14px;font-size:13px;line-height:1.5;color:#eee}
body[data-concept=quiet] .quiet-check{display:grid;grid-template-columns:22px 1fr;gap:3px 8px;border-top:1px solid var(--line);padding:7px 0}
body[data-concept=quiet] .quiet-check b{font:11px var(--mono);color:#c9a45c}
body[data-concept=quiet] .quiet-check strong{font:10px var(--mono);letter-spacing:.06em;color:var(--ok);text-transform:uppercase}
body[data-concept=quiet] .quiet-check[data-polarity=must-not] strong{color:var(--fail)}
body[data-concept=quiet] .quiet-check p{grid-column:2;margin:0;font-size:12px;color:#ccc}
body[data-concept=quiet] .quiet-check small{grid-column:2;font:10px/1.5 var(--mono);color:#aaa}
body[data-concept=quiet] .quiet-locks{padding:9px 0;border-top:1px solid var(--line);display:grid;gap:5px;font:10px var(--mono);color:#c9a45c}
body[data-concept=quiet] .quiet-passive{font-size:11px;color:#999;border-top:1px dashed #444;padding-top:9px;margin:0}
body[data-concept=quiet] .quiet-actions{display:flex;justify-content:space-between;align-items:center;gap:10px;margin-top:14px;color:#888;font:10px var(--mono)}
body[data-concept=quiet] .quiet-primary{padding:8px 13px;border-radius:7px;background:#c9a45c;color:#14110b;font:500 12px var(--sans)}
body[data-concept=quiet] .quiet-primary kbd{color:#524222;border-color:#8c723d;margin-left:5px}
body[data-concept=quiet] button:disabled{opacity:.4;cursor:default}
body[data-concept=quiet] .quiet-status{display:flex;align-items:center;gap:6px;min-height:24px;font:10px var(--mono);color:#c9a45c}
body[data-concept=quiet] .quiet-readbacks{margin-top:12px}
body[data-concept=quiet] .quiet-reading{border-top:1px solid var(--line);padding:9px 0}
body[data-concept=quiet] .quiet-reading header{border:0;padding:0 0 5px;display:flex;justify-content:space-between;gap:8px;font-size:12px}
body[data-concept=quiet] .quiet-reading header span{font:10px var(--mono);color:#999}
body[data-concept=quiet] .quiet-reading[data-state=differs]{border-left:2px solid var(--wait);padding-left:10px}
body[data-concept=quiet] .quiet-choice{font:500 12px var(--mono);color:var(--strain)}
body[data-concept=quiet] .quiet-reading[data-state=matches] header span,body[data-concept=quiet] .quiet-mini[data-state=matches] small{color:var(--ok)}
body[data-concept=quiet] .quiet-reading[data-state=yields] header span,body[data-concept=quiet] .quiet-mini[data-state=yields] small{color:var(--fail)}
body[data-concept=quiet] .quiet-reading[data-state=differs] header span,body[data-concept=quiet] .quiet-reading[data-state=rescoring] header span,body[data-concept=quiet] .quiet-mini[data-state=differs] small,body[data-concept=quiet] .quiet-mini[data-state=rescoring] small{color:var(--wait)}
body[data-concept=quiet] .quiet-reading mark{background:#c9a45c18;color:var(--strain);border-bottom:1px solid var(--wait);padding:0 2px}
body[data-concept=quiet] .quiet-reading p{margin:0;color:#bbb;font-size:11px;line-height:1.5}
body[data-concept=quiet] .quiet-score{display:flex;gap:9px;margin-top:7px;font:10px var(--mono)}
body[data-concept=quiet] .quiet-score [data-result=pass]{color:var(--ok)}
body[data-concept=quiet] .quiet-score [data-result=fail]{color:var(--fail)}
body[data-concept=quiet] .quiet-score [data-result=pending]{color:var(--wait)}
body[data-concept=quiet] .quiet-answer{display:block;width:100%;padding:11px;margin-top:9px;border:1px solid #4b4335;border-radius:8px;text-align:left;font-size:12px;color:#ddd}
body[data-concept=quiet] .quiet-answer:hover{border-color:#c9a45c;background:#c9a45c0c}
body[data-concept=quiet] .quiet-answer kbd{color:#d9ba7e;margin-right:6px}
body[data-concept=quiet] .quiet-answer{display:grid;grid-template-columns:64px 1fr;gap:3px 9px}
body[data-concept=quiet] .quiet-answer kbd{white-space:nowrap;align-self:start;justify-self:start}
body[data-concept=quiet] .quiet-answer small{grid-column:2;color:var(--strain);font:10px var(--mono)}
body[data-concept=quiet] .quiet-question-note{color:#aaa;font:11px/1.6 var(--mono);margin:12px 0 0}
body[data-concept=quiet] .quiet-dock{position:fixed;z-index:10;width:340px;border:1px solid #665333;border-radius:9px;background:rgba(20,19,17,.96);box-shadow:var(--shadow)}
body[data-concept=quiet] .quiet-plate{padding:10px 12px;width:100%;text-align:left;background:#c9a45c13;border-radius:9px 9px 0 0}
body[data-concept=quiet] .quiet-plate b{display:block;font:10px var(--mono);color:#daba7b;letter-spacing:.05em}
body[data-concept=quiet] .quiet-plate span{display:block;font-size:12px;margin-top:5px;color:#eee}
body[data-concept=quiet] .quiet-mini{border-top:1px solid var(--line);padding:9px 12px;font-size:11px;color:#aaa}
body[data-concept=quiet] .quiet-mini>div{display:flex;justify-content:space-between;align-items:flex-start;gap:6px}
body[data-concept=quiet] .quiet-mini small{font:10px/1.5 var(--mono);color:#999;max-width:145px;text-align:right}
body[data-concept=quiet] .quiet-mini .quiet-score{margin-top:5px}
body[data-concept=quiet] .quiet-mini .quiet-agent{margin-left:7px}
body[data-concept=quiet] .quiet-edge{position:fixed;z-index:10;max-width:250px}
body[data-concept=quiet] .quiet-check[data-answer] strong{color:#c9a45c}
body[data-concept=quiet] .chip.quiet-witness{pointer-events:auto;cursor:pointer;border-color:var(--ok);color:var(--ok);font:10px var(--mono);letter-spacing:0;opacity:1!important;translate:0 0!important}
body[data-concept=quiet] .quiet-witness .mono{background:var(--ok)!important;color:#0b1815}
body[data-concept=quiet] .quiet-witness[data-polarity=must-not]{color:var(--fail);border-color:var(--fail)}
body[data-concept=quiet] .quiet-witness[data-polarity=must-not] .mono{background:var(--fail)!important}
body[data-concept=quiet] .quiet-witness .rd{display:none!important}
body[data-concept=quiet] .quiet-link{position:fixed;inset:0;pointer-events:none;z-index:11}
body[data-concept=quiet] .quiet-link path{stroke:#c9a45c88;fill:none;stroke-width:1}
body[data-concept=quiet] .quiet-link circle{fill:#c9a45c}
body[data-concept=quiet] .quiet-fog{position:fixed;pointer-events:none;width:220px;height:130px;border-radius:50%;background:radial-gradient(ellipse,#b3a5901e,#ad9c7d0c 55%,transparent 72%);z-index:2}
body[data-concept=quiet] .terr[data-id=paypal-payouts][data-state=claimed] .iso-line{stroke:#c9a45c}
body[data-concept=quiet] button:focus-visible{outline:1px solid #c9a45c;outline-offset:3px}
@keyframes quiet-ink{from{clip-path:inset(100% 0 0)}to{clip-path:inset(0)}}
body[data-concept=quiet] .quiet-ink{animation:quiet-ink .5s ease-out}
body[data-concept=quiet] .quiet-flagline{display:flex;align-items:baseline;gap:9px;flex-wrap:wrap}
body[data-concept=quiet] [data-tone=wait]{color:var(--wait)!important}
body[data-concept=quiet] [data-tone=local]{color:#b9a273!important}
body[data-concept=quiet] [data-evidence-state=stale]{color:#9b917f!important;text-decoration:underline dashed #9b917f80;text-underline-offset:3px}
body[data-concept=quiet] .quiet-lease{font:10px var(--mono);color:#a99a7c;white-space:nowrap}
body[data-concept=quiet] .quiet-plate .quiet-lease{display:inline;margin:0;font-size:10px}
body[data-concept=quiet] .quiet-kv{border-top:1px solid var(--line);padding:6px 12px 7px;font:10px/1.55 var(--mono);color:#8f897e}
body[data-concept=quiet] .quiet-card .quiet-kv{padding:7px 16px 8px;background:#ffffff03}
body[data-concept=quiet] .quiet-kv header{display:flex;gap:6px;border:0;padding:0;color:#9f9583;font:10px var(--mono)}
body[data-concept=quiet] .quiet-kv header b{margin-left:auto;font-weight:400;color:#c9a45c}
body[data-concept=quiet] .quiet-kv header b[data-tone]{color:var(--wait)}
body[data-concept=quiet] .quiet-kv-row{display:grid;grid-template-columns:auto minmax(0,1fr) auto;gap:8px;white-space:nowrap}
body[data-concept=quiet] .quiet-kv-row b{font-weight:400;color:#c8bfae}
body[data-concept=quiet] .quiet-kv-row span{overflow:hidden;text-overflow:ellipsis;color:#7f796f}
body[data-concept=quiet] .quiet-kv-row i{font-style:normal;color:#9f9583}
body[data-concept=quiet] .quiet-kv-row[data-state=held] b,body[data-concept=quiet] .quiet-kv-row[data-state=held] i{color:var(--wait)}
body[data-concept=quiet] .quiet-kv-row[data-state=failed] i,body[data-concept=quiet] .quiet-kv-row[data-state=rejected] i{color:var(--fail)}
body[data-concept=quiet] .quiet-kv-row[data-state=sending] i{color:#777}
body[data-concept=quiet] .quiet-reading[data-state=held] header span,body[data-concept=quiet] .quiet-mini[data-state=held] small{color:var(--wait)}
body[data-concept=quiet] .quiet-reading[data-state=unknown] header span,body[data-concept=quiet] .quiet-mini[data-state=unknown] small{color:#9a9a9a}
body[data-concept=quiet] .quiet-bezel{display:inline-grid;place-items:center;width:18px;height:14px;margin-right:6px;border:1px dotted #8a8a8a;border-radius:50%;font:9px var(--mono);color:#bbb;background:repeating-linear-gradient(135deg,transparent 0 3px,#ffffff1a 3px 4px);vertical-align:-2px}
body[data-concept=quiet] .quiet-src{margin-left:auto;color:#8a8478}
body[data-concept=quiet] .egress{cursor:pointer}
body[data-concept=quiet] .egress:hover{color:var(--text)}
body[data-concept=quiet] .egress[data-kv=offline] .dot,body[data-concept=quiet] .egress[data-kv=checked] .dot{background:var(--wait)}
body[data-concept=quiet] .quiet-panel{position:fixed;z-index:30;top:62px;right:16px;width:380px;max-height:calc(100vh - 140px)}
body[data-concept=quiet] .quiet-panel .quiet-kv-row{padding:3px 0;border-top:1px solid var(--line)}
body[data-concept=quiet] .quiet-panel .quiet-kv-row span{white-space:normal}
body[data-concept=quiet] .quiet-note{margin:0 0 10px;font-size:11px;line-height:1.5;color:#999}
body[data-concept=quiet] .quiet-panel .quiet-primary{margin:8px 0 12px}
body[data-concept=quiet] .quiet-panel .quiet-section{margin:12px 0 4px}
body[data-concept=quiet] .quiet-prov{position:fixed;z-index:31;width:390px}
body[data-concept=quiet] .quiet-prov p{margin:0 0 10px;font:11px/1.7 var(--mono);color:#ddd}
body[data-concept=quiet] .quiet-prov dl{display:grid;grid-template-columns:86px 1fr;gap:4px 10px;margin:0;font:10px/1.5 var(--mono)}
body[data-concept=quiet] .quiet-prov dt{color:#888}
body[data-concept=quiet] .quiet-prov dd{margin:0;color:#ccc;overflow-wrap:anywhere}
body[data-concept=quiet] .quiet-bindings{border-top:1px solid var(--line);margin-top:10px;padding-top:8px;font:10px var(--mono);color:#999}
body[data-concept=quiet] .quiet-bindings summary{cursor:pointer;list-style:none}
body[data-concept=quiet] .quiet-bindings summary span{float:right;color:#777}
body[data-concept=quiet] .quiet-bindings button{display:grid;grid-template-columns:110px 1fr;gap:8px;width:100%;text-align:left;padding:4px 0;border-top:1px solid var(--line);font:10px var(--mono);color:#bbb}
body[data-concept=quiet] .quiet-bindings button span{color:#888;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
body[data-concept=quiet] .chip.quiet-bot{opacity:1!important}
body[data-concept=quiet] .chip small{font:9px var(--mono);color:#9a9a9a;letter-spacing:0;text-transform:none;font-weight:400}
body[data-concept=quiet] .chip[data-state=dark]{border-color:#555;color:#999}
body[data-concept=quiet] .chip[data-state=dark] .mono{background:#3a3a3a;color:#aaa}
body[data-concept=quiet] .ent[data-state=dark]{opacity:.45}
body[data-concept=quiet] .bl .quiet-holder{color:var(--wait)}
body[data-concept=quiet] #cbar{display:none}
body[data-concept=quiet]:not(.alt) .quiet-say,body[data-concept=quiet]:not(.alt) .br .motion{display:none}
body[data-concept=quiet] .quiet-g{display:inline-flex;gap:3px;font:11px var(--mono);letter-spacing:0}
body[data-concept=quiet] .quiet-g [data-result=pass]{color:var(--ok)!important}
body[data-concept=quiet] .quiet-g [data-result=fail]{color:var(--fail)!important}
body[data-concept=quiet] .quiet-g [data-result=pending]{color:var(--wait)!important}
body[data-concept=quiet] .quiet-n{font:11px var(--mono);color:#cfcfcf!important}
body[data-concept=quiet] .quiet-reading header>span,body[data-concept=quiet] .quiet-mini small{display:inline-flex;align-items:baseline;gap:7px;flex-wrap:wrap;justify-content:flex-end}
body[data-concept=quiet] .quiet-pol{font-style:normal;font-weight:400;margin-right:4px}
body[data-concept=quiet] .quiet-ev{font-style:normal;font-weight:400;color:#9a9a9a;letter-spacing:0}
body[data-concept=quiet] .quiet-check{grid-template-columns:22px auto 1fr;align-items:baseline}
body[data-concept=quiet] .quiet-check small{grid-column:auto;font-size:11px}
body[data-concept=quiet] .quiet-check .quiet-say{grid-column:2 / span 2}
body[data-concept=quiet] .quiet-passive b{font-weight:500;color:#ccc;margin-right:6px}
body.alt[data-concept=quiet] .chip.quiet-bot{pointer-events:auto}
`,
  html: `
<div class="quiet-ui" id="quiet-ui">
  <section class="quiet-guide" id="quiet-guide" aria-label="Work orders">
    <div class="quiet-eyebrow quiet-say">Work orders · from witnesses</div>
    <h2>Pin real moments.</h2>
    <p class="quiet-say">Agents replay their reading before they claim.</p>
    <button data-quiet-go="w1"><i class="quiet-dot"></i>Retry · 15:44 <small class="quiet-say">Replay</small></button>
    <button data-quiet-go="w2"><i class="quiet-dot"></i>PayPal · 15:35 <small class="quiet-say">Replay</small></button>
    <button id="quiet-pin-not">Pin must not <small>M</small></button>
    <button id="quiet-pin-keep">Pin must keep <small>Shift M</small></button>
    <div class="quiet-gate" id="quiet-pin-gate"></div>
    <button id="quiet-post">Post <small>P</small></button>
    <div class="quiet-gate" id="quiet-post-gate"></div>
    <footer class="quiet-say">M pin · Shift M keep · P post / inspect<br>I question · Shift 1 / 2 answer<br>[ ] events · , . minutes · Esc close</footer>
  </section>
  <div class="quiet-hint" id="quiet-hint" hidden>P inspect order · I answer at sqlite</div>
  <div class="quiet-fog" id="quiet-fog"></div>
  <button class="quiet-mark" id="quiet-question-pin" hidden>QUESTION · sqlite <kbd>I</kbd></button>
  <button class="quiet-mark quiet-edge" id="quiet-question-edge" hidden></button>
  <button class="quiet-mark quiet-edge" id="quiet-order-edge" hidden></button>
  <div id="quiet-witness-edges"></div>
  <div class="quiet-dock" id="quiet-dock" hidden>
    <button class="quiet-plate" id="quiet-plate"></button>
    <div class="quiet-kv" id="quiet-kv-dock"></div>
    <div id="quiet-mini-readbacks"></div>
  </div>
  <svg class="quiet-link" id="quiet-link"></svg>
  <section class="quiet-card" id="quiet-card" role="dialog" aria-label="Work order" hidden></section>
  <section class="quiet-card quiet-panel" id="quiet-kv-panel" role="dialog" aria-label="Kvarm connection" hidden></section>
  <section class="quiet-card quiet-prov" id="quiet-prov" role="dialog" aria-label="Provenance" hidden></section>
  <span id="quiet-announcement" role="status" aria-live="polite" style="position:absolute;width:1px;height:1px;overflow:hidden;clip-path:inset(50%)"></span>
</div>`,
  script: `
const quietData = ${JSON.stringify({ ORDER, WITNESSES, QUESTION, READBACKS, SHEET, KV: { origin: KVARM.origin, host: KVARM.host, repo: KVARM.repo, staleAfter: KVARM.staleAfter, ...KVARM.orders } })};
const quietState = {
  witnesses: [], order: { state: "draft", claimedBy: null },
  question: { answered: null }, arrived: [], open: null, sentence: "", scoring: "waiting", rev: null,
  kvarm: { script: "ok", connection: "ok", issue: null, dispatched: false, lease: null, claim: null, log: [], held: [], bytes: 0, panel: false, lastReturn: null }
};
let quietRestoreFocus = null, quietCardTop = null;
let quietLiveMs = 0, quietScoreMs = 0, quietVisibleMs = 0, quietLastFrame = null;
let quietWaitingBase = 0;
let quietInitialized = false;
let quietKvMs = 0, quietInflight = [], quietLeaseShown = null, quietEgressHtml = null;
let quietProvKey = null, quietHover = null, quietTerritory = null;
function quietActive() {
  return conceptActive("quiet") && document.body.dataset.concept === "quiet" && new URLSearchParams(location.hash.slice(1)).get("c") === "quiet";
}
function quietEscape(value) {
  return String(value).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
}
// The live region keeps the full sentence; the visible toast carries the short form.
function quietAnnounce(message, short = message) {
  $("#quiet-announcement").textContent = message;
  toast(short);
}
function quietPoint(id, layer = false, camera = view) {
  const e = ENT[id], a = layer ? k.iso([e.x + 21, e.y - 4, 26 + 14 * e.explode], P) : anchors[id];
  return a ? [camera.x + a[0] * camera.s, camera.y + a[1] * camera.s] : [0, 0];
}
function quietFocusMachine(id) {
  selected = id;
  $("#insp").classList.remove("open");
  document.body.classList.remove("has-insp");
  session = null; review = null;
  if (explode.id) { explode.id = null; explode.t = explode.goal = 0; renderPlant(); }
  const p = quietPoint(id, quietState.open === "question", goal);
  const left = quietState.order.state === "draft" ? 420 : 190;
  const right = innerWidth - (quietState.open ? $("#quiet-card").offsetWidth + 250 : 420);
  const bottom = rpOn() ? $("#rpdeck").getBoundingClientRect().top - 100 : innerHeight - 280;
  const x = Math.max(left, Math.min(right, p[0])), y = Math.max(240, Math.min(bottom, p[1]));
  if (x !== p[0] || y !== p[1]) {
    goal.x += x - p[0]; goal.y += y - p[1];
    animate();
  }
  paintSelection();
}
function quietAgent(r) {
  return r.agent + (r.agent === "Droid" ? " · second session" : "");
}
function quietBytes(n) {
  return n.toLocaleString("en-US") + " B";
}
function quietWinner() {
  return quietData.READBACKS.find((r) => r.answers.q1 === quietState.question.answered && quietState.witnesses.every((w) => w.id === "w3" || r.replay[w.id] === "pass"));
}
function quietUnmatched() {
  const l = quietState.kvarm.lease;
  return !!l && !l.readback;
}
function quietFlag() {
  const o = quietState.order, kv = quietState.kvarm, l = kv.lease;
  if (o.state === "draft") return { text: "Post an order" };
  if (o.state === "stale") return { text: "LEASE STALE · " + l.gen + " expired", ev: "stale" };
  if (o.state === "claimed") return { text: "CLAIMED · " + o.claimedBy + " · lease " + l.gen };
  if (quietUnmatched()) return { text: "UNMATCHED · " + l.holder + " holds " + l.gen + " · no readback", tone: "wait" };
  if (o.state === "matched") return { text: kv.connection === "ok" ? "MATCHED · claiming" : "MATCHED · claim waits for connection", tone: "wait" };
  if (!kv.issue || !kv.dispatched) return { text: kv.connection === "ok" ? "POSTED LOCALLY" : "POSTED LOCALLY · not in Kvarm", tone: "local" };
  return { text: "OPEN · kvarm #" + kv.issue };
}
function quietFlagAttrs(f) {
  return (f.tone ? ' data-tone="' + f.tone + '"' : '') + (f.ev ? ' data-evidence-state="' + f.ev + '"' : '');
}
function quietSetFlag(el, f) {
  el.textContent = f.text;
  if (f.tone) el.dataset.tone = f.tone; else delete el.dataset.tone;
  if (f.ev) el.dataset.evidenceState = f.ev; else delete el.dataset.evidenceState;
}
function quietLeaseText() {
  const l = quietState.kvarm.lease;
  if (!l || l.holder !== (quietState.order.claimedBy || (quietState.order.state === "stale" && l.holder))) return "";
  if (l.state === "expired") return "expired · last renewed " + Math.round((quietKvMs - l.renewedAt) / 1000) + " s ago";
  const left = Math.max(0, Math.ceil(quietData.KV.leaseS - (quietKvMs - l.renewedAt) / 1000));
  return Math.floor(left / 60) + ":" + String(left % 60).padStart(2, "0");
}
function quietEyebrow() {
  return quietEscape(quietData.ORDER.territory + (quietState.rev ? ' · ' + quietState.rev : '')) + '<span class="quiet-say"> · ' + quietEscape(quietData.ORDER.sheet) + '</span>';
}
function quietClaimLine() {
  const o = quietState.order, l = quietState.kvarm.lease, w = quietWinner();
  if (o.state === "claimed") return l.session + " · readback matched every check";
  if (o.state === "stale") return l.session + " · heartbeat rejected · " + quietData.KV.stale.code;
  if (quietUnmatched()) return l.holder + " holds " + l.gen + " in Kvarm · no Acan readback";
  if (o.state === "matched") return w.session + " · readback matched · claim held for Kvarm";
  return "Agents claim after a matching readback.";
}
function quietHeader(label, title, plate = false, html = plate) {
  const f = plate ? quietFlag() : { text: title };
  return '<header' + (plate ? ' data-quiet-plate data-quiet-binding="flag"' : '') + '><div><div class="quiet-eyebrow"' + (plate ? ' id="quiet-card-eyebrow" data-quiet-binding="flag"' : '') + '>' + (html ? label : quietEscape(label)) + '</div>' + (plate ? '<div class="quiet-flagline"><h3 id="quiet-card-status" data-quiet-binding="flag"' + quietFlagAttrs(f) + '>' + quietEscape(f.text) + '</h3><small class="quiet-lease">' + quietLeaseText() + '</small></div>' : '<h3>' + quietEscape(title) + '</h3>') + '</div><button class="quiet-close" data-quiet-close aria-label="Close" title="Close (Esc)">×</button></header>';
}
function quietCheckShort(w) {
  return w.check.split(" · ").slice(1).join(" · ");
}
function quietPolGlyph(polarity) {
  return polarity === "must-not" ? "⊘" : "●";
}
function quietEvGlyph(ev) {
  return { declared: "◇", authenticated: "◆", verified: "◈", stopped: "■" }[ev] || "◇";
}
function quietCheckRows() {
  const posted = quietState.order.state !== "draft";
  return quietState.witnesses.map((w) => {
    const answer = w.id === "w3", pol = w.polarity.replace("-", " ");
    return '<div class="quiet-check" data-polarity="' + w.polarity + '" data-quiet-binding="' + w.id + '"' + (answer ? ' data-answer' : '') + '><b>' + w.id + '</b><strong title="' + pol + '"><i class="quiet-pol">' + quietPolGlyph(w.polarity) + '</i>' + w.clock + '<span class="quiet-say"> · ' + (answer ? "answer · sqlite" : pol) + '</span></strong><small>' + quietEscape(quietCheckShort(w)) + '</small><p class="quiet-say">' + quietEscape(w.reason) + '</p><small class="quiet-say">' + (posted ? 'Locked' : 'Becomes locked') + ' · ' + quietEscape(answer ? "answered at sqlite · " + quietCheckShort(w) : w.check) + '</small></div>';
  }).join("");
}
function quietReadback(r) {
  const arrived = quietState.arrived.includes(r.session), answer = quietState.question.answered;
  const matches = !!answer && r.answers.q1 === answer;
  const settling = quietState.scoring === "rescoring";
  const count = Object.values(r.replay).filter((v) => v === "pass").length + (matches ? 1 : 0);
  const o = quietState.order, claim = quietState.kvarm.claim, mine = claim && claim.session === r.session;
  const outcome = o.state === "claimed" ? ["matches", " · matches · claimed", ""] : o.state === "stale" ? ["held", " · matches · lease expired", "lease expired"] : mine && claim.state === "rejected" ? ["held", " · matches · Kvarm rejected", "claim rejected"] : o.state === "matched" ? ["held", " · matches · claim waits", "claim waits"] : ["matches", " · matches · claiming", "claiming"];
  const state = !arrived ? "waiting" : settling ? "rescoring" : !answer ? "differs" : matches ? outcome[0] : "yields";
  const status = !arrived ? "awaiting readback" : settling ? "re-scoring w3" : !answer ? "p-2307 differs" : count + " of " + quietState.witnesses.length + (matches ? outcome[1] : " · yields");
  const word = !arrived ? "awaiting" : settling ? "checking" : !answer ? "" : matches ? outcome[2] : "yields";
  const tally = arrived && answer && !settling ? count + "/" + quietState.witnesses.length : "";
  return { arrived, matches, state, status, word, tally, choice: r.answers.q1 === "1" ? "pending" : "failed" };
}
function quietSource() {
  return "readback · Acan" + (quietState.rev === "r2" && quietState.scoring === "settled" ? " · re-scored on r2" : quietState.rev ? " · " + quietState.rev : "");
}
function quietGlyphs(r) {
  const rb = quietReadback(r);
  if (!rb.arrived) return "";
  return '<span class="quiet-g">' + quietState.witnesses.map((w) => {
    const result = w.id === "w3" ? quietState.scoring === "rescoring" ? "pending" : rb.matches ? "pass" : "fail" : r.replay[w.id];
    return '<span data-result="' + result + '" title="' + w.id + ' ' + (result === "pending" ? "checking" : result) + '">' + (result === "pass" ? "✓" : result === "fail" ? "✕" : "…") + '</span>';
  }).join("") + '</span>';
}
function quietTally(r, rb) {
  return quietGlyphs(r) + (rb.tally ? '<span class="quiet-n">' + rb.tally + '</span>' : '') + (rb.word ? '<span>' + rb.word + '</span>' : '') + '<span class="quiet-say">' + rb.status + '</span>';
}
function quietScores(r, source = false) {
  const rb = quietReadback(r);
  if (!rb.arrived) return "";
  return '<div class="quiet-score quiet-say">' + quietState.witnesses.map((w) => {
    const result = w.id === "w3" ? quietState.scoring === "rescoring" ? "pending" : rb.matches ? "pass" : "fail" : r.replay[w.id];
    return '<span data-result="' + result + '">' + w.id + ' ' + (result === "pending" ? "checking" : result) + '</span>';
  }).join("") + (source ? '<span class="quiet-src">' + quietSource() + '</span>' : '') + '</div>';
}
function quietHolderHtml(mini) {
  if (!quietUnmatched()) return "";
  const l = quietState.kvarm.lease, name = '<span class="quiet-bezel" data-evidence-state="unknown" title="no readback">?</span>' + l.holder + '<span class="quiet-say"> · ' + l.session + '</span>';
  const right = l.gen + ' · passive<span class="quiet-say"> · holds ' + l.gen + ' · no readback</span>';
  return mini ? '<div class="quiet-mini" data-state="unknown" data-quiet-binding="holder"><div><span>' + name + '</span><small>' + right + '</small></div></div>'
    : '<div class="quiet-reading" data-state="unknown" data-quiet-binding="holder"><header><b>' + name + '</b><span>' + right + '</span></header><p class="quiet-say">Claimed t-41 in Kvarm with its own token. Acan has no readback for ' + quietState.rev + ', so the territory stays fogged.</p></div>';
}
function quietReadingHtml(r) {
  const rb = quietReadback(r);
  const agent = r.agent + (r.agent === "Droid" ? '<span class="quiet-say"> · second session</span>' : '');
  if (!rb.arrived) return '<div class="quiet-reading" data-state="waiting" data-quiet-readback="' + r.session + '"><header>' + agent + '<span>' + quietTally(r, rb) + '</span></header></div>';
  const text = quietEscape(r.text).replace(quietEscape(r.conflict), '<mark>' + quietEscape(r.conflict) + '</mark>');
  const choice = '<em class="quiet-choice' + (quietState.question.answered ? ' quiet-say' : '') + '">' + rb.choice + ' · </em>';
  return '<div class="quiet-reading" data-state="' + rb.state + '" data-quiet-readback="' + r.session + '" data-quiet-binding="rb:' + r.session + '"><header><b>' + choice + agent + '</b><span>' + quietTally(r, rb) + '</span></header><p class="quiet-say">' + text + '</p>' + quietScores(r, true) + '</div>';
}
function quietReadbacksHtml() {
  return quietData.READBACKS.map(quietReadingHtml).join("") + quietHolderHtml(false);
}function quietCanPost() {
  return quietState.order.state === "draft" && quietState.witnesses.length === quietData.WITNESSES.length && !!quietState.sentence.trim();
}
function quietGrowSentence(input) {
  input.style.height = "auto";
  input.style.height = Math.max(58, input.scrollHeight + 2) + "px";
  input.scrollTop = input.scrollLeft = 0;
  $("#quiet-card").scrollTop = 0;
}
function quietRowHtml(r) {
  return '<div class="quiet-kv-row" data-state="' + r.state + '" title="' + quietEscape(r.destination + ' · ' + r.reason) + '"><b>' + r.op + (r.count > 1 ? ' ×' + r.count : '') + '</b><span>' + quietEscape(r.reason) + '</span><i>' + (r.result || (r.state === "held" ? "held" : r.state === "sending" ? "sending" : quietBytes(r.bytes))) + '</i></div>';
}
function quietStripHtml(max) {
  const kv = quietState.kvarm, rows = [...kv.log, ...kv.held.map((h) => h.row)];
  return '<header><span>↗ ' + quietData.KV.host + (kv.connection === "ok" ? '' : ' · ' + (kv.connection === "offline" ? "offline" : "checked")) + ' · ' + rows.length + (rows.length === 1 ? ' op' : ' ops') + '</span>' + (kv.held.length ? '<b data-tone="wait">' + kv.held.length + ' held</b>' : '<b>' + quietBytes(kv.bytes) + '</b>') + '</header>' + rows.slice(-max).map(quietRowHtml).join("");
}
function quietBinding(key) {
  const K = quietData.KV, kv = quietState.kvarm, rev = quietState.rev || "draft", sheet = quietData.SHEET.rev;
  const since = (ms) => ms === null || ms === undefined ? null : Math.max(0, Math.round((quietKvMs - ms) / 1000));
  const remote = kv.connection === "ok" ? "valid" : "stale";
  if (key === "flag") {
    const f = quietFlag(), l = kv.lease;
    return { entity: "Order " + quietData.ORDER.id, channel: "order.claim", value: f.text, state: f.ev || (kv.issue ? remote : "valid"), sourceClass: l ? l.evidence + " · Kvarm lease" : kv.issue ? "authenticated · Kvarm issue" : "local", locator: kv.issue ? "kvarm://" + K.host + "/" + K.repo + "/issues/" + kv.issue + " · task " + K.task : "local://.acan/orders/" + quietData.ORDER.id + ".yaml", revision: rev + " · " + sheet, age: since(kv.lastReturn), policy: "Kvarm fields stale after " + K.staleAfter, meaning: "CLAIMED needs an Acan readback match and a live Kvarm lease of the same generation" };
  }
  if (key === "holder") {
    const l = kv.lease;
    return { entity: l.holder + " · " + l.session, channel: "kvarm.lease", value: "holds " + l.gen, state: remote, sourceClass: l.evidence, locator: "kvarm://" + K.host + "/" + K.repo + "/tasks/" + K.task, revision: rev + " · " + sheet, age: since(kv.lastReturn), policy: "Kvarm fields stale after " + K.staleAfter, meaning: "No Acan readback on " + rev + ". Never drawn as CLAIMED." };
  }
  if (key === "q1") {
    const q = quietData.QUESTION;
    return { entity: "Question q1 · " + q.entity, channel: "readback.disagreement", value: "p-2307 · pending or failed", state: "valid", sourceClass: "observed · Acan replay", locator: "acan://orders/" + quietData.ORDER.id + "/r1/questions/q1", revision: "r1 · " + sheet, age: "replay " + q.clock, policy: "raised when two readbacks differ", meaning: q.text };
  }
  if (key.startsWith("rb:") || key.startsWith("bot:")) {
    const r = quietData.READBACKS.find((r) => key === "rb:" + r.session || key === "bot:" + r.bot);
    if (key.startsWith("bot:")) {
      const l = kv.lease, live = l && l.session === r.session && l.state === "live", bot = ENT[r.bot];
      return { entity: quietAgent(r), channel: "agent.principal", value: r.session, state: bot && bot.state === "dark" ? "stopped" : "valid", sourceClass: live ? l.evidence : "declared", locator: live ? "kvarm token · principal " + r.session : "mcp://local/" + r.session, revision: rev + " · " + sheet, age: null, policy: bot && bot.state === "dark" ? "lifecycle · " + K.stale.code : "session handshake", meaning: live ? "Claimed t-41 with its own Kvarm token" : "Connected over MCP. Identity is declared by the session." };
    }
    const rb = quietReadback(r);
    return { entity: quietAgent(r) + " · " + r.session, channel: "readback.score", value: rb.arrived ? rb.status : "not received", state: !rb.arrived ? "unknown" : quietState.scoring === "rescoring" ? "stale" : "valid", sourceClass: "observed · Acan replay", locator: "acan://orders/" + quietData.ORDER.id + "/" + rev + "/readbacks/" + r.session, revision: rev + " · " + sheet, age: null, policy: "re-scored on every order revision", meaning: "readback · Acan. Kvarm stores no readback." };
  }
  if (key === "budget") {
    const b = quietData.ORDER.budgets[0];
    return { entity: "Order " + quietData.ORDER.id + " · budget", channel: "order.budget", value: b, state: "valid", sourceClass: "declared · your order", locator: "local://.acan/orders/" + quietData.ORDER.id + ".yaml#budgets", revision: rev + " · " + sheet, age: null, policy: quietState.order.state === "draft" ? "editable until Post" : "locked on Post", meaning: "Every attempt keeps " + b };
  }
  const w = quietState.witnesses.find((x) => x.id === key);
  if (!w) return null;
  return { entity: "Witness " + w.id + " · " + w.entity, channel: "witness." + w.polarity, value: w.observed.metric + " " + w.observed.value + (w.observed.unit ? " " + w.observed.unit : ""), state: "valid", sourceClass: w.id === "w3" ? "declared · your answer" : "observed · replay", locator: w.id === "w3" ? "acan://orders/" + quietData.ORDER.id + "/r2#w3" : "replay://" + K.repo + "/" + w.entity + "@" + w.clock, revision: sheet, age: (w.id === "w3" ? "answered at " : "replay ") + w.clock, policy: quietState.order.state === "draft" ? "editable until Post" : "locked on Post", meaning: w.check };
}
function quietBindText(b) {
  return [b.value, b.state, b.sourceClass, b.channel, b.locator, "revision " + b.revision, typeof b.age === "number" ? b.age + " s ago" : b.age || "age not reported"].join(" · ");
}
function quietBindingKeys() {
  return ["flag", "budget", ...quietState.witnesses.map((w) => w.id), ...quietData.READBACKS.filter((r) => quietState.arrived.includes(r.session)).flatMap((r) => ["rb:" + r.session, "bot:" + r.bot]), ...(quietUnmatched() ? ["holder"] : [])];
}
function quietBindingsHtml() {
  return quietBindingKeys().map((key) => { const b = quietBinding(key); return '<button data-quiet-prov="' + key + '"><b>' + quietEscape(b.channel) + '</b><span>' + quietEscape(b.value + ' · ' + b.state + ' · ' + b.sourceClass) + '</span></button>'; }).join("");
}
function quietProvOpen(key, anchor) {
  const b = quietBinding(key);
  if (!b) return;
  quietProvKey = key;
  const el = $("#quiet-prov");
  el.innerHTML = '<header><div><div class="quiet-eyebrow">Binding · ' + quietEscape(b.sourceClass) + '</div><h3>' + quietEscape(b.entity) + '</h3></div><button class="quiet-close" data-quiet-prov-close aria-label="Close provenance" title="Close (Esc)">×</button></header><div class="quiet-body"><p id="quiet-prov-text">' + quietEscape(quietBindText(b)) + '</p><dl><dt>Channel</dt><dd>' + quietEscape(b.channel) + '</dd><dt>Value · state</dt><dd data-evidence-state="' + b.state + '">' + quietEscape(b.value + ' · ' + b.state) + '</dd><dt>Source</dt><dd>' + quietEscape(b.sourceClass) + '</dd><dt>Locator</dt><dd>' + quietEscape(b.locator) + '</dd><dt>Revision</dt><dd>' + quietEscape(b.revision) + '</dd><dt>Freshness</dt><dd>' + quietEscape((typeof b.age === "number" ? b.age + " s ago" : b.age || "age not reported") + ' · ' + b.policy) + '</dd><dt>Meaning</dt><dd>' + quietEscape(b.meaning) + '</dd></dl></div>';
  el.hidden = false;
  const r = anchor.getBoundingClientRect(), w = el.offsetWidth, h = el.offsetHeight;
  const left = Math.max(20, Math.min(innerWidth - w - 20, r.left));
  const top = r.bottom + 10 + h <= innerHeight - 50 ? r.bottom + 10 : Math.max(70, r.top - h - 10);
  el.style.left = left + "px"; el.style.top = top + "px";
}
function quietProvClose() {
  quietProvKey = null;
  $("#quiet-prov").hidden = true;
}
function quietPill() {
  const el = $(".egress"), kv = quietState.kvarm;
  if (!el) return;
  if (quietEgressHtml === null) quietEgressHtml = el.innerHTML;
  el.dataset.kv = kv.connection === "ok" ? "ok" : kv.connection === "offline" ? "offline" : "checked";
  el.setAttribute("role", "button"); el.tabIndex = 0;
  el.title = "Kvarm · " + quietData.KV.origin + " · declared egress · U";
  el.innerHTML = '<span class="dot"></span>' + (kv.connection === "ok" ? 'local + kvarm · ' + quietBytes(kv.bytes) + ' out' : 'kvarm ' + (kv.connection === "offline" ? "offline" : "checked") + ' · ' + kv.held.length + ' held');
}
function quietPanelRender() {
  const kv = quietState.kvarm, K = quietData.KV, p = $("#quiet-kv-panel");
  p.hidden = !kv.panel;
  if (!kv.panel) return;
  const action = kv.connection === "offline" ? '<div class="quiet-status" data-tone="wait">Offline · Kvarm fields are stale · writes are held</div><button class="quiet-primary" data-quiet-kv="reconnect">Reconnect <kbd>Enter</kbd></button>'
    : kv.connection === "checking" ? '<div class="quiet-status">checking heads and leases</div>'
    : kv.connection === "checked" ? '<div class="quiet-status" id="quiet-kv-checked">checked heads and leases · ' + quietEscape(K.checked) + '</div><button class="quiet-primary" data-quiet-kv="send">Send ' + kv.held.length + ' held ' + (kv.held.length === 1 ? 'write' : 'writes') + ' <kbd>Enter</kbd></button>' : '';
  p.innerHTML = '<header><div><div class="quiet-eyebrow">Kvarm · declared egress</div><h3>' + K.host + '</h3></div><button class="quiet-close" data-quiet-panel-close aria-label="Close Kvarm" title="Close (Esc)">×</button></header><div class="quiet-body"><p class="quiet-note">Opt-in connection to your own origin, ' + K.origin + '. Scripted fixture. No browser network requests.</p>' + action +
    '<div class="quiet-section">Held writes · ' + kv.held.length + '</div>' + (kv.held.map((h) => quietRowHtml(h.row)).join("") || '<div class="quiet-kv-row"><b>0</b><span>nothing held locally</span><i></i></div>') +
    '<div class="quiet-section">Network log · ' + quietBytes(kv.bytes) + '</div>' + (kv.log.map(quietRowHtml).join("") || '<div class="quiet-kv-row"><b>0 B</b><span>no outbound operations</span><i></i></div>') + '</div>';
}
function quietPanelToggle(open = !quietState.kvarm.panel) {
  quietState.kvarm.panel = open;
  quietPanelRender();
  if (open) { const b = $("#quiet-kv-panel .quiet-primary") || $("#quiet-kv-panel .quiet-close"); b.focus({ preventScroll: true }); }
}
function quietKvLoud() {
  const kv = quietState.kvarm;
  return kv.connection !== "ok" || kv.held.length > 0;
}
function quietKvRender() {
  quietPill();
  const posted = quietState.order.state !== "draft";
  for (const [id, max] of [["#quiet-kv-dock", 2], ["#quiet-kv-card", 3]]) { const el = $(id); if (el) { el.innerHTML = quietStripHtml(max); el.classList.toggle("quiet-say", !quietKvLoud()); } }
  if (posted && quietState.open === "order" && $("#quiet-card-status")) {
    quietSetFlag($("#quiet-card-status"), quietFlag());
    $("#quiet-card-eyebrow").innerHTML = quietEyebrow();
    $("#quiet-claim-session").textContent = quietClaimLine();
    const list = $("#quiet-bindings-list");
    if (list) list.innerHTML = quietBindingsHtml();
    $("#quiet-bindings-count").textContent = quietBindingKeys().length;
  }
  quietLeaseShown = null;
  if (posted) quietRenderDock();
  if (quietState.kvarm.panel) { const had = document.activeElement && document.activeElement.closest("#quiet-kv-panel"); quietPanelRender(); if (had) quietPanelToggle(true); }
}
function quietAt(ms, done) {
  quietInflight.push({ due: quietKvMs + ms, row: null, done });
}
function quietTransmit(row, done) {
  const kv = quietState.kvarm, last = kv.log[kv.log.length - 1], spec = quietData.KV.ops[row.op];
  if (row.op === "heartbeat_task" && last && last.op === "heartbeat_task" && last.state === "sent") { last.count++; last.bytes += spec.bytes; row = last; }
  else kv.log.push(row);
  kv.bytes += spec.bytes;
  row.state = "sending";
  quietInflight.push({ due: quietKvMs + spec.ms, row, done });
  quietKvRender();
}
function quietSend(op, reason, done, force = false) {
  const kv = quietState.kvarm, spec = quietData.KV.ops[op];
  const row = { op, reason: reason || spec.reason, bytes: spec.bytes, destination: quietData.KV.host, state: "held", count: 1 };
  if (kv.connection !== "ok" && !force) { kv.held.push({ row, done }); quietKvRender(); return; }
  quietTransmit(row, done);
}
function quietReconnect() {
  const kv = quietState.kvarm;
  if (kv.connection !== "offline") return;
  kv.connection = "checking";
  quietSend("check_heads_and_leases", null, () => {
    kv.connection = "checked"; quietKvRender(); quietPanelToggle(true);
    quietAnnounce("Checked heads and leases. Enter sends " + kv.held.length + " held writes.");
  }, true);
  quietPanelToggle(true);
}
function quietSendHeld() {
  const kv = quietState.kvarm, n = kv.held.length;
  if (kv.connection !== "checked") return;
  kv.connection = "ok";
  const next = () => { const h = kv.held.shift(); if (!h) return; quietTransmit(h.row, (row) => { if (h.done) h.done(row); next(); }); };
  next();
  quietPanelToggle(true);
  quietAnnounce("Sending " + n + " held writes to Kvarm, in order.");
}
function quietKvTick(delta) {
  quietKvMs += delta;
  const kv = quietState.kvarm, due = quietInflight.filter((f) => f.due <= quietKvMs);
  if (due.length) {
    quietInflight = quietInflight.filter((f) => f.due > quietKvMs);
    for (const f of due) { if (f.row) { f.row.state = "sent"; kv.lastReturn = quietKvMs; } if (f.done) f.done(f.row); }
    quietKvRender();
  }
  const l = kv.lease;
  if (l && l.state === "live" && quietState.order.state === "claimed" && quietKvMs >= l.beatAt) {
    l.beatAt = Infinity;
    quietSend("heartbeat_task", null, (row) => {
      if (kv.script === "stale") { row.state = "failed"; row.result = quietData.KV.stale.code; quietExpire(); }
      else { l.renewedAt = quietKvMs; l.beatAt = quietKvMs + quietData.KV.heartbeatS * 1000; }
    });
  }
  const text = quietLeaseText();
  if (text !== quietLeaseShown) { quietLeaseShown = text; for (const el of document.querySelectorAll(".quiet-ui .quiet-lease")) el.textContent = text; }
}
function quietRenderCard() {
  const card = $("#quiet-card"), open = quietState.open;
  card.hidden = !open;
  if (!open) return;
  if (open.startsWith("w")) {
    const w = quietState.witnesses.find((x) => x.id === open), locked = quietState.order.state !== "draft";
    card.innerHTML = quietHeader(w.id === "w3" ? "Answered at sqlite" : "Replay · " + w.clock, locked ? "Locked witness · " + w.id : "Pin " + w.polarity.replace("-", " ")).replace("<header", '<header data-quiet-binding="' + w.id + '"') +
      '<div class="quiet-body"><div class="quiet-status" data-quiet-binding="' + w.id + '">' + quietEscape(w.observed.metric + ' · ' + w.observed.value + ' ' + w.observed.unit) + '<span class="quiet-say">' + quietEscape(' · ' + w.context) + '</span></div>' +
      (locked ? '<p class="quiet-sentence">' + quietEscape(w.reason) + '</p>' : '<label for="quiet-reason">Reason<span class="quiet-say"> · editable until Post</span></label><textarea id="quiet-reason">' + quietEscape(w.reason) + '</textarea>') +
      '<div class="quiet-locks" data-quiet-binding="' + w.id + '"><span><span class="quiet-say">' + (locked ? "Locked" : "Becomes locked on Post") + ' · </span>' + quietEscape(quietCheckShort(w)) + '</span></div>' +
      (locked ? "" : '<div class="quiet-actions"><span class="quiet-say">Enter keeps the reason</span><button class="quiet-primary" id="quiet-keep">Pin <kbd>Enter</kbd></button></div>') + '</div>';
    if (!locked) $("#quiet-reason").oninput = (ev) => { w.reason = ev.target.value; };
    return;
  }
  if (open === "question") {
    card.innerHTML = quietHeader('sqlite', 'Answer at p-2307').replace("<header", '<header data-quiet-binding="q1"') +
      '<div class="quiet-body" data-quiet-binding="q1"><p class="quiet-sentence">' + quietData.QUESTION.text + '</p>' +
      quietData.QUESTION.options.map((o) => {
        const r = quietData.READBACKS.find((r) => r.answers.q1 === o.key);
        return '<button class="quiet-answer" data-quiet-answer="' + o.key + '"><kbd>Shift ' + o.key + '</kbd><span>' + o.label + '</span><small title="' + quietAgent(r) + ' reads this">' + r.agent + '<span class="quiet-say"> reads this</span></small></button>';
      }).join("") + '<p class="quiet-question-note quiet-say">Your answer becomes a locked witness.<br>The readbacks must match before an agent claims.</p></div>';
    return;
  }
  const posted = quietState.order.state !== "draft";
  card.innerHTML = (posted ? quietHeader(quietEyebrow(), "", true) + '<div class="quiet-kv' + (quietKvLoud() ? '' : ' quiet-say') + '" id="quiet-kv-card" data-quiet-binding="flag">' + quietStripHtml(3) + '</div>' : quietHeader(quietEyebrow(), "Post an order", false, true)) +
    '<div class="quiet-body">' + (posted ? '<div class="quiet-status quiet-say" id="quiet-claim-session">' + quietEscape(quietClaimLine()) + '</div><p class="quiet-sentence">' + quietEscape(quietState.sentence) + '</p>' : '<div class="quiet-status">DRAFT<span class="quiet-say"> · witnesses become locked on Post</span></div><label for="quiet-sentence">One sentence</label><textarea id="quiet-sentence" rows="2" placeholder="' + quietEscape(quietData.ORDER.placeholder) + '">' + quietEscape(quietState.sentence) + '</textarea>') +
    (posted ? '<section class="quiet-readbacks"><div class="quiet-section quiet-say">Readbacks · before code</div><div id="quiet-readbacks">' + quietReadbacksHtml() + '</div></section>' : "") +
    '<div id="quiet-checks">' + quietCheckRows() + '</div><div class="quiet-locks">' + quietData.ORDER.budgets.map((b) => '<span data-quiet-binding="budget"><span class="quiet-say">' + (posted ? 'Locked' : 'Becomes locked') + ' · </span>' + quietEscape(b) + '</span>').join("") + '</div><div id="quiet-passive">' + quietPassiveHtml() + '</div>' +
    (posted ? '<details class="quiet-bindings" id="quiet-bindings"><summary>Sources <b id="quiet-bindings-count" data-quiet-meta>' + quietBindingKeys().length + '</b><span class="quiet-say">Alt + hover any mark</span></summary><div id="quiet-bindings-list">' + quietBindingsHtml() + '</div></details>' : '<div class="quiet-actions"><span id="quiet-post-note">' + quietPostNote() + '</span><button class="quiet-primary" id="quiet-submit"' + (quietCanPost() ? '' : ' disabled') + '>Post <kbd>Enter</kbd></button></div>') + '</div>';
  if (!posted) {
    const input = $("#quiet-sentence");
    quietGrowSentence(input);
    input.oninput = () => {
      quietState.sentence = input.value;
      quietGrowSentence(input);
      $("#quiet-submit").disabled = !quietCanPost();
      $("#quiet-post-note").innerHTML = quietPostNote();
      quietPosition();
    };
    input.onfocus = () => quietGrowSentence(input);
  }
}
function quietPassiveHtml() {
  return quietUnmatched() ? "" : '<p class="quiet-passive"><b>Codex</b>passive<span class="quiet-say"> · cannot see orders · no MCP</span></p>';
}
function quietPostNote() {
  return quietState.witnesses.length < quietData.WITNESSES.length ? "Pin both witnesses first." : quietState.sentence.trim() ? '<span class="quiet-say">No assignment. Agents claim on a match.</span>' : "Write one sentence.";
}
function quietRenderDock() {
  const posted = quietState.order.state !== "draft", answered = !!quietState.question.answered;
  $("#quiet-guide").hidden = posted;
  $("#quiet-hint").hidden = !posted;
  $("#quiet-hint").textContent = quietState.open ? "Esc close · U log · Alt words" : answered ? "P inspect · U log · Alt words" : "I answer · P inspect · Alt words";
  $("#quiet-dock").hidden = !posted || quietState.open === "order";
  const f = quietFlag(), l = quietState.kvarm.lease, lease = quietLeaseText();
  $("#quiet-plate").innerHTML = '<b data-quiet-binding="flag"' + quietFlagAttrs(f) + '>' + quietEscape(f.text) + '</b><span data-quiet-binding="flag">' + (lease ? '<i class="quiet-lease" title="lease time left">' + lease + '</i>' : '') + '<span class="quiet-say">' + quietEscape((l && l.readback ? l.session + ' · ' : '') + quietData.ORDER.sheet) + '</span></span>';
  const strip = $("#quiet-kv-dock");
  strip.innerHTML = quietStripHtml(2);
  strip.classList.toggle("quiet-say", !quietKvLoud());
  $("#quiet-mini-readbacks").innerHTML = '<div class="quiet-kv quiet-say"><header><span>' + quietSource() + '</span></header></div>' + quietData.READBACKS.map((r) => {
    const rb = quietReadback(r);
    return '<div class="quiet-mini" data-state="' + rb.state + '" data-quiet-session="' + r.session + '"' + (rb.arrived ? ' data-quiet-binding="rb:' + r.session + '"' : '') + '><div><span>' + (rb.arrived ? '<b class="quiet-choice' + (answered ? ' quiet-say' : '') + '">' + rb.choice + '</b>' : '') + '<span class="quiet-agent">' + r.agent + (r.agent === "Droid" ? '<span class="quiet-say"> · second session</span>' : '') + '</span></span><small>' + quietTally(r, rb) + '</small></div>' + quietScores(r) + '</div>';
  }).join("") + quietHolderHtml(true);
  setWaitingCount(quietWaitingBase + (quietState.arrived.length === quietData.READBACKS.length && !quietState.question.answered ? 1 : 0) + (quietUnmatched() ? 1 : 0));
}function quietUpdateReadbacks() {
  if (quietState.open === "order" && quietState.order.state !== "draft") {
    $("#quiet-readbacks").innerHTML = quietReadbacksHtml();
    $("#quiet-checks").innerHTML = quietCheckRows();
    $("#quiet-passive").innerHTML = quietPassiveHtml();
  }
  quietKvRender();
}
function quietClose() {
  quietState.open = null; quietCardTop = null;
  $("#quiet-card").hidden = true;
  $("#quiet-link").innerHTML = "";
  quietRenderDock();
  if (quietRestoreFocus && quietRestoreFocus.isConnected && quietRestoreFocus.getClientRects().length) quietRestoreFocus.focus({ preventScroll: true });
}
function quietOpen(open, machine, focus = true) {
  quietRestoreFocus = document.activeElement;
  quietState.open = open; quietCardTop = null;
  quietRenderCard(); quietRenderDock();
  quietFocusMachine(machine); quietPosition();
  if (focus) {
    const input = open.startsWith("w") ? $("#quiet-reason") : open === "order" && quietState.order.state === "draft" ? $("#quiet-sentence") : $("#quiet-card .quiet-close");
    if (input) { input.focus({ preventScroll: true }); if (input.id === "quiet-sentence") quietGrowSentence(input); }
  }
}
function quietAddPin(w) {
  const c = document.createElement("button");
  c.className = "chip quiet-witness rp loud show";
  c.dataset.id = w.entity; c.dataset.state = "ok"; c.dataset.polarity = w.polarity; c.dataset.quietWitness = w.id; c.dataset.quietBinding = w.id;
  c.innerHTML = '<span class="mono">' + w.id + '</span><i class="quiet-pol" title="' + w.polarity.replace("-", " ") + '">' + quietPolGlyph(w.polarity) + '</i>' + w.clock + '<small class="quiet-say">' + (w.id === "w3" ? 'ANSWER · sqlite' : w.polarity.replace("-", " ").toUpperCase()) + '</small><span class="rd"></span>';
  $("#chips").appendChild(c);
  renderPlant(); apply(); layoutChips();
}
function quietBotChip(r) {
  const bot = ENT[r.bot], l = quietState.kvarm.lease, live = l && l.session === r.session && l.state === "live";
  let c = document.querySelector('.chip[data-id="' + r.bot + '"]');
  if (!c) { c = document.createElement("div"); c.className = "chip quiet-bot"; c.dataset.id = r.bot; $("#chips").appendChild(c); }
  const ev = live ? l.evidence : bot.state === "dark" ? "stopped" : "declared";
  c.dataset.state = bot.state; c.dataset.quietBinding = "bot:" + r.bot;
  c.innerHTML = '<span class="mono">' + bot.mono + '</span>' + r.agent + '<i class="quiet-ev" title="' + ev + '">' + quietEvGlyph(ev) + '</i><small class="quiet-say">' + r.session + ' · ' + ev + '</small><span class="rd">' + bot.reading + '</span>';
}
function quietPin(keep) {
  if (quietState.order.state !== "draft") return quietAnnounce("Posted witnesses are locked.");
  const w = quietData.WITNESSES.find((x) => x.polarity === (keep ? "must-keep" : "must-not"));
  if (!rpOn() || selected !== w.entity || rpClock(RP.t).slice(0, 5) !== w.clock) return quietAnnounce("Replay " + w.clock + " at " + w.entity + ", then Pin.");
  if (!quietState.witnesses.some((x) => x.id === w.id)) {
    const copy = JSON.parse(JSON.stringify(w));
    quietState.witnesses.push(copy); quietAddPin(copy);
  }
  quietOpen(w.id, w.entity);
}
function quietKeep() {
  const w = quietState.witnesses.find((x) => x.id === quietState.open);
  if (w && !w.reason.trim()) w.reason = quietData.WITNESSES.find((x) => x.id === w.id).reason;
  quietClose();
}
function quietDispatched() {
  const kv = quietState.kvarm, U = quietData.KV.unmatched;
  kv.dispatched = true;
  if (kv.script !== "unmatched") return;
  quietAt(U.pollMs, () => quietSend("get_task", null, () => {
    kv.lease = { gen: quietData.KV.generation, holder: U.holder, session: U.session, evidence: U.evidence, readback: false, state: "live", renewedAt: quietKvMs };
    const chip = document.querySelector('.chip[data-id="bot-cx"]');
    if (chip) { chip.dataset.quietBinding = "holder"; chip.innerHTML = '<span class="mono">CX</span>' + U.holder + '<i class="quiet-ev" title="' + U.evidence + ' · no readback">' + quietEvGlyph(U.evidence) + '</i>' + kv.lease.gen + '<small class="quiet-say">' + U.session + ' · ' + U.evidence + ' · holds ' + kv.lease.gen + '</small><span class="rd">no readback</span>'; }
    $(".bl .wait").insertAdjacentHTML("afterend", '<button class="agent quiet-holder" id="quiet-holder-row" data-quiet-binding="holder">' + DRONE_ICON_JS + '<strong>' + U.holder + '</strong> unmatched<span class="quiet-say">holds ' + kv.lease.gen + ' · Kvarm</span></button>');
    $("#quiet-holder-row").onclick = () => { if (quietActive()) quietOpen("order", quietData.ORDER.on); };
    layoutChips(); quietUpdateReadbacks();
    quietAnnounce("Kvarm reports " + U.holder + " holds " + kv.lease.gen + " with no readback. The order stays unclaimed.", U.holder + " holds " + kv.lease.gen + " with no readback.");
  }));
}
function quietPost() {
  if (!quietActive() || !quietCanPost()) return;
  quietState.sentence = quietState.sentence.trim();
  document.activeElement.blur();
  selected = null;
  setMode("live");
  quietState.order.state = "open"; quietState.rev = "r1";
  quietLiveMs = 0;
  quietCardTop = null;
  quietRenderCard(); quietRenderDock(); quietFocusMachine(quietData.ORDER.on);
  const kv = quietState.kvarm, K = quietData.KV;
  quietSend("create_issue", null, () => { kv.issue = K.issue; });
  quietSend("dispatch_issue", K.ops.dispatch_issue.reason + " on #" + K.issue, quietDispatched);
  const plate = $("#quiet-card-status");
  plate.classList.add("quiet-ink");
  plate.onanimationend = () => plate.classList.remove("quiet-ink");
  $("#quiet-card .quiet-close").focus({ preventScroll: true });
  quietAnnounce(kv.connection === "ok" ? "Order r1 posted locally. Witnesses are locked. Agents read back before code." : "Order r1 posted locally, not in Kvarm. Writes are held until you reconnect.", kv.connection === "ok" ? "Posted r1. Witnesses are locked." : "Posted locally. Writes wait for Kvarm.");
}
function quietFloor(r, status) {
  const row = [...document.querySelectorAll(".bl .agent")].find((el) => el.querySelector("strong")?.textContent === r.agent);
  if (!row) return;
  row.classList.remove("off");
  row.dataset.quietSession = r.session;
  row.innerHTML = DRONE_ICON_JS + "<strong>" + r.agent + "</strong> " + (r.agent === "Droid" ? '<i class="quiet-say">second session · </i>' : "") + status + '<span class="quiet-say">MCP</span>';
  row.onclick = () => { if (quietActive()) quietOpen("order", quietData.ORDER.on); };
}
function quietArrive(r) {
  if (!quietActive() || document.body.dataset.mode !== "live" || quietState.order.state !== "open" || quietState.arrived.includes(r.session)) return;
  quietState.arrived.push(r.session);
  const dock = ENT[quietData.ORDER.on];
  if (!ENT[r.bot]) {
    const bot = { id: r.bot, kind: "bot", mono: r.agent === "Droid" ? "D2" : "CC", name: quietAgent(r), x: dock.x - 45, y: dock.y + (r.agent === "Droid" ? 55 : 20), state: "wait", reading: "readback", sub: r.session + " · connected over MCP", params: { target: [dock.x, dock.y, 10] } };
    SCENE.entities.push(bot); ENT[bot.id] = bot;
  }
  quietBotChip(r);
  quietFloor(r, "reading");
  renderPlant(); apply(); layoutChips(); quietUpdateReadbacks();
  if (quietState.arrived.length === quietData.READBACKS.length) {
    layoutChips();
    quietAnnounce("Two readings differ on p-2307. Answer at sqlite.");
  } else quietAnnounce(quietAgent(r) + " sent a readback.", r.agent + " sent a readback.");
}
function quietAnswer(key) {
  const option = quietData.QUESTION.options.find((x) => x.key === key);
  if (!quietActive() || document.body.dataset.mode !== "live" || quietState.open !== "question" || !option || quietState.question.answered || quietState.arrived.length !== quietData.READBACKS.length) return;
  quietState.question.answered = key; quietState.rev = "r2";
  quietState.scoring = "rescoring"; quietScoreMs = quietVisibleMs = 0;
  const w = JSON.parse(JSON.stringify(option.witness));
  quietState.witnesses.push(w); quietAddPin(w);
  quietSend("comment_issue");
  quietClose(); quietRenderDock(); quietPosition();
  quietAnnounce("Answer pinned as w3 at sqlite. Order is now r2. Re-scoring the readbacks.", "Pinned w3. Order is now r2.");
}
function quietClaim() {
  const winner = quietWinner(), kv = quietState.kvarm;
  if (!quietActive() || document.body.dataset.mode !== "live" || !winner || quietState.order.state !== "open" || quietState.scoring !== "settled" || quietVisibleMs < 1400 || !ENT[winner.bot] || kv.claim) return;
  kv.claim = { session: winner.session, state: "sent" };
  quietSend("claim_task", winner.agent + " · " + quietData.KV.ops.claim_task.reason, (row) => {
    if (kv.lease && kv.lease.holder !== winner.agent) {
      kv.claim.state = "rejected"; row.state = "rejected"; row.result = "rejected";
      quietUpdateReadbacks();
      quietAnnounce("Kvarm rejected the claim from " + winner.agent + ". " + kv.lease.holder + " holds " + kv.lease.gen + " with no readback.", "Kvarm rejected the claim from " + winner.agent + ".");
    } else quietGrant(winner);
  });
  if (kv.connection !== "ok") {
    quietState.order.state = "matched";
    quietUpdateReadbacks();
    quietAnnounce(quietAgent(winner) + " matches every check. The claim waits for the Kvarm connection.", winner.agent + " matches. The claim waits for Kvarm.");
  }
}
function quietGrant(winner) {
  const K = quietData.KV, kv = quietState.kvarm;
  quietState.order.state = "claimed"; quietState.order.claimedBy = winner.agent;
  kv.claim.state = "granted";
  kv.lease = { gen: K.generation, holder: winner.agent, session: winner.session, evidence: "authenticated", readback: true, state: "live", renewedAt: quietKvMs, beatAt: quietKvMs + (kv.script === "stale" ? K.stale.heartbeatMs : K.heartbeatS * 1000) };
  const territory = SCENE.territories.find((t) => t.id === quietData.ORDER.territory);
  quietTerritory = quietTerritory || { label: territory.label, fog: TERR[quietData.ORDER.territory].fog };
  territory.state = "claimed"; territory.fog = false; territory.label = quietData.ORDER.territory.toUpperCase() + ' · ' + winner.agent;
  Object.assign(TERR[quietData.ORDER.territory], { state: "claimed", fog: null, owner: winner.agent });
  const bot = ENT[winner.bot], dock = ENT[quietData.ORDER.on];
  bot.x = dock.x - 10; bot.y = dock.y + 15; bot.reading = "claimed · lease " + K.generation; bot.state = "ok";
  const loser = quietData.READBACKS.find((r) => r !== winner);
  ENT[loser.bot].reading = "yields"; ENT[loser.bot].state = "fail";
  $("#quiet-fog").hidden = true;
  for (const r of quietData.READBACKS) { quietFloor(r, r === winner ? "claimed" : "yields"); quietBotChip(r); }
  renderPlant(); apply(); layoutChips(); quietUpdateReadbacks();
  if (!quietState.open || quietState.open === "order") quietFocusMachine(quietData.ORDER.on);
  quietPosition();
  const plate = quietState.open === "order" ? $("#quiet-card-status") : $("#quiet-plate b");
  plate.classList.remove("quiet-ink"); void plate.offsetWidth; plate.classList.add("quiet-ink");
  plate.onanimationend = () => plate.classList.remove("quiet-ink");
  quietAnnounce(quietAgent(winner) + " · " + winner.session + " claimed the order. Readback matched " + quietState.witnesses.length + " of " + quietState.witnesses.length + ".", winner.agent + " claimed. Readback matched " + quietState.witnesses.length + " of " + quietState.witnesses.length + ".");
}
function quietExpire() {
  const kv = quietState.kvarm, l = kv.lease, winner = quietData.READBACKS.find((r) => r.session === l.session);
  l.state = "expired";
  quietState.order.state = "stale"; quietState.order.claimedBy = null;
  const territory = SCENE.territories.find((t) => t.id === quietData.ORDER.territory);
  territory.state = "unclaimed"; territory.fog = true; territory.label = quietData.ORDER.territory.toUpperCase() + " · LEASE STALE";
  Object.assign(TERR[quietData.ORDER.territory], { state: "unclaimed", fog: quietTerritory.fog, owner: null });
  const bot = ENT[winner.bot];
  bot.reading = "stopped · " + quietData.KV.stale.code; bot.state = "dark";
  $("#quiet-fog").hidden = false;
  quietFloor(winner, 'stopped<i class="quiet-say"> · ' + quietData.KV.stale.code + '</i>');
  quietBotChip(winner);
  renderPlant(); apply(); layoutChips(); quietUpdateReadbacks();
  quietAnnounce("Lease " + l.gen + " expired. Kvarm rejected the heartbeat with " + quietData.KV.stale.code + ". " + winner.agent + " stopped.", "Lease " + l.gen + " expired. " + winner.agent + " stopped.");
}
function quietQuestion() {
  if (document.body.dataset.mode !== "live" || quietState.arrived.length !== quietData.READBACKS.length || quietState.question.answered) return quietAnnounce("No open question in this view.");
  quietOpen("question", quietData.QUESTION.entity);
}
function quietOrder() {
  if (!TERR[quietData.ORDER.territory].entities.includes(selected)) return quietAnnounce("Select a machine in paypal-payouts, then Post.");
  quietOpen("order", quietData.ORDER.on);
}
function quietGuide() {
  if (quietState.order.state !== "draft") return;
  const reasons = quietData.WITNESSES.map((w, i) => {
    const ready = rpOn() && selected === w.entity && rpClock(RP.t).slice(0, 5) === w.clock;
    $(i ? "#quiet-pin-keep" : "#quiet-pin-not").disabled = !ready;
    return ready ? "" : "Select " + w.entity + " at " + w.clock;
  }).filter(Boolean);
  $("#quiet-pin-gate").textContent = reasons.slice(0, 1).join("");
  const selectedTerritory = TERR[quietData.ORDER.territory].entities.includes(selected);
  $("#quiet-post").disabled = !selectedTerritory;
  $("#quiet-post-gate").textContent = selectedTerritory ? "" : "Select a paypal-payouts machine.";
}
function quietPosition() {
  const pp = quietPoint(quietData.ORDER.on), qp = quietPoint(quietData.QUESTION.entity, true);
  const card = $("#quiet-card"), dock = $("#quiet-dock"), pin = $("#quiet-question-pin");
  const posted = quietState.order.state !== "draft";
  const inView = (p) => p[0] >= 40 && p[0] <= innerWidth - 70 && p[1] >= 160 && p[1] <= innerHeight - 130;
  const question = quietState.arrived.length === quietData.READBACKS.length && !quietState.question.answered && document.body.dataset.mode === "live";
  const rects = [...document.querySelectorAll(".chip")].filter(chipShown).map((c) => c.getBoundingClientRect());
  const overlaps = (a, b, gap = 10) => a.left < b.right + gap && a.right > b.left - gap && a.top < b.bottom + gap && a.bottom > b.top - gap;
  const nav = $("#nav").getBoundingClientRect(), cbar = $("#cbar").getBoundingClientRect();
  const rightEdge = (top, height) => (top < nav.bottom + 16 && top + height > nav.top - 16 ? nav.left - 16 : innerWidth - 90);
  let lines = "";
  if (quietState.open) {
    const open = quietState.open, id = open === "question" ? quietData.QUESTION.entity : open === "order" ? quietData.ORDER.on : quietState.witnesses.find((x) => x.id === open).entity;
    const p = quietPoint(id, open === "question");
    const bottom = rpOn() ? $("#rpdeck").getBoundingClientRect().top - 16 : innerHeight - 60;
    if (quietCardTop === null) quietCardTop = open === "order" ? 100 : Math.max(140, Math.min(bottom - card.offsetHeight, p[1] - 140));
    if (p[0] + 170 > rightEdge(quietCardTop, card.offsetHeight) - card.offsetWidth && quietCardTop < nav.bottom + 16 && nav.bottom + 16 + card.offsetHeight <= bottom) quietCardTop = nav.bottom + 16;
    const maxLeft = rightEdge(quietCardTop, card.offsetHeight) - card.offsetWidth;
    let left = Math.max(20, Math.min(maxLeft, p[0] + 170));
    if (quietCardTop < cbar.bottom + 10 && left < cbar.right + 12 && left + card.offsetWidth > cbar.left - 12) {
      if (cbar.right + 12 <= maxLeft) left = Math.max(left, cbar.right + 12);
      else quietCardTop = cbar.bottom + 10;
    }
    card.style.left = left + "px"; card.style.top = quietCardTop + "px";
    card.style.maxHeight = Math.max(160, bottom - quietCardTop) + "px";
    rects.push(card.getBoundingClientRect());
    if (inView(p)) lines += '<path data-quiet-card-leader d="M' + p[0] + ' ' + p[1] + 'H' + (left - 14) + 'V' + (quietCardTop + 40) + 'H' + left + '"/><circle cx="' + p[0] + '" cy="' + p[1] + '" r="3"/>';
  }
  const edge = (id, p, visible, text) => {
    const el = $(id);
    el.hidden = !visible;
    if (!visible) return;
    const arrow = p[0] < 40 ? "← " : p[0] > innerWidth - 70 ? "→ " : p[1] < 160 ? "↑ " : "↓ ";
    el.textContent = arrow + text;
    const top = id === "#quiet-order-edge" ? 200 : 160;
    el.style.left = Math.max(20, Math.min(rightEdge(top, el.offsetHeight) - el.offsetWidth, p[0])) + "px";
    el.style.top = top + "px";
  };
  edge("#quiet-question-edge", qp, question && !inView(qp) && quietState.open !== "question", "sqlite · question · I");
  edge("#quiet-order-edge", pp, posted && !inView(pp) && quietState.open !== "order", "PayPal · order · P");
  dock.hidden = !posted || quietState.open === "order" || !inView(pp);
  if (!dock.hidden) {
    const width = dock.offsetWidth, height = dock.offsetHeight;
    const candidates = [[pp[0] + 40, pp[1] + 54], [pp[0] - width - 40, pp[1] + 54], [pp[0] + 40, pp[1] - height - 70], [pp[0] - width - 40, pp[1] - height - 70]];
    if (!card.hidden) {
      const r = card.getBoundingClientRect();
      candidates.push([r.right + 16, pp[1] + 54], [pp[0] + 40, r.bottom + 16], [r.left - width - 16, pp[1] + 54]);
    }
    const boxes = candidates.map(([x, y]) => ({ top: Math.max(155, Math.min(innerHeight - height - 60, y)), x })).map(({ top, x }) => ({ top, left: Math.max(20, Math.min(rightEdge(top, height) - width, x)) })).map((r) => ({ ...r, right: r.left + width, bottom: r.top + height }));
    const r = boxes.find((b) => rects.every((r) => !overlaps(b, r))) || boxes.find((b) => card.hidden || !overlaps(b, card.getBoundingClientRect())) || boxes[0];
    dock.style.left = r.left + "px"; dock.style.top = r.top + "px"; rects.push(r);
    lines += '<path d="M' + pp[0] + ' ' + pp[1] + 'V' + (r.top + 20) + 'H' + (r.left > pp[0] ? r.left : r.right) + '"/><circle cx="' + pp[0] + '" cy="' + pp[1] + '" r="3"/>';
  }
  pin.hidden = !question || !inView(qp) || quietState.open === "question";
  if (!pin.hidden) {
    const width = pin.offsetWidth, height = pin.offsetHeight;
    const candidates = [[qp[0] + 26, qp[1] + 26], [qp[0] + 26, qp[1] - height - 60], [qp[0] - width - 26, qp[1] + 26], [qp[0] - width - 26, qp[1] - height - 60]];
    const r = candidates.map(([x, y]) => ({ left: x, top: y, right: x + width, bottom: y + height })).find((b) => b.left >= 20 && b.right <= innerWidth - 70 && b.top >= 150 && b.bottom <= innerHeight - 80 && rects.every((r) => !overlaps(b, r, 6)));
    pin.hidden = !r;
    if (r) {
      pin.style.left = r.left + "px"; pin.style.top = r.top + "px";
      lines += '<path data-quiet-question-leader d="M' + qp[0] + ' ' + qp[1] + 'V' + (r.top + height / 2) + 'H' + (r.left > qp[0] ? r.left : r.right) + '"/><circle cx="' + qp[0] + '" cy="' + qp[1] + '" r="3"/>';
    } else edge("#quiet-question-edge", qp, true, "sqlite · question · I");
  }
  $("#quiet-fog").style.left = pp[0] - 110 + "px";
  $("#quiet-fog").style.top = pp[1] - 20 + "px";
  $("#quiet-link").innerHTML = lines;
  for (const chip of document.querySelectorAll(".chip")) {
    const r = chip.getBoundingClientRect(), occluded = !card.hidden && overlaps(r, card.getBoundingClientRect(), 0);
    if (!chip.matches(".quiet-witness")) { chip.style.visibility = occluded ? "hidden" : ""; continue; }
    const w = quietState.witnesses.find((w) => w.id === chip.dataset.quietWitness);
    const clipped = r.left < 12 || r.right > innerWidth - 12 || r.top < 120 || r.bottom > innerHeight - 60;
    chip.style.visibility = clipped || occluded ? "hidden" : "";
    let edge = document.querySelector('[data-quiet-witness-edge="' + w.id + '"]');
    if (clipped && !edge) {
      edge = document.createElement("button");
      edge.className = "quiet-mark quiet-edge";
      edge.dataset.quietWitnessEdge = w.id;
      edge.onclick = () => { if (quietActive()) quietOpen(w.id, w.entity); };
      $("#quiet-witness-edges").appendChild(edge);
    }
    if (edge) {
      edge.hidden = !clipped;
      edge.textContent = (r.left < 12 ? "← " : r.right > innerWidth - 12 ? "→ " : r.top < 120 ? "↑ " : "↓ ") + w.entity + " · " + w.id;
      edge.style.left = (r.left < 12 ? 20 : Math.min(innerWidth - 250, r.left)) + "px";
      edge.style.top = (240 + quietState.witnesses.indexOf(w) * 36) + "px";
    }
  }
}
function quietEvidenceVisible() {
  const root = quietState.open === "order" ? $("#quiet-readbacks") : $("#quiet-dock");
  if (!root || root.hidden || !root.getClientRects().length) return false;
  const r = root.getBoundingClientRect();
  return r.left >= 0 && r.right <= innerWidth && r.top >= 100 && r.bottom <= innerHeight - 40 &&
    [[r.left + 8, r.top + 8], [r.right - 8, r.bottom - 8]].every(([x, y]) => root.contains(document.elementFromPoint(x, y)));
}
function quietInit(params) {
  if (!quietActive() || quietInitialized) return;
  quietInitialized = true;
  window.quietState = quietState;
  quietWaitingBase = +$(".bl .wait b").textContent;
  setMode("replay");
  rpSeek(params.has("t") ? +params.get("t") : toSec(quietData.WITNESSES[0].t));
  const kvFlag = params.get("kv");
  if (["unmatched", "stale", "offline"].includes(kvFlag)) quietState.kvarm.script = kvFlag;
  if (kvFlag === "offline") quietState.kvarm.connection = "offline";
  quietPill();
  quietFocusMachine(params.get("sel") || "retry"); quietRenderDock();
  $("#quiet-pin-not").onclick = () => { if (quietActive()) quietPin(false); };
  $("#quiet-pin-keep").onclick = () => { if (quietActive()) quietPin(true); };
  $("#quiet-post").onclick = () => { if (quietActive()) quietOrder(); };
  $("#quiet-plate").onclick = () => { if (quietActive()) quietOpen("order", quietData.ORDER.on); };
  $("#quiet-order-edge").onclick = $("#quiet-plate").onclick;
  $("#quiet-question-pin").onclick = $("#quiet-question-edge").onclick = () => { if (quietActive()) quietQuestion(); };
  for (const b of document.querySelectorAll("[data-quiet-go]")) b.onclick = () => {
    if (!quietActive()) return;
    const w = quietData.WITNESSES.find((x) => x.id === b.dataset.quietGo);
    if (quietState.open) quietClose();
    selected = null;
    setMode("replay"); rpSeek(toSec(w.t)); quietFocusMachine(w.entity);
  };
  $("#quiet-card").onclick = (ev) => {
    if (!quietActive()) return;
    if (ev.target.closest("[data-quiet-close]")) quietClose();
    if (ev.target.closest("#quiet-keep")) quietKeep();
    if (ev.target.closest("#quiet-submit")) quietPost();
    const answer = ev.target.closest("[data-quiet-answer]");
    if (answer) quietAnswer(answer.dataset.quietAnswer);
    const provBtn = ev.target.closest("[data-quiet-prov]");
    if (provBtn) quietProvOpen(provBtn.dataset.quietProv, provBtn);
  };
  $("#quiet-kv-panel").onclick = (ev) => {
    if (!quietActive()) return;
    if (ev.target.closest("[data-quiet-panel-close]")) quietPanelToggle(false);
    const act = ev.target.closest("[data-quiet-kv]");
    if (act) act.dataset.quietKv === "reconnect" ? quietReconnect() : quietSendHeld();
  };
  $("#quiet-prov").onclick = (ev) => { if (ev.target.closest("[data-quiet-prov-close]")) quietProvClose(); };
  const quietEgress = $(".egress");
  if (quietEgress) {
    quietEgress.addEventListener("click", () => { if (quietActive()) { $("#net").classList.remove("open"); quietPanelToggle(); } });
    quietEgress.addEventListener("keydown", (ev) => { if (quietActive() && (ev.key === "Enter" || ev.key === " ")) { ev.preventDefault(); quietPanelToggle(); } });
  }
  document.addEventListener("pointerover", (ev) => {
    if (!quietActive() || !document.body.classList.contains("alt")) return;
    const t = ev.target.closest("[data-quiet-binding]");
    if (t && !t.closest("#quiet-prov")) quietProvOpen(t.dataset.quietBinding, t);
  });
  addEventListener("keyup", (ev) => { if (ev.key === "Alt") quietProvClose(); });
  addEventListener("blur", () => quietProvClose());
  $("#chips").addEventListener("pointerdown", (ev) => { if (quietActive() && ev.target.closest("[data-quiet-witness]")) ev.stopPropagation(); });
  $("#chips").addEventListener("pointerup", (ev) => { if (quietActive() && ev.target.closest("[data-quiet-witness]")) ev.stopPropagation(); });
  $("#chips").addEventListener("click", (ev) => {
    if (!quietActive()) return;
    const pin = ev.target.closest("[data-quiet-witness]");
    if (!pin) return;
    const w = quietState.witnesses.find((x) => x.id === pin.dataset.quietWitness);
    quietOpen(w.id, w.entity);
  });
  addEventListener("keydown", (ev) => {
    if (!quietActive()) return;
    const open = quietState.open;
    const handle = () => { ev.preventDefault(); ev.stopImmediatePropagation(); };
    if (typing(ev)) {
      if (!ev.target.closest("#quiet-card")) return;
      if (ev.key === "Escape") { handle(); ev.target.blur(); return; }
      if (ev.key === "Enter" && !ev.shiftKey) { handle(); if (open === "order") quietPost(); else if (open && open.startsWith("w")) quietKeep(); }
      return;
    }
    if (!bare(ev)) return;
    if (ev.key === "Escape") {
      if (quietProvKey) { handle(); quietProvClose(); return; }
      if (quietState.kvarm.panel) { handle(); quietPanelToggle(false); return; }
      if (open) { handle(); quietClose(); return; }
      return;
    }
    if (open === "question" && ev.shiftKey && ["Digit1", "Digit2"].includes(ev.code)) { handle(); quietAnswer(ev.code.slice(-1)); return; }
    if (ev.key === "Enter" && open) {
      if (ev.target.closest("#quiet-card button, #quiet-kv-panel button, #quiet-prov button")) return;
      handle(); if (open.startsWith("w")) quietKeep(); else if (open === "order") quietPost(); return;
    }
    const key = ev.key.toLowerCase();
    if (key === "u") { handle(); quietPanelToggle(); return; }
    if (key === "m") { handle(); quietPin(ev.shiftKey); }
    if (key === "p") { handle(); quietOrder(); }
    if (key === "i") { handle(); quietQuestion(); }
  }, { capture: true });
  const frame = (now) => {
    const delta = quietLastFrame === null ? 0 : Math.min(100, now - quietLastFrame);
    quietLastFrame = now;
    if (quietActive()) {
      quietKvTick(delta);
      quietGuide(); quietPosition();
      if (document.body.dataset.mode === "live" && quietState.order.state === "open" && !document.hidden) {
        quietLiveMs += delta;
        for (const r of quietData.READBACKS) if (quietLiveMs >= r.arriveMs) quietArrive(r);
        if (quietState.scoring === "rescoring") {
          if (quietEvidenceVisible()) quietScoreMs += delta;
          if (quietScoreMs >= 600) {
            quietState.scoring = "settled"; quietVisibleMs = 0; quietUpdateReadbacks();
            const match = quietData.READBACKS.find((r) => r.answers.q1 === quietState.question.answered);
            quietAnnounce("Readbacks re-scored. " + quietAgent(match) + " matches every locked check.", match.agent + " matches every check.");
          }
        } else if (quietState.scoring === "settled") {
          if (quietEvidenceVisible()) quietVisibleMs += delta;
          else quietVisibleMs = 0;
          quietClaim();
        }
      } else quietVisibleMs = 0;
    } else quietVisibleMs = 0;
    requestAnimationFrame(frame);
  };
  requestAnimationFrame(frame);
}
`,
};
