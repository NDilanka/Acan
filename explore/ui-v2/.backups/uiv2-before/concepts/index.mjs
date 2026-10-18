// Concept registry. Each concept module default-exports:
//   { id, label, css, html, script }
// `script` must declare `function <id>Init(params)`, which runs after every built-in init.
// A concept is active when the URL hash has c=<id>. Guard key handlers with conceptActive("<id>"),
// and scope CSS under body[data-concept="<id>"].
import orders from "./orders.mjs";
import clash from "./clash.mjs";
import gate from "./gate.mjs";
import bench from "./bench.mjs";
import world from "./world.mjs";

export const CONCEPTS = [orders, clash, gate, bench, world];
