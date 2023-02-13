use bevy::prelude::*;
use std::ops::{Add, Sub, Mul};

pub struct PiecewiseLinearSpline<S> {
    control_points: Vec<(f32, S)>,
}

impl<S: Clone + Copy + Add<S, Output=S> + Sub<S, Output=S> + Mul<f32, Output=S>> PiecewiseLinearSpline<S> {
    pub fn new(points: &[(f32, S)]) -> PiecewiseLinearSpline<S> {
        assert!(points.len() >= 2);
        assert!(points[0].0 == 0.0);
        assert!(points[points.len() - 1].0 == 1.0);
        for i in 1 .. points.len() {
            let (lower_x, lower_y) = points[i - 1];
            let (upper_x, upper_y) = points[i];
            assert!((lower_x >= 0.0) && (lower_x <= 1.0));
            assert!((upper_x >= 0.0) && (upper_x <= 1.0));
            assert!(lower_x < upper_x);
        }
        PiecewiseLinearSpline {
            control_points: points.iter().cloned().collect(),
        }
    }

    pub fn interpolate(&self, input: f32) -> S {
        for i in 1 .. self.control_points.len() {
            let input = f32::clamp(input, 0.0, 1.0);
            let (lower_x, lower_y) = self.control_points[i - 1];
            let (upper_x, upper_y) = self.control_points[i];
            if (lower_x <= input) && (input <= upper_x) {
                return
                    ((upper_y - lower_y) * ((input - lower_x) / (upper_x - lower_x)))
                    + lower_y;
            }
        }
        panic!("interpolate received value outside of range");
    }
}

#[derive(Clone, Debug)]
pub struct Frame {
    pub forward: Vec3, // t
    pub up: Vec3, // r
    pub right: Vec3, // s
}

// Compute a series of frames minimizing rotation subject to the given
// constraints:
//
// - The first frame will be equal to the given `initial_frame`
// - The tangent vectors at each point will be equal to the given `tangents`.
// - The frames are continuous with respect to the path given by the given
//   sampled `points`.
pub fn rotation_minimizing_frames(
    initial_frame: &Frame,
    points: &[Vec3],
    tangents: &[Vec3],
) -> Vec<Frame> {
    assert!(tangents[0] == initial_frame.forward);
    assert!(points.len() >= 1);
    let mut result = Vec::new();
    result.push(initial_frame.clone());
    for i in 0 .. points.len() - 1 {
        let v_1 = points[i + 1] - points[i];
        let c_1 = v_1.dot(v_1);
        let t_i = result[i].forward;
        let r_i = result[i].up;
        let s_i = result[i].right;
        let r_l_i = r_i - (2.0 * v_1.dot(r_i) / c_1) * v_1;
        let t_l_i = tangents[i] - (2.0 * v_1.dot(tangents[i]) / c_1) * v_1;
        let v_2 = tangents[i + 1] - t_l_i;
        let c_2 = v_2.dot(v_2);
        let forward = tangents[i + 1];
        let up = r_l_i - ((2.0 * v_2.dot(r_l_i) / c_2) * v_2);
        let right = forward.cross(up);
        result.push(Frame { forward, up, right });
    }
    result
}
