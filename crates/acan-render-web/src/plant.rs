use acan_render::{Polyline, Rect, Scene};

const MACHINE: [f32; 2] = [40.0, 24.0];
const PITCH: [f32; 2] = [100.0, 70.0];
const BRASS: [f32; 3] = [0.71, 0.57, 0.30];
const BELT: [f32; 4] = [0.42, 0.36, 0.25, 1.0];
const BELT_WIDTH: f32 = 3.0;

/// Deterministic synthetic plant: machines on a grid, belts routed with
/// right-angle doglegs from an output edge to an input edge.
pub fn generate(machines: usize, belts: usize) -> Scene {
    let cols = ((machines as f32 * 1.6).sqrt().ceil() as usize).max(1);
    let mut rng = Lcg(0x5eed_aca0);
    let rects: Vec<Rect> = (0..machines)
        .map(|i| {
            let (c, r) = ((i % cols) as f32, (i / cols) as f32);
            let shade = 0.85 + 0.3 * rng.unit();
            let w = MACHINE[0] * (0.8 + 0.5 * rng.unit());
            Rect {
                min: [c * PITCH[0], r * PITCH[1]],
                size: [w, MACHINE[1]],
                color: [BRASS[0] * shade, BRASS[1] * shade, BRASS[2] * shade, 1.0],
            }
        })
        .collect();

    let belts = (0..belts)
        .filter_map(|k| {
            let from = k % machines.max(1);
            let to = if k < machines {
                from + 1
            } else {
                from + cols + rng.below(3)
            };
            let to = to.min(machines.saturating_sub(1));
            if to == from {
                return None;
            }
            let (a, b) = (rects.get(from)?, rects.get(to)?);
            let start = [
                a.min[0] + a.size[0],
                a.center()[1] + 6.0 * (rng.unit() - 0.5),
            ];
            let end = [b.min[0], b.center()[1]];
            let mid_x = start[0] + (PITCH[0] - a.size[0]) * (0.3 + 0.4 * rng.unit());
            let points = if end[0] > start[0] {
                vec![start, [mid_x, start[1]], [mid_x, end[1]], end]
            } else {
                let lane = b.min[1] - 10.0;
                vec![
                    start,
                    [mid_x, start[1]],
                    [mid_x, lane],
                    [end[0] - 12.0, lane],
                    [end[0] - 12.0, end[1]],
                    end,
                ]
            };
            Some(Polyline {
                points,
                width: BELT_WIDTH,
                color: BELT,
            })
        })
        .collect();

    Scene { rects, belts }
}

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }

    fn unit(&mut self) -> f32 {
        (self.next() % 10_000) as f32 / 10_000.0
    }

    fn below(&mut self, n: u64) -> usize {
        (self.next() % n) as usize
    }
}
