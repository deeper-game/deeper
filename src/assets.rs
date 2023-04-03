use std::collections::HashSet;
use bevy::prelude::*;
use bevy::gltf::*;
use bevy_asset_loader::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::level::voxel::{Direction, VoxelShape};
use crate::room_loader::TextFile;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_state::<crate::assets::GameState>()
            .insert_resource(VoxelMeshAssets::default())
            .add_loading_state(
                LoadingState::new(GameState::Loading)
                    .continue_to_state(GameState::Matchmaking))
            .add_collection_to_loading_state::<_, ImageAssets>(GameState::Loading)
            .add_collection_to_loading_state::<_, RoomAssets>(GameState::Loading)
            .add_collection_to_loading_state::<_, FontAssets>(GameState::Loading)
            .add_collection_to_loading_state::<_, GltfAssets>(GameState::Loading)
            .add_system(populate_voxel_meshes
                        .in_schedule(OnExit(GameState::Loading)));
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Default, States)]
pub enum GameState {
    #[default]
    Loading,
    Matchmaking,
    Ready,
}

#[derive(AssetCollection, Resource)]
pub struct ImageAssets {
    #[asset(path = "level.png")]
    pub level: Handle<Image>,
    #[asset(path = "empty.png")]
    pub empty: Handle<Image>,
    #[asset(path = "crosshair.png")]
    pub crosshair: Handle<Image>,
    #[asset(path = "coin.png")]
    pub coin: Handle<Image>,
    #[asset(path = "stone.png")]
    pub stone: Handle<Image>,
    #[asset(path = "solid.png")]
    pub solid: Handle<Image>,
    #[asset(path = "staircase.png")]
    pub staircase: Handle<Image>,
    #[asset(path = "block-debug.png")]
    pub block_debug: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct RoomAssets {
    #[asset(path = "rooms/room1.txt")]
    pub room1: Handle<TextFile>,
    #[asset(path = "rooms/room2.txt")]
    pub room2: Handle<TextFile>,
}

#[derive(AssetCollection, Resource)]
pub struct FontAssets {
    #[asset(path = "DejaVuSans.ttf")]
    pub dejavu_sans: Handle<Font>,
}

#[derive(AssetCollection, Resource)]
pub struct GltfAssets {
    #[asset(path = "gltf/stairs.glb")]
    pub staircase: Handle<Gltf>,
}

#[derive(Clone, Debug)]
pub struct MeshWithMaterial {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

#[derive(Clone, Debug)]
pub enum ColliderMode {
    None,
    ConvexHull,
    Mesh,
}

#[derive(Clone, Debug)]
pub struct VoxelMesh {
    pub neighbors: HashSet<Direction>,
    pub weathering: usize,
    pub meshes: Vec<MeshWithMaterial>,
    pub collider: Handle<Mesh>,
    pub collider_mode: ColliderMode,
    pub friction: Friction,
    pub ghost: bool,
}

impl VoxelMesh {
    pub fn spawn(
        &self,
        transform: &Transform,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) -> Entity {
        let collider = match self.collider_mode {
            ColliderMode::None => None,
            ColliderMode::ConvexHull => {
                use bevy::render::mesh::VertexAttributeValues;
                let VertexAttributeValues::Float32x3(vec) =
                    meshes.get(&self.collider).unwrap()
                    .attribute(Mesh::ATTRIBUTE_POSITION).unwrap()
                else { panic!("Mesh position attribute wasn't 3D float32"); };
                let mut collider_points = Vec::new();
                for pos in vec {
                    let mut point = Vec3::new(pos[0], pos[1], pos[2]);
                    collider_points.push(point);
                }
                Some(Collider::convex_hull(&collider_points).unwrap())
            },
            ColliderMode::Mesh => {
                Some(Collider::from_bevy_mesh(
                    meshes.get(&self.collider).unwrap(),
                    &ComputedColliderShape::TriMesh).unwrap())
            },
        };


        let mut ecmds = commands.spawn((
            SpatialBundle::from(transform.clone()),
            self.friction,
        ));
        if let Some(c) = collider {
            ecmds.insert(c);
        }

        ecmds.with_children(|parent| {
            for mwm in &self.meshes {
                let mut material_hdl = mwm.material.clone();
                if self.ghost {
                    let mut material =
                        materials.get(&material_hdl).unwrap().clone();
                    material.alpha_mode = AlphaMode::Blend;
                    material.base_color.set_a(0.2 * material.base_color.a());
                    material.base_color_texture = None;
                    material.emissive_texture = None;
                    material.metallic_roughness_texture = None;
                    material.normal_map_texture = None;
                    material.occlusion_texture = None;
                    material_hdl = materials.add(material);
                }
                parent.spawn(PbrBundle {
                    mesh: mwm.mesh.clone(),
                    material: material_hdl,
                    ..default()
                });
            }
        });

        ecmds.id()
    }
}

#[derive(Clone, Debug, Default, Resource)]
pub struct VoxelMeshAssets {
    pub solid: Vec<VoxelMesh>,
    pub staircase: Vec<VoxelMesh>,
}

impl VoxelMeshAssets {
    pub fn index(&self, shape: &VoxelShape) -> Option<VoxelMesh> {
        match *shape {
            VoxelShape::Air => None,
            VoxelShape::Solid => Some(self.solid[0].clone()),
            VoxelShape::Staircase => Some(self.staircase[0].clone()),
            VoxelShape::Roof { .. } => None,
        }
    }
}

pub fn apply_transform_to_mesh(transform: &Transform, mesh: &mut Mesh) {
}

pub fn node_to_meshes(node: &GltfNode,
                      meshes: &mut Assets<Mesh>) -> Vec<MeshWithMaterial> {
    let mut result = Vec::new();
    for child in &node.children {
        for mut mwm in node_to_meshes(child, meshes) {
            apply_transform_to_mesh(&node.transform,
                                    meshes.get_mut(&mwm.mesh).unwrap());
            result.push(mwm);
        }
    }
    result
}

pub fn populate_voxel_meshes(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    gltfs: Res<Assets<Gltf>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    gltf_assets: Res<GltfAssets>,
    mut vma: ResMut<VoxelMeshAssets>,
) {
    let get_mesh = |handle: &Handle<GltfMesh>| -> &GltfMesh {
        gltf_meshes.get(handle).unwrap()
    };

    let default_material = materials.add(StandardMaterial {
        ..default()
    });

    {
        let mut cube = Mesh::from(shape::Cube { size: 1.0 });
        use bevy::render::mesh::VertexAttributeValues;
        let positions = cube.attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap();
        let mut pos_vec = Vec::<[f32; 3]>::try_from(positions.clone()).unwrap();
        for [ref mut x, ref mut y, ref mut z] in &mut pos_vec {
            *x += 0.5;
            *y += 0.5;
            *z -= 0.5;
        }
        *positions = VertexAttributeValues::from(pos_vec);
        let cube_handle = meshes.add(cube);

        vma.solid = vec![
            VoxelMesh {
                neighbors: HashSet::new(),
                weathering: 0,
                meshes: vec![
                    MeshWithMaterial {
                        mesh: cube_handle.clone(),
                        material: default_material.clone(),
                    },
                ],
                collider: cube_handle.clone(),
                collider_mode: ColliderMode::Mesh,
                friction: Friction::default(),
                ghost: false,
            },
        ];
    }

    {
        let meshes = &gltfs.get(&gltf_assets.staircase).unwrap().named_meshes;

        let collider_gltf_mesh = get_mesh(&meshes["ColliderMesh"]);
        assert_eq!(collider_gltf_mesh.primitives.len(), 1);
        let collider = &collider_gltf_mesh.primitives[0].mesh;

        let neighbors_empty = HashSet::new();
        let mut neighbors_l = HashSet::new();
        neighbors_l.insert(Direction::North);
        let mut neighbors_r = HashSet::new();
        neighbors_r.insert(Direction::South);
        let mut neighbors_lr = HashSet::new();
        neighbors_lr.insert(Direction::North);
        neighbors_lr.insert(Direction::South);

        let staircases = vec![
            ("StairsWeathered0Mesh", 0, neighbors_empty.clone()),
            ("StairsWeathered1Mesh", 1, neighbors_empty.clone()),
            ("StairsWeathered1LMesh", 1, neighbors_l.clone()),
            ("StairsWeathered1RMesh", 1, neighbors_r.clone()),
            ("StairsWeathered1LRMesh", 1, neighbors_lr.clone()),
            ("StairsWeathered2Mesh", 2, neighbors_empty.clone()),
            ("StairsWeathered2LMesh", 2, neighbors_l.clone()),
            ("StairsWeathered2RMesh", 2, neighbors_r.clone()),
            ("StairsWeathered2LRMesh", 2, neighbors_lr.clone()),
        ];

        for (name, weathering, neighbors) in staircases {
            let mut result = VoxelMesh {
                neighbors,
                weathering,
                meshes: vec![],
                collider: collider.clone(),
                collider_mode: ColliderMode::Mesh,
                friction: Friction::default(),
                ghost: false,
            };

            for prim in &get_mesh(&meshes[name]).primitives {
                result.meshes.push(MeshWithMaterial {
                    mesh: prim.mesh.clone(),
                    material: prim.material.clone().unwrap_or(default_material.clone()),
                });
            }

            vma.staircase.push(result);
        }
    }
}
