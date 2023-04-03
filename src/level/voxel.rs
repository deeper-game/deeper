use bevy::math::Quat;

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct Voxel {
    pub orientation: CardinalDir,
    pub shape: VoxelShape,
    pub texture: Texture,
    pub style: Style,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardinalDir {
    #[default]
    East,
    North,
    West,
    South,
}

impl CardinalDir {
    // Rotate the cardinal direction counterclockwise by 90 degrees.
    pub fn rotate_cw_90(&self) -> CardinalDir {
        match *self {
            CardinalDir::East => CardinalDir::South,
            CardinalDir::North => CardinalDir::East,
            CardinalDir::West => CardinalDir::North,
            CardinalDir::South => CardinalDir::West,
        }
    }

    // Rotate the cardinal direction counterclockwise by 90 degrees.
    pub fn rotate_ccw_90(&self) -> CardinalDir {
        match *self {
            CardinalDir::East => CardinalDir::North,
            CardinalDir::North => CardinalDir::West,
            CardinalDir::West => CardinalDir::South,
            CardinalDir::South => CardinalDir::East,
        }
    }

    // Convert a cardinal direction to a rotation about the Y axis, where east
    // is considered to be a 0 degree rotation.
    pub fn as_rotation(&self) -> Quat {
        Quat::from_rotation_y(match *self {
            CardinalDir::East => 0.0,
            CardinalDir::North => 1.0,
            CardinalDir::West => 2.0,
            CardinalDir::South => 3.0,
        } * std::f32::consts::FRAC_PI_2)
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    East,
    North,
    West,
    South,
    Down,
    Up,
}

#[derive(Clone, Default, Debug, PartialEq, Eq, Hash)]
pub enum VoxelShape {
    #[default]
    Air,
    Solid,
    Staircase,
    Roof { slope: fraction::Fraction },
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub enum Texture {
    #[default]
    None,
    Stone,
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub enum Style {
    #[default]
    Normal,
}
