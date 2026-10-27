//! The kit's isometric projection: azimuth 45 degrees, elevation 30 degrees, scale sqrt(1.6).
//! Plan units, y down. A figure is projected once with this, and the plant layout places
//! each machine with the same function.

const SIN_A: f32 = std::f32::consts::FRAC_1_SQRT_2;
const SIN_E: f32 = 0.5;
const COS_E: f32 = 0.866_025_4;
const K: f32 = 1.264_911;

/// Screen-plane position of the plan point `(x, y, z)`.
pub fn project(x: f32, y: f32, z: f32) -> [f32; 2] {
    [
        (x - y) * SIN_A * K,
        ((x + y) * SIN_A * SIN_E - z * COS_E) * K,
    ]
}

/// Painter depth. Larger is nearer.
pub fn depth(x: f32, y: f32, z: f32) -> f32 {
    (x + y) * SIN_A * COS_E + z * SIN_E
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: [f32; 2], b: [f32; 2]) -> bool {
        (a[0] - b[0]).abs() < 1e-4 && (a[1] - b[1]).abs() < 1e-4
    }

    #[test]
    fn origin_and_axes_match_the_kit() {
        assert!(close(project(0.0, 0.0, 0.0), [0.0, 0.0]));
        assert!(close(project(1.0, 0.0, 0.0), [0.894_427, 0.447_214]));
        assert!(close(project(0.0, 1.0, 0.0), [-0.894_427, 0.447_214]));
        assert!(close(project(0.0, 0.0, 1.0), [0.0, -1.095_445]));
    }

    #[test]
    fn nearer_points_have_larger_depth() {
        assert!(depth(10.0, 10.0, 0.0) > depth(0.0, 0.0, 0.0));
        assert!(depth(0.0, 0.0, 5.0) > depth(0.0, 0.0, 0.0));
    }
}
