export const FAMILIES = {
  machine: "Machines",
  fitting: "Fittings on belts",
  floor: "Floor",
};

export const DEFERRED = [
  { label: "Fold-out cell", maps: "serverless function", why: "no v1 fixture runs serverless" },
  { label: "Pneumatic exchange", maps: "pub/sub", why: "Astronomy Shop uses Kafka as a queue, so Hopper covers it" },
  { label: "Silo", maps: "object storage", why: "no fixture stores objects" },
  { label: "Card catalog", maps: "search / vector index", why: "no fixture has an index" },
  { label: "Fuse", maps: "timeout fitting", why: "drawn as a budget dimension on the belt for now" },
  { label: "Strainer", maps: "validation fitting", why: "no fixture exposes validation as a step" },
  { label: "Meter", maps: "budget probe fitting", why: "budgets live in the inspector in v1" },
];

export const ASSETS = {
  intake: {
    family: "machine", label: "Intake", maps: "clients: web, CLI, MCP, webhooks", fixtures: "Acquit, Astronomy Shop", w: 50, d: 96, top: 30,
    build(c, { x, y }) {
      c.plinth(x, y, 50, 96);
      c.put([x - 19, y, 16], c.box(x - 22, y - 44, 6, 88, 4, 24, { tone: "mid" }, 3, 0.6) + c.D([-30, -10, 10, 30].map((dy) => c.at([x - 19, y + dy, 28])), { size: 0.45 }) + c.lamp(x - 19, y + 40, 28));
      for (let i = 0; i < 4; i++) {
        const yy = y - 38 + i * 20;
        c.put([x + 1, yy + 7, 7], c.box(x - 16, yy, 34, 14, 4, 7, { tone: "mid" }, 2, 0.4) + c.L(c.k.lineOnTop([x - 14, yy + 7], [x + 16, yy + 7], 11, c.P), { tone: "faint" }) + c.D([c.at([x - 8, yy + 7, 11]), c.at([x + 2, yy + 7, 11]), c.at([x + 12, yy + 7, 11])], { size: 0.9, tone: "hi" }));
      }
    },
  },

  splitter: {
    family: "machine", label: "Splitter", maps: "load balancer, gateway, reverse proxy", fixtures: "Astronomy Shop (frontend-proxy)", w: 86, d: 86, top: 34,
    build(c, { x, y }) {
      c.plinth(x, y, 86, 86);
      const chute = (ang, len, wide) => {
        const [ux, uy] = [Math.cos(ang), Math.sin(ang)], [vx, vy] = [-uy, ux];
        const r0 = 10, r1 = r0 + len;
        const ring = [[r0, -wide], [r1, -wide], [r1, wide], [r0, wide]].map(([a, b]) => [x + ux * a + vx * b, y + uy * a + vy * b]);
        const tip = [x + ux * (r1 - 3), y + uy * (r1 - 3)];
        c.put([x + ux * (r0 + len / 2), y + uy * (r0 + len / 2), 9], c.prism(ring, 4, 10, { tone: "mid" }) + c.L(c.k.onTop([[x + ux * (r0 + 2), y + uy * (r0 + 2)], tip], 14, c.P), { tone: "faint" }) + c.D([c.at([...tip, 14])], { size: 0.6, tone: "hi" }));
      };
      chute(Math.PI, 28, 6);
      for (const a of [-0.75, 0, 0.75]) chute(a, 26, 5);
      const hub = c.cyl(x, y, 14, 4, 22, { tone: "mid" }, 40, 0.6);
      const top = c.cyl(x, y, 9, 26, 3, { tone: "hi", crease: "lo" }, 32, 0.4);
      const ticks = c.k.radialTicks(x, y, 8, 24, 6, 29, [0.8, 1.8], c.P);
      const vane = c.L(c.k.lineOnTop([x, y], [x + 7, y - 3], 29.2, c.P), { tone: "lit", free: "start" });
      c.put([x, y, 15], hub + c.L(c.k.sideArc(x, y, 14, 16, c.P, 32), { tone: "lo" }), [[[x, y, 28], top + c.L(ticks.minor, { tone: "lo" }) + c.L(ticks.major, { tone: "mid" }) + vane + c.D([c.at([x, y, 29.3])], { size: 0.8, tone: "hi" }) + c.lamp(x - 6, y + 4, 29)]]);
    },
  },

  gate: {
    family: "machine", label: "Gate", maps: "auth: basic auth, sessions, OAuth", fixtures: "Acquit", w: 44, d: 74, top: 40,
    explodes: 18, parts: "one layer per auth check, plus exempt routes",
    build(c, { x, y, explode = 0, layers = ["outer auth", "sessions", "exempt route"] }) {
      c.plinth(x, y, 44, 74);
      const T = 8 * explode, B = 18 * explode, S = 16 * explode;
      c.put([x, y - 26, 19], c.cyl(x, y - 26, 4, 4, 30, { tone: "mid" }));
      c.put([x, y + 26, 19], c.cyl(x, y + 26, 4, 4, 30, { tone: "mid" }));
      const arms = [0, 2.094, 4.189].map((a) => c.L(c.k.lineOnTop([x, y], [x + Math.sin(a) * 14, y - Math.cos(a) * 14], 18 + T, c.P), { tone: "hi" })).join("");
      c.put([x, y, 11 + T], c.cyl(x, y, 3.2, 4 + T, 14, { tone: "hi" }) + arms + c.D([c.at([x, y, 18.5 + T])], { size: 0.8, tone: "hi" }));
      c.put([x + 13 + S, y + 16, 11], c.box(x + 8 + S, y + 10, 10, 12, 4, 14, { tone: "mid" }, 2, 0.4) + c.L(c.k.ring(x + 13 + S, y + 16, 2.4, 18, c.P, 14), { tone: "hi" }));
      c.put([x, y, 36.5 + B], c.box(x - 5, y - 31, 10, 62, 34 + B, 5, { tone: "mid" }, 3, 0.5) + c.L(c.k.planOutline({ x: x - 3, y: y - 28, w: 6, d: 56, r: 1 }, 39 + B, c.P, 2), { tone: "faint" }) + c.lamp(x, y - 20, 39 + B));
      if (explode > 0.05) { c.label([x, y + 31, 37 + B], layers[0]); c.label([x + 10, y, 18 + T], layers[1]); c.label([x + 18 + S, y + 16, 18], layers[2]); }
    },
  },

  assembler: {
    family: "machine", label: "Assembler", maps: "service: an HTTP API, a worker, a process", fixtures: "all three", w: 96, d: 86, top: 66,
    explodes: 36, parts: "adapters, core, routes (from the code graph)",
    build(c, { x, y, needle = 0.38, explode = 0, layers = ["adapters", "core", "routes"] }) {
      c.plinth(x, y, 96, 86);
      const G = 18 * explode, u = 2 * G;
      const body = { x: x - 38, y: y - 34, w: 76, d: 68, r: 4 };
      const slab = (z, h) => c.S(c.k.slabOf(body, z, h, c.P, 4, 0.8), { tone: "mid" });
      c.put([x, y, 10], slab(4, 12) + (explode ? c.L(c.k.segment([x, y, 16], [x, y, 40 + u], c.P), { tone: "faint", dashed: true }) : ""));
      c.put([x, y, 23 + G], slab(16 + G, 14));
      const Z = 40 + u;
      const shell = slab(30 + u, 10) + c.L(c.k.planOutline(c.k.insetPlan(body, 4), Z, c.P, 4), { tone: "faint" }) + c.screws(body, 3.5, Z) + c.D(c.k.dotGrid(x + 6, y - 30, 8, 2, 3.2, Z, c.P), { size: 0.45, tone: "lo" });
      const [gx, gy] = [x + 18, y + 14];
      const t = c.k.radialTicks(gx, gy, 10, 40, 5, Z + 5, [1.2, 2.6], c.P, -2.4, 4.8);
      const a = -2.4 + 4.8 * needle;
      const gauge = c.cyl(gx, gy, 13, Z, 5, { tone: "hi", crease: "lo" }, 56, 0.6) + c.L(c.k.ring(gx, gy, 11, Z + 5, c.P, 56), { tone: "lo" }) + c.L(t.minor, { tone: "lo" }) + c.L(t.major, { tone: "mid" })
        + `<path class="iso-line needle" data-tone="lit" data-free="start" d="${c.k.lineOnTop([gx, gy], [gx + Math.sin(a) * 9, gy - Math.cos(a) * 9], Z + 5.2, c.P)}" style="stroke-width:1.1"/>` + c.D([c.at([gx, gy, Z + 5.3])], { size: 0.9, tone: "hi" });
      const knob = c.cyl(x - 12, y - 10, 10, Z, 4, { tone: "mid" }, 32, 0.5) + c.cyl(x - 12, y - 10, 5, Z + 4, 14, { tone: "mid" }) + c.L(c.k.knurl(x - 12, y - 10, 5, Z + 6, Z + 16, 18, c.P), { tone: "lo" })
        + c.box(x - 24, y - 22, 24, 24, Z + 18, 6, { tone: "hi", crease: "lo" }, 3, 0.6) + c.screws({ x: x - 24, y: y - 22, w: 24, d: 24 }, 3, Z + 24);
      const dials = [[x - 30, y + 22], [x - 30, y + 8]].map(([ox, oy]) => {
        const ct = c.k.radialTicks(ox, oy, 4.8, 12, 3, Z + 2.4, [0.8, 1.4], c.P);
        return [[ox, oy, Z + 1], c.cyl(ox, oy, 5.5, Z, 2.4, { tone: "mid" }, 28, 0.4) + c.L(ct.minor, { tone: "lo" }) + c.L(ct.major, { tone: "mid" }) + c.L(c.k.lineOnTop([ox, oy], [ox + 3.6, oy - 1.8], Z + 2.5, c.P), { tone: "hi", free: "end" })];
      });
      c.put([x, y, 35 + u], shell, [[[gx, gy, Z + 2], gauge], [[x - 12, y - 10, Z + 10], knob], ...dials, [[x + 32, y - 28, Z + 1], c.lamp(x + 32, y - 28, Z)]]);
      if (explode > 0.05) [[10, layers[0]], [23 + G, layers[1]], [35 + u, layers[2]]].forEach(([z, text]) => c.label(c.rightOf(body, z), text));
    },
  },

  bank: {
    family: "machine", label: "Assembler bank", maps: "worker pool, job runners", fixtures: "Acquit (verifier)", w: 132, d: 58, top: 44,
    explodes: 14, parts: "one cell per worker slot, plus the waiting tray",
    build(c, { x, y, cells = 4, busy = 1, queued = 7, explode = 0 }) {
      const w = cells * 30 + 12;
      c.plinth(x, y, w, 58);
      for (let i = 0; i < cells; i++) {
        const bx = x - w / 2 + 8 + i * 30 + (i - (cells - 1) / 2) * 16 * explode, on = i < busy, L = 14 * explode;
        c.put([bx + 12, y - 7, 17], c.box(bx, y - 22, 24, 30, 4, 26, { tone: "mid" }, 3, 0.6) + c.L(c.k.planOutline({ x: bx + 3, y: y - 19, w: 18, d: 24, r: 1 }, 30, c.P, 2), { tone: "faint" }),
          [[[bx + 12, y - 7, 33 + L], c.cyl(bx + 12, y - 7, 5, 30 + L, 3, { tone: "mid" }) + c.cyl(bx + 12, y - 7, 2.2, 33 + L, on ? 6 : 13, { tone: on ? "hi" : "mid", lit: on }) + c.box(bx + 6, y - 13, 12, 12, (on ? 39 : 46) + L, 3, { tone: on ? "hi" : "mid", lit: on }, 2, 0.4)],
            [[bx + 20, y + 4, 31], c.lamp(bx + 20, y + 4, 30, on ? "ok" : "idle")]]);
        if (explode > 0.05) c.label([bx + 12, y - 7, 50 + L], `slot ${i + 1} · ${on ? "busy" : "idle"}`);
      }
      const tray = { x: x - w / 2 + 4, y: y + 12 + 10 * explode, w: 70, d: 10, r: 1 };
      c.put([tray.x + 35, tray.y + 5, 6], c.S(c.k.slabOf(tray, 4, 3, c.P, 2, 0.4), { tone: "lo" }), Array.from({ length: Math.min(7, queued) }, (_, i) => [[tray.x + 5.5 + i * 9.5, tray.y + 5, 9], c.box(tray.x + 3 + i * 9.5, tray.y + 2.5, 5, 5, 7, 5, { tone: "hi", crease: "lo" }, 2, 0.3)]));
      if (explode > 0.05) c.label([tray.x + 70, tray.y + 5, 8], `${queued} waiting`);
      c.put([x + w / 2 - 10, y + 18, 5], c.lamp(x + w / 2 - 10, y + 18, 4));
    },
  },

  hopper: {
    family: "machine", label: "Hopper + conveyor", maps: "queue: Kafka, SQS, a jobs table, a channel", fixtures: "Astronomy Shop (Kafka), Acan (channels)", w: 100, d: 56, top: 48,
    explodes: 25, parts: "topic rim, one ring per partition, the consumer belt",
    build(c, { x, y, depth = 5, explode = 0, layers = ["topic", "partitions", "consumer belt"] }) {
      c.plinth(x + 14, y, 100, 56);
      for (const [lx, ly] of [[-13, -13], [13, -13], [-13, 13], [13, 13]]) c.put([x + lx, y + ly, 12], c.box(x + lx - 2, y + ly - 2, 4, 4, 4, 18, { tone: "mid" }, 2));
      const E = 5 * explode;
      const rings = [[7, 20], [11, 24], [15, 28], [18, 32], [20, 36]];
      rings.forEach(([r, z], i) => c.put([x, y, z + 2 + i * E], c.cyl(x, y, r, z + i * E, 4, { tone: "mid", crease: "faint" }, 40, 0.3)));
      const rz = 40 + 5 * E;
      c.put([x, y, rz + 1.5], c.cyl(x, y, 22, rz, 3, { tone: "mid" }, 44, 0.5) + c.L(c.k.ring(x, y, 19, rz + 3, c.P, 44), { tone: "lo" }) + c.L(c.k.ring(x, y, 14, rz + 3, c.P, 40), { tone: "faint" }));
      const bx = x + 14 * explode;
      const belt = { x: bx - 4, y: y - 7, w: 64, d: 14, r: 2 };
      const items = Array.from({ length: depth }, (_, i) => [[bx + 10 + i * 10, y, 13], c.box(bx + 7 + i * 10, y - 3.5, 7, 7, 10, 6, { tone: "hi", crease: "lo" }, 2, 0.3)]);
      const rollers = Array.from({ length: 8 }, (_, i) => c.L(c.k.lineOnTop([belt.x + 4 + i * 8, belt.y + 1.5], [belt.x + 4 + i * 8, belt.y + belt.d - 1.5], 10, c.P), { tone: "faint" })).join("");
      c.put([bx + 28, y, 7], c.S(c.k.slabOf(belt, 4, 6, c.P, 3, 0.4), { tone: "mid" }) + rollers, [...items, [[bx + 56, y + 10, 11], c.lamp(bx + 56, y + 10, 10)]]);
      if (explode > 0.05) { c.label([x + 22, y, rz + 2], layers[0]); c.label([x + 18, y, 34 + 3 * E], `${layers[1]} ×${rings.length}`); c.label([bx + 60, y, 10], layers[2]); }
    },
  },

  tank: {
    family: "machine", label: "Tank", maps: "relational database: SQLite, Postgres, MySQL", fixtures: "all three", w: 78, d: 62, top: 56,
    explodes: 42, parts: "tables, indexes, schema lid, side files",
    build(c, { x, y, extra = 2, explode = 0, layers = ["tables", "indexes", "schema"], files = ["-wal", "-shm"] }) {
      c.plinth(x + (extra ? 14 : 0), y, extra ? 78 : 52, 62);
      const G = 14 * explode;
      const [vx, vy] = c.view;
      const fa = Math.atan2(vy, vx) + 0.5;
      const front = [x + Math.cos(fa) * 21, y + Math.sin(fa) * 21];
      const ring = (z0, h, i) => c.cyl(x, y, 21, z0 + i * G, h, { tone: "mid" }, 56, 0.6) + c.L(c.k.sideArc(x, y, 21, z0 + i * G + h * 0.5, c.P, 40), { tone: "lo" });
      c.put([x, y, 11], ring(4, 14, 0) + c.L(c.k.segment([front[0], front[1], 8], [front[0], front[1], 17], c.P), { tone: "lit" }));
      c.put([x, y, 26 + G], ring(18, 16, 1) + c.L(c.k.segment([front[0], front[1], 19 + G], [front[0], front[1], 33 + G], c.P), { tone: "hi" }));
      c.put([x, y, 42 + 2 * G], ring(34, 16, 2) + c.L(c.k.segment([front[0], front[1], 35 + 2 * G], [front[0], front[1], 48 + 2 * G], c.P), { tone: "hi" }));
      const Z = 50 + 3 * G;
      const t = c.k.radialTicks(x, y, 13, 24, 6, Z + 3, [1, 2.2], c.P);
      c.put([x, y, Z + 1.5], c.cyl(x, y, 15, Z, 3, { tone: "mid" }, 48, 0.4) + c.L(c.k.ring(x, y, 18, Z, c.P, 48), { tone: "faint" }) + c.L(t.minor, { tone: "lo" }) + c.L(t.major, { tone: "mid" }) + c.L(c.k.lineOnTop([x, y], [x + 7, y - 9], Z + 3.2, c.P), { tone: "lit", free: "start" }));
      [[x + 32, y - 14, 22], [x + 32, y + 14, 16]].slice(0, extra).forEach(([tx0, ty, h], i) => {
        const tx = tx0 + 22 * explode;
        c.put([tx, ty, 4 + h / 2], c.cyl(tx, ty, 8, 4, h, { tone: "mid" }, 32, 0.4) + c.L(c.k.sideArc(tx, ty, 8, 4 + h * 0.5, c.P, 24), { tone: "lo" }) + c.L(c.k.ring(tx, ty, 5, 4 + h, c.P, 24), { tone: "faint" }));
        if (explode > 0.05) c.label([tx + 8, ty, 4 + h], files[i]);
      });
      c.put([x - 14, y - 18, 5], c.lamp(x - 14, y - 18, 4));
      if (explode > 0.05) [[11, layers[0]], [26 + G, layers[1]], [Z + 1.5, layers[2]]].forEach(([z, text]) => c.label([x + 21, y - 4, z], text));
    },
  },

  buffer: {
    family: "machine", label: "Buffer rack", maps: "cache: Redis, Valkey, memcached, an LRU", fixtures: "Astronomy Shop (valkey cart)", w: 72, d: 36, top: 46,
    build(c, { x, y, fill = 0.7 }) {
      c.plinth(x, y, 72, 36);
      const W = 60, Dp = 22, x0 = x - W / 2, y0 = y - Dp / 2;
      for (const [px, py] of [[x0, y0], [x0 + W - 3, y0], [x0, y0 + Dp - 3], [x0 + W - 3, y0 + Dp - 3]]) c.put([px + 1.5, py + 1.5, 24], c.box(px, py, 3, 3, 4, 40, { tone: "mid" }, 2));
      let n = 0;
      for (const [si, z] of [[0, 10], [1, 24], [2, 38]].map(([i, z]) => [i, z])) {
        const slots = [];
        for (let j = 0; j < 6; j++) {
          n++;
          if ((n * 0.37) % 1 < fill) slots.push([[x0 + 7 + j * 9.4, y, z + 4], c.box(x0 + 4 + j * 9.4, y - 6, 6.5, 12, z + 1.2, 7, { tone: si === 1 && j === 2 ? "hi" : "mid", crease: "lo", lit: si === 1 && j === 2 }, 2, 0.3)]);
        }
        c.put([x, y, z + 0.6], c.S(c.k.slabOf({ x: x0 + 1, y: y0 + 1, w: W - 2, d: Dp - 2, r: 0.6 }, z, 1.2, c.P, 2), { tone: "mid" }), slots);
      }
      c.put([x0 + 1.5, y0 + 1.5, 46], c.lamp(x0 + 1.5, y0 + 1.5, 44));
    },
  },

  kiln: {
    family: "machine", label: "Kiln", maps: "LLM call: a model provider, an agent exec", fixtures: "Acquit (droid exec in docker)", w: 70, d: 66, top: 62,
    explodes: 34, parts: "sealed housing lifts off a fogged core (timed from the host only)",
    build(c, { x, y, explode = 0, layers = ["sealed housing", "agent exec · timed from host only"] }) {
      c.plinth(x, y, 70, 66);
      const H = 34 * explode;
      c.put([x, y, 6], c.box(x - 22, y - 20, 44, 40, 3, 3, { tone: "lo" }, 3, 0.4));
      c.put([x, y, 18], `<g class="fogcore">${c.cyl(x, y, 13, 6, 26, { tone: "mid" }, 36, 0.6)}${c.L(c.k.sideArc(x, y, 13, 16, c.P, 30) + c.k.sideArc(x, y, 13, 24, c.P, 30), { tone: "lo" })}${c.D([c.at([x, y, 32])], { size: 1, tone: "hi" })}</g>`);
      const body = { x: x - 28, y: y - 26, w: 56, d: 52, r: 7 };
      const [n, half] = c.frontFace(28, 26);
      const t = [-n[1], n[0]];
      const fp = (u, z) => c.at([x + n[0] * half + t[0] * u, y + n[1] * half + t[1] * u, z + H]);
      const arch = Array.from({ length: 13 }, (_, i) => { const th = Math.PI * (i / 12); return fp(-4 + Math.cos(th) * 12, 26 + Math.sin(th) * 8); });
      const door = `<path class="iso-line" data-tone="hi" d="${c.k.pathOf([fp(8, 12), fp(8, 26), ...arch, fp(-16, 26), fp(-16, 12)], true)}"/>` + Array.from({ length: 4 }, (_, i) => c.L(c.k.pathOf([fp(-13 + i * 6, 14), fp(-13 + i * 6, 24)]), { tone: "lit" })).join("");
      const shell = c.S(c.k.slabOf(body, 4 + H, 40, c.P, 6, 1.2), { tone: "mid", inner: c.L(c.k.sideSeam(body, 12 + H, c.P, 6), { tone: "lo" }) + door }) + c.L(c.k.planOutline(c.k.insetPlan(body, 4), 44 + H, c.P, 5), { tone: "faint" }) + c.D(c.k.corners(body, 4).map(([px, py]) => c.at([px, py, 44 + H])), { size: 0.5 });
      c.put([x, y, 24 + H], shell, [[[x + 14, y - 12, 52 + H], c.cyl(x + 14, y - 12, 5, 44 + H, 16, { tone: "mid" }) + c.L(c.k.sideArc(x + 14, y - 12, 5, 54 + H, c.P, 16), { tone: "lo" }) + c.cyl(x + 14, y - 12, 6.5, 60 + H, 2, { tone: "mid" })], [[x - 18, y - 16, 45 + H], c.lamp(x - 18, y - 16, 44 + H)]]);
      if (explode > 0.05) { c.label(c.rightOf(body, 30 + H), layers[0]); c.label([x + 13, y, 20], layers[1]); }
    },
  },

  dock: {
    family: "machine", label: "Dock", maps: "external API: PayPal, GitHub, Stripe, a model host", fixtures: "Acquit", w: 60, d: 50, top: 40,
    build(c, { x, y, variant }) {
      c.plinth(x + 6, y, 60, 50);
      c.put([x - 19, y - 15, 18], c.box(x - 22, y - 18, 6, 6, 4, 28, { tone: "mid" }, 2));
      c.put([x - 19, y + 15, 18], c.box(x - 22, y + 12, 6, 6, 4, 28, { tone: "mid" }, 2));
      c.put([x + 9, y, 12], c.box(x - 4, y - 12, 26, 24, 4, 16, { tone: "hi", crease: "lo" }, 3, 0.6) + c.L(c.k.planOutline(c.k.insetPlan({ x: x - 4, y: y - 12, w: 26, d: 24, r: 1 }, 3), 20, c.P, 2), { tone: "faint" }), [[[x + 9, y, 23], c.box(x + 4, y - 8, 10, 16, 20, 6, { tone: "mid" }, 2, 0.4)]]);
      c.put([x - 19, y, 34.5], c.box(x - 24, y - 20, 10, 40, 32, 5, { tone: "mid" }, 3, 0.5) + c.L(c.k.planOutline({ x: x - 22, y: y - 17, w: 6, d: 34, r: 1 }, 37, c.P, 2), { tone: "faint" }) + c.lamp(x - 19, y, 37));
    },
  },

  clock: {
    family: "machine", label: "Cam clock", maps: "scheduler: cron, setInterval, a tick loop", fixtures: "Acquit (30 s tick + house loop)", w: 44, d: 44, top: 34,
    build(c, { x, y, variant, phase = 0.3 }) {
      c.plinth(x, y, 44, 44);
      const cam = Array.from({ length: 28 }, (_, i) => { const th = (i / 28) * Math.PI * 2; const r = 10 + 5 * Math.max(0, Math.cos(th - phase * Math.PI * 2)) ** 2; return [x + Math.cos(th) * r, y + Math.sin(th) * r]; });
      const ticks = c.k.radialTicks(x, y, 15.5, 30, 5, 21, [0.9, 2], c.P);
      c.put([x, y, 12], c.box(x - 15, y - 15, 30, 30, 4, 16, { tone: "mid" }, 4, 0.6) + c.L(ticks.minor, { tone: "lo" }) + c.L(ticks.major, { tone: "mid" }) + c.screws({ x: x - 15, y: y - 15, w: 30, d: 30 }, 2.5, 20),
        [[[x, y, 24], c.prism(cam, 20, 6, { tone: "hi", crease: "lo" }) + c.D([c.at([x, y, 26.2])], { size: 0.9, tone: "hi" })],
          [[x + 15, y - 12, 26], c.cyl(x + 15, y - 12, 2, 20, 10, { tone: "mid" }) + c.L(c.k.lineOnTop([x + 15, y - 12], [x + 6 + Math.cos(phase * 6.283) * 2, y - 6], 29, c.P), { tone: "lit", free: "start" })],
          [[x - 12, y + 12, 21], c.lamp(x - 12, y + 12, 20)]]);
    },
  },

  retry: {
    family: "fitting", label: "Return loop", maps: "retries, backoff", fixtures: "Acquit (paypal x3)", w: 20, d: 20, top: 10,
    build(c, { x, y, along = "x" }) {
      const off = along === "x" ? [0, -11] : [11, 0];
      c.put([x, y, 4], c.box(x - 3, y - 3, 6, 6, 2, 4, { tone: "mid" }, 2) + c.L(c.k.ring(x + off[0], y + off[1], 8, 2.2, c.P, 32), { tone: "red", flow: true, className: "fit-flow" }) + c.D([c.at([x + off[0], y + off[1] - 8, 2.2])], { size: 0.9, tone: "hi" }));
    },
  },

  valve: {
    family: "fitting", label: "Valve", maps: "rate limiter, concurrency cap", fixtures: "Acquit (verifier concurrency 1)", w: 18, d: 18, top: 16,
    build(c, { x, y }) {
      const spokes = [0, 1.047, 2.094].map((a) => c.L(c.k.lineOnTop([x - Math.cos(a) * 6, y - Math.sin(a) * 6], [x + Math.cos(a) * 6, y + Math.sin(a) * 6], 14, c.P), { tone: "hi" })).join("");
      c.put([x, y, 7], c.box(x - 5, y - 5, 10, 10, 2, 6, { tone: "mid" }, 2, 0.4) + c.cyl(x, y, 1.6, 8, 6, { tone: "mid" }, 12), [[[x, y, 14], c.L(c.k.ring(x, y, 6.5, 14, c.P, 28), { tone: "hi" }) + spokes + c.D([c.at([x, y, 14])], { size: 0.8, tone: "hi" })]]);
    },
  },

  breaker: {
    family: "fitting", label: "Breaker", maps: "circuit breaker", fixtures: "Astronomy Shop (failure flags)", w: 18, d: 18, top: 14,
    build(c, { x, y, open = false }) {
      const tip = open ? [x + 6, y - 5] : [x - 6, y - 5];
      c.put([x, y, 6], c.box(x - 6, y - 5, 12, 10, 2, 8, { tone: "mid" }, 2, 0.4) + c.L(c.k.planOutline({ x: x - 4, y: y - 3, w: 8, d: 6, r: 0.5 }, 10, c.P, 2), { tone: "faint" }) + c.L(c.k.segment([x, y, 10], [tip[0], tip[1], 16], c.P), { tone: "hi" }) + c.D([c.at([tip[0], tip[1], 16])], { size: 1.2, tone: "hi" }) + c.lamp(x + 3.5, y + 2.5, 10));
    },
  },

  bot: {
    family: "floor", label: "Construction bot", maps: "an agent session at work", fixtures: "all (Droid, Claude Code, Codex)", w: 44, d: 44, top: 40,
    build(c, { x, y, target, heading }) {
      const k = c.k, V = c.view3, kk = k.cameraOf(c.P).k;
      const h = heading ?? (target ? Math.atan2(target[1] - y, target[0] - x) : 0.5);
      const f = [Math.cos(h), Math.sin(h), 0], s = [-Math.sin(h), Math.cos(h), 0], up = [0, 0, 1];
      const add = (...vs) => vs.reduce((a, b) => [a[0] + b[0], a[1] + b[1], a[2] + b[2]]);
      const mul = (v, m) => [v[0] * m, v[1] * m, v[2] * m];
      const dot = (a, b) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
      const R = 9, C = [x, y, 30];
      const circle = (o, u, v, r, n = 40) => Array.from({ length: n + 1 }, (_, i) => { const t = (i / n) * Math.PI * 2; return add(o, mul(u, r * Math.cos(t)), mul(v, r * Math.sin(t))); });
      const runs = (pts, ok) => { const out = []; let cur = []; for (const p of pts) { if (ok(p)) cur.push(c.at(p)); else { if (cur.length > 1) out.push(cur); cur = []; } } if (cur.length > 1) out.push(cur); return out.map((r) => k.pathOf(r)).join(""); };
      const rel = (p) => mul(add(p, mul(C, -1)), 1 / R);
      const faces = (p) => dot(rel(p), V) > 0.06;
      const hull = (pts) => { const p = pts.slice().sort((a, b) => a[0] - b[0] || a[1] - b[1]); const cr = (o, a, b) => (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0]); const lo = [], hi = []; for (const q of p) { while (lo.length > 1 && cr(lo[lo.length - 2], lo[lo.length - 1], q) <= 0) lo.pop(); lo.push(q); } for (const q of p.reverse()) { while (hi.length > 1 && cr(hi[hi.length - 2], hi[hi.length - 1], q) <= 0) hi.pop(); hi.push(q); } return lo.slice(0, -1).concat(hi.slice(0, -1)); };
      const poly = (pts, cls, tone = "hi") => `<path class="${cls}" d="${k.pathOf(pts, true)}"/><path class="iso-line iso-edge" data-tone="${tone}" d="${k.pathOf(pts, true)}"/>`;
      const lift = Math.sin(h * 3) * 0.0;

      const nacelle = (sg) => {
        const N = add(C, mul(s, sg * (R + 3.4)), [0, 0, -1.5 + lift]), rf = 4.4, Lh = 3.2;
        const Fc = add(N, mul(f, Lh)), Bc = add(N, mul(f, -Lh));
        const fr = circle(Fc, s, up, rf, 32), br = circle(Bc, s, up, rf, 32);
        let svg = c.L(k.segment(add(C, mul(s, sg * R * 0.8)), add(N, mul(s, -sg * rf * 0.6)), c.P), { tone: "mid" });
        svg += poly(hull([...fr, ...br].map(c.at)), "drone-hull");
        const frontOn = dot(f, V) > 0;
        const face = frontOn ? Fc : Bc, ring = frontOn ? fr : br;
        svg += poly(ring.map(c.at), "drone-duct");
        if (frontOn) {
          svg += c.L(Array.from({ length: 14 }, (_, i) => { const a0 = (i / 14) * Math.PI * 2; return k.pathOf([add(face, mul(s, 1.3 * Math.cos(a0)), mul(up, 1.3 * Math.sin(a0))), add(face, mul(s, 3.9 * Math.cos(a0 + 0.5)), mul(up, 3.9 * Math.sin(a0 + 0.5)))].map(c.at)); }).join(""), { tone: "mid", className: "blades" });
          svg += poly(circle(add(face, mul(f, 0.4)), s, up, 1.3, 14).map(c.at), "drone-hub");
        } else {
          svg += c.L(k.pathOf(circle(face, s, up, 3, 28).map(c.at)) + k.pathOf(circle(face, s, up, 1.6, 20).map(c.at)), { tone: "lo" });
        }
        svg += c.L(k.pathOf(circle(add(N, mul(f, frontOn ? -Lh * 0.1 : Lh * 0.1)), s, up, rf, 32).map(c.at)), { tone: "faint" });
        return { d: k.depthOf(N, c.P), svg };
      };
      const tail = (() => {
        const b = add(C, mul(f, -R * 0.75));
        const pts = [add(b, [0, 0, R * 0.25]), add(b, mul(f, -R * 0.75), [0, 0, R * 0.55]), add(b, mul(f, -R * 0.85), [0, 0, R * 0.3]), add(b, mul(f, -0.2), [0, 0, -R * 0.15])];
        return { d: k.depthOf(add(b, mul(f, -R * 0.4)), c.P), svg: poly(pts.map(c.at), "drone-hull", "mid") };
      })();

      const sc = c.at(C), r = R * kk;
      let shell = `<circle class="drone-shell" cx="${sc[0].toFixed(1)}" cy="${sc[1].toFixed(1)}" r="${r.toFixed(2)}"/>`;
      const seams = [];
      for (const hz of [0.38, -0.42]) seams.push(runs(circle(add(C, [0, 0, R * hz]), [1, 0, 0], [0, 1, 0], R * Math.sqrt(1 - hz * hz), 56), faces));
      const notEye = (p) => faces(p) && dot(rel(p), f) < 0.72;
      seams.push(runs(circle(C, s, up, R, 56), notEye));
      seams.push(runs(circle(C, mul(add(f, mul(s, 0.9)), 0.743), up, R, 56), notEye), runs(circle(C, mul(add(f, mul(s, -0.9)), 0.743), up, R, 56), notEye));
      shell += c.L(seams.join(""), { tone: "lo" });
      const grille = [];
      for (let i = -3; i <= 3; i++) for (const lat of [0.62, 0.7]) { const az = h + Math.PI + i * 0.13; const p = add(C, mul([Math.cos(az) * Math.sqrt(1 - lat * lat), Math.sin(az) * Math.sqrt(1 - lat * lat), lat], R)); if (faces(p)) grille.push(c.at(p)); }
      shell += c.D(grille, { size: 0.4, tone: "mid" });
      shell += `<circle class="iso-line iso-edge" data-tone="hi" fill="none" cx="${sc[0].toFixed(1)}" cy="${sc[1].toFixed(1)}" r="${r.toFixed(2)}"/>`;
      if (dot(f, V) > 0.12) {
        const eye = (deg) => { const a = (deg * Math.PI) / 180; return circle(add(C, mul(f, R * Math.cos(a))), s, up, R * Math.sin(a), 40).map(c.at); };
        shell += poly(eye(40), "drone-bezel", "mid") + poly(eye(31), "drone-glass", "hi");
        shell += c.L(k.pathOf(eye(20), true), { tone: "faint" });
        const vents = [-1, 1].map((sg) => k.pathOf([add(C, mul(f, R * 0.8), mul(up, sg * R * 0.52), mul(s, -R * 0.22)), add(C, mul(f, R * 0.8), mul(up, sg * R * 0.52), mul(s, R * 0.22))].map(c.at))).join("");
        shell += c.L(vents, { tone: "mid" });
        shell += `<g class="drone-eye">${c.D([c.at(add(C, mul(f, R)))], { size: 1.6, tone: "hi" })}</g>`;
      }
      shell += `<g class="drone-lamp">${c.D([c.at(add(C, [0, 0, R]))], { size: 1.1, tone: "hi" })}</g>`;
      const nac = [nacelle(1), nacelle(-1), tail];
      const dc = k.depthOf(C, c.P);
      const body = [...nac.filter((n) => n.d < dc).sort((a, b) => a.d - b.d).map((n) => n.svg), shell, ...nac.filter((n) => n.d >= dc).sort((a, b) => a.d - b.d).map((n) => n.svg)].join("");
      const beam = target ? c.L(k.segment(add(C, mul(f, R)), target, c.P), { tone: "lit", className: "beam" }) + c.D([c.at(target)], { size: 1.2, tone: "hi", pulse: true }) : "";
      c.floor(`<g class="drone-shadow">${c.halo({ x: x - R * 1.4, y: y - R * 1.4, w: R * 2.8, d: R * 2.8, r: R * 1.4 }, 0, 0.5)}</g>`);
      c.put([x, y, 30], `<g class="drone">${beam}${body}</g>`);
    },
  },

  scaffold: {
    family: "floor", label: "Scaffolding", maps: "an open PR or branch building a ghost", fixtures: "all", w: 48, d: 44, top: 32,
    build(c, { x, y, w = 48, d = 44, h = 32 }) {
      const sc = { x: x - w / 2, y: y - d / 2, w, d, r: 1 };
      c.floor(`<path class="iso-line scaffold" d="${c.k.planOutline(sc, 0, c.P, 2)}"/>`);
      for (const [px, py] of c.k.corners(sc, 0)) c.put([px, py, h / 2], `<path class="iso-line scaffold" d="${c.k.rise([px, py], 0, h, c.P)}"/><path class="iso-line scaffold" d="${c.k.rise([px, py], h - 0.1, h, c.P)}"/>`);
      c.put([x, y, h], `<path class="iso-line scaffold" d="${c.k.planOutline(sc, h, c.P, 2)}"/>`);
    },
  },

  crane: {
    family: "floor", label: "Crane", maps: "CI: a pipeline run lifting a branch through its checks", fixtures: "all (GitHub Actions, local harness)", w: 30, d: 92, top: 70,
    build(c, { x, y, span = 84, h = 64, at = 0.5, lift = 0.4, run = "running" }) {
      const hs = span / 2, gx = 11, lamp = { running: "run", passed: "ok", failed: "fail" }[run] || "ok";
      for (const sy of [-1, 1]) {
        const ey = y + sy * hs;
        c.floor(c.halo({ x: x - 15, y: ey - 3, w: 30, d: 6, r: 1 }, 0, 3) + c.S(c.k.slabOf({ x: x - 15, y: ey - 3, w: 30, d: 6, r: 1 }, 0, 3, c.P, 2, 0.4), { tone: "lo" }));
        for (const sx of [-1, 1]) c.put([x + sx * gx, ey, h / 2], c.box(x + sx * gx - 1.8, ey - 1.8, 3.6, 3.6, 3, h - 3, { tone: "mid" }, 2, 0.3));
        c.put([x, ey, h - 6], c.L(c.k.segment([x - gx, ey, h - 14], [x + gx, ey, h - 2], c.P) + c.k.segment([x + gx, ey, h - 14], [x - gx, ey, h - 2], c.P), { tone: "lo" }));
      }
      for (const sx of [-1, 1]) c.put([x + sx * gx, y, h + 2], c.box(x + sx * gx - 2.2, y - hs - 3, 4.4, span + 6, h, 4, { tone: "mid" }, 2, 0.4) + c.L(c.k.segment([x + sx * gx, y - hs, h + 4.1], [x + sx * gx, y + hs, h + 4.1], c.P), { tone: "faint" }));
      const ty = y - hs + 8 + (span - 16) * at, hookZ = h - 8 - (h - 22) * (1 - lift);
      const trolley = c.box(x - gx - 1, ty - 5, 2 * gx + 2, 10, h + 4, 5, { tone: "hi", crease: "lo" }, 2, 0.4) + c.lamp(x + 5, ty, h + 9, lamp);
      const cable = c.L(c.k.segment([x - 1.2, ty, h + 4], [x - 1.2, ty, hookZ + 6], c.P) + c.k.segment([x + 1.2, ty, h + 4], [x + 1.2, ty, hookZ + 6], c.P), { tone: "mid" });
      const load = c.box(x - 6, ty - 6, 12, 12, hookZ - 6, 8, { tone: run === "failed" ? "lo" : "hi", crease: "lo" }, 2, 0.4) + c.L(c.k.sideSeam({ x: x - 6, y: ty - 6, w: 12, d: 12, r: 0.6 }, hookZ - 2, c.P, 2), { tone: "lo" });
      c.put([x, ty, hookZ], `<g class="crane-load" data-run="${run}">${cable}${load}</g>`);
      c.put([x, ty, h + 6], trolley);
    },
  },

  inspection: {
    family: "floor", label: "Inspection station", maps: "you: a human reviewer", fixtures: "all", w: 66, d: 40, top: 30,
    build(c, { x, y, waiting = 3 }) {
      c.plinth(x + 8, y, 66, 40);
      c.put([x + 12, y, 11], c.box(x - 6, y - 12, 36, 24, 4, 14, { tone: "mid" }, 3, 0.6), [[[x + 7, y - 1, 18.5], c.box(x - 2, y - 8, 18, 14, 18, 1, { tone: "hi", crease: "lo" }, 2)], [[x + 22, y + 2, 21], c.cyl(x + 22, y + 2, 4, 18, 6, { tone: "mid" }) + c.L(c.k.ring(x + 22, y + 2, 2, 24, c.P, 14), { tone: "hi" })]]);
      for (let i = 0; i < Math.min(waiting, 4); i++) c.put([x - 18.5, y - 8.5 + i * 9, 7.5], c.box(x - 22, y - 12 + i * 9, 7, 7, 4, 7, { tone: "hi", crease: "lo" }, 2, 0.3));
      c.put([x + 34, y - 12, 5], c.lamp(x + 34, y - 12, 4));
    },
  },

  radar: {
    family: "machine", label: "Radar", maps: "telemetry exporter: an OTel SDK, a log shipper", fixtures: "Acquit (D13)", w: 40, d: 36, top: 46,
    build(c, { x, y }) {
      c.plinth(x, y, 40, 36);
      c.put([x, y, 10], c.box(x - 14, y - 12, 28, 24, 4, 12, { tone: "mid" }, 3, 0.5) + c.D(c.k.dotGrid(x - 10, y - 8, 4, 2, 3, 16, c.P), { size: 0.45, tone: "lo" }),
        [[[x + 4, y + 2, 28], c.cyl(x + 4, y + 2, 2, 16, 22, { tone: "mid" }, 14)],
          [[x + 4, y + 2, 40], c.cyl(x + 4, y + 2, 12, 38, 2.5, { tone: "hi", crease: "lo" }, 44, 0.4) + c.L(c.k.ring(x + 4, y + 2, 8, 40.5, c.P, 40), { tone: "lo" }) + c.L(c.k.ring(x + 4, y + 2, 4, 40.5, c.P, 28), { tone: "lo" }) + c.D([c.at([x + 4, y + 2, 41])], { size: 1, tone: "hi" })],
          [[x - 10, y + 8, 17], c.lamp(x - 10, y + 8, 16)]]);
    },
  },

  crate: {
    family: "machine", label: "Crate", maps: "anything not yet placed or recognized", fixtures: "all (unplaced tray)", w: 40, d: 40, top: 30,
    build(c, { x, y, variant = "?" }) {
      c.floor(c.halo({ x: x - 16, y: y - 16, w: 32, d: 32, r: 2 }, 0, 26));
      const b = { x: x - 16, y: y - 16, w: 32, d: 32, r: 1.5 };
      const bands = [9, 18].map((z) => c.L(c.k.sideSeam(b, z, c.P, 2), { tone: "lo" })).join("");
      c.put([x, y, 13], c.S(c.k.slabOf(b, 0, 26, c.P, 2, 0.6), { tone: "mid", inner: bands }) + c.L(c.k.planOutline(c.k.insetPlan(b, 3), 26, c.P, 2), { tone: "faint" }) + c.screws(b, 2.5, 26) + c.text(x, y + 3, 26, variant, { size: 9, anchor: "middle", tone: "mid" }));
    },
  },
};
