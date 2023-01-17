use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bitvec::vec::BitVec;

pub struct Level {
    pub width: usize,
    pub height: usize,
    pub wall_map: BitVec,
    pub floor_map: BitVec,
}

impl Level {
    pub fn from_image(image: &Image) -> Self {
        assert_eq!(image.texture_descriptor.format,
                   TextureFormat::Rgba8UnormSrgb);
        let width = image.texture_descriptor.size.width as usize;
        let height = image.texture_descriptor.size.height as usize;
        let mut wall_map = BitVec::new();
        wall_map.resize(width * height, false);
        let mut floor_map = BitVec::new();
        floor_map.resize(width * height, false);
        for y in 0 .. height {
            for x in 0 .. width {
                let rgb = (
                    image.data[(width * y + x) * 4 + 0],
                    image.data[(width * y + x) * 4 + 1],
                    image.data[(width * y + x) * 4 + 2],
                );
                if rgb == (255, 0, 0) {
                    *wall_map.get_mut(width * y + x).unwrap() = true;
                }
                if rgb == (255, 255, 255) {
                    *floor_map.get_mut(width * y + x).unwrap() = true;
                }
            }
        }
        Level { width, height, wall_map, floor_map }
    }

    pub fn in_bounds(&self, x: usize, y: usize) -> bool {
        (x < self.width) && (y < self.height)
    }

    pub fn has_wall(&self, x: usize, y: usize) -> Option<bool> {
        if !self.in_bounds(x, y) {
            return None;
        }
        Some(self.wall_map[y * self.width + x])
    }

    pub fn has_floor(&self, x: usize, y: usize) -> Option<bool> {
        if !self.in_bounds(x, y) {
            return None;
        }
        Some(self.floor_map[y * self.width + x])
    }
}
