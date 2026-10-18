// Renders the Acquit plant alone on drafting paper, light theme, for share images.
// Run: node build.mjs  -> plant-light.html, then shoot with headless Chrome.
import { writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import * as k from "/home/factory-user/.factory/skills/anatomy/kit/iso-kit.mjs";
import { ASSETS } from "../../ui-v2/assets.mjs";
import { ACQUIT } from "../../ui-v2/acquit.mjs";
import { renderScene, projectionFor, sceneBounds } from "../../ui-v2/scene.mjs";

const HERE = dirname(fileURLToPath(import.meta.url));
const W = 1600, H = 900;

// UI overlays (territory plates, engraved type, the sandbox fence) stay out of a plant-only picture.
const scene = structuredClone(ACQUIT);
scene.territories = [];
for (const e of scene.entities) delete e.variant;
delete scene.wall;

const P = projectionFor(k, scene, ASSETS, { width: W, height: H, azimuth: 35, elevation: 30, pad: 120 });
const { svg } = renderScene(k, ASSETS, scene, P, { width: W, height: H, label: "The Acquit plant" });

const pts = sceneBounds(scene, ASSETS);
const xs = pts.map((p) => p[0]), ys = pts.map((p) => p[1]);
const [x0, x1, y0, y1] = [Math.min(...xs) - 400, Math.max(...xs) + 400, Math.min(...ys) - 400, Math.max(...ys) + 400];
const step = 20, at = (p) => k.iso(p, P).map((v) => v.toFixed(1)).join(" ");
let minor = "", major = "";
for (let x = Math.ceil(x0 / step) * step; x <= x1; x += step) (x % (step * 5) ? (minor += `M${at([x, y0, 0])}L${at([x, y1, 0])}`) : (major += `M${at([x, y0, 0])}L${at([x, y1, 0])}`));
for (let y = Math.ceil(y0 / step) * step; y <= y1; y += step) (y % (step * 5) ? (minor += `M${at([x0, y, 0])}L${at([x1, y, 0])}`) : (major += `M${at([x0, y, 0])}L${at([x1, y, 0])}`));
const c = k.iso([(Math.min(...xs) + Math.max(...xs)) / 2, (Math.min(...ys) + Math.max(...ys)) / 2, 0], P);
const grid = `<defs><radialGradient id="gridFade" gradientUnits="userSpaceOnUse" cx="${c[0].toFixed(0)}" cy="${c[1].toFixed(0)}" r="${(W * 0.56).toFixed(0)}"><stop offset=".35" stop-color="#fff"/><stop offset="1" stop-color="#000"/></radialGradient><mask id="gridMask"><rect width="${W}" height="${H}" fill="url(#gridFade)"/></mask>
<radialGradient id="droneShade" cx=".36" cy=".3" r=".8"><stop offset="0" stop-color="#FFFFFF"/><stop offset=".5" stop-color="#ECE7DC"/><stop offset="1" stop-color="#CFC8BA"/></radialGradient>
<linearGradient id="droneHull" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#F3EFE6"/><stop offset="1" stop-color="#D3CCBE"/></linearGradient>
<radialGradient id="droneGlass" cx=".35" cy=".3" r=".9"><stop offset="0" stop-color="#9FB4BC"/><stop offset=".55" stop-color="#5E7178"/><stop offset="1" stop-color="#3E4C52"/></radialGradient></defs>
<g class="paper-grid" mask="url(#gridMask)"><path d="${minor}" class="g-minor"/><path d="${major}" class="g-major"/></g>`;
const body = svg.replace(/(<svg[^>]*>)/, `$1${grid}`);

// Light finish. The ui's STATE_CSS forces dark --anatomy-mid/lo (#7a7a7a/#474747) on every .iso, which is what
// turned every edge black in light mode, so this page defines its own tone ramp instead of importing it.
const CSS = `
html,body{margin:0;background:#F4F1EA}
body{width:${W}px;height:${H}px;overflow:hidden}
.iso[data-theme="light"]{
  --anatomy-paper:#F4F1EA;--anatomy-face:#FBF9F4;--anatomy-top:#FDFCF8;
  --anatomy-shade-0:#CFC8BA;--anatomy-shade-1:#DCD6CA;--anatomy-shade-2:#E8E4DA;--anatomy-shade-3:#F2EFE7;
  --anatomy-lit-top:#FFFFFF;--anatomy-lit-shade:#EFEBE2;
  --anatomy-hi:#9A9283;--anatomy-mid:#B3AC9E;--anatomy-lo:#C9C3B6;--anatomy-faint:#DDD8CD;--anatomy-lit:#6F6656;
  --anatomy-halo:#8F8574;--anatomy-halo-opacity:.26;--anatomy-dot:#7A7262;
  --anatomy-red:#C2412F;--anatomy-weight:.55px;
  --ok:#2F8F80;--strain:#B7791F;--fail:#C2412F;--ghost:#4F86AD;
}
.iso-svg{overflow:visible}
.g-minor{fill:none;stroke:#EAE6DD;stroke-width:.5}
.g-major{fill:none;stroke:#DFD9CC;stroke-width:.7}
.lamp .iso-top{fill:#DCD6CA}.lamp .iso-shade{fill:#C9C2B4}
.ent[data-state=ok] .lamp:not([data-lamp]) .iso-top,.lamp[data-lamp=ok] .iso-top{fill:#5FB3A5}
.ent[data-state=ok] .lamp:not([data-lamp]) .iso-shade,.lamp[data-lamp=ok] .iso-shade{fill:#3F8F82}
.ent[data-state=strain] .lamp:not([data-lamp]) .iso-top,.ent[data-state=wait] .lamp:not([data-lamp]) .iso-top,.lamp[data-lamp=strain] .iso-top{fill:#E0A64A}
.ent[data-state=strain] .lamp:not([data-lamp]) .iso-shade,.ent[data-state=wait] .lamp:not([data-lamp]) .iso-shade,.lamp[data-lamp=strain] .iso-shade{fill:#B7791F}
.ent[data-state=fail] .lamp:not([data-lamp]) .iso-top{fill:#E0624F}
.ent[data-state=fail] .lamp:not([data-lamp]) .iso-shade{fill:#B13A2A}
.lamp[data-lamp=run] .iso-top{fill:#9CC3DD}.lamp[data-lamp=run] .iso-shade{fill:#5F8DAD}
.ent[data-state=ghost] .iso-fill{fill:rgba(79,134,173,.05)}
.ent[data-state=ghost] .iso-shade,.ent[data-state=ghost] .iso-top,.ent[data-state=ghost] .iso-halo{fill:none;opacity:0}
.ent[data-state=ghost] .iso-line{stroke:var(--ghost);stroke-dasharray:2.6 2.2;opacity:.7}
.ent[data-state=ghost] .iso-dots circle{fill:var(--ghost)}
.ent[data-state=ghost] .lamp{display:none}
.iso-line.scaffold{stroke:var(--anatomy-mid);stroke-dasharray:.01 3;stroke-width:1.1;stroke-linecap:round}
.smoke{display:none}
.belt .iso-line[data-flow]{stroke:#A79F90;animation:none}
.belt[data-tone=red] .iso-line[data-flow]{stroke:#C2412F}
.wall .iso-line[data-dashed]{stroke:#CFC8BA}
.beam{stroke-dasharray:1.5 2;opacity:.6}
.drone-shell{fill:url(#droneShade)}.drone-hull{fill:url(#droneHull)}
.drone-duct{fill:#BDB5A6}.drone-hub{fill:#A39B8C}.drone-bezel{fill:#8E8677}
.drone-glass{fill:url(#droneGlass)}
.drone-eye circle{fill:#F4FBFF}
.drone-lamp circle{fill:#3F8F82}
.ent[data-state=wait] .drone-lamp circle{fill:#B7791F}
.drone-shadow .iso-halo{opacity:.16;filter:blur(3px)}
.fogcore .iso-line{stroke-dasharray:1.4 1.6}
*{animation:none!important}
`;
const html = `<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Acquit plant, light</title><style>${k.ISO_CSS}${CSS}</style></head><body class="iso" data-theme="light">${body}<script>const z=location.hash.match(/z=([0-9.]+),([0-9.]+),([0-9.]+)/);if(z){document.body.style.transformOrigin="0 0";document.body.style.transform="scale("+z[3]+") translate(-"+z[1]+"px,-"+z[2]+"px)"}</script></body></html>`;
writeFileSync(join(HERE, "plant-light.html"), html);
console.log("wrote plant-light.html", (html.length / 1024).toFixed(0), "KB");
