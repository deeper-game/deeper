#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Voxel {
    pub orientation: CardinalDir,
    pub shape: VoxelShape,
    pub texture: Texture,
    pub style: Style,
}

impl Default for Voxel {
    fn default() -> Voxel {
        Voxel {
            orientation: CardinalDir::East,
            shape: VoxelShape::Air,
            texture: Texture::None,
            style: Style::Normal,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardinalDir {
    East,
    North,
    West,
    South,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum VoxelShape {
    Air,
    Solid,
    Staircase,
    Roof { slope: fraction::Fraction },
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Texture {
    None,
    Stone,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Style {
    Normal,
}
