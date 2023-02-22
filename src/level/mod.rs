use bevy::prelude::*;
use bevy::math::IVec3;
use std::collections::HashSet;

use crate::level::aabb::AABB;
use crate::level::brick::Brick;
use crate::level::doorway::{Doorway, DoorwayMode};
use crate::level::erior::{Erior, blocks_to_aabbs};
use crate::level::integer_matrix::IMat3;
use crate::level::voxel::{Voxel, CardinalDir, VoxelShape, Texture, Style};

pub mod aabb;
pub mod brick;
pub mod doorway;
pub mod erior;
pub mod integer_matrix;
pub mod voxel;

#[derive(Clone, PartialEq, Eq)]
pub struct Map {
    pub room_boxes: Vec<AABB>,
    pub open_doorways: HashSet<Doorway>,
    pub voxels: Brick<Voxel>,
}

impl Map {
    pub fn room_gluing(
        starting_room: &Room, size: usize, rooms: &[Room]
    ) -> Map {
        let mut map = Map {
            room_boxes: vec![starting_room.voxels.bounding_box.clone()],
            open_doorways: starting_room.doorways.iter().cloned().collect(),
            voxels: starting_room.voxels.clone(),
        };

        let mut rooms_with_rotations = HashSet::new();
        for room in rooms {
            for rotated_room in room.all_y_rotations() {
                rooms_with_rotations.insert(rotated_room);
            }
        }

        let mut rng = rand::thread_rng();
        for iter in 0 .. size {
            println!("Room gluing iteration {}", iter);
            let mut candidates = Vec::<(DoorwayMatch, &Room)>::new();
            for open_doorway in &map.open_doorways {
                for room in &rooms_with_rotations {
                    for m in match_room_against_doorway(&map, room, open_doorway) {
                        candidates.push((m, room));
                    }
                }
            }
            if candidates.is_empty() {
                println!("Room gluing had no candidates, ending generation");
                break;
            }
            use rand::distributions::Distribution;
            let dist = rand::distributions::Uniform::from(0 .. candidates.len());
            let (ref doorway_match, room) = candidates[dist.sample(&mut rng)];
            let mut room_voxels = room.voxels.clone();
            room_voxels.shift(&doorway_match.offset);
            map.room_boxes.push(room_voxels.bounding_box.clone());
            for doorway in &room.doorways {
                if doorway != &doorway_match.room_doorway {
                    let shifted_doorway = doorway.shift(&doorway_match.offset);
                    map.open_doorways.insert(shifted_doorway);
                }
            }
            map.voxels.blit(&room_voxels);
            map.open_doorways.remove(&doorway_match.map_doorway);
        }

        map
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Room {
    doorways: Vec<Doorway>,
    voxels: Brick<Voxel>,
}

impl Room {
    pub fn parse(string: &str) -> Room {
        let mut solid_blocks = Vec::<IVec3>::new();
        let mut air_blocks = Vec::<IVec3>::new();
        let mut entrance_blocks = Vec::<IVec3>::new();
        let mut exit_blocks = Vec::<IVec3>::new();
        'lines: for line in string.lines() {
            for c in line.chars() {
                if c == ' ' {
                    continue;
                } else if c == '#' {
                    continue 'lines;
                } else {
                    break;
                }
            }
            let chunks = line.split_whitespace().collect::<Vec<&str>>();
            assert_eq!(chunks.len(), 4);
            let x = chunks[0].parse::<i32>().unwrap();
            let z = chunks[1].parse::<i32>().unwrap();
            let y = chunks[2].parse::<i32>().unwrap();
            let pos = IVec3::new(x, y, z);
            if chunks[3] == "ff0000" {
                exit_blocks.push(pos);
            } else if chunks[3] == "00ff00" {
                entrance_blocks.push(pos);
            } else if chunks[3] == "0000ff" {
                air_blocks.push(pos);
            } else {
                solid_blocks.push(pos);
            }
        }

        let mut all_unnormalized_blocks = Vec::new();
        all_unnormalized_blocks.append(&mut solid_blocks.clone());
        all_unnormalized_blocks.append(&mut air_blocks.clone());
        all_unnormalized_blocks.append(&mut entrance_blocks.clone());
        all_unnormalized_blocks.append(&mut exit_blocks.clone());

        let mut minimum = all_unnormalized_blocks[0];
        for pos in &all_unnormalized_blocks {
            minimum = minimum.min(*pos);
        }

        for pos in &mut solid_blocks {
            *pos = *pos - minimum;
        }
        for pos in &mut air_blocks {
            *pos = *pos - minimum;
        }
        for pos in &mut entrance_blocks {
            *pos = *pos - minimum;
        }
        for pos in &mut exit_blocks {
            *pos = *pos - minimum;
        }

        // The room should be watertight with respect to these blocks.
        let mut watertight_blocks = Vec::new();
        watertight_blocks.append(&mut solid_blocks.clone());
        watertight_blocks.append(&mut air_blocks.clone());
        watertight_blocks.append(&mut entrance_blocks.clone());
        watertight_blocks.append(&mut exit_blocks.clone());

        let erior = Erior::from_walls(&watertight_blocks);

        let mut doorways = Vec::new();
        let entrance_aabbs = blocks_to_aabbs(&entrance_blocks);
        let exit_aabbs = blocks_to_aabbs(&exit_blocks);
        for entrance_aabb in &entrance_aabbs {
            doorways.push(
                compute_doorway(&DoorwayMode::Entrance, entrance_aabb, &erior));
        }
        for exit_aabb in &exit_aabbs {
            doorways.push(
                compute_doorway(&DoorwayMode::Exit, exit_aabb, &erior));
        }

        let mut voxels = Brick::new(&erior.bounding_box.minimum,
                                    &erior.bounding_box.dimensions());

        for pos in solid_blocks {
            *voxels.index_mut(&pos) = Voxel {
                orientation: CardinalDir::East,
                shape: VoxelShape::Solid,
                texture: Texture::Stone,
                style: Style::Normal,
            };
        }

        Room { doorways, voxels }
    }

    pub fn reflect(&self) -> Room {
        unimplemented!()
    }

    pub fn rotate(&self, matrix: &IMat3) -> Room {
        let mut result = Room {
            doorways: vec![],
            voxels: Brick::new(&IVec3::ZERO, &(1, 1, 1)), // placeholder
        };
        for doorway in &self.doorways {
            result.doorways.push(doorway.rotate(matrix));
        }
        result.voxels = self.voxels.rotate(matrix);
        result
    }

    pub fn all_y_rotations(&self) -> Vec<Room> {
        let y = IMat3 {
            columns: [
                IVec3::new(0, 0, -1),
                IVec3::new(0, 1, 0),
                IVec3::new(1, 0, 0),
            ],
        };
        let rotations = [
            y,
            y.mul_mat3(&y),
            y.mul_mat3(&y).mul_mat3(&y),
        ];
        let mut result = Vec::new();
        // The identity is a proper rotation, so add in the original room.
        result.push(self.clone());
        for rotation in rotations {
            result.push(self.rotate(&rotation));
        }
        result
    }

    pub fn all_proper_rotations(&self) -> Vec<Room> {
        let x = IMat3 {
            columns: [
                IVec3::new(1, 0, 0),
                IVec3::new(0, 0, 1),
                IVec3::new(0, -1, 0),
            ],
        };
        let y = IMat3 {
            columns: [
                IVec3::new(0, 0, 1),
                IVec3::new(0, 1, 0),
                IVec3::new(-1, 0, 0),
            ],
        };
        // The 23 possible non-identity proper rotations of a cube are given by
        // the following compositions of two 90° rotations.
        // https://www.euclideanspace.com/maths/discrete/groups/categorise/finite/cube/index.htm
        let rotations = [
            x,
            y,
            x.mul_mat3(&x),
            x.mul_mat3(&y),
            y.mul_mat3(&x),
            y.mul_mat3(&y),
            x.mul_mat3(&x).mul_mat3(&x),
            x.mul_mat3(&x).mul_mat3(&y),
            x.mul_mat3(&y).mul_mat3(&x),
            x.mul_mat3(&y).mul_mat3(&y),
            y.mul_mat3(&x).mul_mat3(&x),
            y.mul_mat3(&y).mul_mat3(&x),
            y.mul_mat3(&y).mul_mat3(&y),
            x.mul_mat3(&x).mul_mat3(&x).mul_mat3(&y),
            x.mul_mat3(&x).mul_mat3(&y).mul_mat3(&x),
            x.mul_mat3(&x).mul_mat3(&y).mul_mat3(&y),
            x.mul_mat3(&y).mul_mat3(&x).mul_mat3(&x),
            x.mul_mat3(&y).mul_mat3(&y).mul_mat3(&y),
            y.mul_mat3(&x).mul_mat3(&x).mul_mat3(&x),
            y.mul_mat3(&y).mul_mat3(&y).mul_mat3(&x),
            x.mul_mat3(&x).mul_mat3(&x).mul_mat3(&y).mul_mat3(&x),
            x.mul_mat3(&y).mul_mat3(&x).mul_mat3(&x).mul_mat3(&x),
            x.mul_mat3(&y).mul_mat3(&y).mul_mat3(&y).mul_mat3(&x),
        ];
        let mut result = Vec::new();
        // The identity is a proper rotation, so add in the original room.
        result.push(self.clone());
        for rotation in rotations {
            result.push(self.rotate(&rotation));
        }
        result
    }

    pub fn all_improper_rotations(&self) -> Vec<Room> {
        let mut proper = self.all_proper_rotations();
        let mut improper = self.reflect().all_proper_rotations();
        improper.append(&mut proper);
        improper
    }
}

#[derive(Clone)]
struct DoorwayMatch {
    offset: IVec3,
    map_doorway: Doorway,
    room_doorway: Doorway,
}

fn match_room_against_doorway(
    map: &Map,
    room: &Room,
    map_doorway: &Doorway,
) -> Vec<DoorwayMatch> {
    let map_doorway_aabb = AABB {
        minimum: map_doorway.bounding_box.minimum + map_doorway.normal,
        maximum: map_doorway.bounding_box.maximum + map_doorway.normal,
    };
    let mut result = Vec::new();
    'doorway_loop: for room_doorway in &room.doorways {
        if !AABB::has_same_shape(&room_doorway.bounding_box,
                                 &map_doorway_aabb) {
            // TODO: allow partial matches in doorway shape
            println!("Room pair did not have the same shape: {:?} versus {:?}",
                     room_doorway.bounding_box, map_doorway_aabb);
            continue;
        }

        if !DoorwayMode::compatible(&room_doorway.mode, &map_doorway.mode) {
            continue;
        }

        if room_doorway.normal != -map_doorway.normal {
            continue;
        }

        let offset: IVec3 =
            //map.voxels.bounding_box.minimum
            map_doorway_aabb.minimum
            //- room.voxels.bounding_box.minimum
            - room_doorway.bounding_box.minimum;
        let mut shifted_room = room.voxels.clone();
        shifted_room.shift(&offset);

        if let Some(sliced) = map.voxels.slice(&shifted_room.bounding_box) {
            let mut changed_something = false;
            for pos in sliced.bounding_box.iter() {
                let voxel = sliced.index(&pos);
                let room_voxel = shifted_room.index(&pos);
                if voxel != room_voxel {
                    changed_something = true;
                }
                if (voxel != &Voxel::default()) && (voxel != room_voxel) {
                    continue 'doorway_loop;
                }
            }
            if !changed_something && sliced.bounding_box == shifted_room.bounding_box {
                continue 'doorway_loop;
            }
        }

        result.push(DoorwayMatch {
            offset,
            map_doorway: map_doorway.clone(),
            room_doorway: room_doorway.clone(),
        });
    }

    result
}

fn compute_doorway(
    mode: &DoorwayMode, aabb: &AABB, erior: &Brick<Erior>
) -> Doorway {
    let possible_normals = [
        IVec3::new(1, 0, 0),
        IVec3::new(-1, 0, 0),
        IVec3::new(0, 1, 0),
        IVec3::new(0, -1, 0),
        IVec3::new(0, 0, 1),
        IVec3::new(0, 0, -1),
    ];
    let mut compatible_normals = Vec::new();
    'outer: for possible_normal in possible_normals {
        for pos in aabb.iter() {
            let shifted = pos + possible_normal;
            if erior.bounding_box.contains(&shifted) {
                if erior.index(&shifted) != &Erior::Exterior {
                    continue 'outer;
                }
            }
        }
        compatible_normals.push(possible_normal);
    }
    assert_eq!(compatible_normals.len(), 1);
    let normal = compatible_normals[0];
    Doorway { mode: *mode, normal, bounding_box: aabb.clone() }
}
