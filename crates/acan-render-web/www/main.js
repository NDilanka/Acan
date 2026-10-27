// The product screen. The wasm renderer draws the plant. This page owns the HTML panels, the
// pins, the settle card, and the usage table, and talks to the server with the owner token.
//
// The token lives in sessionStorage only. It is never logged, and it never leaves the
// Authorization header or the first frame of the stream.
import init, { App } from "./pkg/acan_render_web.js";

const TOKEN_KEY = "acan.owner";
const USAGE_DELAY_MS = 800;
const RECONNECT_MS = 1000;
const ID_PREFIX = { order: "o", clash: "c", question: "q", task: "t" };

const $ = (id) => document.getElementById(id);
const canvas = $("plant");
const pins = $("pins");
const card = $("card");
const gate = $("gate");
const link = $("link");
const orderLine = $("order-line");
const usageBody = $("usage-body");

let app = null;
let token = null;
let view = null;
let renderedVersion = -1;
let selectedClash = null;
let socket = null;
let usageTimer = null;
let settling = false;
let placed = [];
let note = "";
const drag = { active: false, x: 0, y: 0 };

function setLink(text) {
  link.textContent = text;
}

function label(kind, id) {
  return `${ID_PREFIX[kind]}-${id}`;
}

function forgetToken() {
  sessionStorage.removeItem(TOKEN_KEY);
  token = null;
  if (socket) {
    const old = socket;
    socket = null;
    old.close();
  }
  gate.hidden = false;
  setLink("sign in");
}

async function get(path) {
  const res = await fetch(path, { headers: { Authorization: `Bearer ${token}` } });
  if (res.status === 401 || res.status === 403) {
    forgetToken();
    throw new Error("the owner token was refused");
  }
  if (!res.ok) {
    throw new Error(`${path} answered ${res.status}`);
  }
  return res.text();
}

async function start() {
  setLink("loading");
  app.set_plant(await get("/api/plant"));
  await reload();
  connect();
  refreshUsage();
}

async function reload() {
  app.load_log(await get("/api/events?from=0"));
}

function connect() {
  const next = app.next_seq();
  const scheme = location.protocol === "https:" ? "wss" : "ws";
  const ws = new WebSocket(`${scheme}://${location.host}/api/stream?from=${next}`);
  socket = ws;
  ws.onopen = () => {
    ws.send(JSON.stringify({ token }));
    setLink(`live · seq ${app.next_seq()}`);
  };
  ws.onmessage = (message) => {
    try {
      app.ingest(message.data);
    } catch (err) {
      console.error("the stream could not be applied; reloading the log", err);
      ws.close();
    }
  };
  ws.onclose = (event) => {
    if (socket !== ws) return;
    socket = null;
    if (!token) return;
    if (event.code === 1008) {
      forgetToken();
      return;
    }
    setLink("reconnecting");
    setTimeout(resync, RECONNECT_MS);
  };
}

async function resync() {
  if (!token) return;
  try {
    await reload();
    connect();
  } catch (err) {
    console.error(err);
    setLink("reconnecting");
    setTimeout(resync, RECONNECT_MS * 2);
  }
}

function frame() {
  try {
    if (app) {
      app.render();
      const version = app.view_version();
      if (version !== renderedVersion) {
        renderedVersion = version;
        view = JSON.parse(app.view_json());
        paint();
        scheduleUsage();
        if (socket) setLink(`live · seq ${app.next_seq()}`);
      }
      placePins();
    }
  } catch (err) {
    console.error(err);
  }
  requestAnimationFrame(frame);
}

function paint() {
  if (selectedClash === null) {
    const open = view.clashes.find((clash) => clash.state === "open" || clash.state === "proposed");
    if (open) selectedClash = open.id;
  }
  paintOrderLine();
  paintPins();
  paintCard();
}

function paintOrderLine() {
  const order = view.orders.find((item) => item.phase !== "cancelled" && item.phase !== "landed")
    ?? view.orders.at(-1);
  if (!order) {
    orderLine.textContent = "";
    return;
  }
  orderLine.textContent = `${label("order", order.id)} · ${order.machine} · ${order.phase.replace("_", " ")} · ${order.sentence}`;
}

function paintPins() {
  placed = [];
  pins.replaceChildren();
  const machines = new Map(view.machines.map((machine) => [machine.name, machine]));

  for (const machine of view.machines) {
    addPin("pin machine", machine.min[0], machine.min[1], machine.name, { dy: -18 });
  }
  for (const order of view.orders) {
    const home = machines.get(order.machine);
    if (!home || !order.at) continue;
    const x = home.min[0] + home.size[0] / 2;
    addPin("pin order", x, home.min[1], `${label("order", order.id)} ${order.phase.replace("_", " ")}`, { dy: -22, dx: -40 });
  }
  for (const agent of view.agents) {
    if (!agent.body) continue;
    const [x, y] = agent.body;
    addPin("pin flag", x, y, `${agent.id} · ${agent.activity}`, { dy: -22, color: agent.color });
  }
  for (const question of view.questions) {
    if (!question.at) continue;
    addPin("pin question", question.at[0], question.at[1], `${label("question", question.id)} ${question.text}`, { dy: -14 });
  }
  for (const clash of view.clashes) {
    if (!clash.at) continue;
    const el = addPin(`pin clash ${clash.state === "open" || clash.state === "proposed" ? "" : "resolved"}`, clash.at[0], clash.at[1], clash.label, { diamond: true });
    if (clash.id === selectedClash) el.classList.add("selected");
    el.addEventListener("click", () => {
      selectedClash = clash.id;
      note = "";
      paint();
    });
  }
}

function addPin(className, wx, wy, text, options = {}) {
  const el = document.createElement("div");
  el.className = className.trim();
  if (options.diamond) {
    const diamond = document.createElement("span");
    diamond.className = "diamond";
    el.append(diamond);
  }
  const span = document.createElement("span");
  span.className = options.diamond ? "text" : "";
  span.textContent = text;
  el.append(span);
  if (options.color) el.style.color = options.color;
  pins.append(el);
  placed.push({ el, wx, wy, dx: options.dx ?? 0, dy: options.dy ?? 0 });
  return el;
}

function placePins() {
  for (const pin of placed) {
    const [x, y] = app.world_to_screen(pin.wx, pin.wy);
    pin.el.style.transform = `translate(${x + pin.dx}px, ${y + pin.dy}px)`;
  }
}

function currentClash() {
  return view.clashes.find((clash) => clash.id === selectedClash) ?? null;
}

function paintCard() {
  const clash = currentClash();
  if (!clash) {
    card.hidden = true;
    return;
  }
  card.hidden = false;
  const unsettled = clash.state === "open" || clash.state === "proposed";
  const title = document.createElement("h2");
  title.textContent = clash.label;

  const kind = document.createElement("p");
  kind.className = "muted";
  kind.textContent = `${clash.kind.replace(/_/g, " ")} between ${label("task", clash.a)} and ${label("task", clash.b)}`;

  const body = document.createElement("p");
  body.textContent = clash.consequence ?? "explaining the overlap";
  if (clash.consequence === null) body.className = "muted";

  const children = [title, kind, body];
  if (clash.breaks) {
    const breaks = document.createElement("p");
    breaks.textContent = `breaks ${clash.breaks}`;
    children.push(breaks);
  }

  if (unsettled) {
    const list = document.createElement("ol");
    clash.options.forEach((option) => {
      const item = document.createElement("li");
      const key = document.createElement("span");
      key.className = "key";
      key.textContent = String(option.key);
      const button = document.createElement("button");
      button.type = "button";
      button.textContent = option.label;
      button.disabled = settling;
      button.addEventListener("click", () => settle(clash, option));
      item.append(key, button);
      list.append(item);
    });
    children.push(list);
    const hint = document.createElement("p");
    hint.className = "hint";
    hint.textContent = "press 1, 2 or 3 to settle";
    children.push(hint);
  } else {
    const resolved = document.createElement("p");
    resolved.textContent = clash.resolved ?? clash.state;
    children.push(resolved);
  }

  if (note) {
    const error = document.createElement("p");
    error.className = "note";
    error.textContent = note;
    children.push(error);
  }
  card.replaceChildren(...children);
}

async function settle(clash, option) {
  if (!token || settling) return;
  settling = true;
  note = "";
  paintCard();
  try {
    const res = await fetch("/api/commands", {
      method: "POST",
      headers: { Authorization: `Bearer ${token}`, "Content-Type": "application/json" },
      body: JSON.stringify({ cmd: "settle_clash", clash: clash.id, choice: option.choice }),
    });
    if (res.status === 401 || res.status === 403) {
      forgetToken();
    } else if (!res.ok) {
      note = `the origin refused: ${(await res.text()).slice(0, 160)}`;
    }
  } catch (err) {
    note = "the settle did not reach the server";
  } finally {
    settling = false;
    paintCard();
  }
}

function scheduleUsage() {
  if (usageTimer) return;
  usageTimer = setTimeout(() => {
    usageTimer = null;
    refreshUsage();
  }, USAGE_DELAY_MS);
}

async function refreshUsage() {
  if (!token) return;
  try {
    renderUsage(JSON.parse(await get("/api/usage")));
  } catch (err) {
    console.error(err);
  }
}

function renderUsage(rows) {
  if (rows.length === 0) {
    const empty = document.createElement("tr");
    const cell = document.createElement("td");
    cell.className = "muted";
    cell.colSpan = 6;
    cell.textContent = "no model calls yet";
    empty.append(cell);
    usageBody.replaceChildren(empty);
    return;
  }
  usageBody.replaceChildren(...rows.map((row) => {
    const tr = document.createElement("tr");
    const cells = [row.role, row.model, row.provider, row.calls, row.prompt_tokens, row.completion_tokens];
    cells.forEach((value, i) => {
      const td = document.createElement("td");
      td.textContent = String(value);
      if (i === 2) td.className = row.provider === "offline" ? "offline" : "live";
      tr.append(td);
    });
    return tr;
  }));
}

function bindCanvas() {
  canvas.addEventListener("pointerdown", (event) => {
    drag.active = true;
    drag.x = event.clientX;
    drag.y = event.clientY;
    canvas.setPointerCapture(event.pointerId);
  });
  canvas.addEventListener("pointermove", (event) => {
    if (!drag.active) return;
    app.pan(event.clientX - drag.x, event.clientY - drag.y);
    drag.x = event.clientX;
    drag.y = event.clientY;
  });
  canvas.addEventListener("pointerup", () => {
    drag.active = false;
  });
  canvas.addEventListener("wheel", (event) => {
    event.preventDefault();
    const rect = canvas.getBoundingClientRect();
    app.zoom_at(event.clientX - rect.left, event.clientY - rect.top, event.deltaY < 0 ? 1.1 : 1 / 1.1);
  }, { passive: false });
}

function sizeCanvas() {
  const rect = canvas.getBoundingClientRect();
  const dpr = window.devicePixelRatio || 1;
  canvas.width = Math.round(rect.width * dpr);
  canvas.height = Math.round(rect.height * dpr);
  if (app) app.resize(rect.width, rect.height, dpr);
}

function bindKeys() {
  window.addEventListener("keydown", (event) => {
    if (event.target instanceof HTMLInputElement) return;
    if (event.key === "f") {
      app.fit();
      return;
    }
    if (!/^[1-3]$/.test(event.key)) return;
    const clash = currentClash();
    if (!clash || (clash.state !== "open" && clash.state !== "proposed")) return;
    const option = clash.options[Number(event.key) - 1];
    if (option) settle(clash, option);
  });
  $("token-form").addEventListener("submit", (event) => {
    event.preventDefault();
    const value = $("token").value.trim();
    $("token").value = "";
    if (!value) return;
    sessionStorage.setItem(TOKEN_KEY, value);
    token = value;
    gate.hidden = true;
    start().catch((err) => {
      console.error(err);
      setLink("failed");
    });
  });
}

function adoptToken() {
  const fragment = new URLSearchParams(location.hash.slice(1)).get("token");
  if (fragment) {
    sessionStorage.setItem(TOKEN_KEY, fragment);
    history.replaceState(null, "", location.pathname + location.search);
  }
  return sessionStorage.getItem(TOKEN_KEY);
}

async function boot() {
  sizeCanvas();
  await init();
  try {
    app = await App.create(canvas, navigator.gpu ? "webgpu" : "webgl");
  } catch (err) {
    console.error(err);
    app = await App.create(canvas, "webgl");
  }
  sizeCanvas();
  bindCanvas();
  bindKeys();
  window.addEventListener("resize", sizeCanvas);
  requestAnimationFrame(frame);

  token = adoptToken();
  if (token) {
    start().catch((err) => {
      console.error(err);
      setLink("failed");
    });
  } else {
    gate.hidden = false;
    setLink("sign in");
  }
}

boot().catch((err) => {
  console.error(err);
  setLink("renderer failed");
});
