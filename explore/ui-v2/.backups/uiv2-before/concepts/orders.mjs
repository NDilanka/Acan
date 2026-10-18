import { ORDER, WITNESSES, QUESTION, READBACKS, SHEET, KVARM } from "./core-data.mjs";

export default {
  id: "orders",
  label: "Work orders",
  hint: "M pin must not · Shift M pin must keep · P post / inspect · I question · Shift 1 / 2 answer · U Kvarm · Alt hover provenance",
  css: `
body[data-concept=orders] *{transition:none!important}
body[data-concept=orders] .stage *,body[data-concept=orders] .bl *{animation:none!important}
body[data-concept=orders] .hint{display:none}
body[data-concept=orders] .toast{left:20px;bottom:200px;max-width:288px;transform:none;z-index:13}
body[data-concept=orders] .cbar{top:83px}
body[data-concept=orders] .orders-ui{display:block}
body[data-concept=orders] .orders-ui [hidden]{display:none!important}
.orders-ui{display:none}
body[data-concept=orders] .orders-hint{position:fixed;left:20px;top:130px;font:11px var(--mono);color:#aaa;z-index:8}
body[data-concept=orders] .orders-gate{margin:4px 0 8px;color:var(--wait);font:10px/1.5 var(--mono)}
body[data-concept=orders] .orders-guide{position:fixed;left:20px;top:130px;z-index:8;width:288px;padding:14px;border:1px solid var(--line);border-radius:12px;background:var(--glass);box-shadow:var(--shadow)}
body[data-concept=orders] .orders-eyebrow{font:10px var(--mono);letter-spacing:.12em;text-transform:uppercase;color:#c9a45c}
body[data-concept=orders] .orders-guide h2{margin:6px 0;font-size:17px;font-weight:500;color:#eee}
body[data-concept=orders] .orders-guide p{margin:0 0 12px;color:#999;font-size:12px}
body[data-concept=orders] .orders-guide button{display:flex;align-items:center;gap:8px;width:100%;padding:8px 0;text-align:left;border-top:1px solid var(--line);font-size:12px}
body[data-concept=orders] .orders-guide button small{margin-left:auto;color:#888;font:10px var(--mono)}
body[data-concept=orders] .orders-guide .orders-dot{width:6px;height:6px;border-radius:50%;background:var(--fail)}
body[data-concept=orders] .orders-guide [data-orders-go=w2] .orders-dot{background:var(--ok)}
body[data-concept=orders] .orders-guide footer{margin-top:8px;font:10px/1.8 var(--mono);color:#888}
body[data-concept=orders] .orders-mark{position:fixed;z-index:9;border:1px solid #c9a45c;border-radius:7px;background:#19160f;color:#e6cb96;padding:5px 8px;font:10px var(--mono);white-space:nowrap}
body[data-concept=orders] .orders-card{position:fixed;z-index:12;width:470px;max-width:calc(100vw - 40px);border:1px solid #665333;border-radius:14px;background:rgba(22,22,21,.97);box-shadow:var(--shadow);overflow:auto;scrollbar-width:thin;scrollbar-color:#55462f transparent}
body[data-concept=orders] .orders-card header{display:flex;gap:10px;align-items:center;padding:12px 16px;border-bottom:1px solid var(--line)}
body[data-concept=orders] .orders-card header[data-orders-plate]{background:#c9a45c13}
body[data-concept=orders] .orders-card h3{font-size:15px;font-weight:500;color:#eee;margin:3px 0 0}
body[data-concept=orders] .orders-close{margin-left:auto;align-self:flex-start;color:#999;width:26px;height:26px;border-radius:5px}
body[data-concept=orders] .orders-close:hover{background:#333;color:#fff}
body[data-concept=orders] .orders-body{padding:12px 16px}
body[data-concept=orders] .orders-body label,body[data-concept=orders] .orders-section{display:block;font:10px var(--mono);letter-spacing:.07em;text-transform:uppercase;color:#999;margin-bottom:8px}
body[data-concept=orders] .orders-body input,body[data-concept=orders] .orders-body textarea{width:100%;border-radius:7px;background:#111;border:1px solid #444;color:#eee;padding:10px;font:12px/1.5 var(--sans);outline:none}
body[data-concept=orders] .orders-body input:focus,body[data-concept=orders] .orders-body textarea:focus{border-color:#c9a45c;box-shadow:0 0 0 2px #c9a45c22}
body[data-concept=orders] .orders-body textarea{resize:vertical;min-height:70px}
body[data-concept=orders] #orders-sentence{resize:none;min-height:58px;overflow:hidden;white-space:pre-wrap;overflow-wrap:anywhere;display:block;margin-bottom:12px}
body[data-concept=orders] .orders-sentence{margin:0 0 14px;font-size:13px;line-height:1.5;color:#eee}
body[data-concept=orders] .orders-check{display:grid;grid-template-columns:22px 1fr;gap:3px 8px;border-top:1px solid var(--line);padding:7px 0}
body[data-concept=orders] .orders-check b{font:11px var(--mono);color:#c9a45c}
body[data-concept=orders] .orders-check strong{font:10px var(--mono);letter-spacing:.06em;color:var(--ok);text-transform:uppercase}
body[data-concept=orders] .orders-check[data-polarity=must-not] strong{color:var(--fail)}
body[data-concept=orders] .orders-check p{grid-column:2;margin:0;font-size:12px;color:#ccc}
body[data-concept=orders] .orders-check small{grid-column:2;font:10px/1.5 var(--mono);color:#aaa}
body[data-concept=orders] .orders-locks{padding:9px 0;border-top:1px solid var(--line);display:grid;gap:5px;font:10px var(--mono);color:#c9a45c}
body[data-concept=orders] .orders-passive{font-size:11px;color:#999;border-top:1px dashed #444;padding-top:9px;margin:0}
body[data-concept=orders] .orders-actions{display:flex;justify-content:space-between;align-items:center;gap:10px;margin-top:14px;color:#888;font:10px var(--mono)}
body[data-concept=orders] .orders-primary{padding:8px 13px;border-radius:7px;background:#c9a45c;color:#14110b;font:500 12px var(--sans)}
body[data-concept=orders] .orders-primary kbd{color:#524222;border-color:#8c723d;margin-left:5px}
body[data-concept=orders] button:disabled{opacity:.4;cursor:default}
body[data-concept=orders] .orders-status{display:flex;align-items:center;gap:6px;min-height:24px;font:10px var(--mono);color:#c9a45c}
body[data-concept=orders] .orders-readbacks{margin-top:12px}
body[data-concept=orders] .orders-reading{border-top:1px solid var(--line);padding:9px 0}
body[data-concept=orders] .orders-reading header{border:0;padding:0 0 5px;display:flex;justify-content:space-between;gap:8px;font-size:12px}
body[data-concept=orders] .orders-reading header span{font:10px var(--mono);color:#999}
body[data-concept=orders] .orders-reading[data-state=differs]{border-left:2px solid var(--wait);padding-left:10px}
body[data-concept=orders] .orders-choice{font:500 12px var(--mono);color:var(--strain)}
body[data-concept=orders] .orders-reading[data-state=matches] header span,body[data-concept=orders] .orders-mini[data-state=matches] small{color:var(--ok)}
body[data-concept=orders] .orders-reading[data-state=yields] header span,body[data-concept=orders] .orders-mini[data-state=yields] small{color:var(--fail)}
body[data-concept=orders] .orders-reading[data-state=differs] header span,body[data-concept=orders] .orders-reading[data-state=rescoring] header span,body[data-concept=orders] .orders-mini[data-state=differs] small,body[data-concept=orders] .orders-mini[data-state=rescoring] small{color:var(--wait)}
body[data-concept=orders] .orders-reading mark{background:#c9a45c18;color:var(--strain);border-bottom:1px solid var(--wait);padding:0 2px}
body[data-concept=orders] .orders-reading p{margin:0;color:#bbb;font-size:11px;line-height:1.5}
body[data-concept=orders] .orders-score{display:flex;gap:9px;margin-top:7px;font:10px var(--mono)}
body[data-concept=orders] .orders-score [data-result=pass]{color:var(--ok)}
body[data-concept=orders] .orders-score [data-result=fail]{color:var(--fail)}
body[data-concept=orders] .orders-score [data-result=pending]{color:var(--wait)}
body[data-concept=orders] .orders-answer{display:block;width:100%;padding:11px;margin-top:9px;border:1px solid #4b4335;border-radius:8px;text-align:left;font-size:12px;color:#ddd}
body[data-concept=orders] .orders-answer:hover{border-color:#c9a45c;background:#c9a45c0c}
body[data-concept=orders] .orders-answer kbd{color:#d9ba7e;margin-right:6px}
body[data-concept=orders] .orders-answer{display:grid;grid-template-columns:64px 1fr;gap:3px 9px}
body[data-concept=orders] .orders-answer kbd{white-space:nowrap;align-self:start;justify-self:start}
body[data-concept=orders] .orders-answer small{grid-column:2;color:var(--strain);font:10px var(--mono)}
body[data-concept=orders] .orders-question-note{color:#aaa;font:11px/1.6 var(--mono);margin:12px 0 0}
body[data-concept=orders] .orders-dock{position:fixed;z-index:10;width:340px;border:1px solid #665333;border-radius:9px;background:rgba(20,19,17,.96);box-shadow:var(--shadow)}
body[data-concept=orders] .orders-plate{padding:10px 12px;width:100%;text-align:left;background:#c9a45c13;border-radius:9px 9px 0 0}
body[data-concept=orders] .orders-plate b{display:block;font:10px var(--mono);color:#daba7b;letter-spacing:.05em}
body[data-concept=orders] .orders-plate span{display:block;font-size:12px;margin-top:5px;color:#eee}
body[data-concept=orders] .orders-mini{border-top:1px solid var(--line);padding:9px 12px;font-size:11px;color:#aaa}
body[data-concept=orders] .orders-mini>div{display:flex;justify-content:space-between;align-items:flex-start;gap:6px}
body[data-concept=orders] .orders-mini small{font:10px/1.5 var(--mono);color:#999;max-width:145px;text-align:right}
body[data-concept=orders] .orders-mini .orders-score{margin-top:5px}
body[data-concept=orders] .orders-mini .orders-agent{margin-left:7px}
body[data-concept=orders] .orders-edge{position:fixed;z-index:10;max-width:250px}
body[data-concept=orders] .orders-check[data-answer] strong{color:#c9a45c}
body[data-concept=orders] .chip.orders-witness{pointer-events:auto;cursor:pointer;border-color:var(--ok);color:var(--ok);font:10px var(--mono);letter-spacing:0;opacity:1!important;translate:0 0!important}
body[data-concept=orders] .orders-witness .mono{background:var(--ok)!important;color:#0b1815}
body[data-concept=orders] .orders-witness[data-polarity=must-not]{color:var(--fail);border-color:var(--fail)}
body[data-concept=orders] .orders-witness[data-polarity=must-not] .mono{background:var(--fail)!important}
body[data-concept=orders] .orders-witness .rd{display:none!important}
body[data-concept=orders] .orders-link{position:fixed;inset:0;pointer-events:none;z-index:11}
body[data-concept=orders] .orders-link path{stroke:#c9a45c88;fill:none;stroke-width:1}
body[data-concept=orders] .orders-link circle{fill:#c9a45c}
body[data-concept=orders] .orders-fog{position:fixed;pointer-events:none;width:220px;height:130px;border-radius:50%;background:radial-gradient(ellipse,#b3a5901e,#ad9c7d0c 55%,transparent 72%);z-index:2}
body[data-concept=orders] .terr[data-id=paypal-payouts][data-state=claimed] .iso-line{stroke:#c9a45c}
body[data-concept=orders] button:focus-visible{outline:1px solid #c9a45c;outline-offset:3px}
@keyframes orders-ink{from{clip-path:inset(100% 0 0)}to{clip-path:inset(0)}}
body[data-concept=orders] .orders-ink{animation:orders-ink .5s ease-out}
body[data-concept=orders] .orders-flagline{display:flex;align-items:baseline;gap:9px;flex-wrap:wrap}
body[data-concept=orders] [data-tone=wait]{color:var(--wait)!important}
body[data-concept=orders] [data-tone=local]{color:#b9a273!important}
body[data-concept=orders] [data-evidence-state=stale]{color:#9b917f!important;text-decoration:underline dashed #9b917f80;text-underline-offset:3px}
body[data-concept=orders] .orders-lease{font:10px var(--mono);color:#a99a7c;white-space:nowrap}
body[data-concept=orders] .orders-plate .orders-lease{display:inline;margin:0;font-size:10px}
body[data-concept=orders] .orders-kv{border-top:1px solid var(--line);padding:6px 12px 7px;font:10px/1.55 var(--mono);color:#8f897e}
body[data-concept=orders] .orders-card .orders-kv{padding:7px 16px 8px;background:#ffffff03}
body[data-concept=orders] .orders-kv header{display:flex;gap:6px;border:0;padding:0;color:#9f9583;font:10px var(--mono)}
body[data-concept=orders] .orders-kv header b{margin-left:auto;font-weight:400;color:#c9a45c}
body[data-concept=orders] .orders-kv header b[data-tone]{color:var(--wait)}
body[data-concept=orders] .orders-kv-row{display:grid;grid-template-columns:auto minmax(0,1fr) auto;gap:8px;white-space:nowrap}
body[data-concept=orders] .orders-kv-row b{font-weight:400;color:#c8bfae}
body[data-concept=orders] .orders-kv-row span{overflow:hidden;text-overflow:ellipsis;color:#7f796f}
body[data-concept=orders] .orders-kv-row i{font-style:normal;color:#9f9583}
body[data-concept=orders] .orders-kv-row[data-state=held] b,body[data-concept=orders] .orders-kv-row[data-state=held] i{color:var(--wait)}
body[data-concept=orders] .orders-kv-row[data-state=failed] i,body[data-concept=orders] .orders-kv-row[data-state=rejected] i{color:var(--fail)}
body[data-concept=orders] .orders-kv-row[data-state=sending] i{color:#777}
body[data-concept=orders] .orders-reading[data-state=held] header span,body[data-concept=orders] .orders-mini[data-state=held] small{color:var(--wait)}
body[data-concept=orders] .orders-reading[data-state=unknown] header span,body[data-concept=orders] .orders-mini[data-state=unknown] small{color:#9a9a9a}
body[data-concept=orders] .orders-bezel{display:inline-grid;place-items:center;width:18px;height:14px;margin-right:6px;border:1px dotted #8a8a8a;border-radius:50%;font:9px var(--mono);color:#bbb;background:repeating-linear-gradient(135deg,transparent 0 3px,#ffffff1a 3px 4px);vertical-align:-2px}
body[data-concept=orders] .orders-src{margin-left:auto;color:#8a8478}
body[data-concept=orders] .egress{cursor:pointer}
body[data-concept=orders] .egress:hover{color:var(--text)}
body[data-concept=orders] .egress[data-kv=offline] .dot,body[data-concept=orders] .egress[data-kv=checked] .dot{background:var(--wait)}
body[data-concept=orders] .orders-panel{position:fixed;z-index:30;top:62px;right:20px;width:380px;max-height:calc(100vh - 140px)}
body[data-concept=orders] .orders-panel .orders-kv-row{padding:3px 0;border-top:1px solid var(--line)}
body[data-concept=orders] .orders-panel .orders-kv-row span{white-space:normal}
body[data-concept=orders] .orders-note{margin:0 0 10px;font-size:11px;line-height:1.5;color:#999}
body[data-concept=orders] .orders-panel .orders-primary{margin:8px 0 12px}
body[data-concept=orders] .orders-panel .orders-section{margin:12px 0 4px}
body[data-concept=orders] .orders-prov{position:fixed;z-index:31;width:390px}
body[data-concept=orders] .orders-prov p{margin:0 0 10px;font:11px/1.7 var(--mono);color:#ddd}
body[data-concept=orders] .orders-prov dl{display:grid;grid-template-columns:86px 1fr;gap:4px 10px;margin:0;font:10px/1.5 var(--mono)}
body[data-concept=orders] .orders-prov dt{color:#888}
body[data-concept=orders] .orders-prov dd{margin:0;color:#ccc;overflow-wrap:anywhere}
body[data-concept=orders] .orders-bindings{border-top:1px solid var(--line);margin-top:10px;padding-top:8px;font:10px var(--mono);color:#999}
body[data-concept=orders] .orders-bindings summary{cursor:pointer;list-style:none}
body[data-concept=orders] .orders-bindings summary span{float:right;color:#777}
body[data-concept=orders] .orders-bindings button{display:grid;grid-template-columns:110px 1fr;gap:8px;width:100%;text-align:left;padding:4px 0;border-top:1px solid var(--line);font:10px var(--mono);color:#bbb}
body[data-concept=orders] .orders-bindings button span{color:#888;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
body[data-concept=orders] .chip.orders-bot{opacity:1!important;translate:0 0!important}
body[data-concept=orders] .chip small{font:9px var(--mono);color:#9a9a9a;letter-spacing:0;text-transform:none;font-weight:400}
body[data-concept=orders] .chip[data-state=dark]{border-color:#555;color:#999}
body[data-concept=orders] .chip[data-state=dark] .mono{background:#3a3a3a;color:#aaa}
body[data-concept=orders] .ent[data-state=dark]{opacity:.45}
body[data-concept=orders] .bl .orders-holder{color:var(--wait)}
`,
  html: `
<div class="orders-ui" id="orders-ui">
  <section class="orders-guide" id="orders-guide" aria-label="Work orders">
    <div class="orders-eyebrow">Work orders · from witnesses</div>
    <h2>Point to what matters.</h2>
    <p>Pin real moments. Agents replay their reading before they claim.</p>
    <button data-orders-go="w1"><i class="orders-dot"></i>Retry · 15:44 <small>Replay</small></button>
    <button data-orders-go="w2"><i class="orders-dot"></i>PayPal · 15:35 <small>Replay</small></button>
    <button id="orders-pin-not">Pin must not <small>M</small></button>
    <button id="orders-pin-keep">Pin must keep <small>Shift M</small></button>
    <div class="orders-gate" id="orders-pin-gate"></div>
    <button id="orders-post">Post <small>P</small></button>
    <div class="orders-gate" id="orders-post-gate"></div>
    <footer>M pin · Shift M keep · P post / inspect<br>I question · Shift 1 / 2 answer<br>[ ] events · , . minutes · Esc close</footer>
  </section>
  <div class="orders-hint" id="orders-hint" hidden>P inspect order · I answer at sqlite</div>
  <div class="orders-fog" id="orders-fog"></div>
  <button class="orders-mark" id="orders-question-pin" hidden>QUESTION · sqlite <kbd>I</kbd></button>
  <button class="orders-mark orders-edge" id="orders-question-edge" hidden></button>
  <button class="orders-mark orders-edge" id="orders-order-edge" hidden></button>
  <div id="orders-witness-edges"></div>
  <div class="orders-dock" id="orders-dock" hidden>
    <button class="orders-plate" id="orders-plate"></button>
    <div class="orders-kv" id="orders-kv-dock"></div>
    <div id="orders-mini-readbacks"></div>
  </div>
  <svg class="orders-link" id="orders-link"></svg>
  <section class="orders-card" id="orders-card" role="dialog" aria-label="Work order" hidden></section>
  <section class="orders-card orders-panel" id="orders-kv-panel" role="dialog" aria-label="Kvarm connection" hidden></section>
  <section class="orders-card orders-prov" id="orders-prov" role="dialog" aria-label="Provenance" hidden></section>
  <span id="orders-announcement" role="status" aria-live="polite" style="position:absolute;width:1px;height:1px;overflow:hidden;clip-path:inset(50%)"></span>
</div>`,
  script: `
const ordersData = ${JSON.stringify({ ORDER, WITNESSES, QUESTION, READBACKS, SHEET, KV: { origin: KVARM.origin, host: KVARM.host, repo: KVARM.repo, staleAfter: KVARM.staleAfter, ...KVARM.orders } })};
const ordersState = {
  witnesses: [], order: { state: "draft", claimedBy: null },
  question: { answered: null }, arrived: [], open: null, sentence: "", scoring: "waiting", rev: null,
  kvarm: { script: "ok", connection: "ok", issue: null, dispatched: false, lease: null, claim: null, log: [], held: [], bytes: 0, panel: false, lastReturn: null }
};
let ordersRestoreFocus = null, ordersCardTop = null;
let ordersLiveMs = 0, ordersScoreMs = 0, ordersVisibleMs = 0, ordersLastFrame = null;
let ordersWaitingBase = 0;
let ordersInitialized = false;
let ordersKvMs = 0, ordersInflight = [], ordersLeaseShown = null, ordersEgressHtml = null;
let ordersProvKey = null, ordersHover = null, ordersTerritory = null;
function ordersActive() {
  return conceptActive("orders") && document.body.dataset.concept === "orders" && new URLSearchParams(location.hash.slice(1)).get("c") === "orders";
}
function ordersEscape(value) {
  return String(value).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
}
function ordersAnnounce(message) {
  $("#orders-announcement").textContent = message;
  toast(message);
}
function ordersPoint(id, layer = false, camera = view) {
  const e = ENT[id], a = layer ? k.iso([e.x + 21, e.y - 4, 26 + 14 * e.explode], P) : anchors[id];
  return a ? [camera.x + a[0] * camera.s, camera.y + a[1] * camera.s] : [0, 0];
}
function ordersFocusMachine(id) {
  selected = id;
  $("#insp").classList.remove("open");
  document.body.classList.remove("has-insp");
  session = null; review = null;
  if (explode.id) { explode.id = null; explode.t = explode.goal = 0; renderPlant(); }
  const p = ordersPoint(id, ordersState.open === "question", goal);
  const left = ordersState.order.state === "draft" ? 420 : 190;
  const right = innerWidth - (ordersState.open ? $("#orders-card").offsetWidth + 250 : 420);
  const bottom = rpOn() ? $("#rpdeck").getBoundingClientRect().top - 100 : innerHeight - 280;
  const x = Math.max(left, Math.min(right, p[0])), y = Math.max(240, Math.min(bottom, p[1]));
  if (x !== p[0] || y !== p[1]) {
    goal.x += x - p[0]; goal.y += y - p[1];
    animate();
  }
  paintSelection();
}
function ordersAgent(r) {
  return r.agent + (r.agent === "Droid" ? " · second session" : "");
}
function ordersBytes(n) {
  return n.toLocaleString("en-US") + " B";
}
function ordersWinner() {
  return ordersData.READBACKS.find((r) => r.answers.q1 === ordersState.question.answered && ordersState.witnesses.every((w) => w.id === "w3" || r.replay[w.id] === "pass"));
}
function ordersUnmatched() {
  const l = ordersState.kvarm.lease;
  return !!l && !l.readback;
}
function ordersFlag() {
  const o = ordersState.order, kv = ordersState.kvarm, l = kv.lease;
  if (o.state === "draft") return { text: "Post an order" };
  if (o.state === "stale") return { text: "LEASE STALE · " + l.gen + " expired", ev: "stale" };
  if (o.state === "claimed") return { text: "CLAIMED · " + o.claimedBy + " · lease " + l.gen };
  if (ordersUnmatched()) return { text: "UNMATCHED · " + l.holder + " holds " + l.gen + " · no readback", tone: "wait" };
  if (o.state === "matched") return { text: kv.connection === "ok" ? "MATCHED · claiming" : "MATCHED · claim waits for connection", tone: "wait" };
  if (!kv.issue || !kv.dispatched) return { text: kv.connection === "ok" ? "POSTED LOCALLY" : "POSTED LOCALLY · not in Kvarm", tone: "local" };
  return { text: "OPEN · kvarm #" + kv.issue };
}
function ordersFlagAttrs(f) {
  return (f.tone ? ' data-tone="' + f.tone + '"' : '') + (f.ev ? ' data-evidence-state="' + f.ev + '"' : '');
}
function ordersSetFlag(el, f) {
  el.textContent = f.text;
  if (f.tone) el.dataset.tone = f.tone; else delete el.dataset.tone;
  if (f.ev) el.dataset.evidenceState = f.ev; else delete el.dataset.evidenceState;
}
function ordersLeaseText() {
  const l = ordersState.kvarm.lease;
  if (!l || l.holder !== (ordersState.order.claimedBy || (ordersState.order.state === "stale" && l.holder))) return "";
  if (l.state === "expired") return "expired · last renewed " + Math.round((ordersKvMs - l.renewedAt) / 1000) + " s ago";
  const left = Math.max(0, Math.ceil(ordersData.KV.leaseS - (ordersKvMs - l.renewedAt) / 1000));
  return Math.floor(left / 60) + ":" + String(left % 60).padStart(2, "0") + " left";
}
function ordersEyebrow() {
  return ordersData.ORDER.territory + (ordersState.rev ? ' · ' + ordersState.rev : '') + ' · ' + ordersData.ORDER.sheet;
}
function ordersClaimLine() {
  const o = ordersState.order, l = ordersState.kvarm.lease, w = ordersWinner();
  if (o.state === "claimed") return l.session + " · readback matched every check";
  if (o.state === "stale") return l.session + " · heartbeat rejected · " + ordersData.KV.stale.code;
  if (ordersUnmatched()) return l.holder + " holds " + l.gen + " in Kvarm · no Acan readback";
  if (o.state === "matched") return w.session + " · readback matched · claim held for Kvarm";
  return "Agents claim after a matching readback.";
}
function ordersHeader(label, title, plate = false) {
  const f = plate ? ordersFlag() : { text: title };
  return '<header' + (plate ? ' data-orders-plate' : '') + '><div><div class="orders-eyebrow"' + (plate ? ' id="orders-card-eyebrow"' : '') + '>' + ordersEscape(label) + '</div>' + (plate ? '<div class="orders-flagline"><h3 id="orders-card-status" data-orders-binding="flag"' + ordersFlagAttrs(f) + '>' + ordersEscape(f.text) + '</h3><small class="orders-lease">' + ordersLeaseText() + '</small></div>' : '<h3>' + ordersEscape(title) + '</h3>') + '</div><button class="orders-close" data-orders-close aria-label="Close" title="Close (Esc)">×</button></header>';
}
function ordersCheckRows() {
  const posted = ordersState.order.state !== "draft";
  return ordersState.witnesses.map((w) => {
    const answer = w.id === "w3", check = answer ? "answered at sqlite · " + w.check.split(" · ").slice(1).join(" · ") : w.check;
    return '<div class="orders-check" data-polarity="' + w.polarity + '" data-orders-binding="' + w.id + '"' + (answer ? ' data-answer' : '') + '><b>' + w.id + '</b><strong>' + (answer ? "Answer · sqlite" : w.polarity.replace("-", " ") + " · " + w.clock) + '</strong><p>' + ordersEscape(w.reason) + '</p><small>' + (posted ? 'Locked' : 'Becomes locked') + ' · ' + ordersEscape(check) + '</small></div>';
  }).join("");
}
function ordersReadback(r) {
  const arrived = ordersState.arrived.includes(r.session), answer = ordersState.question.answered;
  const matches = !!answer && r.answers.q1 === answer;
  const settling = ordersState.scoring === "rescoring";
  const count = Object.values(r.replay).filter((v) => v === "pass").length + (matches ? 1 : 0);
  const o = ordersState.order, claim = ordersState.kvarm.claim, mine = claim && claim.session === r.session;
  const outcome = o.state === "claimed" ? ["matches", " · matches · claimed"] : o.state === "stale" ? ["held", " · matches · lease expired"] : mine && claim.state === "rejected" ? ["held", " · matches · Kvarm rejected"] : o.state === "matched" ? ["held", " · matches · claim waits"] : ["matches", " · matches · claiming"];
  const state = !arrived ? "waiting" : settling ? "rescoring" : !answer ? "differs" : matches ? outcome[0] : "yields";
  const status = !arrived ? "awaiting readback" : settling ? "re-scoring w3" : !answer ? "p-2307 differs" : count + " of " + ordersState.witnesses.length + (matches ? outcome[1] : " · yields");
  return { arrived, matches, state, status, choice: r.answers.q1 === "1" ? "pending" : "failed" };
}
function ordersSource() {
  return "readback · Acan" + (ordersState.rev === "r2" && ordersState.scoring === "settled" ? " · re-scored on r2" : ordersState.rev ? " · " + ordersState.rev : "");
}
function ordersScores(r, source = false) {
  const rb = ordersReadback(r);
  if (!rb.arrived) return "";
  return '<div class="orders-score">' + ordersState.witnesses.map((w) => {
    const result = w.id === "w3" ? ordersState.scoring === "rescoring" ? "pending" : rb.matches ? "pass" : "fail" : r.replay[w.id];
    return '<span data-result="' + result + '">' + w.id + ' ' + (result === "pending" ? "checking" : result) + '</span>';
  }).join("") + (!ordersState.question.answered ? '<span data-result="' + (r.answers.q1 === "1" ? "pending" : "fail") + '">p-2307 ' + rb.choice + '</span>' : "") + (source ? '<span class="orders-src">' + ordersSource() + '</span>' : '') + '</div>';
}
function ordersHolderHtml(mini) {
  if (!ordersUnmatched()) return "";
  const l = ordersState.kvarm.lease, name = '<span class="orders-bezel" data-evidence-state="unknown">?</span>' + l.holder + ' · ' + l.session;
  return mini ? '<div class="orders-mini" data-state="unknown" data-orders-binding="holder"><div><span>' + name + '</span><small>holds ' + l.gen + ' · no readback</small></div></div>'
    : '<div class="orders-reading" data-state="unknown" data-orders-binding="holder"><header><b>' + name + '</b><span>holds ' + l.gen + ' · no readback</span></header><p>Claimed t-41 in Kvarm with its own token. Acan has no readback for ' + ordersState.rev + ', so the territory stays fogged.</p></div>';
}
function ordersReadingHtml(r) {
  const rb = ordersReadback(r);
  if (!rb.arrived) return '<div class="orders-reading" data-state="waiting" data-orders-readback="' + r.session + '"><header>' + ordersAgent(r) + '<span>awaiting readback</span></header></div>';
  const text = ordersEscape(r.text).replace(ordersEscape(r.conflict), '<mark>' + ordersEscape(r.conflict) + '</mark>');
  return '<div class="orders-reading" data-state="' + rb.state + '" data-orders-readback="' + r.session + '" data-orders-binding="rb:' + r.session + '"><header><b><em class="orders-choice">' + rb.choice + '</em> · ' + ordersAgent(r) + '</b><span>' + rb.status + '</span></header><p>' + text + '</p>' + ordersScores(r, true) + '</div>';
}
function ordersReadbacksHtml() {
  return ordersData.READBACKS.map(ordersReadingHtml).join("") + ordersHolderHtml(false);
}
function ordersCanPost() {
  return ordersState.order.state === "draft" && ordersState.witnesses.length === ordersData.WITNESSES.length && !!ordersState.sentence.trim();
}
function ordersGrowSentence(input) {
  input.style.height = "auto";
  input.style.height = Math.max(58, input.scrollHeight + 2) + "px";
  input.scrollTop = input.scrollLeft = 0;
  $("#orders-card").scrollTop = 0;
}
function ordersRowHtml(r) {
  return '<div class="orders-kv-row" data-state="' + r.state + '" title="' + ordersEscape(r.destination + ' · ' + r.reason) + '"><b>' + r.op + (r.count > 1 ? ' ×' + r.count : '') + '</b><span>' + ordersEscape(r.reason) + '</span><i>' + (r.result || (r.state === "held" ? "held" : r.state === "sending" ? "sending" : ordersBytes(r.bytes))) + '</i></div>';
}
function ordersStripHtml(max) {
  const kv = ordersState.kvarm, rows = [...kv.log, ...kv.held.map((h) => h.row)];
  return '<header><span>↗ ' + ordersData.KV.host + (kv.connection === "ok" ? '' : ' · ' + (kv.connection === "offline" ? "offline" : "checked")) + ' · ' + rows.length + (rows.length === 1 ? ' op' : ' ops') + '</span>' + (kv.held.length ? '<b data-tone="wait">' + kv.held.length + ' held</b>' : '<b>' + ordersBytes(kv.bytes) + '</b>') + '</header>' + rows.slice(-max).map(ordersRowHtml).join("");
}
function ordersBinding(key) {
  const K = ordersData.KV, kv = ordersState.kvarm, rev = ordersState.rev || "draft", sheet = ordersData.SHEET.rev;
  const since = (ms) => ms === null || ms === undefined ? null : Math.max(0, Math.round((ordersKvMs - ms) / 1000));
  const remote = kv.connection === "ok" ? "valid" : "stale";
  if (key === "flag") {
    const f = ordersFlag(), l = kv.lease;
    return { entity: "Order " + ordersData.ORDER.id, channel: "order.claim", value: f.text, state: f.ev || (kv.issue ? remote : "valid"), sourceClass: l ? l.evidence + " · Kvarm lease" : kv.issue ? "authenticated · Kvarm issue" : "local", locator: kv.issue ? "kvarm://" + K.host + "/" + K.repo + "/issues/" + kv.issue + " · task " + K.task : "local://.acan/orders/" + ordersData.ORDER.id + ".yaml", revision: rev + " · " + sheet, age: since(kv.lastReturn), policy: "Kvarm fields stale after " + K.staleAfter, meaning: "CLAIMED needs an Acan readback match and a live Kvarm lease of the same generation" };
  }
  if (key === "holder") {
    const l = kv.lease;
    return { entity: l.holder + " · " + l.session, channel: "kvarm.lease", value: "holds " + l.gen, state: remote, sourceClass: l.evidence, locator: "kvarm://" + K.host + "/" + K.repo + "/tasks/" + K.task, revision: rev + " · " + sheet, age: since(kv.lastReturn), policy: "Kvarm fields stale after " + K.staleAfter, meaning: "No Acan readback on " + rev + ". Never drawn as CLAIMED." };
  }
  if (key === "q1") {
    const q = ordersData.QUESTION;
    return { entity: "Question q1 · " + q.entity, channel: "readback.disagreement", value: "p-2307 · pending or failed", state: "valid", sourceClass: "observed · Acan replay", locator: "acan://orders/" + ordersData.ORDER.id + "/r1/questions/q1", revision: "r1 · " + sheet, age: "replay " + q.clock, policy: "raised when two readbacks differ", meaning: q.text };
  }
  if (key.startsWith("rb:") || key.startsWith("bot:")) {
    const r = ordersData.READBACKS.find((r) => key === "rb:" + r.session || key === "bot:" + r.bot);
    if (key.startsWith("bot:")) {
      const l = kv.lease, live = l && l.session === r.session && l.state === "live", bot = ENT[r.bot];
      return { entity: ordersAgent(r), channel: "agent.principal", value: r.session, state: bot && bot.state === "dark" ? "stopped" : "valid", sourceClass: live ? l.evidence : "declared", locator: live ? "kvarm token · principal " + r.session : "mcp://local/" + r.session, revision: rev + " · " + sheet, age: null, policy: bot && bot.state === "dark" ? "lifecycle · " + K.stale.code : "session handshake", meaning: live ? "Claimed t-41 with its own Kvarm token" : "Connected over MCP. Identity is declared by the session." };
    }
    const rb = ordersReadback(r);
    return { entity: ordersAgent(r) + " · " + r.session, channel: "readback.score", value: rb.arrived ? rb.status : "not received", state: !rb.arrived ? "unknown" : ordersState.scoring === "rescoring" ? "stale" : "valid", sourceClass: "observed · Acan replay", locator: "acan://orders/" + ordersData.ORDER.id + "/" + rev + "/readbacks/" + r.session, revision: rev + " · " + sheet, age: null, policy: "re-scored on every order revision", meaning: "readback · Acan. Kvarm stores no readback." };
  }
  const w = ordersState.witnesses.find((x) => x.id === key);
  if (!w) return null;
  return { entity: "Witness " + w.id + " · " + w.entity, channel: "witness." + w.polarity, value: w.observed.metric + " " + w.observed.value + (w.observed.unit ? " " + w.observed.unit : ""), state: "valid", sourceClass: w.id === "w3" ? "declared · your answer" : "observed · replay", locator: w.id === "w3" ? "acan://orders/" + ordersData.ORDER.id + "/r2#w3" : "replay://" + K.repo + "/" + w.entity + "@" + w.clock, revision: sheet, age: (w.id === "w3" ? "answered at " : "replay ") + w.clock, policy: ordersState.order.state === "draft" ? "editable until Post" : "locked on Post", meaning: w.check };
}
function ordersBindText(b) {
  return [b.value, b.state, b.sourceClass, b.channel, b.locator, "revision " + b.revision, typeof b.age === "number" ? b.age + " s ago" : b.age || "age not reported"].join(" · ");
}
function ordersBindingKeys() {
  return ["flag", ...ordersState.witnesses.map((w) => w.id), ...ordersData.READBACKS.filter((r) => ordersState.arrived.includes(r.session)).flatMap((r) => ["rb:" + r.session, "bot:" + r.bot]), ...(ordersUnmatched() ? ["holder"] : [])];
}
function ordersBindingsHtml() {
  return ordersBindingKeys().map((key) => { const b = ordersBinding(key); return '<button data-orders-prov="' + key + '"><b>' + ordersEscape(b.channel) + '</b><span>' + ordersEscape(b.value + ' · ' + b.state + ' · ' + b.sourceClass) + '</span></button>'; }).join("");
}
function ordersProvOpen(key, anchor) {
  const b = ordersBinding(key);
  if (!b) return;
  ordersProvKey = key;
  const el = $("#orders-prov");
  el.innerHTML = '<header><div><div class="orders-eyebrow">Binding · ' + ordersEscape(b.sourceClass) + '</div><h3>' + ordersEscape(b.entity) + '</h3></div><button class="orders-close" data-orders-prov-close aria-label="Close provenance" title="Close (Esc)">×</button></header><div class="orders-body"><p id="orders-prov-text">' + ordersEscape(ordersBindText(b)) + '</p><dl><dt>Channel</dt><dd>' + ordersEscape(b.channel) + '</dd><dt>Value · state</dt><dd data-evidence-state="' + b.state + '">' + ordersEscape(b.value + ' · ' + b.state) + '</dd><dt>Source</dt><dd>' + ordersEscape(b.sourceClass) + '</dd><dt>Locator</dt><dd>' + ordersEscape(b.locator) + '</dd><dt>Revision</dt><dd>' + ordersEscape(b.revision) + '</dd><dt>Freshness</dt><dd>' + ordersEscape((typeof b.age === "number" ? b.age + " s ago" : b.age || "age not reported") + ' · ' + b.policy) + '</dd><dt>Meaning</dt><dd>' + ordersEscape(b.meaning) + '</dd></dl></div>';
  el.hidden = false;
  const r = anchor.getBoundingClientRect(), w = el.offsetWidth, h = el.offsetHeight;
  const left = Math.max(20, Math.min(innerWidth - w - 20, r.left));
  const top = r.bottom + 10 + h <= innerHeight - 50 ? r.bottom + 10 : Math.max(70, r.top - h - 10);
  el.style.left = left + "px"; el.style.top = top + "px";
}
function ordersProvClose() {
  ordersProvKey = null;
  $("#orders-prov").hidden = true;
}
function ordersPill() {
  const el = $(".egress"), kv = ordersState.kvarm;
  if (!el) return;
  if (ordersEgressHtml === null) ordersEgressHtml = el.innerHTML;
  el.dataset.kv = kv.connection === "ok" ? "ok" : kv.connection === "offline" ? "offline" : "checked";
  el.setAttribute("role", "button"); el.tabIndex = 0;
  el.title = "Kvarm · " + ordersData.KV.origin + " · declared egress · U";
  el.innerHTML = '<span class="dot"></span>' + (kv.connection === "ok" ? 'local + kvarm · ' + ordersBytes(kv.bytes) + ' out' : 'kvarm ' + (kv.connection === "offline" ? "offline" : "checked") + ' · ' + kv.held.length + ' held');
}
function ordersPanelRender() {
  const kv = ordersState.kvarm, K = ordersData.KV, p = $("#orders-kv-panel");
  p.hidden = !kv.panel;
  if (!kv.panel) return;
  const action = kv.connection === "offline" ? '<div class="orders-status" data-tone="wait">Offline · Kvarm fields are stale · writes are held</div><button class="orders-primary" data-orders-kv="reconnect">Reconnect <kbd>Enter</kbd></button>'
    : kv.connection === "checking" ? '<div class="orders-status">checking heads and leases</div>'
    : kv.connection === "checked" ? '<div class="orders-status" id="orders-kv-checked">checked heads and leases · ' + ordersEscape(K.checked) + '</div><button class="orders-primary" data-orders-kv="send">Send ' + kv.held.length + ' held ' + (kv.held.length === 1 ? 'write' : 'writes') + ' <kbd>Enter</kbd></button>' : '';
  p.innerHTML = '<header><div><div class="orders-eyebrow">Kvarm · declared egress</div><h3>' + K.host + '</h3></div><button class="orders-close" data-orders-panel-close aria-label="Close Kvarm" title="Close (Esc)">×</button></header><div class="orders-body"><p class="orders-note">Opt-in connection to your own origin, ' + K.origin + '. Scripted fixture. No browser network requests.</p>' + action +
    '<div class="orders-section">Held writes · ' + kv.held.length + '</div>' + (kv.held.map((h) => ordersRowHtml(h.row)).join("") || '<div class="orders-kv-row"><b>0</b><span>nothing held locally</span><i></i></div>') +
    '<div class="orders-section">Network log · ' + ordersBytes(kv.bytes) + '</div>' + (kv.log.map(ordersRowHtml).join("") || '<div class="orders-kv-row"><b>0 B</b><span>no outbound operations</span><i></i></div>') + '</div>';
}
function ordersPanelToggle(open = !ordersState.kvarm.panel) {
  ordersState.kvarm.panel = open;
  ordersPanelRender();
  if (open) { const b = $("#orders-kv-panel .orders-primary") || $("#orders-kv-panel .orders-close"); b.focus({ preventScroll: true }); }
}
function ordersKvRender() {
  ordersPill();
  const posted = ordersState.order.state !== "draft";
  for (const [id, max] of [["#orders-kv-dock", 2], ["#orders-kv-card", 3]]) { const el = $(id); if (el) el.innerHTML = ordersStripHtml(max); }
  if (posted && ordersState.open === "order" && $("#orders-card-status")) {
    ordersSetFlag($("#orders-card-status"), ordersFlag());
    $("#orders-card-eyebrow").textContent = ordersEyebrow();
    $("#orders-claim-session").textContent = ordersClaimLine();
    const list = $("#orders-bindings-list");
    if (list) list.innerHTML = ordersBindingsHtml();
    $("#orders-bindings-count").textContent = ordersBindingKeys().length;
  }
  ordersLeaseShown = null;
  if (posted) ordersRenderDock();
  if (ordersState.kvarm.panel) { const had = document.activeElement && document.activeElement.closest("#orders-kv-panel"); ordersPanelRender(); if (had) ordersPanelToggle(true); }
}
function ordersAt(ms, done) {
  ordersInflight.push({ due: ordersKvMs + ms, row: null, done });
}
function ordersTransmit(row, done) {
  const kv = ordersState.kvarm, last = kv.log[kv.log.length - 1], spec = ordersData.KV.ops[row.op];
  if (row.op === "heartbeat_task" && last && last.op === "heartbeat_task" && last.state === "sent") { last.count++; last.bytes += spec.bytes; row = last; }
  else kv.log.push(row);
  kv.bytes += spec.bytes;
  row.state = "sending";
  ordersInflight.push({ due: ordersKvMs + spec.ms, row, done });
  ordersKvRender();
}
function ordersSend(op, reason, done, force = false) {
  const kv = ordersState.kvarm, spec = ordersData.KV.ops[op];
  const row = { op, reason: reason || spec.reason, bytes: spec.bytes, destination: ordersData.KV.host, state: "held", count: 1 };
  if (kv.connection !== "ok" && !force) { kv.held.push({ row, done }); ordersKvRender(); return; }
  ordersTransmit(row, done);
}
function ordersReconnect() {
  const kv = ordersState.kvarm;
  if (kv.connection !== "offline") return;
  kv.connection = "checking";
  ordersSend("check_heads_and_leases", null, () => {
    kv.connection = "checked"; ordersKvRender(); ordersPanelToggle(true);
    ordersAnnounce("Checked heads and leases. Enter sends " + kv.held.length + " held writes.");
  }, true);
  ordersPanelToggle(true);
}
function ordersSendHeld() {
  const kv = ordersState.kvarm, n = kv.held.length;
  if (kv.connection !== "checked") return;
  kv.connection = "ok";
  const next = () => { const h = kv.held.shift(); if (!h) return; ordersTransmit(h.row, (row) => { if (h.done) h.done(row); next(); }); };
  next();
  ordersPanelToggle(true);
  ordersAnnounce("Sending " + n + " held writes to Kvarm, in order.");
}
function ordersKvTick(delta) {
  ordersKvMs += delta;
  const kv = ordersState.kvarm, due = ordersInflight.filter((f) => f.due <= ordersKvMs);
  if (due.length) {
    ordersInflight = ordersInflight.filter((f) => f.due > ordersKvMs);
    for (const f of due) { if (f.row) { f.row.state = "sent"; kv.lastReturn = ordersKvMs; } if (f.done) f.done(f.row); }
    ordersKvRender();
  }
  const l = kv.lease;
  if (l && l.state === "live" && ordersState.order.state === "claimed" && ordersKvMs >= l.beatAt) {
    l.beatAt = Infinity;
    ordersSend("heartbeat_task", null, (row) => {
      if (kv.script === "stale") { row.state = "failed"; row.result = ordersData.KV.stale.code; ordersExpire(); }
      else { l.renewedAt = ordersKvMs; l.beatAt = ordersKvMs + ordersData.KV.heartbeatS * 1000; }
    });
  }
  const text = ordersLeaseText();
  if (text !== ordersLeaseShown) { ordersLeaseShown = text; for (const el of document.querySelectorAll(".orders-ui .orders-lease")) el.textContent = text; }
}
function ordersRenderCard() {
  const card = $("#orders-card"), open = ordersState.open;
  card.hidden = !open;
  if (!open) return;
  if (open.startsWith("w")) {
    const w = ordersState.witnesses.find((x) => x.id === open), locked = ordersState.order.state !== "draft";
    card.innerHTML = ordersHeader(w.id === "w3" ? "Answered at sqlite" : "Replay · " + w.clock, locked ? "Locked witness · " + w.id : "Pin " + w.polarity.replace("-", " ")) +
      '<div class="orders-body"><div class="orders-status" data-orders-binding="' + w.id + '">' + ordersEscape(w.observed.metric + ' · ' + w.observed.value + ' ' + w.observed.unit + ' · ' + w.context) + '</div>' +
      (locked ? '<p class="orders-sentence">' + ordersEscape(w.reason) + '</p>' : '<label for="orders-reason">Reason · editable until Post</label><textarea id="orders-reason">' + ordersEscape(w.reason) + '</textarea>') +
      '<div class="orders-locks">' + (locked ? "Locked" : "Becomes locked on Post") + ' · ' + ordersEscape(w.id === "w3" ? "answered at sqlite · " + w.check.split(" · ").slice(1).join(" · ") : w.check) + '</div>' +
      (locked ? "" : '<div class="orders-actions"><span>Enter keeps the reason</span><button class="orders-primary" id="orders-keep">Pin <kbd>Enter</kbd></button></div>') + '</div>';
    if (!locked) $("#orders-reason").oninput = (ev) => { w.reason = ev.target.value; };
    return;
  }
  if (open === "question") {
    card.innerHTML = ordersHeader('sqlite · ' + ordersData.QUESTION.layer, 'Answer at p-2307') +
      '<div class="orders-body"><p class="orders-sentence">' + ordersData.QUESTION.text + '</p>' +
      ordersData.QUESTION.options.map((o) => {
        const r = ordersData.READBACKS.find((r) => r.answers.q1 === o.key);
        return '<button class="orders-answer" data-orders-answer="' + o.key + '"><kbd>Shift ' + o.key + '</kbd><span>' + o.label + '</span><small>' + ordersAgent(r) + ' reads this</small></button>';
      }).join("") + '<p class="orders-question-note">Your answer becomes a locked witness.<br>The readbacks must match before an agent claims.</p></div>';
    return;
  }
  const posted = ordersState.order.state !== "draft";
  card.innerHTML = (posted ? ordersHeader(ordersEyebrow(), "", true) + '<div class="orders-kv" id="orders-kv-card" data-orders-binding="flag">' + ordersStripHtml(3) + '</div>' : ordersHeader(ordersData.ORDER.territory + ' · ' + ordersData.ORDER.sheet, "Post an order")) +
    '<div class="orders-body">' + (posted ? '<div class="orders-status" id="orders-claim-session">' + ordersEscape(ordersClaimLine()) + '</div><p class="orders-sentence">' + ordersEscape(ordersState.sentence) + '</p>' : '<div class="orders-status">DRAFT · witnesses become locked on Post</div><label for="orders-sentence">One sentence</label><textarea id="orders-sentence" rows="2" placeholder="' + ordersEscape(ordersData.ORDER.placeholder) + '">' + ordersEscape(ordersState.sentence) + '</textarea>') +
    (posted ? '<section class="orders-readbacks"><div class="orders-section">Readbacks · before code</div><div id="orders-readbacks">' + ordersReadbacksHtml() + '</div></section>' : "") +
    '<div id="orders-checks">' + ordersCheckRows() + '</div><div class="orders-locks">' + ordersData.ORDER.budgets.map((b) => '<span>' + (posted ? 'Locked' : 'Becomes locked') + ' · ' + ordersEscape(b) + '</span>').join("") + '</div><p class="orders-passive">Codex cannot see orders. Passive session, no MCP.</p>' +
    (posted ? '<details class="orders-bindings" id="orders-bindings"><summary>Evidence bindings · <b id="orders-bindings-count">' + ordersBindingKeys().length + '</b><span>Alt + hover any mark</span></summary><div id="orders-bindings-list">' + ordersBindingsHtml() + '</div></details>' : '<div class="orders-actions"><span id="orders-post-note">' + (ordersState.witnesses.length === ordersData.WITNESSES.length ? ordersState.sentence.trim() ? 'No assignment. Agents claim on a match.' : 'Write a sentence before Post.' : 'Pin both replay witnesses before Post.') + '</span><button class="orders-primary" id="orders-submit"' + (ordersCanPost() ? '' : ' disabled') + '>Post <kbd>Enter</kbd></button></div>') + '</div>';
  if (!posted) {
    const input = $("#orders-sentence");
    ordersGrowSentence(input);
    input.oninput = () => {
      ordersState.sentence = input.value;
      ordersGrowSentence(input);
      $("#orders-submit").disabled = !ordersCanPost();
      $("#orders-post-note").textContent = ordersState.witnesses.length < ordersData.WITNESSES.length ? "Pin both replay witnesses before Post." : ordersState.sentence.trim() ? "No assignment. Agents claim on a match." : "Write a sentence before Post.";
      ordersPosition();
    };
    input.onfocus = () => ordersGrowSentence(input);
  }
}
function ordersRenderDock() {
  const posted = ordersState.order.state !== "draft";
  $("#orders-guide").hidden = posted;
  $("#orders-hint").hidden = !posted;
  $("#orders-hint").textContent = ordersState.question.answered ? "P inspect · click a pin · U Kvarm" : "P inspect · I question · U Kvarm";
  $("#orders-dock").hidden = !posted || ordersState.open === "order";
  const f = ordersFlag(), l = ordersState.kvarm.lease, lease = ordersLeaseText();
  $("#orders-plate").innerHTML = '<b data-orders-binding="flag"' + ordersFlagAttrs(f) + '>' + ordersEscape(f.text) + '</b><span>' + (l && l.readback ? l.session + ' · ' : '') + ordersData.ORDER.sheet + (lease ? ' · <i class="orders-lease">' + lease + '</i>' : '') + '</span>';
  $("#orders-kv-dock").innerHTML = ordersStripHtml(2);
  $("#orders-mini-readbacks").innerHTML = '<div class="orders-kv"><header><span>' + ordersSource() + '</span></header></div>' + ordersData.READBACKS.map((r) => {
    const rb = ordersReadback(r);
    return '<div class="orders-mini" data-state="' + rb.state + '" data-orders-session="' + r.session + '"' + (rb.arrived ? ' data-orders-binding="rb:' + r.session + '"' : '') + '><div><span>' + (rb.arrived ? '<b class="orders-choice">' + rb.choice + '</b>' : '') + '<span class="orders-agent">' + ordersAgent(r) + '</span></span><small>' + rb.status + '</small></div>' + ordersScores(r) + '</div>';
  }).join("") + ordersHolderHtml(true);
  $(".bl .wait b").textContent = ordersWaitingBase + (ordersState.arrived.length === ordersData.READBACKS.length && !ordersState.question.answered ? 1 : 0) + (ordersUnmatched() ? 1 : 0);
}
function ordersUpdateReadbacks() {
  if (ordersState.open === "order" && ordersState.order.state !== "draft") {
    $("#orders-readbacks").innerHTML = ordersReadbacksHtml();
    $("#orders-checks").innerHTML = ordersCheckRows();
  }
  ordersKvRender();
}
function ordersClose() {
  ordersState.open = null; ordersCardTop = null;
  $("#orders-card").hidden = true;
  $("#orders-link").innerHTML = "";
  ordersRenderDock();
  if (ordersRestoreFocus && ordersRestoreFocus.isConnected && ordersRestoreFocus.getClientRects().length) ordersRestoreFocus.focus({ preventScroll: true });
}
function ordersOpen(open, machine, focus = true) {
  ordersRestoreFocus = document.activeElement;
  ordersState.open = open; ordersCardTop = null;
  ordersRenderCard(); ordersRenderDock();
  ordersFocusMachine(machine); ordersPosition();
  if (focus) {
    const input = open.startsWith("w") ? $("#orders-reason") : open === "order" && ordersState.order.state === "draft" ? $("#orders-sentence") : $("#orders-card .orders-close");
    if (input) { input.focus({ preventScroll: true }); if (input.id === "orders-sentence") ordersGrowSentence(input); }
  }
}
function ordersAddPin(w) {
  const c = document.createElement("button");
  c.className = "chip orders-witness rp loud show";
  c.dataset.id = w.entity; c.dataset.state = "ok"; c.dataset.polarity = w.polarity; c.dataset.ordersWitness = w.id; c.dataset.ordersBinding = w.id;
  c.innerHTML = '<span class="mono">' + w.id + '</span>' + (w.id === "w3" ? 'ANSWER · sqlite' : w.polarity.replace("-", " ").toUpperCase() + ' · ' + w.clock) + '<span class="rd"></span>';
  $("#chips").appendChild(c);
  renderPlant(); apply(); layoutChips();
}
function ordersBotChip(r) {
  const bot = ENT[r.bot], l = ordersState.kvarm.lease, live = l && l.session === r.session && l.state === "live";
  let c = document.querySelector('.chip[data-id="' + r.bot + '"]');
  if (!c) { c = document.createElement("div"); c.className = "chip orders-bot"; c.dataset.id = r.bot; $("#chips").appendChild(c); }
  c.dataset.state = bot.state;
  c.innerHTML = '<span class="mono">' + bot.mono + '</span>' + r.agent + '<small>' + r.session + ' · ' + (live ? l.evidence : bot.state === "dark" ? "stopped" : "declared") + '</small><span class="rd">' + bot.reading + '</span>';
}
function ordersPin(keep) {
  if (ordersState.order.state !== "draft") return ordersAnnounce("Posted witnesses are locked.");
  const w = ordersData.WITNESSES.find((x) => x.polarity === (keep ? "must-keep" : "must-not"));
  if (!rpOn() || selected !== w.entity || rpClock(RP.t).slice(0, 5) !== w.clock) return ordersAnnounce("Replay " + w.clock + " at " + w.entity + ", then Pin.");
  if (!ordersState.witnesses.some((x) => x.id === w.id)) {
    const copy = JSON.parse(JSON.stringify(w));
    ordersState.witnesses.push(copy); ordersAddPin(copy);
  }
  ordersOpen(w.id, w.entity);
}
function ordersKeep() {
  const w = ordersState.witnesses.find((x) => x.id === ordersState.open);
  if (w && !w.reason.trim()) w.reason = ordersData.WITNESSES.find((x) => x.id === w.id).reason;
  ordersClose();
}
function ordersDispatched() {
  const kv = ordersState.kvarm, U = ordersData.KV.unmatched;
  kv.dispatched = true;
  if (kv.script !== "unmatched") return;
  ordersAt(U.pollMs, () => ordersSend("get_task", null, () => {
    kv.lease = { gen: ordersData.KV.generation, holder: U.holder, session: U.session, evidence: U.evidence, readback: false, state: "live", renewedAt: ordersKvMs };
    const chip = document.querySelector('.chip[data-id="bot-cx"]');
    if (chip) chip.innerHTML = '<span class="mono">CX</span>' + U.holder + '<small>' + U.session + ' · ' + U.evidence + ' · holds ' + kv.lease.gen + '</small><span class="rd">no readback</span>';
    $(".bl .wait").insertAdjacentHTML("afterend", '<button class="agent orders-holder" id="orders-holder-row" data-orders-binding="holder">' + DRONE_ICON_JS + '<strong>' + U.holder + '</strong> holds ' + kv.lease.gen + ' · no readback<span>Kvarm</span></button>');
    $("#orders-holder-row").onclick = () => { if (ordersActive()) ordersOpen("order", ordersData.ORDER.on); };
    layoutChips(); ordersUpdateReadbacks();
    ordersAnnounce("Kvarm reports " + U.holder + " holds " + kv.lease.gen + " with no readback. The order stays unclaimed.");
  }));
}
function ordersPost() {
  if (!ordersActive() || !ordersCanPost()) return;
  ordersState.sentence = ordersState.sentence.trim();
  document.activeElement.blur();
  selected = null;
  setMode("live");
  ordersState.order.state = "open"; ordersState.rev = "r1";
  ordersLiveMs = 0;
  ordersCardTop = null;
  ordersRenderCard(); ordersRenderDock(); ordersFocusMachine(ordersData.ORDER.on);
  const kv = ordersState.kvarm, K = ordersData.KV;
  ordersSend("create_issue", null, () => { kv.issue = K.issue; });
  ordersSend("dispatch_issue", K.ops.dispatch_issue.reason + " on #" + K.issue, ordersDispatched);
  const plate = $("#orders-card-status");
  plate.classList.add("orders-ink");
  plate.onanimationend = () => plate.classList.remove("orders-ink");
  $("#orders-card .orders-close").focus({ preventScroll: true });
  ordersAnnounce(kv.connection === "ok" ? "Order r1 posted locally. Witnesses are locked. Agents read back before code." : "Order r1 posted locally, not in Kvarm. Writes are held until you reconnect.");
}
function ordersFloor(r, status) {
  const row = [...document.querySelectorAll(".bl .agent")].find((el) => el.querySelector("strong")?.textContent === r.agent);
  if (!row) return;
  row.classList.remove("off");
  row.dataset.ordersSession = r.session;
  row.innerHTML = DRONE_ICON_JS + "<strong>" + r.agent + "</strong> " + (r.agent === "Droid" ? "second session · " : "") + status + "<span>MCP</span>";
  row.onclick = () => { if (ordersActive()) ordersOpen("order", ordersData.ORDER.on); };
}
function ordersArrive(r) {
  if (!ordersActive() || document.body.dataset.mode !== "live" || ordersState.order.state !== "open" || ordersState.arrived.includes(r.session)) return;
  ordersState.arrived.push(r.session);
  const dock = ENT[ordersData.ORDER.on];
  if (!ENT[r.bot]) {
    const bot = { id: r.bot, kind: "bot", mono: r.agent === "Droid" ? "D2" : "CC", name: ordersAgent(r), x: dock.x - 45, y: dock.y + (r.agent === "Droid" ? 55 : 20), state: "wait", reading: "readback", sub: r.session + " · connected over MCP", params: { target: [dock.x, dock.y, 10] } };
    SCENE.entities.push(bot); ENT[bot.id] = bot;
  }
  ordersBotChip(r);
  ordersFloor(r, "reading payout order");
  renderPlant(); apply(); layoutChips(); ordersUpdateReadbacks();
  if (ordersState.arrived.length === ordersData.READBACKS.length) {
    layoutChips();
    ordersAnnounce("Two readings differ on p-2307. Answer at sqlite.");
  } else ordersAnnounce(ordersAgent(r) + " sent a readback.");
}
function ordersAnswer(key) {
  const option = ordersData.QUESTION.options.find((x) => x.key === key);
  if (!ordersActive() || document.body.dataset.mode !== "live" || ordersState.open !== "question" || !option || ordersState.question.answered || ordersState.arrived.length !== ordersData.READBACKS.length) return;
  ordersState.question.answered = key; ordersState.rev = "r2";
  ordersState.scoring = "rescoring"; ordersScoreMs = ordersVisibleMs = 0;
  const w = JSON.parse(JSON.stringify(option.witness));
  ordersState.witnesses.push(w); ordersAddPin(w);
  ordersSend("comment_issue");
  ordersClose(); ordersRenderDock(); ordersPosition();
  ordersAnnounce("Answer pinned as w3 at sqlite. Order is now r2. Re-scoring the readbacks.");
}
function ordersClaim() {
  const winner = ordersWinner(), kv = ordersState.kvarm;
  if (!ordersActive() || document.body.dataset.mode !== "live" || !winner || ordersState.order.state !== "open" || ordersState.scoring !== "settled" || ordersVisibleMs < 1400 || !ENT[winner.bot] || kv.claim) return;
  kv.claim = { session: winner.session, state: "sent" };
  ordersSend("claim_task", winner.agent + " · " + ordersData.KV.ops.claim_task.reason, (row) => {
    if (kv.lease && kv.lease.holder !== winner.agent) {
      kv.claim.state = "rejected"; row.state = "rejected"; row.result = "rejected";
      ordersUpdateReadbacks();
      ordersAnnounce("Kvarm rejected the claim from " + winner.agent + ". " + kv.lease.holder + " holds " + kv.lease.gen + " with no readback.");
    } else ordersGrant(winner);
  });
  if (kv.connection !== "ok") {
    ordersState.order.state = "matched";
    ordersUpdateReadbacks();
    ordersAnnounce(ordersAgent(winner) + " matches every check. The claim waits for the Kvarm connection.");
  }
}
function ordersGrant(winner) {
  const K = ordersData.KV, kv = ordersState.kvarm;
  ordersState.order.state = "claimed"; ordersState.order.claimedBy = winner.agent;
  kv.claim.state = "granted";
  kv.lease = { gen: K.generation, holder: winner.agent, session: winner.session, evidence: "authenticated", readback: true, state: "live", renewedAt: ordersKvMs, beatAt: ordersKvMs + (kv.script === "stale" ? K.stale.heartbeatMs : K.heartbeatS * 1000) };
  const territory = SCENE.territories.find((t) => t.id === ordersData.ORDER.territory);
  ordersTerritory = ordersTerritory || { label: territory.label, fog: TERR[ordersData.ORDER.territory].fog };
  territory.state = "claimed"; territory.fog = false; territory.label = ordersData.ORDER.territory.toUpperCase() + ' · CLAIMED · ' + winner.agent;
  Object.assign(TERR[ordersData.ORDER.territory], { state: "claimed", fog: null, owner: winner.agent });
  const bot = ENT[winner.bot], dock = ENT[ordersData.ORDER.on];
  bot.x = dock.x - 10; bot.y = dock.y + 15; bot.reading = "claimed · lease " + K.generation; bot.state = "ok";
  const loser = ordersData.READBACKS.find((r) => r !== winner);
  ENT[loser.bot].reading = "yields"; ENT[loser.bot].state = "fail";
  $("#orders-fog").hidden = true;
  for (const r of ordersData.READBACKS) { ordersFloor(r, r === winner ? "claimed payout order" : "yields payout order"); ordersBotChip(r); }
  renderPlant(); apply(); layoutChips(); ordersUpdateReadbacks();
  if (!ordersState.open || ordersState.open === "order") ordersFocusMachine(ordersData.ORDER.on);
  ordersPosition();
  const plate = ordersState.open === "order" ? $("#orders-card-status") : $("#orders-plate b");
  plate.classList.remove("orders-ink"); void plate.offsetWidth; plate.classList.add("orders-ink");
  plate.onanimationend = () => plate.classList.remove("orders-ink");
  ordersAnnounce(ordersAgent(winner) + " · " + winner.session + " claimed the order. Readback matched " + ordersState.witnesses.length + " of " + ordersState.witnesses.length + ".");
}
function ordersExpire() {
  const kv = ordersState.kvarm, l = kv.lease, winner = ordersData.READBACKS.find((r) => r.session === l.session);
  l.state = "expired";
  ordersState.order.state = "stale"; ordersState.order.claimedBy = null;
  const territory = SCENE.territories.find((t) => t.id === ordersData.ORDER.territory);
  territory.state = "unclaimed"; territory.fog = true; territory.label = ordersData.ORDER.territory.toUpperCase() + " · LEASE STALE";
  Object.assign(TERR[ordersData.ORDER.territory], { state: "unclaimed", fog: ordersTerritory.fog, owner: null });
  const bot = ENT[winner.bot];
  bot.reading = "stopped · " + ordersData.KV.stale.code; bot.state = "dark";
  $("#orders-fog").hidden = false;
  ordersFloor(winner, "stopped · " + ordersData.KV.stale.code);
  ordersBotChip(winner);
  renderPlant(); apply(); layoutChips(); ordersUpdateReadbacks();
  ordersAnnounce("Lease " + l.gen + " expired. Kvarm rejected the heartbeat with " + ordersData.KV.stale.code + ". " + winner.agent + " stopped.");
}
function ordersQuestion() {
  if (document.body.dataset.mode !== "live" || ordersState.arrived.length !== ordersData.READBACKS.length || ordersState.question.answered) return ordersAnnounce("No open question in this view.");
  ordersOpen("question", ordersData.QUESTION.entity);
}
function ordersOrder() {
  if (!TERR[ordersData.ORDER.territory].entities.includes(selected)) return ordersAnnounce("Select a machine in paypal-payouts, then Post.");
  ordersOpen("order", ordersData.ORDER.on);
}
function ordersGuide() {
  if (ordersState.order.state !== "draft") return;
  const reasons = ordersData.WITNESSES.map((w, i) => {
    const ready = rpOn() && selected === w.entity && rpClock(RP.t).slice(0, 5) === w.clock;
    $(i ? "#orders-pin-keep" : "#orders-pin-not").disabled = !ready;
    return ready ? "" : w.polarity.replace("-", " ") + " · select " + w.entity + " at " + w.clock;
  }).filter(Boolean);
  $("#orders-pin-gate").textContent = reasons.join(". ");
  const selectedTerritory = TERR[ordersData.ORDER.territory].entities.includes(selected);
  $("#orders-post").disabled = !selectedTerritory;
  $("#orders-post-gate").textContent = selectedTerritory ? "" : "Select a machine in paypal-payouts before Post.";
}
function ordersPosition() {
  const pp = ordersPoint(ordersData.ORDER.on), qp = ordersPoint(ordersData.QUESTION.entity, true);
  const card = $("#orders-card"), dock = $("#orders-dock"), pin = $("#orders-question-pin");
  const posted = ordersState.order.state !== "draft";
  const inView = (p) => p[0] >= 40 && p[0] <= innerWidth - 70 && p[1] >= 160 && p[1] <= innerHeight - 130;
  const question = ordersState.arrived.length === ordersData.READBACKS.length && !ordersState.question.answered && document.body.dataset.mode === "live";
  const rects = [...document.querySelectorAll(".chip")].filter(chipShown).map((c) => c.getBoundingClientRect());
  const overlaps = (a, b, gap = 10) => a.left < b.right + gap && a.right > b.left - gap && a.top < b.bottom + gap && a.bottom > b.top - gap;
  let lines = "";
  if (ordersState.open) {
    const open = ordersState.open, id = open === "question" ? ordersData.QUESTION.entity : open === "order" ? ordersData.ORDER.on : ordersState.witnesses.find((x) => x.id === open).entity;
    const p = ordersPoint(id, open === "question");
    const left = Math.max(20, Math.min(innerWidth - card.offsetWidth - 90, p[0] + 170));
    const bottom = rpOn() ? $("#rpdeck").getBoundingClientRect().top - 16 : innerHeight - 60;
    if (ordersCardTop === null) ordersCardTop = open === "order" ? 100 : Math.max(140, Math.min(bottom - card.offsetHeight, p[1] - 140));
    card.style.left = left + "px"; card.style.top = ordersCardTop + "px";
    card.style.maxHeight = Math.max(160, bottom - ordersCardTop) + "px";
    rects.push(card.getBoundingClientRect());
    if (inView(p)) lines += '<path data-orders-card-leader d="M' + p[0] + ' ' + p[1] + 'H' + (left - 14) + 'V' + (ordersCardTop + 40) + 'H' + left + '"/><circle cx="' + p[0] + '" cy="' + p[1] + '" r="3"/>';
  }
  const edge = (id, p, visible, text) => {
    const el = $(id);
    el.hidden = !visible;
    if (!visible) return;
    const arrow = p[0] < 40 ? "← " : p[0] > innerWidth - 70 ? "→ " : p[1] < 160 ? "↑ " : "↓ ";
    el.textContent = arrow + text;
    el.style.left = Math.max(20, Math.min(innerWidth - el.offsetWidth - 90, p[0])) + "px";
    el.style.top = (id === "#orders-order-edge" ? 200 : 160) + "px";
  };
  edge("#orders-question-edge", qp, question && !inView(qp) && ordersState.open !== "question", "sqlite · question · I");
  edge("#orders-order-edge", pp, posted && !inView(pp) && ordersState.open !== "order", "PayPal · order · P");
  dock.hidden = !posted || ordersState.open === "order" || !inView(pp);
  if (!dock.hidden) {
    const width = dock.offsetWidth, height = dock.offsetHeight;
    const candidates = [[pp[0] + 40, pp[1] + 54], [pp[0] - width - 40, pp[1] + 54], [pp[0] + 40, pp[1] - height - 70], [pp[0] - width - 40, pp[1] - height - 70]];
    if (!card.hidden) {
      const r = card.getBoundingClientRect();
      candidates.push([r.right + 16, pp[1] + 54], [pp[0] + 40, r.bottom + 16], [r.left - width - 16, pp[1] + 54]);
    }
    const boxes = candidates.map(([x, y]) => ({ left: Math.max(20, Math.min(innerWidth - width - 90, x)), top: Math.max(155, Math.min(innerHeight - height - 60, y)), right: 0, bottom: 0 })).map((r) => ({ ...r, right: r.left + width, bottom: r.top + height }));
    const r = boxes.find((b) => rects.every((r) => !overlaps(b, r))) || boxes.find((b) => card.hidden || !overlaps(b, card.getBoundingClientRect())) || boxes[0];
    dock.style.left = r.left + "px"; dock.style.top = r.top + "px"; rects.push(r);
    lines += '<path d="M' + pp[0] + ' ' + pp[1] + 'V' + (r.top + 20) + 'H' + (r.left > pp[0] ? r.left : r.right) + '"/><circle cx="' + pp[0] + '" cy="' + pp[1] + '" r="3"/>';
  }
  pin.hidden = !question || !inView(qp) || ordersState.open === "question";
  if (!pin.hidden) {
    const width = pin.offsetWidth, height = pin.offsetHeight;
    const candidates = [[qp[0] + 26, qp[1] + 26], [qp[0] + 26, qp[1] - height - 60], [qp[0] - width - 26, qp[1] + 26], [qp[0] - width - 26, qp[1] - height - 60]];
    const r = candidates.map(([x, y]) => ({ left: x, top: y, right: x + width, bottom: y + height })).find((b) => b.left >= 20 && b.right <= innerWidth - 70 && b.top >= 150 && b.bottom <= innerHeight - 80 && rects.every((r) => !overlaps(b, r, 6)));
    pin.hidden = !r;
    if (r) {
      pin.style.left = r.left + "px"; pin.style.top = r.top + "px";
      lines += '<path data-orders-question-leader d="M' + qp[0] + ' ' + qp[1] + 'V' + (r.top + height / 2) + 'H' + (r.left > qp[0] ? r.left : r.right) + '"/><circle cx="' + qp[0] + '" cy="' + qp[1] + '" r="3"/>';
    } else edge("#orders-question-edge", qp, true, "sqlite · question · I");
  }
  $("#orders-fog").style.left = pp[0] - 110 + "px";
  $("#orders-fog").style.top = pp[1] - 20 + "px";
  $("#orders-link").innerHTML = lines;
  for (const chip of document.querySelectorAll(".chip")) {
    const r = chip.getBoundingClientRect(), occluded = !card.hidden && overlaps(r, card.getBoundingClientRect(), 0);
    if (!chip.matches(".orders-witness")) { chip.style.visibility = occluded ? "hidden" : ""; continue; }
    const w = ordersState.witnesses.find((w) => w.id === chip.dataset.ordersWitness);
    const clipped = r.left < 12 || r.right > innerWidth - 12 || r.top < 120 || r.bottom > innerHeight - 60;
    chip.style.visibility = clipped || occluded ? "hidden" : "";
    let edge = document.querySelector('[data-orders-witness-edge="' + w.id + '"]');
    if (clipped && !edge) {
      edge = document.createElement("button");
      edge.className = "orders-mark orders-edge";
      edge.dataset.ordersWitnessEdge = w.id;
      edge.onclick = () => { if (ordersActive()) ordersOpen(w.id, w.entity); };
      $("#orders-witness-edges").appendChild(edge);
    }
    if (edge) {
      edge.hidden = !clipped;
      edge.textContent = (r.left < 12 ? "← " : r.right > innerWidth - 12 ? "→ " : r.top < 120 ? "↑ " : "↓ ") + w.entity + " · " + w.id;
      edge.style.left = (r.left < 12 ? 20 : Math.min(innerWidth - 250, r.left)) + "px";
      edge.style.top = (240 + ordersState.witnesses.indexOf(w) * 36) + "px";
    }
  }
}
function ordersEvidenceVisible() {
  const root = ordersState.open === "order" ? $("#orders-readbacks") : $("#orders-dock");
  if (!root || root.hidden || !root.getClientRects().length) return false;
  const r = root.getBoundingClientRect();
  return r.left >= 0 && r.right <= innerWidth && r.top >= 100 && r.bottom <= innerHeight - 40 &&
    [[r.left + 8, r.top + 8], [r.right - 8, r.bottom - 8]].every(([x, y]) => root.contains(document.elementFromPoint(x, y)));
}
function ordersInit(params) {
  if (!ordersActive() || ordersInitialized) return;
  ordersInitialized = true;
  window.ordersState = ordersState;
  ordersWaitingBase = +$(".bl .wait b").textContent;
  setMode("replay");
  rpSeek(params.has("t") ? +params.get("t") : toSec(ordersData.WITNESSES[0].t));
  const kvFlag = params.get("kv");
  if (["unmatched", "stale", "offline"].includes(kvFlag)) ordersState.kvarm.script = kvFlag;
  if (kvFlag === "offline") ordersState.kvarm.connection = "offline";
  ordersPill();
  ordersFocusMachine(params.get("sel") || "retry"); ordersRenderDock();
  $("#orders-pin-not").onclick = () => { if (ordersActive()) ordersPin(false); };
  $("#orders-pin-keep").onclick = () => { if (ordersActive()) ordersPin(true); };
  $("#orders-post").onclick = () => { if (ordersActive()) ordersOrder(); };
  $("#orders-plate").onclick = () => { if (ordersActive()) ordersOpen("order", ordersData.ORDER.on); };
  $("#orders-order-edge").onclick = $("#orders-plate").onclick;
  $("#orders-question-pin").onclick = $("#orders-question-edge").onclick = () => { if (ordersActive()) ordersQuestion(); };
  for (const b of document.querySelectorAll("[data-orders-go]")) b.onclick = () => {
    if (!ordersActive()) return;
    const w = ordersData.WITNESSES.find((x) => x.id === b.dataset.ordersGo);
    if (ordersState.open) ordersClose();
    selected = null;
    setMode("replay"); rpSeek(toSec(w.t)); ordersFocusMachine(w.entity);
  };
  $("#orders-card").onclick = (ev) => {
    if (!ordersActive()) return;
    if (ev.target.closest("[data-orders-close]")) ordersClose();
    if (ev.target.closest("#orders-keep")) ordersKeep();
    if (ev.target.closest("#orders-submit")) ordersPost();
    const answer = ev.target.closest("[data-orders-answer]");
    if (answer) ordersAnswer(answer.dataset.ordersAnswer);
    const provBtn = ev.target.closest("[data-orders-prov]");
    if (provBtn) ordersProvOpen(provBtn.dataset.ordersProv, provBtn);
  };
  $("#orders-kv-panel").onclick = (ev) => {
    if (!ordersActive()) return;
    if (ev.target.closest("[data-orders-panel-close]")) ordersPanelToggle(false);
    const act = ev.target.closest("[data-orders-kv]");
    if (act) act.dataset.ordersKv === "reconnect" ? ordersReconnect() : ordersSendHeld();
  };
  $("#orders-prov").onclick = (ev) => { if (ev.target.closest("[data-orders-prov-close]")) ordersProvClose(); };
  const ordersEgress = $(".egress");
  if (ordersEgress) {
    ordersEgress.addEventListener("click", () => { if (ordersActive()) ordersPanelToggle(); });
    ordersEgress.addEventListener("keydown", (ev) => { if (ordersActive() && (ev.key === "Enter" || ev.key === " ")) { ev.preventDefault(); ordersPanelToggle(); } });
  }
  document.addEventListener("pointerover", (ev) => {
    if (!ordersActive() || !document.body.classList.contains("alt")) return;
    const t = ev.target.closest("[data-orders-binding]");
    if (t && !t.closest("#orders-prov")) ordersProvOpen(t.dataset.ordersBinding, t);
  });
  addEventListener("keyup", (ev) => { if (ev.key === "Alt") ordersProvClose(); });
  addEventListener("blur", () => ordersProvClose());
  $("#chips").addEventListener("pointerdown", (ev) => { if (ordersActive() && ev.target.closest("[data-orders-witness]")) ev.stopPropagation(); });
  $("#chips").addEventListener("pointerup", (ev) => { if (ordersActive() && ev.target.closest("[data-orders-witness]")) ev.stopPropagation(); });
  $("#chips").addEventListener("click", (ev) => {
    if (!ordersActive()) return;
    const pin = ev.target.closest("[data-orders-witness]");
    if (!pin) return;
    const w = ordersState.witnesses.find((x) => x.id === pin.dataset.ordersWitness);
    ordersOpen(w.id, w.entity);
  });
  addEventListener("keydown", (ev) => {
    if (!ordersActive()) return;
    const open = ordersState.open;
    const handle = () => { ev.preventDefault(); ev.stopImmediatePropagation(); };
    if (typing(ev)) {
      if (!ev.target.closest("#orders-card")) return;
      if (ev.key === "Escape") { handle(); ev.target.blur(); return; }
      if (ev.key === "Enter" && !ev.shiftKey) { handle(); if (open === "order") ordersPost(); else if (open && open.startsWith("w")) ordersKeep(); }
      return;
    }
    if (!bare(ev)) return;
    if (ev.key === "Escape") {
      if (ordersProvKey) { handle(); ordersProvClose(); return; }
      if (ordersState.kvarm.panel) { handle(); ordersPanelToggle(false); return; }
      if (open) { handle(); ordersClose(); return; }
      return;
    }
    if (open === "question" && ev.shiftKey && ["Digit1", "Digit2"].includes(ev.code)) { handle(); ordersAnswer(ev.code.slice(-1)); return; }
    if (ev.key === "Enter" && open) {
      if (ev.target.closest("#orders-card button, #orders-kv-panel button, #orders-prov button")) return;
      handle(); if (open.startsWith("w")) ordersKeep(); else if (open === "order") ordersPost(); return;
    }
    const key = ev.key.toLowerCase();
    if (key === "u") { handle(); ordersPanelToggle(); return; }
    if (key === "m") { handle(); ordersPin(ev.shiftKey); }
    if (key === "p") { handle(); ordersOrder(); }
    if (key === "i") { handle(); ordersQuestion(); }
  }, { capture: true });
  const frame = (now) => {
    const delta = ordersLastFrame === null ? 0 : Math.min(100, now - ordersLastFrame);
    ordersLastFrame = now;
    if (ordersActive()) {
      ordersKvTick(delta);
      ordersGuide(); ordersPosition();
      if (document.body.dataset.mode === "live" && ordersState.order.state === "open" && !document.hidden) {
        ordersLiveMs += delta;
        for (const r of ordersData.READBACKS) if (ordersLiveMs >= r.arriveMs) ordersArrive(r);
        if (ordersState.scoring === "rescoring") {
          if (ordersEvidenceVisible()) ordersScoreMs += delta;
          if (ordersScoreMs >= 600) {
            ordersState.scoring = "settled"; ordersVisibleMs = 0; ordersUpdateReadbacks();
            ordersAnnounce("Readbacks re-scored. " + ordersAgent(ordersData.READBACKS.find((r) => r.answers.q1 === ordersState.question.answered)) + " matches every locked check.");
          }
        } else if (ordersState.scoring === "settled") {
          if (ordersEvidenceVisible()) ordersVisibleMs += delta;
          else ordersVisibleMs = 0;
          ordersClaim();
        }
      } else ordersVisibleMs = 0;
    } else ordersVisibleMs = 0;
    requestAnimationFrame(frame);
  };
  requestAnimationFrame(frame);
}
`,
};
