use bevy::prelude::*;
use std::ops::{Add, Sub, Mul};

pub trait Vector: Clone
    + Copy
    + Add<Self, Output=Self>
    + Sub<Self, Output=Self>
    + Mul<f32, Output=Self> {}
impl<V: Clone + Copy + Add<V, Output=V> + Sub<V, Output=V> + Mul<f32, Output=V>>
    Vector for V {}

pub struct PiecewiseLinearSpline<V> {
    control_points: Vec<(f32, V)>,
}

impl<V: Vector> PiecewiseLinearSpline<V> {
    pub fn new(points: &[(f32, V)]) -> PiecewiseLinearSpline<V> {
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

    pub fn interpolate(&self, input: f32) -> V {
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

pub struct QuadraticBezierCurve<V> {
    pub p0: V,
    pub p1: V,
    pub p2: V,
}

impl<V: Vector> QuadraticBezierCurve<V> {
    pub fn interpolate(&self, t: f32) -> V {
        ((self.p0 * (1.0 - t) + self.p1 * t) * (1.0 - t))
            + ((self.p1 * (1.0 - t) + self.p2 * t) * t)
    }

    pub fn derivative(&self, t: f32) -> V {
        unimplemented!()
    }
}

pub struct CubicBezierCurve<V> {
    pub p0: V,
    pub p1: V,
    pub p2: V,
    pub p3: V,
}

impl<V: Vector> CubicBezierCurve<V> {
    pub fn interpolate(&self, t: f32) -> V {
        let a = QuadraticBezierCurve {
            p0: self.p0,
            p1: self.p1,
            p2: self.p2,
        };
        let b = QuadraticBezierCurve {
            p0: self.p1,
            p1: self.p2,
            p2: self.p3,
        };
        (a.interpolate(t) * (1.0 - t)) + (b.interpolate(t) * t)
    }

    pub fn derivative(&self, t: f32) -> V {
        (self.p1 - self.p0) * (3.0 * (1.0 - t) * (1.0 - t))
            + (self.p2 - self.p1) * (6.0 * (1.0 - t) * t)
            + (self.p3 - self.p2) * (3.0 * t * t)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Frame {
    pub forward: Vec3, // t
    pub up: Vec3, // r
    pub right: Vec3, // s
}

impl Add<Frame> for Frame {
    type Output = Frame;
    fn add(self, rhs: Frame) -> Frame {
        return Frame {
            forward: self.forward + rhs.forward,
            up: self.up + rhs.up,
            right: self.right + rhs.right,
        };
    }
}

impl Sub<Frame> for Frame {
    type Output = Frame;
    fn sub(self, rhs: Frame) -> Frame {
        return Frame {
            forward: self.forward - rhs.forward,
            up: self.up - rhs.up,
            right: self.right - rhs.right,
        };
    }
}

impl Mul<f32> for Frame {
    type Output = Frame;
    fn mul(self, rhs: f32) -> Frame {
        return Frame {
            forward: self.forward * rhs,
            up: self.up * rhs,
            right: self.right * rhs,
        };
    }
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
