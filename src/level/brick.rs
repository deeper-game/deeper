use bevy::math::IVec3;
use crate::level::aabb::AABB;
use crate::level::integer_matrix::IMat3;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Brick<T> {
    pub bounding_box: AABB,
    pub contents: Vec<T>,
}

impl<T> Brick<T> {
    pub fn new(position: &IVec3, dimensions: &(u32, u32, u32)) -> Brick<T>
    where T: Clone + Default
    {
        let bounding_box = AABB {
            minimum: *position,
            maximum: *position + IVec3::new(dimensions.0 as i32 - 1,
                                            dimensions.1 as i32 - 1,
                                            dimensions.2 as i32 - 1),
        };
        let mut contents = Vec::new();
        contents.resize((dimensions.0 * dimensions.1 * dimensions.2) as usize,
                        Default::default());
        Brick { bounding_box, contents }
    }

    pub fn index(&self, position: &IVec3) -> &T {
        let (width, height, depth) = self.bounding_box.dimensions();
        let [x_i32, y_i32, z_i32] =
            (*position - self.bounding_box.minimum).to_array();
        assert!(x_i32 >= 0);
        assert!(y_i32 >= 0);
        assert!(z_i32 >= 0);
        let (x, y, z) = (x_i32 as u32, y_i32 as u32, z_i32 as u32);
        &self.contents[(width * height * z + width * y + x) as usize]
    }

    pub fn index_mut(&mut self, position: &IVec3) -> &mut T {
        let (width, height, depth) = self.bounding_box.dimensions();
        let [x_i32, y_i32, z_i32] =
            (*position - self.bounding_box.minimum).to_array();
        assert!(x_i32 >= 0);
        assert!(y_i32 >= 0);
        assert!(z_i32 >= 0);
        let (x, y, z) = (x_i32 as u32, y_i32 as u32, z_i32 as u32);
        &mut self.contents[(width * height * z + width * y + x) as usize]
    }

    pub fn rotate(&self, matrix: &IMat3) -> Brick<T> where T: Clone + Default {
        let bounding_box = self.bounding_box.rotate(matrix);
        let mut result: Brick<T> =
            Brick::new(&bounding_box.minimum, &bounding_box.dimensions());
        for pos in self.bounding_box.iter() {
            *result.index_mut(&matrix.mul_vec3(&pos)) =
                self.index(&pos).clone();
        }
        result
    }

    pub fn shift(&mut self, offset: &IVec3) {
        self.bounding_box = self.bounding_box.shift(offset);
    }

    pub fn blit(&mut self, other: &Brick<T>) where T: Clone + Default {
        let bounding_box = AABB::convex_hull(&[
            self.bounding_box.clone(),
            other.bounding_box.clone(),
        ]).unwrap();

        let mut result =
            Brick::new(&bounding_box.minimum, &bounding_box.dimensions());

        for pos in self.bounding_box.clone().iter() {
            *result.index_mut(&pos) = self.index(&pos).clone();
        }

        for pos in other.bounding_box.clone().iter() {
            *result.index_mut(&pos) = other.index(&pos).clone();
        }

        self.bounding_box = bounding_box;
        self.contents = result.contents;
    }

    pub fn slice(&self, region: &AABB) -> Option<Brick<T>>
    where T: Clone + Default
    {
        let intersection = AABB::intersection(&self.bounding_box, region)?;
        let mut result = Brick::new(&intersection.minimum,
                                    &intersection.dimensions());
        for pos in intersection.iter() {
            *result.index_mut(&pos) = self.index(&pos).clone();
        }
        Some(result)
    }
}
