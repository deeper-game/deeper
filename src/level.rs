use bevy::prelude::*;
use bevy::math::IVec3;
use bevy::render::render_resource::TextureFormat;
use bitvec::vec::BitVec;
use std::collections::{HashSet, HashMap};

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
                        default());
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

#[derive(Clone, PartialEq, Eq)]
pub struct Map {
    pub room_boxes: Vec<AABB>,
    pub open_doorways: HashSet<Doorway>,
    pub voxels: Brick<Voxel>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DoorwayMode {
    Neither,
    Entrance,
    Exit,
}

impl DoorwayMode {
    fn inverse(&self) -> DoorwayMode {
        match *self {
            DoorwayMode::Neither => DoorwayMode::Neither,
            DoorwayMode::Entrance => DoorwayMode::Exit,
            DoorwayMode::Exit => DoorwayMode::Entrance,
        }
    }

    fn compatible(x: &DoorwayMode, y: &DoorwayMode) -> bool {
        *x == y.inverse()
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Doorway {
    mode: DoorwayMode,
    normal: IVec3,
    bounding_box: AABB,
}

impl Doorway {
    fn rotate(&self, matrix: &IMat3) -> Doorway {
        let mut result = self.clone();
        result.normal = matrix.mul_vec3(&result.normal);
        result.bounding_box = result.bounding_box.rotate(matrix);
        result
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

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Room {
    doorways: Vec<Doorway>,
    voxels: Brick<Voxel>,
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

fn positions_to_aabb(positions: &[IVec3]) -> AABB {
    let mut minimum = *positions.iter().nth(0).unwrap();
    let mut maximum = *positions.iter().nth(0).unwrap();
    for pos in positions {
        minimum = minimum.min(pos.clone());
        maximum = maximum.max(pos.clone());
    }
    AABB { minimum, maximum }
}

fn blocks_to_aabbs(blocks: &[IVec3]) -> Vec<AABB> {
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

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Erior { Interior, Wall, Exterior }

impl Default for Erior {
    fn default() -> Erior {
        Erior::Interior
    }
}

fn compute_erior(walls: &[IVec3]) -> Brick<Erior> {
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

        let erior = compute_erior(&watertight_blocks);

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
                IVec3::new(0, 0, 1),
                IVec3::new(0, 1, 0),
                IVec3::new(-1, 0, 0),
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

pub fn room_gluing(starting_room: &Room, size: usize, rooms: &[Room]) -> Map {
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
                map.open_doorways.insert(doorway.clone());
            }
        }
        map.voxels.blit(&room_voxels);
        map.open_doorways.remove(&doorway_match.map_doorway);
    }

    map
}

#[derive(Clone, Copy)]
pub struct IMat3 {
    pub columns: [IVec3; 3],
}

impl IMat3 {
    fn mul_vec3(&self, rhs: &IVec3) -> IVec3 {
        let x = IVec3::new(
            self.columns[0].x, self.columns[1].x, self.columns[2].x
        ).dot(*rhs);
        let y = IVec3::new(
            self.columns[0].y, self.columns[1].y, self.columns[2].y
        ).dot(*rhs);
        let z = IVec3::new(
            self.columns[0].z, self.columns[1].z, self.columns[2].z
        ).dot(*rhs);
        IVec3::new(x, y, z)
    }

    fn mul_mat3(&self, rhs: &IMat3) -> IMat3 {
        IMat3 {
            columns: [
                self.mul_vec3(&rhs.columns[0]),
                self.mul_vec3(&rhs.columns[1]),
                self.mul_vec3(&rhs.columns[2]),
            ],
        }
    }

    fn inverse(&self) -> Self {
        let mut tmp0 = self.columns[1].cross(self.columns[2]);
        let mut tmp1 = self.columns[2].cross(self.columns[0]);
        let mut tmp2 = self.columns[0].cross(self.columns[1]);
        let det = self.columns[2].dot(tmp2);
        assert_ne!(det, 0);
        assert_eq!(tmp0.x % det, 0);
        assert_eq!(tmp0.y % det, 0);
        assert_eq!(tmp0.z % det, 0);
        assert_eq!(tmp1.x % det, 0);
        assert_eq!(tmp1.y % det, 0);
        assert_eq!(tmp1.z % det, 0);
        assert_eq!(tmp2.x % det, 0);
        assert_eq!(tmp2.y % det, 0);
        assert_eq!(tmp2.z % det, 0);
        tmp0 /= det;
        tmp1 /= det;
        tmp2 /= det;
        IMat3 {
            columns: [
                IVec3::new(tmp0.x, tmp1.x, tmp2.x),
                IVec3::new(tmp0.y, tmp1.y, tmp2.y),
                IVec3::new(tmp0.z, tmp1.z, tmp2.z),
            ],
        }
    }
}
