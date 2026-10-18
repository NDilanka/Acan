import { writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { ISO_CSS } from "/home/factory-user/.factory/skills/anatomy/kit/iso-kit.mjs";
import { runtimeScript, STATE_CSS, FOG_DEFS } from "./lib.mjs";
import { ACQUIT, INSPECT } from "./acquit.mjs";
import { BP_HTML, BP_CSS, BP_SCRIPT } from "./blueprint.mjs";
import { floorHtml, FLOOR_CSS, FLOOR_SCRIPT } from "./floor.mjs";
import { CH_HTML, CH_CSS, CH_SCRIPT } from "./choices.mjs";
import { SIM_HTML, SIM_CSS, SIM_SCRIPT } from "./sim.mjs";
import { RP_HTML, RP_CSS, RP_SCRIPT } from "./replay.mjs";
import { ACT_SCRIPT } from "./actions.mjs";
import { PAL_HTML, PAL_CSS, PAL_SCRIPT } from "./palette.mjs";
import { FR_HTML, FR_CSS, FR_SCRIPT } from "./first.mjs";
import { SH_HTML, SH_CSS, SH_SCRIPT } from "./share.mjs";
import { FOG_CSS, FOG_SCRIPT } from "./fog.mjs";
import { CONCEPTS } from "./concepts/index.mjs";

const ONLY = process.argv[2];
const LIST = ONLY ? CONCEPTS.filter((c) => c.id === ONLY) : CONCEPTS;
if (ONLY && !LIST.length) throw new Error("unknown concept " + ONLY + ", expected one of " + CONCEPTS.map((c) => c.id).join(", "));
const OUT = process.env.OUT || (ONLY ? "live-" + ONLY + ".html" : "live.html");
const CBAR = '<nav class="cbar" id="cbar">' + LIST.map((c) => '<a href="#c=' + c.id + '" data-c="' + c.id + '">' + c.label + "</a>").join("") + "</nav>";
const HINTS = Object.fromEntries(LIST.map((c) => [c.id, c.hint || ""]));
const CBAR_CSS = ".cbar{position:fixed;top:92px;left:50%;transform:translateX(-50%);display:flex;gap:2px;padding:3px;border-radius:9px;background:var(--glass);border:1px solid var(--line);z-index:40;font:500 11px var(--sans)}.cbar a{color:var(--dim);text-decoration:none;padding:4px 10px;border-radius:6px}.cbar a:hover{color:var(--text)}body[data-concept] .cbar a[aria-current]{color:#111;background:#c9a45c}body[data-concept] .toast,body[data-concept] .toast.on{bottom:auto;transform:none;transition:none;max-width:440px;z-index:60;line-height:1.45}";
// Concept pages: a toast describes only the latest action, so any key or press clears it before handlers run.
// Registered in <head> so it runs ahead of every other capture listener.
const TOAST_DISMISS = '<script>for (const t of ["keydown", "pointerdown"]) addEventListener(t, (e) => { if (!document.body || !document.body.dataset.concept || (t === "keydown" && ["Shift", "Alt", "Control", "Meta"].includes(e.key))) return; const el = document.getElementById("toast"); if (el) el.classList.remove("on"); }, true);</script>';

const HERE = dirname(fileURLToPath(import.meta.url));
const W = 1600, H = 900;

const LOGO = `<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.1"><circle cx="12" cy="12" r="10.5" opacity=".45"/><circle cx="12" cy="12" r="6.5" opacity=".7"/><circle cx="12" cy="12" r="2.2" fill="currentColor"/><circle cx="18.5" cy="12" r="1.4" fill="currentColor"/><circle cx="4.6" cy="7.6" r="1" fill="currentColor"/></svg>`;
const DRONE_ICON = `<svg viewBox="0 0 24 16" width="22" height="15"><circle cx="5" cy="9" r="3.2" fill="#1a1a1a" stroke="currentColor" stroke-width=".8"/><circle cx="19" cy="9" r="3.2" fill="#1a1a1a" stroke="currentColor" stroke-width=".8"/><circle cx="12" cy="8" r="6" fill="#2c2c2c" stroke="currentColor" stroke-width=".9"/><circle cx="12" cy="8.6" r="2.6" fill="#080808" stroke="currentColor" stroke-width=".6"/><circle class="eye" cx="12" cy="8.6" r="1" fill="#fff"/></svg>`;

const css = `
${ISO_CSS}
${STATE_CSS}
:root{--glass:rgba(18,18,18,.7);--glass-hi:rgba(26,26,26,.82);--line:rgba(255,255,255,.08);--line2:rgba(255,255,255,.13);--text:#d6d6d6;--dim:#7c7c7c;--faint:#4a4a4a;--wait:#e1a43c;
--sans:"Inter","IBM Plex Sans",system-ui,-apple-system,"Segoe UI",sans-serif;--mono:"IBM Plex Mono",ui-monospace,"SF Mono",Menlo,Consolas,monospace;--ease:cubic-bezier(.32,.72,0,1);
--shadow:0 1px 0 rgba(255,255,255,.04) inset,0 12px 32px rgba(0,0,0,.45),0 2px 6px rgba(0,0,0,.3)}
*{box-sizing:border-box}
html,body{margin:0;height:100%;background:#0a0a0a;color:var(--text);font:13px/1.4 var(--sans);-webkit-font-smoothing:antialiased;overflow:hidden}
button{font:inherit;color:inherit;background:none;border:0;cursor:pointer;padding:0}
kbd{font:10px var(--mono);color:var(--faint);border:1px solid var(--line2);border-radius:4px;padding:0 4px}
.dot{width:6px;height:6px;border-radius:50%;background:var(--ok);flex:none}

.stage{position:fixed;inset:0;overflow:hidden;cursor:grab;background:var(--paper);transition:background .6s;touch-action:none}
.stage.drag{cursor:grabbing}.stage.orbiting{cursor:ew-resize}
.grid{position:absolute;inset:0;width:100%;height:100%;pointer-events:none;-webkit-mask-image:radial-gradient(70% 70% at 50% 50%,#000 30%,transparent 100%);mask-image:radial-gradient(70% 70% at 50% 50%,#000 30%,transparent 100%)}
.grid .minor{stroke:var(--grid-minor)}.grid .major{stroke:var(--grid-major)}.grid .axis{stroke:var(--grid-axis)}
.world{position:absolute;left:0;top:0;width:${W}px;height:${H}px;transform-origin:0 0;will-change:transform}
.world .iso{background:transparent}
.world svg{width:${W}px;height:${H}px}
.chips{position:absolute;inset:0;pointer-events:none}
.chip{position:absolute;transform:translate(-50%,calc(-100% - 20px - var(--lift,0px)));display:flex;align-items:center;gap:6px;padding:3px 8px 3px 3px;border-radius:7px;background:rgba(14,14,14,.72);backdrop-filter:blur(8px);-webkit-backdrop-filter:blur(8px);border:1px solid var(--line2);white-space:nowrap;font-size:10.5px;letter-spacing:.09em;text-transform:uppercase;color:#d8d8d8;font-weight:600;opacity:0;translate:0 4px;transition:opacity .2s,translate .25s var(--ease),border-color .2s;transform-origin:50% 100%}
.chip::before{content:"";position:absolute;left:50%;top:100%;width:1px;height:calc(18px + var(--lift,0px));background:linear-gradient(rgba(255,255,255,.3),rgba(255,255,255,.06))}
.chip .mono{display:grid;place-items:center;min-width:19px;height:19px;padding:0 3px;border-radius:5px;background:#e8e8e8;color:#0a0a0a;font:700 9.5px var(--mono);letter-spacing:0}
.chip .rd{display:none;font:10px var(--mono);color:#8a8a8a;letter-spacing:0;text-transform:none;font-weight:400}
.chip.show,.chip.hover,.chip.sel,body.alt .chip{opacity:1;translate:0 0}
body.alt .chip .rd{display:inline}
.chip[data-state=strain] .mono,.chip[data-state=wait] .mono{background:var(--strain)}
.chip[data-state=fail]{border-color:rgba(232,96,74,.55)}
.chip[data-state=fail] .mono{background:var(--fail)}
.chip[data-state=ghost]{border-style:dashed;border-color:rgba(122,184,230,.5);color:#bcd9ef}
.chip[data-state=ghost] .mono{background:transparent;border:1px dashed var(--ghost);color:var(--ghost)}
.chip[data-state=run] .mono{background:#cfe6fb}
.chip.sel{border-color:rgba(255,255,255,.4)}
body.no-labels .chip,.chip.exp{opacity:0!important}
.has-sel .ent:not(.sel){opacity:.35}
.ent{transition:opacity .3s}
.ent.hover .iso-edge{stroke:var(--anatomy-hi)}
.ent.sel .iso-edge{stroke:var(--anatomy-lit)}
.has-sel .belt:not(.lit){opacity:.3}
.belt.lit .iso-line[data-flow]{stroke:var(--anatomy-lit)}
body:not(.traffic) .belt .iso-line[data-flow]{animation:none;stroke-dasharray:none;opacity:.3}
body:not(.agents) [data-kind=bot],body:not(.agents) [data-kind=scaffold]{display:none}
body:not(.ghosts) .ent[data-state=ghost]{display:none}
body.still .iso-line[data-flow],body.still .smoke circle,body.still .lamp,body.still .pulse,body.still .drone,body.still .drone-shadow .iso-halo{animation-play-state:paused}
.callouts{position:absolute;inset:0;pointer-events:none}
.xl{position:absolute;transform:translate(0,-50%);font:10.5px var(--mono);color:#e0e0e0;white-space:nowrap;padding:2px 6px;border-radius:5px;background:rgba(10,10,10,.6)}
.leaders{position:absolute;left:0;top:0;overflow:visible}
.leaders path{fill:none;stroke:rgba(255,255,255,.35);stroke-width:1}
.leaders circle{fill:#e6e6e6}

.fl{position:fixed;z-index:6;display:flex;align-items:center;gap:4px;padding:4px;border-radius:12px;background:var(--glass);backdrop-filter:blur(16px) saturate(1.2);-webkit-backdrop-filter:blur(16px) saturate(1.2);border:1px solid var(--line);box-shadow:var(--shadow)}
.fl .b{display:flex;align-items:center;gap:7px;height:30px;padding:0 10px;border-radius:8px;color:var(--dim);font-size:12px;white-space:nowrap;transition:background .15s,color .15s}
.fl .b:hover{background:rgba(255,255,255,.06);color:var(--text)}
.fl .sep{width:1px;height:18px;background:var(--line2);margin:0 2px}
.tl{left:16px;top:16px}
.brand{color:#ececec!important;font-weight:600}
.tc{left:50%;top:16px;transform:translateX(-50%)}
.modes{position:relative;display:flex}
.modes button{position:relative;z-index:1;height:30px;padding:0 14px;border-radius:8px;color:var(--dim);font-size:12px;display:flex;gap:8px;align-items:center;transition:color .2s}
.modes button[aria-pressed=true]{color:#f2f2f2}
.modes kbd{border:0;padding:0}
.modes .pill{position:absolute;z-index:0;top:0;bottom:0;border-radius:8px;background:rgba(255,255,255,.09);box-shadow:inset 0 1px 0 rgba(255,255,255,.06);transition:left .35s var(--ease),width .35s var(--ease)}
.tr{right:16px;top:16px}
.egress{display:flex;align-items:center;gap:6px;height:24px;padding:0 9px;border-radius:999px;border:1px solid var(--line);font:11px var(--mono);color:var(--dim);margin:0 4px}
.bl{left:16px;bottom:16px;flex-direction:column;align-items:stretch;gap:2px;padding:6px;min-width:264px}
.wait{display:flex;align-items:center;gap:8px;height:34px;padding:0 10px;border-radius:8px;background:rgba(225,164,60,.09);color:#f1d5a2;font-size:12px}
.wait b{margin-left:auto;font:600 12px var(--mono);background:rgba(225,164,60,.18);border-radius:5px;padding:1px 6px}
.agent{display:flex;align-items:center;gap:8px;height:30px;padding:0 8px;border-radius:8px;color:var(--dim);font-size:12px}
.agent:hover{background:rgba(255,255,255,.05);color:var(--text)}
.agent svg{color:#bbb}.agent.off svg{color:#555;opacity:.7}.agent.off .eye{fill:#555}
.agent strong{color:var(--text);font-weight:500}
.agent span{margin-left:auto;font:10.5px var(--mono);color:var(--faint)}
.br{right:16px;bottom:16px;gap:10px;padding:4px 12px 4px 4px}
.zoom{display:flex;align-items:center;gap:2px}
.zoom button{width:28px;height:28px;border-radius:7px;color:var(--dim);font-size:14px}
.zoom button:hover{background:rgba(255,255,255,.06);color:#fff}
.zoom span{font:10.5px var(--mono);color:var(--faint);min-width:38px;text-align:center}
.readout{font:11px var(--mono);color:var(--dim)}
.live{display:flex;align-items:center;gap:7px;font:11px var(--mono);color:var(--dim)}
.live b{color:#e8e8e8;letter-spacing:.12em;font-weight:500}
.motion{font:10.5px var(--mono);color:var(--faint)}
.hint{position:fixed;left:50%;top:68px;white-space:nowrap;transform:translateX(-50%);font:10.5px var(--mono);letter-spacing:.06em;color:var(--dim);text-transform:uppercase;transition:opacity .8s;pointer-events:none;z-index:5}
body.used .hint,body.has-insp .hint{opacity:0}
.watermark{position:fixed;left:50%;top:64px;transform:translateX(-50%);font:10.5px var(--mono);letter-spacing:.2em;color:var(--dim);opacity:0;transition:opacity .4s;pointer-events:none}
body:not([data-mode=live]) .watermark{opacity:1}

.nav{position:fixed;right:16px;top:70px;display:flex;flex-direction:column;align-items:center;gap:8px;z-index:5}
.has-insp .nav,.has-insp .br{right:420px}
.compass{width:74px;height:74px;border-radius:50%;background:var(--glass);backdrop-filter:blur(16px);border:1px solid var(--line);position:relative;cursor:ew-resize;box-shadow:var(--shadow)}
.compass svg{position:absolute;inset:0}
.compass text{font:9px var(--mono);fill:var(--dim);text-anchor:middle;dominant-baseline:central}
.compass text.n{fill:#f2f2f2}
.compass .read{position:absolute;inset:0;display:grid;place-items:center;font:10px var(--mono);color:#e6e6e6;pointer-events:none}
.navrow{display:flex;gap:2px;padding:3px;border-radius:10px;background:var(--glass);backdrop-filter:blur(16px);border:1px solid var(--line);box-shadow:var(--shadow)}
.navrow button{min-width:26px;height:24px;padding:0 6px;border-radius:7px;color:var(--dim);font:11px var(--mono)}
.navrow button:hover{background:rgba(255,255,255,.06);color:#fff}
.navrow button[aria-pressed=true]{background:rgba(255,255,255,.1);color:#fff}
.tilt{font:10px var(--mono);color:var(--faint)}

.pop{position:fixed;top:62px;right:16px;width:250px;padding:8px;border-radius:12px;background:var(--glass-hi);backdrop-filter:blur(18px);border:1px solid var(--line2);box-shadow:var(--shadow);opacity:0;transform:translateY(-4px) scale(.98);transform-origin:top right;pointer-events:none;z-index:10}
.pop.open{opacity:1;transform:none;pointer-events:auto}
.pop h4{margin:4px 6px 6px;font:10px var(--mono);letter-spacing:.12em;color:var(--faint);text-transform:uppercase;font-weight:400}
.pop label{display:flex;align-items:center;justify-content:space-between;padding:6px;border-radius:7px;cursor:pointer;color:var(--text)}
.pop label:hover{background:rgba(255,255,255,.04)}
.pop label.off{color:var(--faint);cursor:default}
.pop label small{font:10px var(--mono);color:var(--faint)}
.sw{width:26px;height:15px;border-radius:999px;background:#2a2a2a;position:relative;transition:background .2s;flex:none}
.sw::after{content:"";position:absolute;left:2px;top:2px;width:11px;height:11px;border-radius:50%;background:#bbb;transition:transform .25s var(--ease)}
input:checked+.sw{background:#3d3d3d}input:checked+.sw::after{transform:translateX(11px);background:#fff}
.pop input{display:none}

.insp{position:fixed;top:62px;right:16px;bottom:16px;width:392px;border-radius:16px;background:var(--glass-hi);backdrop-filter:blur(20px) saturate(1.2);-webkit-backdrop-filter:blur(20px) saturate(1.2);border:1px solid var(--line2);box-shadow:var(--shadow);transform:translateX(calc(100% + 80px));z-index:7;display:flex;flex-direction:column;overflow:hidden}
.insp.open{transform:none}
.ih{display:flex;align-items:center;gap:10px;padding:14px 14px 10px}
.ih .mono{display:grid;place-items:center;min-width:30px;height:30px;padding:0 5px;border-radius:8px;background:#e8e8e8;color:#0a0a0a;font:700 12px var(--mono)}
.ih[data-state=run] .mono{background:#cfe6fb}.ih[data-state=fail] .mono{background:var(--fail)}.ih[data-state=strain] .mono{background:var(--strain)}.ih[data-state=ghost] .mono{background:transparent;color:var(--ghost);border:1px dashed var(--ghost)}
.ih h3{margin:0;font-size:15px;font-weight:600;color:#f0f0f0}
.ih .kind{font:10.5px var(--mono);color:var(--dim)}
.ih .x{margin-left:auto;width:28px;height:28px;border-radius:8px;color:var(--dim)}
.ih .x:hover{background:rgba(255,255,255,.06);color:#fff}
.ibody>*{flex:none}
.ibody{overflow:auto;padding:0 14px 14px;display:flex;flex-direction:column;gap:14px;scrollbar-width:thin;scrollbar-color:#333 transparent}
.fig{position:relative;height:210px;border-radius:12px;background:radial-gradient(110% 90% at 50% 40%,#181818,#0d0d0d);border:1px solid var(--line);overflow:hidden}
.fig .iso{height:100%;background:transparent}.fig svg{width:100%;height:100%;display:block}
.fig .tools{position:absolute;left:8px;bottom:8px;display:flex;gap:4px}
.fig .tools button{height:24px;padding:0 8px;border-radius:7px;background:rgba(0,0,0,.45);border:1px solid var(--line);font:10.5px var(--mono);color:var(--dim)}
.fig .tools button:hover,.fig .tools button[aria-pressed=true]{color:#fff;border-color:var(--line2)}
.fig .read{position:absolute;right:10px;top:8px;font:15px var(--mono);color:#f2f2f2;font-variant-numeric:tabular-nums}
.fig .read small{display:block;font:10px var(--mono);color:var(--dim);text-align:right}
.sec h5{margin:0 0 8px;display:flex;align-items:center;gap:8px;font:500 10px var(--mono);letter-spacing:.14em;text-transform:uppercase;color:var(--dim)}
.sec h5 i{font-style:normal;margin-left:auto;letter-spacing:.02em;text-transform:none;color:var(--faint)}
.dim{display:block;width:100%;height:56px;overflow:visible}
.dim text{font:10px var(--mono);fill:#bdbdbd}
.dim .t{fill:var(--dim)}
.dim .rail{stroke:#3a3a3a}.dim .tick{stroke:#4a4a4a}
.dim .val{stroke:#e8e8e8;stroke-width:1.4}.dim .over{stroke:var(--fail);stroke-width:1.4}
.dim .ext{stroke:#8a8a8a;stroke-dasharray:2 2}
.dim .arr{fill:#e8e8e8}.dim .arr.over{fill:var(--fail);stroke:none}
.lock{font:10px var(--mono);color:var(--faint);margin-top:8px}
.src{display:grid;grid-template-columns:70px 10px 1fr;gap:2px 8px;align-items:baseline;padding:6px 0;border-top:1px solid var(--line)}
.src:first-of-type{border-top:0}
.src .n{font:10.5px var(--mono);color:var(--dim);text-transform:uppercase;letter-spacing:.08em}
.src .ok{color:var(--ok)}.src .no{color:var(--faint)}
.src b{font-weight:500;font-size:12px;color:#e2e2e2;overflow-wrap:anywhere}.src .d{grid-column:3;font:10.5px var(--mono);color:var(--dim)}
.match{display:flex;align-items:center;gap:8px;margin-top:6px;font:10.5px var(--mono);color:var(--dim)}
.bar{flex:1;height:4px;border-radius:2px;background:#262626;overflow:hidden}.bar i{display:block;height:100%;background:#d6d6d6;border-radius:2px}
.drift{display:flex;gap:8px;padding:7px 0;border-top:1px solid var(--line);font-size:12px;color:#cfcfcf}
.drift:first-of-type{border-top:0}
.drift .sv{flex:none;font:9.5px var(--mono);text-transform:uppercase;letter-spacing:.08em;padding:1px 5px;border-radius:4px;height:fit-content;border:1px solid var(--line2);color:var(--dim)}
.drift .sv[data-s=fail]{color:var(--fail);border-color:rgba(232,96,74,.4)}.drift .sv[data-s=drift]{color:var(--strain);border-color:rgba(225,164,60,.35)}.drift .sv[data-s=fog]{color:#9a9a9a;border-style:dashed}
.drift small{display:block;font:10px var(--mono);color:var(--faint);margin-top:2px}
.fog{display:grid;grid-template-columns:repeat(6,1fr);gap:4px}
.fog div{display:flex;flex-direction:column;gap:4px;font:9.5px var(--mono);color:var(--dim)}
.fog div i{height:6px;border-radius:2px;background:#e0e0e0}
.fog div[data-v="0.5"] i{background:repeating-linear-gradient(90deg,#8a8a8a 0 2px,transparent 2px 4px)}
.fog div[data-v="0"] i{background:#262626}
.cal{display:flex;align-items:center;gap:12px}
.cal b{font:22px var(--mono);font-weight:400;color:#f0f0f0}
.cal svg{flex:1;height:30px}
.cal path{fill:none;stroke:#d6d6d6;stroke-width:1.2}
.choice{display:grid;grid-template-columns:1fr auto;gap:2px 10px;padding:7px 0;border-top:1px solid var(--line);font-size:12px;color:#d8d8d8}
.choice:first-of-type{border-top:0}
.choice small{font:10px var(--mono);color:var(--faint)}
.choice .risk{grid-row:span 2;align-self:center;font:11px var(--mono);color:var(--dim)}
.acts{display:flex;gap:6px}
.acts button{flex:1;height:30px;border-radius:8px;border:1px solid var(--line2);color:var(--text);font-size:12px;background:rgba(255,255,255,.03)}
.acts button:hover{background:rgba(255,255,255,.07)}
.note{padding:10px;border:1px dashed var(--line2);border-radius:10px;color:var(--dim);font-size:12px}
`;

const papers = `
body[data-mode=live]{--paper:radial-gradient(120% 90% at 50% 45%,#141414,#0a0a0a 70%);--grid-minor:rgba(255,255,255,.035);--grid-major:rgba(255,255,255,.075);--grid-axis:rgba(255,255,255,.12)}
body[data-mode=blueprint]{--paper:radial-gradient(120% 90% at 50% 45%,#0f2136,#07111d 70%);--grid-minor:rgba(140,190,240,.06);--grid-major:rgba(140,190,240,.13);--grid-axis:rgba(140,190,240,.22)}
body[data-mode=blueprint] .iso{--anatomy-paper:#0d1a2a;--anatomy-face:#11223a;--anatomy-top:#13263f;--anatomy-shade-0:#0a1524;--anatomy-shade-1:#0c182a;--anatomy-shade-2:#0e1c30;--anatomy-shade-3:#102035;--anatomy-hi:#cfe0f2;--anatomy-mid:#5f7a96;--anatomy-lo:#344c66;--anatomy-faint:#203449;--anatomy-dot:#cfe0f2;--anatomy-halo:#030a12}
body[data-mode=simulate]{--paper:radial-gradient(120% 90% at 50% 45%,#141b17,#090d0b 70%);--grid-minor:rgba(170,230,190,.04);--grid-major:rgba(170,230,190,.09);--grid-axis:rgba(170,230,190,.15)}
body[data-mode=simulate] .iso{--anatomy-paper:#111714;--anatomy-face:#17201b;--anatomy-top:#18221c;--anatomy-shade-0:#0d120f;--anatomy-shade-1:#0f1511;--anatomy-shade-2:#121914;--anatomy-shade-3:#141c17;--anatomy-hi:#cfe0d4;--anatomy-mid:#62786a;--anatomy-lo:#36463c;--anatomy-faint:#233029}
body[data-mode=replay]{--paper:radial-gradient(120% 90% at 50% 45%,#1b160f,#0d0a07 70%);--grid-minor:rgba(240,210,160,.04);--grid-major:rgba(240,210,160,.09);--grid-axis:rgba(240,210,160,.15)}
body[data-mode=replay] .iso{--anatomy-paper:#16120d;--anatomy-face:#1e1912;--anatomy-top:#201a13;--anatomy-shade-0:#120e0a;--anatomy-shade-1:#14100b;--anatomy-shade-2:#17130e;--anatomy-shade-3:#1a1510;--anatomy-hi:#e3d3b8;--anatomy-mid:#7f6f58;--anatomy-lo:#4a3f31;--anatomy-faint:#2e271e}
`;

const chips = ACQUIT.entities.filter((e) => e.kind !== "scaffold").map((e) => `<div class="chip" data-id="${e.id}" data-state="${e.state}"><span class="mono">${e.mono}</span>${e.name}<span class="rd">${e.reading}</span></div>`).join("");

const rose = `<svg viewBox="-37 -37 74 74" width="74" height="74"><g class="rose">${Array.from({ length: 72 }, (_, i) => { const a = (i / 72) * Math.PI * 2, r0 = i % 18 === 0 ? 26 : i % 6 === 0 ? 29 : 31; return `<line x1="${(Math.sin(a) * r0).toFixed(2)}" y1="${(-Math.cos(a) * r0).toFixed(2)}" x2="${(Math.sin(a) * 33).toFixed(2)}" y2="${(-Math.cos(a) * 33).toFixed(2)}" stroke="${i === 0 ? "#f2f2f2" : "rgba(255,255,255,.22)"}" stroke-width="${i % 18 === 0 ? 1.2 : 0.7}"/>`; }).join("")}</g>${["N", "E", "S", "W"].map((t, i) => `<text class="${i ? "" : "n"}" data-a="${i * 90}">${t}</text>`).join("")}</svg>`;

const body = `
<main class="stage" id="stage">
  <svg class="grid" id="grid"></svg>
  <div class="world" id="world"><div class="iso" id="plant"></div><div class="chips" id="chips">${chips}</div></div>
  <div class="callouts" id="callouts"></div>
</main>
<div class="watermark" id="watermark"></div>
<div class="fl tl"><button class="b brand">${LOGO}Acan</button><span class="sep"></span><button class="b">Acquit <span style="opacity:.5">▾</span></button><button class="b" data-do="palette">Go to <kbd>Ctrl K</kbd></button></div>
<div class="fl tc"><div class="modes" id="modes"><span class="pill"></span>
  <button data-mode="live" aria-pressed="true">Live <kbd>1</kbd></button><button data-mode="blueprint">Blueprint <kbd>2</kbd></button><button data-mode="simulate">Simulate <kbd>3</kbd></button><button data-mode="replay">Replay <kbd>4</kbd></button></div></div>
<div class="fl tr"><button class="b" id="layersBtn">Layers <kbd>L</kbd></button><button class="b" id="exportBtn">Export</button><span class="egress" title="Outbound connections since launch"><span class="dot"></span>local · 0 out</span></div>
<div class="pop" id="layers">
  <h4>Layers</h4>
  <label>Labels<input type="checkbox" data-layer="labels" checked><span class="sw"></span></label>
  <label>All labels <small>hold Alt</small><input type="checkbox" data-layer="alt"><span class="sw"></span></label>
  <label>Traffic<input type="checkbox" data-layer="traffic" checked><span class="sw"></span></label>
  <label>Ghosts<input type="checkbox" data-layer="ghosts" checked><span class="sw"></span></label>
  <label>Agents<input type="checkbox" data-layer="agents" checked><span class="sw"></span></label>
  <label>Territories<input type="checkbox" data-layer="territories" checked><span class="sw"></span></label>
  <label>Choices <small>pins</small><input type="checkbox" data-layer="choices" checked><span class="sw"></span></label>
  <label>Fog <small>what Acan doesn't know</small><input type="checkbox" data-layer="fogged"><span class="sw"></span></label>
</div>
<div class="nav" id="nav">
  <div class="compass" id="compass" title="Drag to orbit. Double-click to reset.">${rose}<div class="read" id="azRead">35°</div></div>
  <div class="navrow"><button id="rotL" title="Rotate left (Q)">⟲</button><button id="top" title="Top view (T)">Top</button><button id="rotR" title="Rotate right (E)">⟳</button></div>
  <div class="tilt" id="tiltRead">tilt 30°</div>
</div>
<div class="fl bl">
  <button class="wait"><span class="dot pulse" style="background:var(--wait)"></span>Waiting on you<b>7</b></button>
  <button class="agent" data-session="droid-radar">${DRONE_ICON}<strong>Droid</strong> verifying otel radar<span>38m</span></button>
  <button class="agent blocked" data-session="codex-paypal">${DRONE_ICON}<strong>Codex</strong> blocked on you<span>9m</span></button>
  <button class="agent off">${DRONE_ICON}<strong>Claude Code</strong> idle<span></span></button>
  <button class="tl" id="tlBtn" aria-pressed="false">Floor timeline <kbd>G</kbd></button>
</div>
<div class="fl br">
  <div class="zoom"><button id="zout">−</button><span id="zpct">100%</span><button id="zin">+</button><button id="zfit" title="Fit (F)">⤢</button></div>
  <span class="sep"></span>
  <span class="readout">15 rps · p99 184 ms · 0.6% err</span>
  <span class="sep"></span>
  <span class="live"><span class="dot pulse"></span><b>LIVE</b><span id="time">14:22:07</span></span>
  <span class="motion" id="motion">still in 2:00</span>
</div>
<div class="hint">Ctrl K go to · Tab next issue · drag pan · right-drag orbit · scroll zoom · Alt labels</div>
<aside class="insp" id="insp">
  <div class="ih" id="ih"><span class="mono" id="iMono">JS</span><div><h3 id="iName">acquit-api</h3><div class="kind" id="iKind">Assembler</div></div><button class="x" id="close" title="Close (Esc)">✕</button></div>
  <div class="ibody" id="ibody"></div>
</aside>`;

const script = `${runtimeScript()}
const SCENE = ${JSON.stringify(ACQUIT)};
const INSPECT = ${JSON.stringify(INSPECT)};
const ENT = Object.fromEntries(SCENE.entities.map((e) => [e.id, e]));
const W = ${W}, H = ${H};
const SHOW = new Set(["fail", "strain", "wait", "ghost", "run"]);
const $ = (s) => document.querySelector(s);
const typing = (ev) => { const t = ev.target; return !!t && (t.isContentEditable || t.tagName === "TEXTAREA" || (t.tagName === "INPUT" && !["range", "checkbox", "radio", "button"].includes(t.type))); };
const bare = (ev) => !typing(ev) && !ev.ctrlKey && !ev.metaKey && !ev.altKey;
const stage = $("#stage"), world = $("#world"), plant = $("#plant"), grid = $("#grid");
const view = { x: 0, y: 0, s: 1 }, goal = { x: 0, y: 0, s: 1 };
const cam = { az: 35, el: 30 }, camGoal = { az: 35, el: 30 };
const SCALE = projectionFor(k, SCENE, ASSETS, { width: W, height: H, azimuth: 35, elevation: 30, pad: 90 }).scale;
let pivot = projectionFor(k, SCENE, ASSETS, { width: W, height: H, azimuth: 35, elevation: 30, scale: SCALE }).pivot;
let P, anchors = {}, labels = [], selected = null, hovered = null, raf = 0, dirtyScene = true;
const explode = { id: null, t: 0, goal: 0 };

function project() { P = projectionFor(k, SCENE, ASSETS, { width: W, height: H, azimuth: cam.az, elevation: cam.el, scale: SCALE, pivot }); }
function renderPlant() {
  project();
  for (const e of SCENE.entities) e.explode = e.id === explode.id ? explode.t : 0;
  const out = renderScene(k, ASSETS, SCENE, P, { width: W, height: H, label: "The Acquit plant" });
  plant.innerHTML = out.svg; anchors = out.anchors; labels = out.labels;
  for (const chip of document.querySelectorAll(".chip")) { const [x, y] = anchors[chip.dataset.id]; chip.style.left = x + "px"; chip.style.top = y + "px"; chip.classList.toggle("show", SHOW.has(chip.dataset.state) && ASSETS[ENT[chip.dataset.id].kind].family !== "fitting"); chip.classList.toggle("exp", chip.dataset.id === explode.id && explode.t > 0.2); }
  paintSelection();
  const rot = -cam.az + 35;
  $("#azRead").textContent = Math.round(((cam.az % 360) + 360) % 360) + "°";
  $(".compass .rose").style.transform = "rotate(" + rot + "deg)";
  for (const t of document.querySelectorAll(".compass text")) { const th = (+t.dataset.a + rot) * Math.PI / 180; t.setAttribute("x", (Math.sin(th) * 19).toFixed(2)); t.setAttribute("y", (-Math.cos(th) * 19).toFixed(2)); }
  $("#tiltRead").textContent = "tilt " + Math.round(cam.el) + "°";
  $("#top").setAttribute("aria-pressed", cam.el > 85);
  afterRender(); floorAfterRender(); choicesAfterRender(); simAfterRender(); rpAfterRender(); fogAfterRender(); layoutChips();
  if (selected) drawCloseup();
}
function drawCallouts() {
  const box = $("#callouts");
  if (!labels.length) { box.innerHTML = ""; return; }
  const r = stage.getBoundingClientRect();
  box.innerHTML = calloutHtml(layoutCallouts(labels, { scale: view.s, offset: [view.x, view.y], gap: 20, pad: 36 }), r.width, r.height);
}

function toWorld(sx, sy) {
  const px = (sx - view.x) / view.s, py = (sy - view.y) / view.s;
  const o = k.iso([0, 0, 0], P), ax = k.iso([1, 0, 0], P), ay = k.iso([0, 1, 0], P);
  const a = ax[0] - o[0], b = ax[1] - o[1], c = ay[0] - o[0], d = ay[1] - o[1], det = a * d - b * c;
  const dx = px - o[0], dy = py - o[1];
  return [(d * dx - c * dy) / det, (-b * dx + a * dy) / det];
}
function drawGrid() {
  const r = stage.getBoundingClientRect();
  const corners = [[0, 0], [r.width, 0], [0, r.height], [r.width, r.height]].map(([x, y]) => toWorld(x, y));
  const xs = corners.map((p) => p[0]), ys = corners.map((p) => p[1]);
  const unit = Math.hypot(...[0, 1].map((i) => k.iso([1, 0, 0], P)[i] - k.iso([0, 0, 0], P)[i])) * view.s;
  let minor = 20, major = 100;
  while (minor * unit < 9) { minor *= 5; major *= 5; }
  const toS = (x, y) => { const p = k.iso([x, y, 0], P); return [(p[0] * view.s + view.x).toFixed(1), (p[1] * view.s + view.y).toFixed(1)]; };
  const lines = { minor: "", major: "", axis: "" };
  const x0 = Math.floor(Math.min(...xs) / minor) * minor, x1 = Math.max(...xs), y0 = Math.floor(Math.min(...ys) / minor) * minor, y1 = Math.max(...ys);
  if ((x1 - x0) / minor + (y1 - y0) / minor < 1600) {
    for (let x = x0; x <= x1; x += minor) { const cls = x === 0 ? "axis" : x % major === 0 ? "major" : "minor"; const [a, b] = toS(x, y0), [c, d] = toS(x, y1); lines[cls] += "M" + a + " " + b + "L" + c + " " + d; }
    for (let y = y0; y <= y1; y += minor) { const cls = y === 0 ? "axis" : y % major === 0 ? "major" : "minor"; const [a, b] = toS(x0, y), [c, d] = toS(x1, y); lines[cls] += "M" + a + " " + b + "L" + c + " " + d; }
  }
  grid.innerHTML = Object.entries(lines).map(([c, d]) => '<path class="' + c + '" d="' + d + '"/>').join("");
}

function fit() {
  const r = stage.getBoundingClientRect();
  if (frOn()) return frFit();
  const s = Math.min(r.width / W, r.height / H) * 1.02;
  goal.s = s; goal.x = (r.width - W * s) / 2; goal.y = (r.height - H * s) / 2 + 10; animate();
}
function apply() {
  world.style.transform = "translate(" + view.x + "px," + view.y + "px) scale(" + view.s + ")";
  const sc = Math.min(1.6, Math.max(0.75, 0.9 / view.s));
  for (const c of document.querySelectorAll(".chip")) c.style.scale = sc;
  pinScale(sc); layoutChips();
  const far = view.s < 0.55; document.body.classList.toggle("far", far);
  $("#zpct").textContent = Math.round(view.s * 100) + "%" + (far ? " far" : "");
  drawGrid(); drawCallouts();
}
const CHIP_RANK = { fail: 2, strain: 1, wait: 1 };
function chipShown(c) {
  if (!c.offsetParent) return false;
  const fade = c.getAnimations().find((a) => a.transitionProperty === "opacity");
  return +(fade ? fade.effect.getKeyframes().at(-1).opacity : getComputedStyle(c).opacity) > 0.01;
}
function layoutChips() {
  const all = [...document.querySelectorAll(".chip")];
  for (const c of all) c.style.removeProperty("--lift");
  const items = all.filter(chipShown).map((c) => ({ c, r: c.getBoundingClientRect(), rank: c.classList.contains("sel") ? 8 : c.classList.contains("show") || document.body.classList.contains("alt") ? CHIP_RANK[c.dataset.state] || 0 : -1 }));
  items.sort((a, b) => b.rank - a.rank || b.r.bottom - a.r.bottom);
  const placed = [], gap = 3;
  const cover = (r, up) => placed.reduce((s, q) => s + Math.max(0, Math.min(r.right, q.right + gap) - Math.max(r.left, q.left - gap)) * Math.max(0, Math.min(r.bottom - up, q.bottom + gap) - Math.max(r.top - up, q.top - gap)), 0);
  for (const it of items) {
    const step = it.r.height + gap;
    // With no free slot within four steps, take the lift that hides the least chip area rather than stacking at zero.
    let up = 0, best = cover(it.r, 0);
    for (let u = step / 2; best && u <= step * 4; u += step / 2) { const n = cover(it.r, u); if (n < best) { best = n; up = u; } }
    placed.push({ left: it.r.left, right: it.r.right, top: it.r.top - up, bottom: it.r.bottom - up });
    if (up) it.c.style.setProperty("--lift", (up / (it.r.width / it.c.offsetWidth)).toFixed(1) + "px");
  }
}
function repivot() {
  const r = stage.getBoundingClientRect(), cx = r.width / 2, cy = r.height / 2;
  const [wx, wy] = toWorld(cx, cy);
  pivot = [wx, wy, 0]; project();
  view.x = goal.x = cx - (W / 2) * view.s; view.y = goal.y = cy - (H / 2) * view.s;
}
function animate() {
  wake();
  if (raf) return;
  const step = () => {
    let moving = false, redraw = dirtyScene;
    for (const key of ["x", "y", "s"]) { const d = goal[key] - view[key]; if (Math.abs(d) > (key === "s" ? 0.0005 : 0.1)) { view[key] += d * 0.2; moving = true; } else view[key] = goal[key]; }
    for (const key of ["az", "el"]) { const d = camGoal[key] - cam[key]; if (Math.abs(d) > 0.05) { cam[key] += d * 0.22; moving = redraw = true; } else if (cam[key] !== camGoal[key]) { cam[key] = camGoal[key]; redraw = true; } }
    const de = explode.goal - explode.t;
    if (Math.abs(de) > 0.004) { explode.t += de * 0.16; moving = redraw = true; } else if (explode.t !== explode.goal) { explode.t = explode.goal; redraw = true; if (!explode.t) explode.id = null; }
    if (redraw) { renderPlant(); dirtyScene = false; }
    apply();
    raf = moving ? requestAnimationFrame(step) : 0;
  };
  raf = requestAnimationFrame(step);
}
function orbitTo(az, el = camGoal.el) { if (!drag || !drag.orbit) repivot(); camGoal.az = az; camGoal.el = Math.min(89, Math.max(12, el)); animate(); }
function zoomAt(px, py, f) {
  const ns = Math.min(5, Math.max(0.3, goal.s * f));
  goal.x = px - (px - goal.x) * (ns / goal.s); goal.y = py - (py - goal.y) * (ns / goal.s); goal.s = ns; animate();
}
function toggleExplode(id = selected) {
  if (!id || !ASSETS[ENT[id].kind].explodes) return;
  if (explode.id === id && explode.goal === 1) explode.goal = 0; else { explode.id = id; explode.goal = 1; }
  const b = $("#exBtn"); if (b) b.setAttribute("aria-pressed", explode.goal === 1);
  animate();
}

stage.addEventListener("wheel", (e) => { e.preventDefault(); const r = stage.getBoundingClientRect(); zoomAt(e.clientX - r.left, e.clientY - r.top, Math.exp(-e.deltaY * 0.0015)); used(); }, { passive: false });
stage.addEventListener("contextmenu", (e) => e.preventDefault());
let drag = null;
stage.addEventListener("pointerdown", (e) => {
  const orbit = e.button === 2 || e.shiftKey;
  if (orbit) repivot();
  drag = { x: e.clientX, y: e.clientY, gx: goal.x, gy: goal.y, az: camGoal.az, el: camGoal.el, orbit, moved: false };
  stage.classList.add(orbit ? "orbiting" : "drag");
});
$("#compass").addEventListener("pointerdown", (e) => { repivot(); drag = { x: e.clientX, y: e.clientY, az: camGoal.az, el: camGoal.el, orbit: true, moved: false }; e.preventDefault(); });
addEventListener("pointermove", (e) => {
  if (!drag) {
    const g = e.target.closest && e.target.closest(".ent");
    const id = g ? g.dataset.id : null;
    if (id !== hovered) { hovered = id; for (const n of document.querySelectorAll(".ent,.chip")) n.classList.toggle("hover", n.dataset.id === id); layoutChips(); }
    return;
  }
  const dx = e.clientX - drag.x, dy = e.clientY - drag.y;
  if (Math.abs(dx) + Math.abs(dy) > 3) { drag.moved = true; used(); }
  if (drag.orbit) { camGoal.az = drag.az - dx * 0.4; camGoal.el = Math.min(89, Math.max(12, drag.el + dy * 0.25)); cam.az = camGoal.az; cam.el = camGoal.el; renderPlant(); apply(); wake(); }
  else { goal.x = drag.gx + dx; goal.y = drag.gy + dy; view.x = goal.x; view.y = goal.y; apply(); wake(); }
});
addEventListener("pointerup", (e) => {
  if (drag && !drag.moved && !drag.orbit && e.target.closest && e.target.closest("#stage") && !e.target.closest(".cpin")) { const g = e.target.closest(".ent"); select(g ? g.dataset.id : null); }
  drag = null; stage.classList.remove("drag", "orbiting");
});
$("#compass").addEventListener("dblclick", () => orbitTo(35, 30));
$("#rotL").onclick = () => orbitTo(Math.round((camGoal.az - 45) / 45) * 45);
$("#rotR").onclick = () => orbitTo(Math.round((camGoal.az + 45) / 45) * 45);
$("#top").onclick = () => orbitTo(camGoal.az, camGoal.el > 85 ? 30 : 89);
$("#zin").onclick = () => zoomAt(innerWidth / 2, innerHeight / 2, 1.3);
$("#zout").onclick = () => zoomAt(innerWidth / 2, innerHeight / 2, 1 / 1.3);
$("#zfit").onclick = fit;
function used() { document.body.classList.add("used"); }

function paintSelection() {
  stage.classList.toggle("has-sel", !!selected);
  for (const g of document.querySelectorAll(".ent,.chip")) { g.classList.toggle("sel", g.dataset.id === selected); g.classList.toggle("hover", g.dataset.id === hovered); }
  for (const b of plant.querySelectorAll(".belt")) b.classList.toggle("lit", !!selected && (b.dataset.from === selected || b.dataset.to === selected));
  layoutChips();
}

const pct = (v, max) => Math.max(0, Math.min(1, v / max));
function dimSvg(b) {
  const x0 = 6, x1 = 356, X = (v) => x0 + (x1 - x0) * pct(v, b.max), y = 26, over = b.value > b.budget;
  let s = '<svg class="dim" viewBox="0 0 364 56"><line class="rail" x1="' + x0 + '" y1="' + y + '" x2="' + x1 + '" y2="' + y + '"/>';
  for (let i = 0; i <= 10; i++) { const x = x0 + ((x1 - x0) * i) / 10; s += '<line class="tick" x1="' + x + '" y1="' + (y - (i % 5 ? 2 : 4)) + '" x2="' + x + '" y2="' + (y + (i % 5 ? 2 : 4)) + '"/>'; }
  const vb = X(Math.min(b.value, b.budget)), vx = X(b.value), bx = X(b.budget);
  s += '<line class="val" x1="' + x0 + '" y1="' + y + '" x2="' + vb + '" y2="' + y + '"/>';
  if (over) s += '<line class="over" x1="' + bx + '" y1="' + y + '" x2="' + vx + '" y2="' + y + '"/>';
  s += '<path class="arr' + (over ? " over" : "") + '" d="M' + vx + " " + y + "l-6 -3v6z" + '"/><path class="arr" d="M' + x0 + " " + y + "l6 -3v6z" + '"/>';
  s += '<line class="ext" x1="' + bx + '" y1="6" x2="' + bx + '" y2="44"/>';
  s += '<text x="' + Math.min(vx, 300) + '" y="' + (y - 9) + '" text-anchor="middle">' + b.value + " " + b.unit + "</text>";
  s += '<text class="t" x="' + bx + '" y="' + 52 + '" text-anchor="' + (bx > 300 ? "end" : "middle") + '">budget ' + b.budget + " " + b.unit + "</text>";
  return s + "</svg>";
}
function spark(series) {
  const w = 200, h = 30, min = Math.min(...series) - 0.05, max = Math.max(...series) + 0.05;
  return '<svg viewBox="0 0 ' + w + " " + h + '" preserveAspectRatio="none"><path d="' + series.map((v, i) => (i ? "L" : "M") + ((i / (series.length - 1)) * w).toFixed(1) + " " + (h - ((v - min) / (max - min)) * h).toFixed(1)).join("") + '"/></svg>';
}
function inspectorHtml(e) {
  if (simOn() && (SM[e.id] || SIMD.ghosts[e.id])) return simInspector(e);
  if (rpOn() && RPD.series[e.id]) return rpInspector(e);
  if (GHOST[e.id]) return ghostHtml(e);
  if (e.kind === "crane") return ciHtml(e);
  const a = ASSETS[e.kind], d = INSPECT[e.id];
  let h = '<div class="fig" id="fig"><div class="iso" id="figPlant"></div><div class="read">' + e.reading + "<small>" + e.sub + "</small></div>";
  h += '<div class="tools">' + (a.explodes ? '<button id="exBtn" aria-pressed="' + (explode.id === e.id && explode.goal === 1) + '">Explode <kbd>X</kbd></button>' : "") + '<button data-do="open">Open code <kbd>O</kbd></button><button data-do="hand">Hand to agent <kbd>H</kbd></button></div></div>';
  if (!d) return h + fogStrip(e.id) + '<div class="note">' + a.label + " · " + a.maps + ". The full inspector is drawn for acquit-api, verifier and paypal in this prototype.</div>";
  h += '<div class="sec"><h5>Budgets<i>set on the drawing · agents read only</i></h5>' + d.budgets.map((b) => '<div style="font-size:12px;color:#cfcfcf;margin-bottom:-4px">' + b.metric + '</div>' + dimSvg(b)).join("") + '<div class="lock">🔒 Agents can propose a change. You approve it (K16).</div></div>';
  h += '<div class="sec"><h5>Three sources<i>match ' + d.match.score.toFixed(2) + "</i></h5>" + d.sources.map((s) => '<div class="src"><span class="n">' + s.src + '</span><span class="' + (s.ok ? "ok" : "no") + '">' + (s.ok ? "●" : "○") + "</span><b>" + s.what + "</b><span class='d'>" + s.detail + "</span></div>").join("");
  h += '<div class="match"><div class="bar"><i style="width:' + d.match.score * 100 + '%"></i></div>' + d.match.rules.join(" · ") + "</div></div>";
  h += '<div class="sec"><h5>Drift and fog<i>' + d.drift.length + " open</i></h5>" + d.drift.map((x) => '<div class="drift"><span class="sv" data-s="' + x.sev + '">' + x.sev + "</span><div>" + x.text + "<small>" + x.ref + "</small></div></div>").join("");
  h += '<div class="fog" style="margin-top:8px">' + Object.entries(d.fog).map(([n, v]) => '<div data-v="' + v + '"><i></i>' + n + "</div>").join("") + "</div></div>";
  h += '<div class="sec"><h5>Calibration<i>nightly backtest</i></h5><div class="cal"><b>' + d.calibration.score.toFixed(2) + "</b>" + spark(d.calibration.series) + "</div></div>";
  h += '<div class="sec"><h5>Choices here<i>riskiest first</i></h5>' + d.choices.map((c) => '<div class="choice"><span>' + c.text + '</span><span class="risk">risk ' + c.risk.toFixed(2) + "</span><small>confidence " + c.conf + " · reversal " + c.cost + "</small></div>").join("") + "</div>";
  h += '<div class="acts"><button data-walk="1">Walk the choices</button><button>Set a budget</button></div>';
  return h;
}
function drawCloseup() {
  const e = ENT[selected], host = $("#figPlant");
  if (!e || !host) return;
  const a = ASSETS[e.kind], ent = { ...e, x: 0, y: 0, size: 1, explode: e.id === explode.id ? explode.t : 0 };
  const scene = { entities: [ent] }, FW = 364, FH = 210;
  if (!host.scale) host.scale = Math.min(4, projectionFor(k, scene, ASSETS, { width: FW, height: FH, azimuth: 45, elevation: 30, pad: 30 }).scale * (a.explodes ? 1.05 : 1.25));
  const top = a.top + (a.explodes || 0) * ent.explode;
  const Pc = projectionFor(k, scene, ASSETS, { width: FW, height: FH, azimuth: cam.az, elevation: Math.min(cam.el, 60), scale: host.scale, pivot: [a.explodes ? 26 : 0, a.explodes ? -26 : 0, top * 0.45] });
  const out = renderScene(k, ASSETS, scene, Pc, { width: FW, height: FH });
  host.innerHTML = out.svg;
}
function select(id) {
  if (frOn()) return frClickEnt(id);
  if (id && exOpen()) return;
  if (id && BOT_SESSION[id]) return openSession(BOT_SESSION[id]);
  if (id === "you") return toggleQueue(true);
  const wasReview = review; review = null;
  session = null; for (const r of document.querySelectorAll("[data-session]")) r.classList.remove("on");
  if (id && ENT[id] && ENT[id].of) id = ENT[id].of;
  selected = id;
  const open = !!id && id !== "you";
  $("#insp").classList.toggle("open", open); document.body.classList.toggle("has-insp", open);
  if (explode.id && explode.id !== id) { explode.goal = 0; animate(); }
  if (open) {
    const [ax, ay] = anchors[id];
    goal.x += (innerWidth - 420) / 2 - (goal.x + ax * goal.s); goal.y += innerHeight / 2 - (goal.y + ay * goal.s); animate();
    const e = ENT[id], a = ASSETS[e.kind];
    $("#ih").dataset.state = e.state; $("#iMono").textContent = e.mono; $("#iName").textContent = e.name; $("#iKind").textContent = a.label + " · " + a.maps;
    $("#ibody").innerHTML = inspectorHtml(e); $("#ibody").scrollTop = 0;
    if ($("#simNow")) { paintSimNow(id); $("#iKind").textContent = "Simulated · queueing model on recorded traffic"; }
    rpSelected(id);
    const b = $("#exBtn"); if (b) b.onclick = () => toggleExplode(id);
    requestAnimationFrame(drawCloseup);
  }
  if (wasReview && !id) fit();
  paintSelection(); floorAfterRender(); choicesAfterRender();
}
$("#close").onclick = () => select(null);

const MODES = { live: "", blueprint: "BLUEPRINT · 1 GHOST · PAGE 3", simulate: "SIMULATE · RECORDED TRAFFIC, YESTERDAY 14:00 TO 15:00 · SEED 418", replay: "REPLAY · RECORDED, TODAY 15:30 TO 16:30 · READ ONLY" };
function setMode(m) {
  const keep = document.body.dataset.mode !== m && selected;
  document.body.dataset.mode = m;
  const btns = [...document.querySelectorAll("#modes button")];
  btns.forEach((b) => b.setAttribute("aria-pressed", b.dataset.mode === m));
  const b = btns.find((b) => b.dataset.mode === m), pill = $("#modes .pill");
  pill.style.left = b.offsetLeft + "px"; pill.style.width = b.offsetWidth + "px";
  $("#watermark").textContent = MODES[m];
  // Exits restore Live states before an entry snapshots them.
  if (m !== "simulate") simMode(m);
  if (m !== "replay") rpMode(m);
  if (m === "simulate") simMode(m);
  if (m === "replay") rpMode(m);
  if (keep && ENT[keep] && !$("#rpHide").textContent.includes('"' + keep + '"')) select(keep);
  wake();
}
for (const b of document.querySelectorAll("#modes button")) b.onclick = () => setMode(b.dataset.mode);

$("#layersBtn").onclick = (e) => { e.stopPropagation(); $("#layers").classList.toggle("open"); };
addEventListener("click", (e) => { if (!e.target.closest("#layers,#layersBtn")) $("#layers").classList.remove("open"); });
for (const i of document.querySelectorAll("[data-layer]")) {
  const set = () => { if (i.dataset.layer === "labels") document.body.classList.toggle("no-labels", !i.checked); else document.body.classList.toggle(i.dataset.layer, i.checked); };
  i.onchange = () => { set(); wake(); if (i.dataset.layer === "fogged") fogAfterRender(); }; set();
}

const IDLE = 120;
let lastInput = performance.now();
function wake() { lastInput = performance.now(); document.body.classList.remove("still"); }
for (const ev of ["pointermove", "keydown", "wheel"]) addEventListener(ev, wake, { passive: true });
setInterval(() => {
  const left = Math.max(0, IDLE - (performance.now() - lastInput) / 1000);
  if (left === 0) document.body.classList.add("still");
  const m = Math.floor(left / 60), s = String(Math.floor(left % 60)).padStart(2, "0");
  $("#motion").textContent = left === 0 ? "still" : "still in " + m + ":" + s;
  $("#time").textContent = new Date().toTimeString().slice(0, 8);
}, 1000);

addEventListener("keydown", (e) => {
  if (e.key === "Alt") { e.preventDefault(); if (!document.body.classList.contains("alt")) { document.body.classList.add("alt"); layoutChips(); } return; }
  if (e.key === "Escape") {
    if (typing(e)) e.target.blur();
    else if ($("#pal").classList.contains("open")) palClose();
    else if ($("#layers").classList.contains("open")) $("#layers").classList.remove("open");
    else if ($("#insp").classList.contains("open")) select(null);
    else if (document.body.classList.contains("queue-open")) toggleQueue(false);
    else if (document.body.classList.contains("floor-open")) toggleFloor(false);
    return;
  }
  if (!bare(e)) return;
  const m = !e.shiftKey && { 1: "live", 2: "blueprint", 3: "simulate", 4: "replay" }[e.key];
  if (m) setMode(m);
  const key = e.key.toLowerCase();
  if (key === "f") fit();
  if (key === "l") $("#layers").classList.toggle("open");
  if (key === "q") $("#rotL").click();
  if (key === "e") $("#rotR").click();
  if (key === "t") $("#top").click();
  if (key === "x") toggleExplode();
  used();
});
addEventListener("keyup", (e) => { if (e.key === "Alt") { document.body.classList.remove("alt"); layoutChips(); } });
addEventListener("blur", () => document.body.classList.remove("alt"));
addEventListener("resize", () => { if (review && review.i >= 0) flyTo(CH[review.i].on); else fit(); });

const DRONE_ICON_JS = ${JSON.stringify(DRONE_ICON)};
${BP_SCRIPT}
${FLOOR_SCRIPT}
${CH_SCRIPT}
${SIM_SCRIPT}
${RP_SCRIPT}
${ACT_SCRIPT}
${PAL_SCRIPT}
${FR_SCRIPT}
${SH_SCRIPT}
${FOG_SCRIPT}
const CONCEPT = new URLSearchParams(location.hash.slice(1)).get("c") || "";
const conceptActive = (id) => CONCEPT === id;
if (CONCEPT) document.body.dataset.concept = CONCEPT;
const CONCEPT_HINTS = ${JSON.stringify(HINTS)};
if (CONCEPT_HINTS[CONCEPT]) $(".hint").textContent = CONCEPT_HINTS[CONCEPT];
function setWaitingCount(n) {
  const b = $(".bl .wait b");
  if (!b || (+b.textContent === n && ENT.you.params.waiting === n)) return;
  b.textContent = n; ENT.you.params.waiting = n; ENT.you.reading = n + " waiting"; dirtyScene = true; animate();
}
// Concept panels move with the camera and open beside machines, so a fixed toast slot always collides with one of them.
// Place the toast at the free spot nearest the ui-v1 slot (bottom centre); plant labels are avoided when possible.
function conceptToastObstacles(toastEl) {
  const hard = [], soft = [], area = innerWidth * innerHeight;
  const walk = (n) => {
    for (const e of n.children) {
      if (e === toastEl || e.id === "stage" || e.tagName === "SCRIPT" || e instanceof SVGElement || !e.getClientRects().length) continue;
      const cs = getComputedStyle(e);
      if (cs.visibility === "hidden" || +cs.opacity < 0.05) continue;
      if (cs.position === "fixed" || cs.position === "absolute") {
        const r = e.getBoundingClientRect();
        if (r.width > 4 && r.height > 4 && r.width * r.height < area * 0.5) hard.push(r);
      }
      walk(e);
    }
  };
  walk(document.body);
  for (const c of document.querySelectorAll("#chips .chip")) if (chipShown(c) && c.style.visibility !== "hidden") soft.push(c.getBoundingClientRect());
  return { hard, soft };
}
function conceptToastHits(list, x, y, w, h, m = 8) {
  let n = 0;
  for (const r of list) if (x < r.right + m && x + w > r.left - m && y < r.bottom + m && y + h > r.top - m) n++;
  return n;
}
function conceptToastPlace(force) {
  const el = $("#toast");
  if (!el.classList.contains("on")) return;
  const w = el.offsetWidth, h = el.offsetHeight, { hard, soft } = conceptToastObstacles(el);
  if (!force) { const r = el.getBoundingClientRect(); if (!conceptToastHits(hard, r.left, r.top, w, h)) return; }
  const px = (innerWidth - w) / 2, py = innerHeight - 76 - h;
  let best = null;
  for (let y = 64; y <= innerHeight - h - 16; y += 8) for (let x = 16; x <= innerWidth - w - 16; x += 16) {
    const score = conceptToastHits(hard, x, y, w, h) * 1e6 + conceptToastHits(soft, x, y, w, h, 2) * 1e3 + Math.hypot(x - px, (y - py) * 1.5);
    if (!best || score < best.score) best = { x, y, score };
  }
  if (best) { el.style.left = best.x + "px"; el.style.top = best.y + "px"; }
}
if (CONCEPT) {
  let conceptToastTimer = null;
  new MutationObserver(() => {
    const on = $("#toast").classList.contains("on");
    if (on) conceptToastPlace(true);
    clearInterval(conceptToastTimer);
    conceptToastTimer = on ? setInterval(() => conceptToastPlace(false), 200) : null;
  }).observe($("#toast"), { attributes: true, attributeFilter: ["class"], childList: true, characterData: true, subtree: true });
}
for (const a of document.querySelectorAll("#cbar a")) { if (a.dataset.c === CONCEPT) a.setAttribute("aria-current", "true"); a.onclick = (e) => { e.preventDefault(); location.hash = "c=" + a.dataset.c; location.reload(); }; }
${LIST.map((c) => c.script).join("\n")}
const params = new URLSearchParams(location.hash.slice(1));
if (params.get("az")) cam.az = camGoal.az = +params.get("az");
if (params.get("el")) cam.el = camGoal.el = +params.get("el");
if (params.get("alt")) document.body.classList.add("alt");
if (params.get("fog")) { document.body.classList.add("fogged"); document.querySelector('[data-layer="fogged"]').checked = true; }
renderPlant();
requestAnimationFrame(() => {
  setMode(params.get("mode") || "live");
  fit(); view.x = goal.x; view.y = goal.y; view.s = goal.s; apply();
  if (params.get("zoom")) { zoomAt(innerWidth * 0.45, innerHeight * 0.5, +params.get("zoom")); view.x = goal.x; view.y = goal.y; view.s = goal.s; apply(); }
  if (params.get("sel")) select(params.get("sel"));
  if (params.get("explode")) { toggleExplode(params.get("sel")); explode.t = 1; renderPlant(); apply(); }
  if (params.get("hover")) { hovered = params.get("hover"); paintSelection(); }
  bpInit(params); floorInit(params); chInit(params); simInit(params); rpInit(params); frInit(params); shInit(params);
  ${LIST.map((c) => c.id + "Init(params);").join(" ")}
});
`;

const html = `<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Acan · v1 · Live</title>${TOAST_DISMISS}<style>${css}${papers}${BP_CSS}${FLOOR_CSS}${CH_CSS}${SIM_CSS}${RP_CSS}${PAL_CSS}${FR_CSS}${SH_CSS}${FOG_CSS}${CBAR_CSS}${LIST.map((c) => c.css).join("")}</style></head><body class="traffic ghosts agents territories choices" data-mode="live">${FOG_DEFS}${body}${BP_HTML}${floorHtml(DRONE_ICON)}${CH_HTML}${SIM_HTML}${RP_HTML}${PAL_HTML}${FR_HTML}${SH_HTML}${CBAR}${LIST.map((c) => c.html).join("")}<script>${script}</script></body></html>`;
writeFileSync(join(HERE, OUT), html);
console.log("wrote " + OUT, (html.length / 1024).toFixed(0) + " KB");
