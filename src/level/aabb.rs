use bevy::math::IVec3;
use crate::level::integer_matrix::IMat3;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AABB {
    pub minimum: IVec3,
    pub maximum: IVec3,
}

impl AABB {
    pub fn contains(&self, pos: &IVec3) -> bool {
        pos.cmpge(self.minimum).all() && pos.cmple(self.maximum).all()
    }

    pub fn shift_to_zero(&self) -> AABB {
        AABB {
            minimum: IVec3::ZERO,
            maximum: self.maximum - self.minimum
        }
    }

    pub fn shift(&self, offset: &IVec3) -> AABB {
        AABB {
            minimum: self.minimum + *offset,
            maximum: self.maximum + *offset,
        }
    }

    pub fn convex_hull(bounding_boxes: &[AABB]) -> Option<AABB> {
        if bounding_boxes.is_empty() {
            return None;
        }
        let mut minimum = bounding_boxes[0].minimum;
        let mut maximum = bounding_boxes[0].maximum;
        for bb in bounding_boxes {
            minimum = bb.clone().minimum.min(minimum);
            maximum = bb.clone().maximum.max(maximum);
        }
        Some(AABB { minimum, maximum })
    }

    pub fn dimensions(&self) -> (u32, u32, u32) {
        let dx = (self.maximum.x - self.minimum.x + 1) as u32;
        let dy = (self.maximum.y - self.minimum.y + 1) as u32;
        let dz = (self.maximum.z - self.minimum.z + 1) as u32;
        (dx, dy, dz)
    }

    pub fn iter(&self) -> impl Iterator<Item=IVec3> + '_ {
        let (dx, dy, dz) = self.dimensions();
        let size = dx * dy * dz;
        (0 .. size).map(move |i: u32| -> IVec3 {
            let (i, z) = (i % (dx * dy), i / (dx * dy));
            let (i, y) = (i % dx, i / dx);
            let (i, x) = (i % 1, i / 1);
            self.minimum + IVec3::new(x as i32, y as i32, z as i32)
        })
    }

    pub fn has_intersection(lhs: &AABB, rhs: &AABB) -> bool {
        ((lhs.minimum.cmple(rhs.minimum) & rhs.minimum.cmple(lhs.maximum))
            | (rhs.minimum.cmple(lhs.minimum) & lhs.minimum.cmple(rhs.maximum))).all()
    }

    pub fn intersection(lhs: &AABB, rhs: &AABB) -> Option<AABB> {
        if !AABB::has_intersection(lhs, rhs) { return None; }
        let p = lhs.minimum.max(rhs.minimum);
        let q = lhs.maximum.min(rhs.maximum);
        Some(AABB {
            minimum: p.min(q),
            maximum: p.max(q),
        })
    }

    pub fn has_same_shape(lhs: &AABB, rhs: &AABB) -> bool {
        (lhs.maximum - lhs.minimum) == (rhs.maximum - rhs.minimum)
    }

    pub fn rotate(&self, matrix: &IMat3) -> AABB {
        let p = matrix.mul_vec3(&self.minimum);
        let q = matrix.mul_vec3(&self.maximum);
        AABB {
            minimum: p.min(q),
            maximum: p.max(q),
        }
    }
}
