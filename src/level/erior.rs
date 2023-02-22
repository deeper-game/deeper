use bevy::math::IVec3;
use std::collections::{HashSet, HashMap};
use crate::level::aabb::AABB;
use crate::level::brick::Brick;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Erior { Interior, Wall, Exterior }

impl Default for Erior {
    fn default() -> Erior {
        Erior::Interior
    }
}

impl Erior {
    pub fn from_walls(walls: &[IVec3]) -> Brick<Erior> {
        let aabb = positions_to_aabb(walls);
        let mut result =
            Brick::new(&aabb.minimum, &aabb.dimensions());
        for wall in walls {
            *result.index_mut(wall) = Erior::Wall;
        }
        let mut not_walls = Vec::new();
        for pos in aabb.iter() {
            if *result.index(&pos) != Erior::Wall {
                not_walls.push(pos);
            }
        }
        let blobs = blocks_to_blobs(&not_walls);
        for blob in blobs {
            let mut is_exterior_blob = false;
            for pos in &blob {
                is_exterior_blob = is_exterior_blob
                    || pos.x == aabb.minimum.x || pos.x == aabb.maximum.x
                    || pos.y == aabb.minimum.y || pos.y == aabb.maximum.y
                    || pos.z == aabb.minimum.z || pos.z == aabb.maximum.z;
                if is_exterior_blob {
                    break;
                }
            }
            if is_exterior_blob {
                for pos in &blob {
                    *result.index_mut(pos) = Erior::Exterior;
                }
            }
        }
        result
    }
}

pub fn blocks_to_aabbs(blocks: &[IVec3]) -> Vec<AABB> {
    let blobs = blocks_to_blobs(blocks);
    let mut result = Vec::<AABB>::new();
    for blob in blobs {
        let aabb = positions_to_aabb(&blob);
        assert!((aabb.dimensions().0 == 1)
                || (aabb.dimensions().1 == 1)
                || (aabb.dimensions().2 == 1));
        result.push(aabb);
    }
    result
}

fn positions_to_aabb(positions: &[IVec3]) -> AABB {
    let mut minimum = *positions.iter().nth(0).unwrap();
    let mut maximum = *positions.iter().nth(0).unwrap();
    for pos in positions {
        minimum = minimum.min(pos.clone());
        maximum = maximum.max(pos.clone());
    }
    AABB { minimum, maximum }
}

fn blocks_to_blobs(blocks: &[IVec3]) -> Vec<Vec<IVec3>> {
    use petgraph::unionfind::UnionFind;
    let mut blocks_map = HashMap::<IVec3, usize>::new();
    for (i, pos) in blocks.iter().enumerate() {
        blocks_map.insert(pos.clone(), i);
    }
    let mut blobs = UnionFind::<usize>::new(blocks.len());
    for (i, pos) in blocks.iter().enumerate() {
        let mut lambda = |neighboring_pos: IVec3| {
            if let Some(j) = blocks_map.get(&neighboring_pos) {
                blobs.union(i, *j);
            }
        };
        lambda(IVec3::new(pos.x + 1, pos.y, pos.z));
        lambda(IVec3::new(pos.x - 1, pos.y, pos.z));
        lambda(IVec3::new(pos.x, pos.y + 1, pos.z));
        lambda(IVec3::new(pos.x, pos.y - 1, pos.z));
        lambda(IVec3::new(pos.x, pos.y, pos.z + 1));
        lambda(IVec3::new(pos.x, pos.y, pos.z - 1));
    }
    let mut rep_to_class = HashMap::<usize, HashSet<IVec3>>::new();
    for i in 0 .. blocks.len() {
        let rep = blobs.find_mut(i);
        if let Some(set) = rep_to_class.get_mut(&rep) {
            set.insert(blocks[i]);
        } else {
            let mut set = HashSet::new();
            set.insert(blocks[i]);
            rep_to_class.insert(rep, set);
        }
    }
    let mut result = Vec::new();
    for (_, class) in rep_to_class {
        let mut class_vec = class.iter().cloned().collect::<Vec<IVec3>>();
        result.push(class_vec);
    }
    result
}
