// Benchmarks, loaded only with `?bench=1`. They drive the same App the page uses. Frame
// intervals come from requestAnimationFrame timestamps, and CPU comes from timing App.render.
// Run them from the console: `await bench.pan(10)`, `await bench.idle(3)`, `bench.timing`.

const PAN_PX_PER_FRAME = 3;

const round = (x) => Math.round(x * 100) / 100;

function summary(values) {
  const sorted = [...values].sort((a, b) => a - b);
  const at = (q) => sorted[Math.min(sorted.length - 1, Math.floor(q * (sorted.length - 1)))];
  return {
    samples: sorted.length,
    p50: round(at(0.5)),
    p95: round(at(0.95)),
    max: round(sorted[sorted.length - 1]),
  };
}

export function installBench({ App, app, canvas, timing, redraw }) {
  let rafCalls = 0;
  let renders = 0;
  let renderMs = 0;
  const rawRaf = window.requestAnimationFrame.bind(window);
  window.requestAnimationFrame = (callback) => {
    rafCalls += 1;
    return rawRaf(callback);
  };
  const rawRender = App.prototype.render;
  App.prototype.render = function render(...args) {
    renders += 1;
    const started = performance.now();
    const result = rawRender.apply(this, args);
    renderMs += performance.now() - started;
    return result;
  };

  window.bench = {
    timing,

    /** Pans at a constant speed for `seconds`, one full frame (render and labels) per vsync. */
    pan(seconds = 10) {
      return new Promise((resolve) => {
        const intervals = [];
        const frameCpu = [];
        const rendererCpu = [];
        const begin = performance.now();
        let last = null;
        const tick = (now) => {
          if (last !== null) intervals.push(now - last);
          last = now;
          app.pan(PAN_PX_PER_FRAME, PAN_PX_PER_FRAME / 3);
          const rendererBefore = renderMs;
          const started = performance.now();
          redraw();
          frameCpu.push(performance.now() - started);
          rendererCpu.push(renderMs - rendererBefore);
          if (now - begin < seconds * 1000) {
            rawRaf(tick);
          } else {
            resolve({
              frames: frameCpu.length,
              intervalMs: summary(intervals),
              frameCpuMs: summary(frameCpu),
              rendererCpuMs: summary(rendererCpu),
              detail: app.detail(),
              machines: app.machine_count(),
              canvas: [canvas.width, canvas.height],
              backend: app.backend(),
            });
          }
        };
        rawRaf(tick);
      });
    },

    /** Counts requestAnimationFrame calls and renders over `seconds` with no input. */
    idle(seconds = 3) {
      const rafBefore = rafCalls;
      const rendersBefore = renders;
      return new Promise((resolve) => {
        setTimeout(() => {
          resolve({
            seconds,
            rafCalls: rafCalls - rafBefore,
            renders: renders - rendersBefore,
          });
        }, seconds * 1000);
      });
    },

    /** Zooms to `ratio` times fit on machine `index`, so a run can measure a chosen detail. */
    focus(index, ratio) {
      app.focus(index, ratio);
      redraw();
      return { detail: app.detail() };
    },
  };
}
