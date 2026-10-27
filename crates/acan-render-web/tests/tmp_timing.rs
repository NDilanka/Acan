// Temporary timing probe. Removed before commit.
use acan_render_web::layout::layout;
use acan_render_web::plant::stress;
use acan_render_web::stand_in::{models, scene};
use std::time::Instant;

#[test]
fn time_stress_pipeline() {
    for count in [200_usize, 500, 1000, 2000] {
        let t0 = Instant::now();
        let plant = stress(count);
        let t1 = Instant::now();
        let (kinds, edges) = plant.layout_input();
        let laid = layout(&kinds, &edges);
        let t2 = Instant::now();
        let s = scene(&laid);
        let t3 = Instant::now();
        let m = models();
        let t4 = Instant::now();
        eprintln!(
            "count {count}: stress {:?}, layout {:?} (belts {}), scene {:?} (instances {}), models {:?}, edges {}",
            t1 - t0,
            t2 - t1,
            laid.belts.len(),
            t3 - t2,
            s.instances.len(),
            t4 - t3,
            edges.len()
        );
        assert!(m.len() > 0);
    }
}
