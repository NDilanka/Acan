// The Acan product screen. The wasm renderer draws the plant and folds the origin's log into a
// view. This page owns the canvas input, the label and pin overlays, the HTML panels, the owner
// token and the stream. It draws only when something changed: input, a resize, new data, or a
// turn that is still easing. An idle page asks for no frames.
import init, { App } from "./pkg/acan_render_web.js";

const TOKEN_KEY = "acan.owner";
const RECONNECT_MS = 1000;
// The usage panel shows this many rows at rest. Alt shows every row.
const USAGE_REST_ROWS = 3;
const ID_PREFIX = { order: "o", clash: "c", question: "q", task: "t" };
const PAN_KEY_PX = 80;
const ZOOM_KEY = 1.25;
const WHEEL_RATE = 0.0015;
const TURN_DEG = 45;
const LABEL_OFFSET_PX = 6;
const LABEL_PAD_PX = 4;
// A press that moves less than this is a click, and a click within PIN_PICK_PX of a clash pin's
// tip opens its card.
const CLICK_PX = 4;
const PIN_PICK_PX = 18;
// The gap between the flags of one machine, stacked in a column above its bots.
const FLAG_ROW_GAP_PX = 4;
// A settled card shows its one line this long, then fades out over FADE_MS.
const FADE_HOLD_MS = 1500;
const FADE_MS = 400;
const SVG_NS = "http://www.w3.org/2000/svg";
// The point of view. The reticle is a square POV_SQUARE_PX across, with arms POV_ARM_PX long. A
// click counts as on a drone within DRONE_PICK_PX of its disc's edge. The chip and the frame fade
// over POV_FADE_MS, the time the CSS gives --pov-fade.
const POV_FADE_MS = 200;
const POV_SQUARE_PX = 40;
const POV_ARM_PX = 8;
const DRONE_PICK_PX = 6;
const RIDE_HINT = "<kbd>Esc</kbd> leave&nbsp;&nbsp;<kbd>Tab</kbd> next&nbsp;&nbsp;<kbd>Shift</kbd><kbd>Tab</kbd> back&nbsp;&nbsp;<kbd>Alt</kbd> words&nbsp;&nbsp;wheel closer";

const params = new URLSearchParams(location.search);
// `?pov=agent-a` rides that agent once the view places its drone, for screenshots.
let povWanted = params.get("pov");
// `?freeze=1` holds the plant still for screenshots. The page loads the log and folds only the
// events of its own settles, so the server's later events, such as a lease ending, do not move
// the marks.
const freeze = params.get("freeze") === "1";
// `?upto=N` folds only the events before seq N. A frozen screenshot uses it to show the demo at
// an earlier point, such as after a witness passes and before its task lands.
const upto = params.has("upto") ? Number(params.get("upto")) : null;
const $ = (id) => document.getElementById(id);
const canvas = $("plant");
const labelsEl = $("labels");
const leadersEl = $("leaders");
const flagsEl = $("flags");
const card = $("card");
const gate = $("gate");
const link = $("link");
const stale = $("stale");
const orderEl = $("order");
const orderLine = $("order-line");
const usagePanel = $("usage");
const usageBody = $("usage-body");
const repoEl = $("repo");
const noticeEl = $("notice");
const hintEl = $("hint");
const hintRest = hintEl.innerHTML;
const povEls = ["pov-vignette", "pov-frame", "pov-reticle", "pov-chip"].map((id) => $(id));
const povChip = $("pov-chip");
const povDot = $("pov-dot");
const povAgent = $("pov-agent");
const povLine = $("pov-line");
const povAlt = $("pov-alt");
const povBracket = $("pov-bracket");
const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)");

const timing = { started: performance.now() };
let app = null;
let token = null;
let view = null;
let renderedVersion = -1;
// The marks as the wasm side last gave them: the flags in order, and the open clash card.
let markState = { flags: [], card: null };
let flagEls = [];
let leaderEls = [];
let fadeTimer = null;
let socket = null;
let settling = false;
let note = "";
let scheduled = false;
let lastFrame = null;
let drag = null;
let press = null;
let altDown = false;
// The agent the page rides, or "" when it does not ride.
let ridden = "";
// Whether the view holds an order to show in the order island.
let hasOrder = false;
let labelGeneration = 0;
const labels = new Map();

function requestRender() {
  if (scheduled || app === null) return;
  scheduled = true;
  requestAnimationFrame(drawFrame);
}

// `now` is the frame time. The bench calls this with no argument, which reads as a frame with
// no time elapsed on a turn.
function drawFrame(now = performance.now()) {
  scheduled = false;
  const elapsed = lastFrame === null ? 0 : now - lastFrame;
  lastFrame = now;
  try {
    const turning = app.advance(elapsed);
    // A landed dot going quiet or a fade ending changes the flag list, so the flags are redrawn.
    if (app.take_flags_changed()) paintMarks();
    if (!app.render()) requestRender();
    if (timing.firstFrame === undefined) timing.firstFrame = performance.now();
    syncView();
    paintRide();
    placeLabels();
    if (turning) {
      requestRender();
    } else {
      lastFrame = null;
    }
  } catch (err) {
    console.error(err);
  }
}

// The view is rebuilt by the wasm side after the log changes. The panels are redrawn only when
// the version moves, so a camera move never rebuilds them.
function syncView() {
  const version = app.view_version();
  if (version === 0 || version === renderedVersion) return;
  renderedVersion = version;
  view = JSON.parse(app.view_json());
  paint();
  if (povWanted !== null && app.ride(povWanted)) povWanted = null;
  // The marks ease in from where they start, so the next frame must advance them.
  requestRender();
  if (socket) setLink(`live · seq ${app.next_seq()}`);
}

// The ride as the wasm side has it now: the chip's words, the reticle on the beam's target, and
// the frame, vignette and chip shown or hidden as the ride starts or ends.
function paintRide() {
  const ride = JSON.parse(app.pov_json());
  const agent = ride === null ? "" : ride.agent;
  if ((agent === "") !== (ridden === "")) rideToggled(agent !== "");
  ridden = agent;
  if (ride === null) return;
  document.documentElement.style.setProperty("--pov", ride.hue);
  paintChip(ride);
  paintReticle(ride.target_css);
}

function rideToggled(riding) {
  for (const el of povEls) {
    if (riding) showPov(el);
    else hidePov(el);
  }
  // The order island fades out with the point of view, and comes back when the ride ends.
  if (riding && !orderEl.hidden) hidePov(orderEl);
  if (!riding && hasOrder) showPov(orderEl);
  hintEl.innerHTML = riding ? RIDE_HINT : hintRest;
}

// Showing removes `hidden`, then adds `on` once the element has a style to fade from. Hiding
// removes `on` and sets `hidden` after the fade.
const povTimers = new WeakMap();
// The svg reticle has no `hidden` property (only HTML elements do), so the attribute is set and
// removed directly. Setting the property would leave the svg at display none.
function setHidden(el, hidden) {
  el.toggleAttribute("hidden", hidden);
}

function showPov(el) {
  clearTimeout(povTimers.get(el));
  setHidden(el, false);
  void el.getBoundingClientRect();
  requestAnimationFrame(() => el.classList.add("on"));
}

function hidePov(el) {
  el.classList.remove("on");
  povTimers.set(el, setTimeout(() => {
    setHidden(el, true);
  }, POV_FADE_MS));
}

function setText(el, text) {
  if (el.textContent !== text) el.textContent = text;
}

// The chip's words are the machine and the activity word its flag shows. Alt adds a row with the
// task and the file, and the row lines up under the agent's name.
function paintChip(ride) {
  povDot.classList.toggle("wait", ride.waiting);
  setText(povAgent, ride.agent);
  setText(povLine, `${ride.machine} \u00B7 ${ride.activity}`);
  povAlt.hidden = !altDown;
  if (!altDown) return;
  setText(povAlt, [ride.task, ride.file].filter((part) => part).join(" \u00B7 "));
  const pad = parseFloat(getComputedStyle(povChip).paddingLeft);
  povAlt.style.marginLeft = `${povAgent.offsetLeft - pad}px`;
}

// The reticle's four brackets sit on the target, a square POV_SQUARE_PX across. Off screen or
// behind the camera it is hidden at once, without a fade.
function paintReticle(target) {
  if (target === null) {
    povBracket.style.visibility = "hidden";
    return;
  }
  const [x, y] = target;
  const half = POV_SQUARE_PX / 2;
  const [x0, x1] = [x - half, x + half];
  const [y0, y1] = [y - half, y + half];
  const arm = POV_ARM_PX;
  povBracket.setAttribute("d", [
    `M${x0 + arm} ${y0}H${x0}V${y0 + arm}`,
    `M${x1 - arm} ${y0}H${x1}V${y0 + arm}`,
    `M${x0 + arm} ${y1}H${x0}V${y1 - arm}`,
    `M${x1 - arm} ${y1}H${x1}V${y1 - arm}`,
  ].join(""));
  povBracket.style.visibility = "visible";
}

// The riding agent's own flag is hidden: the chip replaces it.
function isRiderFlag(flag) {
  return ridden !== "" && flag !== undefined && flag.kind === "agent"
    && flag.text.startsWith(`${ridden} \u00B7 `);
}

// Labels come in priority order, larger and nearer first. A label that would overlap one
// already placed stays hidden, so the plant never shows a pile of names. The flags are placed
// first, so a machine name never hides one.
function placeLabels() {
  const width = canvas.clientWidth;
  const height = canvas.clientHeight;
  labelGeneration += 1;
  const placedBoxes = [];
  placeFlags(placedBoxes);
  const quads = app.label_layout(altDown);
  for (let k = 0; k < quads.length; k += 4) {
    const index = quads[k];
    const x = quads[k + 1];
    const y = quads[k + 2];
    const opacity = quads[k + 3];
    let el = labels.get(index);
    if (el === undefined) {
      el = makeLabel(index);
      labels.set(index, el);
      labelsEl.appendChild(el);
    }
    el.generation = labelGeneration;
    if (el.size === undefined || el.sizedAlt !== altDown) {
      el.size = [el.offsetWidth, el.offsetHeight];
      el.sizedAlt = altDown;
    }
    const [w, h] = el.size;
    const box = {
      left: x - w / 2 - LABEL_PAD_PX,
      right: x + w / 2 + LABEL_PAD_PX,
      top: y + LABEL_OFFSET_PX - LABEL_PAD_PX,
      bottom: y + LABEL_OFFSET_PX + h + LABEL_PAD_PX,
    };
    const inside = box.right >= 0 && box.left <= width && box.bottom >= 0 && box.top <= height;
    const clear = inside && placedBoxes.every((other) => !overlaps(box, other));
    if (clear) placedBoxes.push(box);
    el.style.visibility = clear ? "visible" : "hidden";
    el.style.opacity = String(opacity);
    el.style.transform = `translate(${x}px, ${y}px) translate(-50%, ${LABEL_OFFSET_PX}px)`;
  }
  for (const [index, el] of labels) {
    if (el.generation !== labelGeneration) {
      el.remove();
      labels.delete(index);
    }
  }
}

function overlaps(a, b) {
  return a.left < b.right && b.left < a.right && a.top < b.bottom && b.top < a.bottom;
}

function makeLabel(index) {
  const el = document.createElement("div");
  el.className = "label";
  el.textContent = app.machine_name(index);
  const files = app.machine_files(index);
  const count = document.createElement("span");
  count.className = "count";
  count.textContent = `${files} ${files === 1 ? "file" : "files"}`;
  el.appendChild(count);
  return el;
}

function label(kind, id) {
  return `${ID_PREFIX[kind]}-${id}`;
}

function setLink(text) {
  link.textContent = text;
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
  if (freeze) {
    setLink(`frozen · seq ${app.next_seq()}`);
  } else {
    connect();
  }
}

async function reload() {
  const log = await get("/api/events?from=0");
  app.load_log(upto === null ? log : keepBefore(log, upto));
  requestRender();
}

function keepBefore(text, seq) {
  return JSON.stringify(JSON.parse(text).filter((event) => event.seq < seq));
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
      requestRender();
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

function paint() {
  paintOrderLine();
  paintMarks();
  paintUsage();
  paintStale();
}

// A page older than the origin passes over the events it cannot read, and says so here.
function paintStale() {
  const passed = app.not_shown();
  stale.hidden = passed === 0;
  stale.textContent = passed === 0
    ? ""
    : `this page is older than the server: ${passed} ${passed === 1 ? "event" : "events"} not shown, rebuild the UI`;
}

function paintOrderLine() {
  const order = view.orders.find((item) => item.phase !== "cancelled" && item.phase !== "landed")
    ?? view.orders.at(-1);
  hasOrder = order !== undefined;
  orderLine.textContent = hasOrder
    ? `${label("order", order.id)} · ${order.machine} · ${order.phase.replace("_", " ")} · ${order.sentence}`
    : "";
  paintOrderShown();
}

// A ride holds the island: it fades with the point of view, so rideToggled shows and hides it.
function paintOrderShown() {
  if (ridden !== "") return;
  orderEl.hidden = !hasOrder;
  orderEl.classList.toggle("on", hasOrder);
}

// The marks come from the wasm side: each agent with its activity, the active order, each open
// clash and each question. They are rebuilt when the view changes, and placed every frame.
function paintMarks() {
  markState = JSON.parse(app.marks_json());
  flagsEl.replaceChildren();
  flagEls = markState.flags.map((flag) => {
    const el = flagElement(flag);
    flagsEl.append(el);
    return el;
  });
  paintCard();
}

function flagElement(flag) {
  const el = document.createElement("div");
  el.className = `flag ${flag.kind}`;
  el.classList.toggle("leaving", flag.leaving);
  if (flag.hue) {
    const dot = document.createElement("span");
    dot.className = "dot";
    dot.style.background = flag.hue;
    el.append(dot);
  }
  if (flag.kind === "landed") {
    const dot = document.createElement("span");
    dot.className = "dot land";
    el.append(dot);
    el.classList.toggle("quiet", flag.text === "");
  }
  if (flag.text !== "") {
    const text = document.createElement("span");
    text.textContent = flag.text;
    el.append(text);
  }
  if (flag.check) el.append(checkChip(flag.check));
  if (flag.waiting) {
    const amber = document.createElement("span");
    amber.className = "dot wait";
    el.append(amber);
  }
  if (flag.clash !== null) {
    el.addEventListener("click", () => openCard(flag.clash));
  }
  return el;
}

// A witness chip: a grey ring while the check is pending, a dark tick when it passed, a red dot
// when it failed. Alt adds its words, such as `w3 · passed`.
function checkChip(check) {
  const chip = document.createElement("span");
  chip.className = `chip ${check.state}`;
  chip.title = `${check.name} · ${check.state}`;
  chip.append(document.createElement("i"));
  if (altDown) {
    const words = document.createElement("span");
    words.textContent = `${check.name} · ${check.state}`;
    chip.append(words);
  }
  return chip;
}

// Flags are placed before the names and are never hidden for overlap; a flag is hidden only when
// it is off the canvas. An agent's flags stack in a column above their machine's drones, each with
// a leader line to its drone's crown, and a flag never sits on a drone because the column starts
// above the crowns. A clash label starts to the right of its diamond, and the order flag sits
// above its machine. A flag that would cover a flag placed before it moves up clear of it. Each
// flag keeps its place as the drones and diamonds move.
function placeFlags(placedBoxes) {
  // The drones come first, so no flag or name is placed over one.
  const discs = app.drone_layout();
  for (let k = 0; k < discs.length; k += 3) {
    const [x, y, r] = [discs[k], discs[k + 1], discs[k + 2]];
    placedBoxes.push({ left: x - r, right: x + r, top: y - r, bottom: y + r });
  }
  const quads = app.flag_layout();
  const columns = new Map();
  for (let k = 0; k < quads.length; k += 4) {
    const el = flagEls[quads[k]];
    if (el === undefined) continue;
    el.hidden = isRiderFlag(markState.flags[quads[k]]);
    if (el.hidden) continue;
    const machine = quads[k + 1];
    const x = quads[k + 2];
    const y = quads[k + 3];
    if (el.size === undefined) el.size = [el.offsetWidth, el.offsetHeight];
    if (machine >= 0) {
      if (!columns.has(machine)) columns.set(machine, []);
      columns.get(machine).push({ el, x, y });
      continue;
    }
    const [w, h] = el.size;
    const beside = el.classList.contains("clash") || el.classList.contains("landed");
    const left = beside ? x : x - w / 2;
    const top = clearOf(left, beside ? y - h / 2 : y - LABEL_OFFSET_PX - h, w, h, placedBoxes);
    placeFlag(el, left, top, placedBoxes);
  }
  let leaders = 0;
  for (const column of columns.values()) {
    const centre = column.reduce((sum, item) => sum + item.x, 0) / column.length;
    let bottom = Math.min(...column.map((item) => item.y)) - LABEL_OFFSET_PX;
    for (const item of column) {
      const [w, h] = item.el.size;
      const top = clearOf(centre - w / 2, bottom - h, w, h, placedBoxes);
      placeFlag(item.el, centre - w / 2, top, placedBoxes);
      drawLeader(leaders, centre, top + h, item.x, item.y);
      leaders += 1;
      bottom = top - FLAG_ROW_GAP_PX;
    }
  }
  for (let i = leaders; i < leaderEls.length; i += 1) {
    leaderEls[i].style.display = "none";
  }
}

// The top a flag takes so it clears every box placed before it. A flag that overlaps one moves up
// to sit on its top edge, and the check starts again, so the flag ends clear of all of them. A
// flag only ever moves up, so the search ends.
function clearOf(left, top, w, h, placedBoxes) {
  let y = top;
  let moved = true;
  while (moved) {
    moved = false;
    for (const box of placedBoxes) {
      const overlapsX = left - LABEL_PAD_PX < box.right && left + w + LABEL_PAD_PX > box.left;
      const overlapsY = y - LABEL_PAD_PX < box.bottom && y + h + LABEL_PAD_PX > box.top;
      if (overlapsX && overlapsY) {
        y = box.top - LABEL_PAD_PX - h;
        moved = true;
      }
    }
  }
  return y;
}

function placeFlag(el, left, top, placedBoxes) {
  const [w, h] = el.size;
  const box = {
    left: left - LABEL_PAD_PX,
    right: left + w + LABEL_PAD_PX,
    top: top - LABEL_PAD_PX,
    bottom: top + h + LABEL_PAD_PX,
  };
  placedBoxes.push(box);
  const inside = box.right >= 0 && box.left <= canvas.clientWidth
    && box.bottom >= 0 && box.top <= canvas.clientHeight;
  el.style.visibility = inside ? "visible" : "hidden";
  el.style.transform = `translate(${left}px, ${top}px)`;
}

// A thin line from a flag's bottom edge to its bot's crown. The leaders sit under the flags, so
// a flag hides the start of its own line.
function drawLeader(index, x1, y1, x2, y2) {
  let line = leaderEls[index];
  if (line === undefined) {
    line = document.createElementNS(SVG_NS, "line");
    leadersEl.append(line);
    leaderEls[index] = line;
  }
  line.setAttribute("x1", String(x1));
  line.setAttribute("y1", String(y1));
  line.setAttribute("x2", String(x2));
  line.setAttribute("y2", String(y2));
  line.style.display = "";
}

function openCard(index) {
  app.open_card(index);
  note = "";
  cancelFade();
  showCard();
}

function toggleCard() {
  app.toggle_card();
  note = "";
  cancelFade();
  showCard();
}

function showCard() {
  markState = JSON.parse(app.marks_json());
  paintCard();
  requestRender();
}

// The pin whose tip lies within PIN_PICK_PX of the CSS point, nearest first, or null.
function pinAt(x, y) {
  const quads = app.pin_layout();
  let found = null;
  let best = PIN_PICK_PX;
  for (let k = 0; k < quads.length; k += 3) {
    const distance = Math.hypot(quads[k + 1] - x, quads[k + 2] - y);
    if (distance < best) {
      best = distance;
      found = quads[k];
    }
  }
  return found;
}

// The clash card: the two declarations with their hues, and the witness the clash would break.
// Below them, the settlement options as a list, each with its key chip. The suggested option says
// so. Once the clash is settled, one line says how, and the card fades out.
function paintCard() {
  const data = markState.card;
  if (!data || !view) {
    card.hidden = true;
    return;
  }
  card.hidden = false;
  const clash = view.clashes[data.clash];
  const unsettled = clash.state === "open" || clash.state === "proposed";

  const head = document.createElement("div");
  head.className = "head";
  const title = document.createElement("h2");
  title.textContent = data.title;
  const close = document.createElement("button");
  close.type = "button";
  close.className = "close";
  close.textContent = "close";
  close.addEventListener("click", toggleCard);
  head.append(title, close);
  const children = [head];

  if (unsettled) {
    cancelFade();
    for (const row of data.rows) {
      if (row.kind !== "settle") children.push(cardRow(row));
    }
    const suggested = data.rows.find((row) => row.kind === "settle" && row.suggested);
    children.push(optionList(clash, suggested ? suggested.key : null));
  } else if (data.resolved) {
    const settled = document.createElement("p");
    settled.className = "resolved";
    settled.textContent = `settled · ${data.resolved}`;
    children.push(settled);
    scheduleFade();
  }
  if (note) {
    const error = document.createElement("p");
    error.className = "note";
    error.textContent = note;
    children.push(error);
  }
  card.replaceChildren(...children);
}

function cardRow(row) {
  const el = document.createElement("p");
  el.className = `row ${row.kind}`;
  if (row.kind === "declaration") {
    const dot = document.createElement("span");
    dot.className = "dot";
    dot.style.background = row.hue;
    const text = document.createElement("span");
    text.textContent = row.text;
    el.title = row.agent;
    el.append(dot, text);
  } else {
    el.textContent = row.text;
  }
  return el;
}

function optionList(clash, suggestedKey) {
  const list = document.createElement("ol");
  list.className = "options";
  for (const option of clash.options) {
    const item = document.createElement("li");
    const button = document.createElement("button");
    button.type = "button";
    button.className = "option";
    button.disabled = settling;
    const chip = document.createElement("kbd");
    chip.textContent = String(option.key);
    const label = document.createElement("span");
    label.textContent = option.label;
    button.append(chip, label);
    if (option.key === suggestedKey) {
      const tag = document.createElement("span");
      tag.className = "tag";
      tag.textContent = "suggested";
      button.append(tag);
    }
    button.addEventListener("click", () => settle(clash, option));
    item.append(button);
    list.append(item);
  }
  return list;
}

// A settled card shows its one line for FADE_HOLD_MS, then fades out and closes.
function scheduleFade() {
  if (fadeTimer !== null) return;
  fadeTimer = setTimeout(() => {
    card.classList.add("fading");
    fadeTimer = setTimeout(closeSettledCard, FADE_MS);
  }, FADE_HOLD_MS);
}

function cancelFade() {
  if (fadeTimer !== null) clearTimeout(fadeTimer);
  fadeTimer = null;
  card.classList.remove("fading");
}

function closeSettledCard() {
  fadeTimer = null;
  card.classList.remove("fading");
  app.close_card();
  markState = JSON.parse(app.marks_json());
  paintCard();
  requestRender();
}

// Keys 1, 2 and 3 settle the open clash on the card with the matching choice.
function settleByKey(key) {
  const data = markState.card;
  if (!data || !view) return;
  const clash = view.clashes[data.clash];
  if (!clash || (clash.state !== "open" && clash.state !== "proposed")) return;
  const option = clash.options[key - 1];
  if (option) settle(clash, option);
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
    } else if (freeze) {
      // The frozen page does not hear the stream, so it folds the settle's own events here.
      for (const event of await res.json()) app.ingest(JSON.stringify(event));
      requestRender();
    }
  } catch (err) {
    note = "the settle did not reach the server";
  } finally {
    settling = false;
    paintCard();
  }
}

// The usage panel reads the model calls the view has recorded: one row per model and step. It is
// hidden until the first call. At rest it shows the first few rows; Alt shows every row.
function paintUsage() {
  const rows = view === null ? [] : view.usage;
  usagePanel.hidden = rows.length === 0;
  const shown = altDown ? rows : rows.slice(0, USAGE_REST_ROWS);
  usageBody.replaceChildren(...shown.map(usageRow));
}

function usageRow(row) {
  const tr = document.createElement("tr");
  const cells = [row.role, row.model, row.calls, row.input, row.output];
  cells.forEach((value, i) => {
    const td = document.createElement("td");
    td.textContent = typeof value === "number" ? value.toLocaleString("en-US") : String(value);
    if (i >= 2) td.className = "num";
    tr.append(td);
  });
  return tr;
}

// Sets the drawing buffer to the CSS box times the device pixel ratio. The wasm side rounds
// the same product, so the two sizes agree.
function setDrawingSize() {
  const rect = canvas.getBoundingClientRect();
  const dpr = window.devicePixelRatio || 1;
  canvas.width = Math.max(1, Math.round(rect.width * dpr));
  canvas.height = Math.max(1, Math.round(rect.height * dpr));
  return { rect, dpr };
}

function sizeCanvas() {
  const { rect, dpr } = setDrawingSize();
  if (app !== null) {
    app.resize(rect.width, rect.height, dpr);
    requestRender();
  }
}

function showNotice(text) {
  noticeEl.textContent = text;
  noticeEl.hidden = false;
}

function zoomAtCentre(factor) {
  const rect = canvas.getBoundingClientRect();
  app.zoom_at(rect.width / 2, rect.height / 2, factor);
  requestRender();
}

function turn(degrees) {
  app.turn(degrees);
  requestRender();
}

// Whether a drone's disc lies under the CSS point, within its radius plus DRONE_PICK_PX.
function droneAt(x, y) {
  const discs = app.drone_layout();
  for (let k = 0; k < discs.length; k += 3) {
    if (Math.hypot(discs[k] - x, discs[k + 1] - y) <= discs[k + 2] + DRONE_PICK_PX) return true;
  }
  return false;
}

// Left drag moves the ground under the pointer, so the point you grab stays under the cursor.
// Right drag, or Alt with left drag, orbits and tilts about the ground point under the screen
// centre.
function bindCanvas() {
  canvas.addEventListener("contextmenu", (event) => event.preventDefault());

  canvas.addEventListener("pointerdown", (event) => {
    if (app === null || (event.button !== 0 && event.button !== 2)) return;
    canvas.setPointerCapture(event.pointerId);
    drag = {
      x: event.clientX,
      y: event.clientY,
      orbit: event.button === 2 || event.altKey,
    };
    press = event.button === 0 ? { x: event.clientX, y: event.clientY } : null;
    canvas.classList.add("dragging");
  });

  canvas.addEventListener("pointermove", (event) => {
    if (drag === null) return;
    if (drag.orbit) {
      app.orbit(event.clientX - drag.x, event.clientY - drag.y);
    } else {
      const rect = canvas.getBoundingClientRect();
      app.drag(
        drag.x - rect.left,
        drag.y - rect.top,
        event.clientX - rect.left,
        event.clientY - rect.top,
      );
    }
    drag.x = event.clientX;
    drag.y = event.clientY;
    requestRender();
  });

  // A press that did not move is a click. A click near a clash pin's tip opens its card; the
  // camera stays where it is.
  // A click on a clash pin opens its card. A click on a drone rides it, or switches the ride to
  // the drone nearest the click.
  const endDrag = (event) => {
    const pressed = press;
    press = null;
    drag = null;
    canvas.classList.remove("dragging");
    if (event.type !== "pointerup" || pressed === null) return;
    if (Math.hypot(event.clientX - pressed.x, event.clientY - pressed.y) >= CLICK_PX) return;
    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;
    const clash = pinAt(x, y);
    if (clash !== null) {
      openCard(clash);
    } else if (droneAt(x, y)) {
      app.ride_nearest(x, y);
      requestRender();
    }
  };
  canvas.addEventListener("pointerup", endDrag);
  canvas.addEventListener("pointercancel", endDrag);

  canvas.addEventListener(
    "wheel",
    (event) => {
      event.preventDefault();
      if (app === null) return;
      const rect = canvas.getBoundingClientRect();
      const lines = event.deltaMode === 1 ? 16 : 1;
      const factor = Math.exp(-event.deltaY * lines * WHEEL_RATE);
      // Wheel up brings the drone closer: the distance scale shrinks as the wheel turns up.
      if (ridden !== "") app.pov_scale(1 / factor);
      else app.zoom_at(event.clientX - rect.left, event.clientY - rect.top, factor);
      requestRender();
    },
    { passive: false },
  );
}

const ARROW_PAN = {
  ArrowLeft: [PAN_KEY_PX, 0],
  ArrowRight: [-PAN_KEY_PX, 0],
  ArrowUp: [0, PAN_KEY_PX],
  ArrowDown: [0, -PAN_KEY_PX],
};

function setAlt(down) {
  if (altDown === down) return;
  altDown = down;
  labelsEl.classList.toggle("alt", down);
  paintUsage();
  // The chips and the landed dots change their words with Alt, so the flags are redrawn.
  if (view !== null) paintMarks();
  requestRender();
}

function bindKeys() {
  window.addEventListener("keydown", (event) => {
    if (event.target instanceof HTMLInputElement) return;
    if (event.key === "Alt") {
      event.preventDefault();
      setAlt(true);
      return;
    }
    if (app === null || event.ctrlKey || event.metaKey) return;
    if (ARROW_PAN[event.key]) {
      event.preventDefault();
      app.pan(...ARROW_PAN[event.key]);
      requestRender();
      return;
    }
    if (event.key === "5" || event.code === "Numpad5") {
      if (!event.repeat) {
        app.toggle_projection();
        requestRender();
      }
      return;
    }
    if (/^[1-3]$/.test(event.key)) {
      settleByKey(Number(event.key));
      return;
    }
    switch (event.key) {
      case "c":
      case "C":
        if (!event.repeat) toggleCard();
        break;
      case "+":
      case "=":
        zoomAtCentre(ZOOM_KEY);
        break;
      case "-":
      case "_":
        zoomAtCentre(1 / ZOOM_KEY);
        break;
      case "0":
        app.fit();
        requestRender();
        break;
      case "f":
      case "F":
        if (!event.repeat) {
          const rect = canvas.getBoundingClientRect();
          app.ride_nearest(rect.width / 2, rect.height / 2);
          requestRender();
        }
        break;
      case "Tab":
        event.preventDefault();
        app.ride_step(event.shiftKey ? -1 : 1);
        requestRender();
        break;
      case "Escape":
        if (ridden !== "") {
          app.leave();
          requestRender();
        }
        break;
      case "q":
      case "Q":
        if (!event.repeat) turn(-TURN_DEG);
        break;
      case "e":
      case "E":
        if (!event.repeat) turn(TURN_DEG);
        break;
      default:
    }
  });

  window.addEventListener("keyup", (event) => {
    if (event.key === "Alt") setAlt(false);
  });
  window.addEventListener("blur", () => setAlt(false));

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

// The owner token arrives as `#token=` from `acan serve`. It is kept for the session only, then
// the fragment is removed so it does not sit in the address bar or the history.
function adoptToken() {
  const fragment = new URLSearchParams(location.hash.slice(1)).get("token");
  if (fragment) {
    sessionStorage.setItem(TOKEN_KEY, fragment);
    history.replaceState(null, "", location.pathname + location.search);
  }
  return sessionStorage.getItem(TOKEN_KEY);
}

async function createApp() {
  const backend = params.get("backend") || (navigator.gpu ? "webgpu" : "webgl");
  try {
    return await App.create(canvas, backend);
  } catch (err) {
    if (backend === "webgl") throw err;
    console.error(err);
    return App.create(canvas, "webgl");
  }
}

// The page loads its plant one of three ways. `?plant=stress&machines=N` builds a generated
// plant for the benchmark. `?plant=fixture` loads the demo plant with no log. Otherwise the
// page reads the live origin, which needs the owner token.
async function loadPlant() {
  const source = params.get("plant");
  if (source === "stress") {
    const machines = Number(params.get("machines") || 2000);
    const started = performance.now();
    app.set_stress(machines);
    timing.load = performance.now() - started;
  } else if (source === "fixture") {
    const response = await fetch("fixtures/demo-plant.json");
    if (!response.ok) throw new Error(`fixture HTTP ${response.status}`);
    app.set_plant(await response.text());
  } else {
    token = adoptToken();
    if (!token) {
      gate.hidden = false;
      setLink("sign in");
      return;
    }
    try {
      await start();
    } catch (err) {
      console.error(err);
      setLink("failed");
    }
  }
  repoEl.textContent = app.repo_name();
}

function followReducedMotion() {
  app.set_bob(!reducedMotion.matches);
  requestRender();
}

async function boot() {
  const started = performance.now();
  await init();
  timing.init = performance.now() - started;
  setDrawingSize();
  const createStarted = performance.now();
  app = await createApp();
  timing.create = performance.now() - createStarted;
  sizeCanvas();
  bindCanvas();
  bindKeys();
  window.addEventListener("resize", sizeCanvas);
  app.set_still(freeze);
  // Reduced motion turns the hover bob off, and the page follows the setting as it changes.
  followReducedMotion();
  reducedMotion.addEventListener("change", followReducedMotion);
  await loadPlant();
  // `?view=azimuth,elevation` turns to a camera angle in degrees, such as a low tilt.
  if (params.has("view")) {
    const [azimuth, elevation] = params.get("view").split(",").map(Number);
    app.view(azimuth, elevation);
  }
  // `?focus=agent-a&zoom=8` closes up on one agent's drone, at 8 times the fit zoom. The zoom is
  // the scale at the drone, so 8 draws it about 375 px tall. It applies once the log places the
  // drone, so the page can ask before the log arrives.
  if (params.has("focus")) {
    app.focus_agent(params.get("focus"), Number(params.get("zoom") || 8));
  }
  requestRender();
  if (params.get("bench") === "1") {
    const { installBench } = await import("./bench.js");
    installBench({ App, app, canvas, timing, redraw: drawFrame, setAlt });
  }
}

boot().catch((error) => {
  console.error(error);
  showNotice(String(error?.message ?? error));
});
