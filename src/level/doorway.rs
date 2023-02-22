use bevy::math::IVec3;
use crate::level::aabb::AABB;
use crate::level::integer_matrix::IMat3;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Doorway {
    pub mode: DoorwayMode,
    pub normal: IVec3,
    pub bounding_box: AABB,
}

impl Doorway {
    pub fn rotate(&self, matrix: &IMat3) -> Doorway {
        let mut result = self.clone();
        result.normal = matrix.mul_vec3(&result.normal);
        result.bounding_box = result.bounding_box.rotate(matrix);
        result
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DoorwayMode {
    Neither,
    Entrance,
    Exit,
}

impl DoorwayMode {
    pub fn inverse(&self) -> DoorwayMode {
        match *self {
            DoorwayMode::Neither => DoorwayMode::Neither,
            DoorwayMode::Entrance => DoorwayMode::Exit,
            DoorwayMode::Exit => DoorwayMode::Entrance,
        }
    }

    pub fn compatible(x: &DoorwayMode, y: &DoorwayMode) -> bool {
        *x == y.inverse()
    }
}
