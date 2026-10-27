//! Two-dimensional helpers for solids: winding, hulls, facing edges and runs of edges.

use std::cmp::Ordering;

pub(crate) type Point = [f32; 2];

/// Shoelace area of a ring, positive when it winds counter-clockwise in plan.
pub(crate) fn signed_area(ring: &[Point]) -> f32 {
    let cross: f32 = ring
        .iter()
        .zip(ring.iter().cycle().skip(1))
        .map(|(a, b)| a[0] * b[1] - b[0] * a[1])
        .sum();
    cross / 2.0
}

/// The ring, reversed if needed so that it winds counter-clockwise.
pub(crate) fn oriented(ring: &[Point]) -> Vec<Point> {
    if signed_area(ring) < 0.0 {
        ring.iter().rev().copied().collect()
    } else {
        ring.to_vec()
    }
}

/// The convex hull of `points`, by monotone chain, in the same order the kit produces.
pub(crate) fn convex_hull(points: &[Point]) -> Vec<Point> {
    let mut sorted = points.to_vec();
    sorted.sort_by(|a, b| {
        a[0].partial_cmp(&b[0])
            .unwrap_or(Ordering::Equal)
            .then(a[1].partial_cmp(&b[1]).unwrap_or(Ordering::Equal))
    });
    if sorted.len() < 3 {
        return sorted;
    }
    let cross = |o: Point, a: Point, b: Point| {
        (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
    };
    let mut lower: Vec<Point> = Vec::new();
    for &point in &sorted {
        while lower.len() >= 2
            && cross(lower[lower.len() - 2], lower[lower.len() - 1], point) <= 0.0
        {
            lower.pop();
        }
        lower.push(point);
    }
    let mut upper: Vec<Point> = Vec::new();
    for &point in sorted.iter().rev() {
        while upper.len() >= 2
            && cross(upper[upper.len() - 2], upper[upper.len() - 1], point) <= 0.0
        {
            upper.pop();
        }
        upper.push(point);
    }
    lower.pop();
    upper.pop();
    lower.extend(upper);
    lower
}

/// Outward unit normal of the edge from `a` to `b`. `counter` says the ring winds counter-clockwise.
pub(crate) fn edge_normal(a: Point, b: Point, counter: bool) -> Point {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let length = dx.hypot(dy);
    let length = if length > 0.0 { length } else { 1.0 };
    if counter {
        [dy / length, -dx / length]
    } else {
        [-dy / length, dx / length]
    }
}

/// Outward normal of every edge of `ring`, and whether each edge faces the viewer.
pub(crate) fn facing_edges(ring: &[Point], toward: Point) -> (Vec<Point>, Vec<bool>) {
    let count = ring.len();
    let counter = signed_area(ring) > 0.0;
    let normals: Vec<Point> = (0..count)
        .map(|i| edge_normal(ring[i], ring[(i + 1) % count], counter))
        .collect();
    let facing = normals
        .iter()
        .map(|n| n[0] * toward[0] + n[1] * toward[1] > 1e-9)
        .collect();
    (normals, facing)
}

/// The runs of consecutive edges whose flag equals `want`, wrapping past the first vertex.
///
/// Each run lists the vertices it passes through, so a run of `n` edges has `n + 1` vertices.
pub(crate) fn runs_of(flags: &[bool], want: bool) -> Vec<Vec<usize>> {
    let count = flags.len();
    if count == 0 {
        return Vec::new();
    }
    let Some(start) = (0..count).find(|&i| flags[i] != flags[(i + count - 1) % count]) else {
        return if flags[0] == want {
            vec![(0..=count).map(|i| i % count).collect()]
        } else {
            Vec::new()
        };
    };
    let mut runs = Vec::new();
    let mut run: Option<Vec<usize>> = None;
    for step in 0..count {
        let index = (start + step) % count;
        if flags[index] == want {
            run.get_or_insert_with(|| vec![index])
                .push((index + 1) % count);
        } else if let Some(finished) = run.take() {
            runs.push(finished);
        }
    }
    runs.extend(run);
    runs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hull_of_a_square_with_an_interior_point_is_the_square() {
        let points = [[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0], [1.0, 1.0]];
        let hull = convex_hull(&points);
        assert_eq!(hull.len(), 4);
        assert!(!hull.contains(&[1.0, 1.0]));
    }

    #[test]
    fn oriented_gives_counter_clockwise_rings() {
        let clockwise = [[0.0, 0.0], [0.0, 2.0], [2.0, 2.0], [2.0, 0.0]];
        assert!(signed_area(&oriented(&clockwise)) > 0.0);
    }

    #[test]
    fn runs_wrap_past_the_first_vertex() {
        let flags = [true, false, false, true];
        assert_eq!(runs_of(&flags, true), vec![vec![3, 0, 1]]);
        assert_eq!(runs_of(&flags, false), vec![vec![1, 2, 3]]);
    }

    #[test]
    fn all_true_flags_give_one_closed_run() {
        assert_eq!(runs_of(&[true, true, true], true), vec![vec![0, 1, 2, 0]]);
        assert!(runs_of(&[true, true, true], false).is_empty());
    }
}
