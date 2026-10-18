import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const KIT = "/home/factory-user/.factory/skills/anatomy/kit/iso-kit.mjs";

export function runtimeScript() {
  const kit = readFileSync(KIT, "utf8");
  const names = [...kit.matchAll(/^export (?:function|const) (\w+)/gm)].map((m) => m[1]);
  const strip = (f) => readFileSync(join(HERE, f), "utf8").replace(/^export /gm, "");
  return `const k = (() => {\n${kit.replace(/^export /gm, "")}\nreturn { ${names.join(", ")} };\n})();\n${strip("assets.mjs")}\n${strip("scene.mjs")}`.replace(/<\/(script)/gi, "<\\/$1");
}

export const STATE_CSS = `
:root{--ok:#35b7a2;--strain:#e1a43c;--fail:#e8604a;--ghost:#7ab8e6;--idle:#4a4a4a}
.iso{--anatomy-red:#e8604a;--anatomy-mid:#7a7a7a;--anatomy-lo:#474747;--anatomy-weight:.75px}
.iso-svg{overflow:visible}
.lamp .iso-top{fill:#3a3a3a}.lamp .iso-shade{fill:#262626}
.ent[data-state=ok] .lamp:not([data-lamp]) .iso-top,.lamp[data-lamp=ok] .iso-top{fill:var(--ok)}
.ent[data-state=ok] .lamp:not([data-lamp]) .iso-shade,.lamp[data-lamp=ok] .iso-shade{fill:#1f6e62}
.ent[data-state=strain] .lamp:not([data-lamp]) .iso-top,.lamp[data-lamp=strain] .iso-top{fill:var(--strain)}
.ent[data-state=strain] .lamp:not([data-lamp]) .iso-shade,.lamp[data-lamp=strain] .iso-shade{fill:#8a6322}
.ent[data-state=strain] .lamp:not([data-lamp]){animation:lampPulse 1.6s ease-in-out infinite}
.ent[data-state=wait] .lamp:not([data-lamp]) .iso-top{fill:var(--strain)}
.ent[data-state=wait] .lamp:not([data-lamp]) .iso-shade{fill:#8a6322}
.ent[data-state=wait] .lamp:not([data-lamp]){animation:lampPulse 2.4s ease-in-out infinite}
.ent[data-state=fail] .lamp:not([data-lamp]) .iso-top{fill:var(--fail)}
.ent[data-state=fail] .lamp:not([data-lamp]) .iso-shade{fill:#8b3427}
.ent[data-state=fail] .lamp:not([data-lamp]){animation:lampPulse .9s steps(2) infinite}
.ent[data-state=idle] .lamp .iso-top,.ent[data-state=dark] .lamp .iso-top,.ent[data-state=fog] .lamp .iso-top{fill:#3a3a3a}
.ent[data-state=idle] .lamp .iso-shade,.ent[data-state=dark] .lamp .iso-shade{fill:#262626}
.ent[data-state=idle] .needle,.ent[data-state=dark] .needle{opacity:.35}
@keyframes lampPulse{50%{opacity:.4}}
.lamp[data-lamp=run] .iso-top{fill:#cfe6fb}.lamp[data-lamp=run] .iso-shade{fill:#56718a}.lamp[data-lamp=run]{animation:lampPulse 1.1s ease-in-out infinite}
.crane-load[data-run=running]{animation:craneSway 2.8s ease-in-out infinite;transform-box:fill-box;transform-origin:50% 0}
@keyframes craneSway{50%{transform:translateY(-3px)}}
.ent[data-state=dark]{opacity:.32}
.ent[data-state=dark] [data-lit]{--anatomy-lit:#777}
.ent[data-state=fog]{filter:url(#fog);opacity:.7}
.ent[data-state=ghost] .iso-fill{fill:rgba(122,184,230,.035)}
.ent[data-state=ghost] .iso-shade,.ent[data-state=ghost] .iso-top,.ent[data-state=ghost] .iso-halo{fill:none;opacity:0}
.ent[data-state=ghost] .iso-line{stroke:var(--ghost);stroke-dasharray:2.6 2.2;opacity:.85}
.ent[data-state=ghost] .iso-dots circle{fill:var(--ghost)}
.ent[data-state=ghost] .iso-face-text{fill:var(--ghost)}
.ent[data-state=ghost] .lamp{display:none}
.iso-line.scaffold{stroke:var(--anatomy-mid);stroke-dasharray:.01 3;stroke-width:1.1;stroke-linecap:round}
.smoke{display:none}
.ent[data-state=fail] .smoke{display:inline}
.smoke circle{fill:#8d8d8d;opacity:0;transform-box:fill-box;transform-origin:center;animation:smoke 3.2s ease-out infinite;animation-delay:calc(var(--i)*.8s)}
@keyframes smoke{0%{opacity:0;transform:translate(0,0)}15%{opacity:.35}100%{opacity:0;transform:translate(9px,-34px) scale(1.8)}}
.ent:not([data-state=fail]):not([data-state=strain]) .fit-flow{stroke:var(--anatomy-mid);animation:none}
.beam{stroke-dasharray:1.5 2;opacity:.8}
.drone{animation:bob 3.6s ease-in-out infinite}
@keyframes bob{50%{transform:translateY(-2.2px)}}
.drone-shadow .iso-halo{opacity:.55;filter:blur(2px);animation:shadow 3.6s ease-in-out infinite;transform-box:fill-box;transform-origin:center}
@keyframes shadow{50%{transform:scale(.9);opacity:.4}}
.drone-shell{fill:url(#droneShade)}
.drone-hull{fill:url(#droneHull)}
.drone-duct{fill:#070707}
.drone-hub{fill:#3a3a3a}
.drone-bezel{fill:#262626}
.drone-glass{fill:url(#droneGlass)}
.drone-eye circle{fill:#fff;filter:drop-shadow(0 0 2px rgba(255,255,255,.9))}
.drone-lamp circle{fill:var(--ok)}
.ent[data-state=idle] .drone-lamp circle{fill:#555}.ent[data-state=idle] .drone-eye circle{fill:#777;filter:none}.ent[data-state=idle] .beam{display:none}
.ent[data-state=wait] .drone-lamp circle{fill:var(--strain)}
.fogcore{filter:url(#fog);opacity:.8}
.fogcore .iso-line{stroke-dasharray:1.4 1.6}
@media (prefers-reduced-motion:reduce){.iso-line[data-flow],.smoke circle,.lamp,.crane-load{animation:none!important}}
`;

export const FOG_DEFS = `<svg width="0" height="0" style="position:absolute"><defs><radialGradient id="droneShade" cx=".36" cy=".3" r=".8"><stop offset="0" stop-color="#6a6a6a"/><stop offset=".45" stop-color="#363636"/><stop offset="1" stop-color="#151515"/></radialGradient><linearGradient id="droneHull" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#4a4a4a"/><stop offset="1" stop-color="#1a1a1a"/></linearGradient><radialGradient id="droneGlass" cx=".35" cy=".3" r=".9"><stop offset="0" stop-color="#3c3c3c"/><stop offset=".5" stop-color="#121212"/><stop offset="1" stop-color="#050505"/></radialGradient></defs><filter id="fog" x="-20%" y="-20%" width="140%" height="140%"><feTurbulence type="fractalNoise" baseFrequency=".9" numOctaves="2" seed="4"/><feDisplacementMap in="SourceGraphic" scale="3"/><feGaussianBlur stdDeviation=".7"/></filter></svg>`;
