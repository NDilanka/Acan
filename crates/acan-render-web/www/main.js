import init, { App } from './pkg/acan_render_web.js';

const params = new URLSearchParams(location.search);
const backend = params.get('backend') ?? 'webgpu';
const machines = Number(params.get('machines') ?? 300);
const belts = Number(params.get('belts') ?? 600);
const LABELED = [0, Math.floor(machines / 2), machines - 1];

const canvas = document.getElementById('plant');
const labelLayer = document.getElementById('labels');
const hud = document.getElementById('hud');

const spike = {
  backend, machines, belts,
  coldStart: { moduleStartMs: performance.now() },
  rafCallbacks: 0,
  frames: 0,
  error: null,
};
window.spike = spike;

function sizeCanvas() {
  const dpr = window.devicePixelRatio || 1;
  canvas.width = Math.round(canvas.clientWidth * dpr);
  canvas.height = Math.round(canvas.clientHeight * dpr);
  return dpr;
}

let app;
try {
  sizeCanvas();
  await init();
  spike.coldStart.wasmReadyMs = performance.now();
  app = await App.create(canvas, backend, machines, belts);
  spike.coldStart.rendererReadyMs = performance.now();
  spike.adapter = app.backend();
} catch (e) {
  spike.error = String(e);
  hud.textContent = `failed: ${spike.error}`;
  throw e;
}

const labels = LABELED.map((i) => {
  const el = document.createElement('div');
  el.className = 'label';
  el.dataset.machine = String(i);
  el.textContent = `M-${String(i).padStart(3, '0')}`;
  labelLayer.append(el);
  return el;
});

function placeLabels() {
  for (const el of labels) {
    const [x, y] = app.machine_rect(Number(el.dataset.machine));
    const [sx, sy] = app.world_to_screen(x, y);
    el.style.left = `${sx}px`;
    el.style.top = `${sy - 2}px`;
  }
}

let scheduled = false;
let bench = null;

function invalidate() {
  if (!scheduled) {
    scheduled = true;
    requestAnimationFrame(frame);
  }
}

function frame(ts) {
  scheduled = false;
  spike.rafCallbacks++;
  if (bench) bench.step(ts);
  const t = performance.now();
  app.render();
  const cpu = performance.now() - t;
  spike.frames++;
  placeLabels();
  if (bench) bench.cpu.push(cpu);
  if (spike.frames === 1) {
    spike.coldStart.firstFrameMs = performance.now();
    requestAnimationFrame(() => { spike.coldStart.firstFramePresentedMs = performance.now(); });
  }
  hud.textContent = `${spike.adapter}\n${machines} machines, ${belts} belts, zoom ${app.zoom().toFixed(2)}, frames ${spike.frames}`;
  if (bench) bench.after();
}

const quantile = (xs, q) => {
  const s = [...xs].sort((a, b) => a - b);
  return s[Math.min(s.length - 1, Math.floor(q * s.length))];
};

window.runPanBench = (seconds = 10, pxPerFrame = 6) => new Promise((resolve) => {
  const intervals = [];
  const cpu = [];
  let last = null;
  let start = null;
  let angle = 0;
  bench = {
    cpu,
    step(ts) {
      if (start === null) start = ts;
      if (last !== null) intervals.push(ts - last);
      last = ts;
      angle += 0.02;
      app.pan(Math.cos(angle) * pxPerFrame, Math.sin(angle) * pxPerFrame);
    },
    after() {
      if (last - start < seconds * 1000) {
        invalidate();
        return;
      }
      bench = null;
      resolve({
        frames: intervals.length,
        intervalP50: quantile(intervals, 0.5),
        intervalP95: quantile(intervals, 0.95),
        intervalMax: Math.max(...intervals),
        cpuP50: quantile(cpu, 0.5),
        cpuP95: quantile(cpu, 0.95),
        cpuMax: Math.max(...cpu),
      });
    },
  };
  invalidate();
});

window.measureIdle = (seconds = 5) => new Promise((resolve) => {
  const before = { raf: spike.rafCallbacks, frames: spike.frames };
  setTimeout(() => resolve({
    seconds,
    rafCallbacks: spike.rafCallbacks - before.raf,
    frames: spike.frames - before.frames,
  }), seconds * 1000);
});

window.spikeApi = { pan: (dx, dy) => { app.pan(dx, dy); invalidate(); }, zoomAt: (x, y, f) => { app.zoom_at(x, y, f); invalidate(); } };

let drag = null;
canvas.addEventListener('pointerdown', (e) => {
  drag = { x: e.clientX, y: e.clientY };
  canvas.setPointerCapture(e.pointerId);
  canvas.focus();
});
canvas.addEventListener('pointermove', (e) => {
  if (!drag) return;
  app.pan(e.clientX - drag.x, e.clientY - drag.y);
  drag = { x: e.clientX, y: e.clientY };
  invalidate();
});
canvas.addEventListener('pointerup', () => { drag = null; });
canvas.addEventListener('wheel', (e) => {
  e.preventDefault();
  app.zoom_at(e.offsetX, e.offsetY, Math.exp(-e.deltaY * 0.0015));
  invalidate();
}, { passive: false });
window.addEventListener('keydown', (e) => {
  const step = 40;
  const center = [canvas.clientWidth / 2, canvas.clientHeight / 2];
  const actions = {
    ArrowLeft: () => app.pan(step, 0),
    ArrowRight: () => app.pan(-step, 0),
    ArrowUp: () => app.pan(0, step),
    ArrowDown: () => app.pan(0, -step),
    '+': () => app.zoom_at(...center, 1.25),
    '=': () => app.zoom_at(...center, 1.25),
    '-': () => app.zoom_at(...center, 0.8),
    '0': () => app.fit(),
  };
  const act = actions[e.key];
  if (act) {
    act();
    invalidate();
    e.preventDefault();
  }
});

new ResizeObserver(() => {
  const dpr = window.devicePixelRatio || 1;
  app.resize(canvas.clientWidth, canvas.clientHeight, dpr);
  invalidate();
}).observe(canvas);

invalidate();
