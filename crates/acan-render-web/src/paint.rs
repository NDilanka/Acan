//! Turns the plan and the view into the geometry the renderer draws.
//!
//! Machines stay grey until a clash stands on them. Agent bodies are the only saturated marks
//! on the plan. Pins, labels and cards are HTML, so they are not drawn here.

use acan_render::{Polyline, Rect, Scene};

use crate::layout::Plan;
use crate::mirror::View;

/// Belt stroke width, in world units.
const BELT_WIDTH: f32 = 4.0;
/// Belts are a neutral grey, so the machines and agents read first.
const BELT_COLOR: [f32; 4] = [0.80, 0.80, 0.78, 1.0];
/// Machine face, the white card the machine is printed on.
const FACE_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
/// Machine outline at rest, in world units of border.
const EDGE_WIDTH: f32 = 1.5;
const EDGE_COLOR: [f32; 4] = [0.72, 0.72, 0.70, 1.0];
/// Outline of a machine a clash stands on. The clash pin is the same red.
const ABNORMAL_EDGE_WIDTH: f32 = 3.0;
const ABNORMAL_EDGE_COLOR: [f32; 4] = [0.76, 0.25, 0.18, 1.0];

/// The scene for `view`, laid out on `plan`. `view.machines` is in plan order.
#[must_use]
pub fn scene(plan: &Plan, view: &View) -> Scene {
    let belts = plan
        .belts
        .iter()
        .map(|belt| Polyline {
            points: belt.points.clone(),
            width: BELT_WIDTH,
            color: BELT_COLOR,
        })
        .collect();

    let mut rects = Vec::with_capacity(plan.machines.len() * 2 + view.agents.len());
    for (machine, shown) in plan.machines.iter().zip(&view.machines) {
        let (grow, color) = if shown.abnormal {
            (ABNORMAL_EDGE_WIDTH, ABNORMAL_EDGE_COLOR)
        } else {
            (EDGE_WIDTH, EDGE_COLOR)
        };
        rects.push(Rect {
            min: [machine.min[0] - grow, machine.min[1] - grow],
            size: [machine.size[0] + 2.0 * grow, machine.size[1] + 2.0 * grow],
            color,
        });
        rects.push(Rect {
            min: machine.min,
            size: machine.size,
            color: FACE_COLOR,
        });
    }
    for agent in &view.agents {
        if let Some([x, y, w, h]) = agent.body {
            rects.push(Rect {
                min: [x, y],
                size: [w, h],
                color: rgba(agent.color),
            });
        }
    }

    Scene { rects, belts }
}

/// Parses `#rrggbb` into straight-alpha sRGB. A malformed channel reads as zero.
fn rgba(hex: &str) -> [f32; 4] {
    let digits = hex.trim_start_matches('#');
    let channel = |range: std::ops::Range<usize>| {
        digits
            .get(range)
            .and_then(|s| u8::from_str_radix(s, 16).ok())
            .map_or(0.0, |v| f32::from(v) / 255.0)
    };
    [channel(0..2), channel(2..4), channel(4..6), 1.0]
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::layout::{BeltJson, FileJson, MachineJson, PlantJson};
    use crate::mirror::{AgentView, MachineView};

    fn plant() -> PlantJson {
        PlantJson {
            machines: vec![
                MachineJson {
                    id: 1,
                    name: "api".to_owned(),
                    files: vec![10],
                },
                MachineJson {
                    id: 2,
                    name: "ui".to_owned(),
                    files: vec![11],
                },
            ],
            belts: vec![BeltJson { from: 2, to: 1 }],
            files: vec![
                FileJson {
                    id: 10,
                    path: "src/api.ts".to_owned(),
                },
                FileJson {
                    id: 11,
                    path: "src/ui.ts".to_owned(),
                },
            ],
        }
    }

    fn empty_view(abnormal_ui: bool) -> View {
        let plan = Plan::new(&plant());
        View {
            next_seq: acan_core::Seq::new(0),
            machines: plan
                .machines
                .iter()
                .map(|m| MachineView {
                    name: m.name.clone(),
                    min: m.min,
                    size: m.size,
                    abnormal: abnormal_ui && m.name == "ui",
                })
                .collect(),
            orders: Vec::new(),
            tasks: Vec::new(),
            agents: Vec::new(),
            questions: Vec::new(),
            clashes: Vec::new(),
        }
    }

    #[test]
    fn each_machine_is_an_outline_and_a_face() {
        let plan = Plan::new(&plant());
        let scene = scene(&plan, &empty_view(false));
        assert_eq!(scene.rects.len(), plan.machines.len() * 2);
        assert_eq!(scene.belts.len(), plan.belts.len());
        assert_eq!(
            scene.rects[1].color, FACE_COLOR,
            "the face follows its outline"
        );
    }

    #[test]
    fn a_machine_under_a_clash_gets_the_red_outline() {
        let plan = Plan::new(&plant());
        let at_rest = scene(&plan, &empty_view(false));
        let abnormal = scene(&plan, &empty_view(true));
        let ui = plan.index_of("ui").expect("ui is on the plan");
        assert_eq!(at_rest.rects[ui * 2].color, EDGE_COLOR);
        assert_eq!(abnormal.rects[ui * 2].color, ABNORMAL_EDGE_COLOR);
    }

    #[test]
    fn an_agent_body_is_drawn_in_its_color() {
        let plan = Plan::new(&plant());
        let mut view = empty_view(false);
        view.agents.push(AgentView {
            id: acan_core::AgentId::from_str("agent-a").expect("valid agent id"),
            color: "#ff8000",
            task: None,
            activity: "claimed",
            machine: Some("api".to_owned()),
            body: Some([1.0, 2.0, 3.0, 4.0]),
        });
        let scene = scene(&plan, &view);
        let body = scene.rects.last().expect("the body is the last rect");
        assert_eq!(body.min, [1.0, 2.0]);
        assert_eq!(body.size, [3.0, 4.0]);
        assert_eq!(body.color, [1.0, 128.0 / 255.0, 0.0, 1.0]);
    }

    #[test]
    fn a_hex_color_parses_per_channel() {
        assert_eq!(rgba("#000000"), [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(rgba("#ffffff"), [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(rgba("#zz0000")[0], 0.0, "a malformed channel reads as zero");
    }
}
