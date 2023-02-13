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
