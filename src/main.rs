#![allow(dead_code)]
#![allow(deprecated)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_variables)]

use std::f32::consts::{PI, TAU};
use std::collections::{HashSet, HashMap};
use num_traits::float::FloatConst;
use bevy::prelude::*;
use bevy::window::WindowResized;
use bevy_rapier3d::prelude::*;
//use bevy_inspector_egui::{quick::ResourceInspectorPlugin, quick::FilterQueryInspectorPlugin};
use crate::add_bloom::AddBloom;
use crate::assets::{GameState, VoxelMeshAssets};
use crate::key_translator::TranslatedKey;
use crate::interact::{Interactable, Item};
use crate::inventory::{Inventory, InventoryItem, ItemType};
use crate::netcode::{NetcodeIdProvider, Session, Peer};
use crate::postprocessing::PostprocessingMaterial;
use crate::projectile::Projectile;
use crate::fps_controller::{
    FpsController, FpsControllerInput, LogicalPlayer, RenderPlayer
};
use crate::importable_shaders::ImportableShader;

pub mod checksum;
pub mod netcode;
pub mod postprocessing;
pub mod terminal_key;
pub mod key_translator;
pub mod magic;
pub mod level;
pub mod ui;
pub mod assets;
pub mod inventory;
pub mod interact;
pub mod editor;
pub mod crt;
pub mod projectile;
pub mod enemies;
pub mod fps_controller;
pub mod room_loader;
pub mod circles;
pub mod spline;
pub mod shapes;
pub mod self_destruct;
pub mod importable_shaders;
pub mod explosion;
pub mod trail;
pub mod add_bloom;
pub mod voxel_editor;

pub fn main() {
    let mut default_plugins = DefaultPlugins.build();
    #[cfg(target_arch = "x86_64")]
    {
        default_plugins = default_plugins.set(AssetPlugin {
            watch_for_changes: true,
            ..Default::default()
        });
    }

    App::new()
        .insert_resource(FixedTime::new_from_secs(1.0))
        .insert_resource(Msaa::Sample4)
        .add_plugins(default_plugins)
        .insert_resource(RapierConfiguration::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        //.add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(bevy_scene_hook::HookPlugin)
        .add_plugin(bevy_egui::EguiPlugin)
        .add_plugin(bevy_mod_outline::OutlinePlugin)
        .add_plugin(bevy_mod_outline::AutoGenerateOutlineNormalsPlugin)
        .add_plugin(bevy_framepace::FramepacePlugin)
        .add_plugin(crate::netcode::NetcodePlugin)
        .insert_resource(crate::netcode::connect("testing", 1)) // FIXME
        .add_plugin(crate::postprocessing::PostprocessingPlugin)
        .add_plugin(crate::room_loader::TxtPlugin)
        .add_plugin(crate::fps_controller::FpsControllerPlugin)
        .add_plugin(crate::assets::AssetsPlugin)
        .add_plugin(crate::ui::UiPlugin)
        .add_plugin(crate::inventory::InventoryPlugin)
        .add_plugin(crate::interact::InteractPlugin)
        .add_plugin(crate::key_translator::KeyTranslatorPlugin)
        .add_plugin(crate::crt::CrtPlugin)
        .add_plugin(crate::projectile::ProjectilePlugin)
        .add_plugin(crate::enemies::EnemiesPlugin)
        .add_plugin(crate::circles::CirclePlugin)
        .add_plugin(crate::self_destruct::SelfDestructPlugin)
        .add_plugin(crate::importable_shaders::ImportableShadersPlugin)
        .add_plugin(crate::explosion::ExplosionPlugin)
        .add_plugin(crate::trail::TrailPlugin)
        .add_plugin(crate::add_bloom::AddBloomPlugin)
        .add_plugin(crate::level::LevelPlugin)
        .add_plugin(crate::voxel_editor::VoxelEditorPlugin)
        //.add_plugin(Sprite3dPlugin)
        //.add_plugin(crate::camera::PlayerPlugin)
        .add_system(setup.in_schedule(OnEnter(GameState::Ready)))
        .add_system(respawn_falling
                    .before(crate::fps_controller::fps_controller_input))
        .add_system(respawn
                    .in_schedule(OnEnter(GameState::Ready))
                    .after(spawn_level))
        .add_system(resize_camera_texture)
        .add_system(spawn_projectiles)
        .add_system(toggle_msaa)
        .add_system(debug_scenes)
        .add_system(add_convex_hull_colliders)
        .add_system(spawn_level.in_schedule(OnEnter(GameState::Ready)))
        .add_system(reload_level.run_if(in_state(GameState::Ready)))
        //.add_system(movement)
        .run();
}



fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut pp_materials: ResMut<Assets<PostprocessingMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut windows: Query<&mut Window>,
    session: Res<Session>,
    mut nip: ResMut<NetcodeIdProvider>,
    asset_server: Res<AssetServer>,
    voxel_mesh_assets: Res<VoxelMeshAssets>,
) {
    for mut window in windows.iter_mut() {
        window.present_mode = bevy::window::PresentMode::AutoVsync;
    }

    commands.spawn(ImportableShader::new("animation"));

    let spawn_point = Transform::from_xyz(10.0, 10.0, 10.0);

    commands.spawn((
        (
            Collider::capsule(Vec3::Y * 0.125, Vec3::Y * 0.375, 0.125),
            CollisionGroups::new(Group::from_bits(1u32).unwrap(),
                                 Group::from_bits(1u32).unwrap()),
            ActiveEvents::COLLISION_EVENTS,
            Velocity::zero(),
            RigidBody::Dynamic,
            Sleeping::disabled(),
            LockedAxes::ROTATION_LOCKED,
            AdditionalMassProperties::Mass(1.0),
            GravityScale(0.0),
            Ccd { enabled: true },
        ),
        TransformBundle::from(spawn_point),
        LogicalPlayer(0),
        FpsControllerInput {
            pitch: -TAU / 12.0,
            yaw: TAU * 5.0 / 8.0,
            ..default()
        },
        FpsController {
            key_fly: KeyCode::Grave,
            enable_input: false,
            ..default()
        },
        Inventory::new(),
        nip.next(),
    ));

    let player_mesh = meshes.add(Mesh::from(shape::Cube { size: 0.5 }));
    for (i, peer) in session.info.as_ref().unwrap().peers().iter().enumerate() {
        info!("Spawning player {}: {:?}", i + 1, peer);
        assert!(i < 30);
        commands.spawn((
            (
                Collider::capsule(Vec3::Y * 0.125, Vec3::Y * 0.375, 0.125),
                CollisionGroups::new(Group::from_bits(1u32 << (i + 1)).unwrap(),
                                     Group::from_bits(1u32 << (i + 1)).unwrap()),
                ActiveEvents::COLLISION_EVENTS,
                Velocity::zero(),
                RigidBody::Dynamic,
                Sleeping::disabled(),
                LockedAxes::ROTATION_LOCKED,
                AdditionalMassProperties::Mass(1.0),
                GravityScale(0.0),
                Ccd { enabled: true },
            ),
            TransformBundle::from(spawn_point),
            LogicalPlayer((i as u8) + 1),
            FpsControllerInput {
                pitch: -TAU / 12.0,
                yaw: TAU * 5.0 / 8.0,
                ..default()
            },
            FpsController {
                enable_input: false,
                ..default()
            },
            peer.clone(),
        ));

        commands.spawn((
            RenderPlayer((i as u8) + 1),
            nip.next(),
            peer.clone(),
            PbrBundle {
                mesh: player_mesh.clone(),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgb(1.0, 0.2, 0.2),
                    ..default()
                }),
                ..default()
            },
        )).insert(TransformBundle::from(spawn_point));
    }

    use bevy::core_pipeline::bloom::BloomSettings;
    use bevy::render::camera::RenderTarget;
    use bevy::render::view::RenderLayers;
    use bevy::core_pipeline::clear_color::ClearColorConfig;
    use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};

    let post_processing_pass_layer = RenderLayers::layer((RenderLayers::TOTAL_LAYERS - 1) as u8);

    let render_target = images.add(make_camera_image(1.0));

    let mut camera = Camera::default();
    camera.hdr = true;

    commands.spawn((
        Camera3dBundle {
            // camera_3d: Camera3d {
            //     clear_color: ClearColorConfig::Custom(Color::WHITE),
            //     ..default()
            // },
            camera: Camera {
                target: RenderTarget::Image(render_target.clone()),
                ..camera.clone()
            },
            tonemapping: Tonemapping::AgX,
            dither: DebandDither::Enabled,
            ..default()
        },
        RenderPlayer(0),
        BloomSettings::default(),
        UiCameraConfig { show_ui: false },
    )).insert(TransformBundle::from(spawn_point));

    let palette: Handle<Image> = asset_server.load("palette.png");

    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(
                Mesh::from(shape::Plane { size: 1.0, ..default() })),
            material: pp_materials.add(PostprocessingMaterial {
                input: render_target.clone(),
                palette: palette.clone(),
                ..default()
            }),
            // I don't really understand why the translation here needs to be
            // 298.8 instead of 299, but it does seem to be the maximal value
            // that leads to zero crop.
            transform: Transform::IDENTITY
                .with_rotation(Quat::from_rotation_x(f32::PI() / 2.0))
                .with_translation(Vec3::new(300.0, 300.0, 298.8))
                .with_scale(Vec3::new(1.0, 1.0, 1.0)),
            ..default()
        },
        CameraTexture(render_target),
        post_processing_pass_layer,
    ));

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(300.0, 300.0, 300.0),
            camera: Camera {
                order: 1,
                ..camera.clone()
            },
            tonemapping: Tonemapping::None,
            dither: DebandDither::Disabled,
            ..default()
        },
        post_processing_pass_layer,
    ));

    voxel_mesh_assets.staircase[0].spawn(
        &Transform::from_xyz(0.0, 20.0, 0.0),
        &mut commands, &mut meshes, &mut materials);
    voxel_mesh_assets.staircase[0].spawn(
        &Transform::from_xyz(-1.0, 21.0, 0.0),
        &mut commands, &mut meshes, &mut materials);
    voxel_mesh_assets.staircase[0].spawn(
        &Transform::from_xyz(-2.0, 22.0, 0.0),
        &mut commands, &mut meshes, &mut materials);
}

fn add_convex_hull_colliders(
    mut commands: Commands,
    mut scenes: ResMut<Assets<Scene>>,
    meshes: Res<Assets<Mesh>>,
    scene_entities: Query<(Entity, &GlobalTransform, &Handle<Scene>), Without<Collider>>,
) {
    let mut mapping = HashMap::new();
    for (entity, transform, scene_handle) in scene_entities.iter() {
        mapping.insert(scene_handle.id(), (entity, transform.clone()));
    }
    for (id, mut scene) in scenes.iter_mut() {
        let Some((entity, outer_transform)) = mapping.get(&id) else {
            continue;
        };
        let inverted_outer_matrix = outer_transform.compute_matrix().inverse();
        let mut transform_mapping = HashMap::new();
        {
            let mut transforms_query = scene.world.query::<(Entity, &Transform)>();
            for (entity, transform) in transforms_query.iter_mut(&mut scene.world) {
                transform_mapping.insert(entity, transform.clone());
            }
        }
        let mut meshes_query = scene.world.query::<(&Handle<Mesh>, &GlobalTransform, &Parent)>();
        let mut points = Vec::new();
        for (mesh_handle, transform, parent) in meshes_query.iter_mut(&mut scene.world) {
            println!("Mesh transform: {:?}", transform);
            let transform = transform_mapping.get(&parent.get()).unwrap();
            println!("Parent transform: {:?}", transform);
            use bevy::render::mesh::VertexAttributeValues;
            // if name.as_str() != "ColliderSphere" {
            //     continue;
            // }
            let VertexAttributeValues::Float32x3(vec) =
                meshes.get(mesh_handle).unwrap()
                .attribute(Mesh::ATTRIBUTE_POSITION).unwrap()
                else { panic!("Mesh position attribute wasn't 3D float32"); };
            for pos in vec {
                let mut point = Vec3::new(pos[0], pos[1], pos[2]);
                point = transform.transform_point(point);
                //point = outer_transform.compute_matrix().transform_point3(point);
                //point = outer_transform.transform_point(point);
                points.push(point);
            }
            // colliders.push((
            //     Vec3::ZERO,
            //     Quat::IDENTITY,
            //     Collider::from_bevy_mesh(meshes.get(mesh_handle).unwrap(),
            //                              &ComputedColliderShape::TriMesh).unwrap(),
            // ));
        }
        commands.entity(*entity).insert(Collider::convex_hull(&points).unwrap());
    }
}

fn toggle_msaa(
    keyboard: Res<Input<KeyCode>>,
    mut msaa: ResMut<Msaa>,
) {
    if keyboard.just_pressed(KeyCode::M) {
        *msaa = match *msaa {
            Msaa::Off => Msaa::Sample2,
            Msaa::Sample2 => Msaa::Sample4,
            Msaa::Sample4 => Msaa::Sample8,
            Msaa::Sample8 => Msaa::Off,
        };
    }
}

#[derive(Component)]
struct CameraTexture(Handle<Image>);

fn make_camera_image(aspect_ratio: f32) -> Image {
    use bevy::render::camera::Camera;
    use bevy::render::render_resource::*;
    use bevy::render::texture::ImageSampler;

    let width = 1024;
    let height = (width as f32 / aspect_ratio).round() as u32;

    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size: Extent3d { width, height, ..default() },
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            view_formats: &[TextureFormat::Bgra8UnormSrgb],
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
        },
        sampler_descriptor: ImageSampler::nearest(),
        ..default()
    };

    image.resize(image.texture_descriptor.size);

    image
}

fn resize_camera_texture(
    mut resize_reader: EventReader<WindowResized>,
    mut images: ResMut<Assets<Image>>,
    mut pp_materials: ResMut<Assets<PostprocessingMaterial>>,
    mut camera_textures: Query<(&mut Transform, &Handle<PostprocessingMaterial>, &CameraTexture)>,
) {
    let mut last_event = None;
    for e in resize_reader.iter() {
        last_event = Some(e);
    }
    if let Some(window_resized) = last_event {
        for (mut transform, mat, camera_texture) in camera_textures.iter_mut() {
            let CameraTexture(handle) = camera_texture;
            let aspect_ratio = window_resized.width / window_resized.height;
            transform.scale.x = aspect_ratio;
            *images.get_mut(&handle).unwrap() = make_camera_image(aspect_ratio);
            pp_materials.get_mut(&mat).unwrap().input = handle.clone();
        }
    }
}

#[derive(Component)]
struct PartOfMap;

fn spawn_voxels(
    seed: u64,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    texture_pack: &Option<Handle<Image>>,
    start_room: &crate::level::Room,
    rooms: &[crate::level::Room],
) {
    let pos_to_transform = |pos: bevy::math::IVec3| -> Transform {
        // Annoying hack because camera position is weird
        Transform::from_xyz(pos.x as f32 - 0.5,
                            pos.y as f32 - 0.5,
                            pos.z as f32 - 0.5)
    };
    use crate::level::{self, voxel, UVRect};
    let mut map = level::Map::room_gluing(seed, start_room, 20, rooms);
    let stone = level::Block {
        orientation: voxel::CardinalDir::East,
        texture: voxel::Texture::Stone,
        style: voxel::Style::Normal,
    };
    map.uv_rects.insert(
        (stone.clone(), voxel::Direction::North),
        UVRect {
            minimum: Vec2::new(1.0 / 3.0, 3.0 / 4.0),
            maximum: Vec2::new(2.0 / 3.0, 4.0 / 4.0),
        });
    map.uv_rects.insert(
        (stone.clone(), voxel::Direction::Down),
        UVRect {
            minimum: Vec2::new(1.0 / 3.0, 2.0 / 4.0),
            maximum: Vec2::new(2.0 / 3.0, 3.0 / 4.0),
        });
    map.uv_rects.insert(
        (stone.clone(), voxel::Direction::South),
        UVRect {
            minimum: Vec2::new(1.0 / 3.0, 1.0 / 4.0),
            maximum: Vec2::new(2.0 / 3.0, 2.0 / 4.0),
        });
    map.uv_rects.insert(
        (stone.clone(), voxel::Direction::Up),
        UVRect {
            minimum: Vec2::new(1.0 / 3.0, 0.0 / 4.0),
            maximum: Vec2::new(2.0 / 3.0, 1.0 / 4.0),
        });
    map.uv_rects.insert(
        (stone.clone(), voxel::Direction::West),
        UVRect {
            minimum: Vec2::new(0.0 / 3.0, 1.0 / 4.0),
            maximum: Vec2::new(1.0 / 3.0, 2.0 / 4.0),
        });
    map.uv_rects.insert(
        (stone.clone(), voxel::Direction::East),
        UVRect {
            minimum: Vec2::new(2.0 / 3.0, 1.0 / 4.0),
            maximum: Vec2::new(3.0 / 3.0, 2.0 / 4.0),
        });
    let cube = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    let brown1 = materials.add(StandardMaterial {
        base_color: Color::rgb(0.8, 0.7, 0.6),
        base_color_texture: texture_pack.clone(),
        //emissive: Color::rgb(0.03, 0.03, 0.03),
        ..default()
    });
    let brown2 = materials.add(StandardMaterial {
        base_color: Color::rgb(0.5, 0.7, 0.6),
        //emissive: Color::rgb(0.03, 0.03, 0.03),
        ..default()
    });
    let map_mesh = map.generate_mesh();
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(map_mesh.clone()),
            material: brown1,
            transform: pos_to_transform(IVec3::new(0, 0, 0)),
            ..default()
        },
        PartOfMap,
        Collider::from_bevy_mesh(
            &map_mesh, &ComputedColliderShape::TriMesh).unwrap(),
    ));

    // Useful for debugging map generation
    if false {
        let room_box_corner = meshes.add(Mesh::from(shape::Cube { size: 1.75 }));
        let room_box_material = materials.add(Color::rgba(0.5, 0.0, 0.0, 0.3).into());
        for room_box in map.room_boxes {
            commands.spawn(PbrBundle {
                mesh: room_box_corner.clone(),
                material: room_box_material.clone(),
                transform: pos_to_transform(room_box.minimum),
                ..default()
            })
                .insert(PartOfMap);
            commands.spawn(PbrBundle {
                mesh: room_box_corner.clone(),
                material: room_box_material.clone(),
                transform: pos_to_transform(room_box.maximum),
                ..default()
            })
                .insert(PartOfMap);
        }
        let red = materials.add(Color::rgba(1.0, 0.0, 0.0, 0.5).into());
        let green = materials.add(Color::rgba(0.0, 1.0, 0.0, 0.5).into());
        let magenta = materials.add(Color::rgba(1.0, 0.0, 1.0, 0.5).into());
        for doorway in map.open_doorways {
            for pos in doorway.bounding_box.iter() {
                use crate::level::doorway::DoorwayMode;
                commands.spawn(PbrBundle {
                    mesh: cube.clone(),
                    material: match doorway.mode {
                        DoorwayMode::Neither => magenta.clone(),
                        DoorwayMode::Entrance => green.clone(),
                        DoorwayMode::Exit => red.clone(),
                    },
                    transform: pos_to_transform(pos),
                    ..default()
                })
                    .insert(PartOfMap);
            }
        }
    }
}

fn debug_scenes(
    mut commands: Commands,
    scenes: Res<Assets<Scene>>,
    materials: Res<Assets<StandardMaterial>>,
    keyboard: Res<Input<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::M) {
        for (_, scene) in scenes.iter() {
            if let Some(scene_materials) = scene.world.get_resource::<Assets<StandardMaterial>>() {
                for (_, material) in scene_materials.iter() {
                    println!("DEBUG: scene material: {:?}", material);
                }
            }
        }
    }
}

fn reload_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    rooms: Res<Assets<crate::room_loader::TextFile>>,
    image_assets: Res<crate::assets::ImageAssets>,
    room_assets: Res<crate::assets::RoomAssets>,
    keyboard: Res<Input<KeyCode>>,
    preexisting_voxels: Query<Entity, With<PartOfMap>>,
) {
    if keyboard.just_pressed(KeyCode::L) {
        for entity in preexisting_voxels.iter() {
            commands.entity(entity).despawn();
        }

        let room1 = crate::level::Room::parse(&rooms.get(&room_assets.room1).unwrap().contents);
        let room2 = crate::level::Room::parse(&rooms.get(&room_assets.room2).unwrap().contents);
        let rooms = [room1.clone(), room2.clone()];

        use rand::Rng;
        let mut rng = rand::thread_rng();
        spawn_voxels(rng.gen(),
                     &mut commands, &mut meshes, &mut materials,
                     &Some(image_assets.stone.clone()), &room1, &rooms);
    }
}

fn spawn_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    rooms: Res<Assets<crate::room_loader::TextFile>>,
    image_assets: Res<crate::assets::ImageAssets>,
    room_assets: Res<crate::assets::RoomAssets>,
) {
    let room1 = crate::level::Room::parse(&rooms.get(&room_assets.room1).unwrap().contents);
    let room2 = crate::level::Room::parse(&rooms.get(&room_assets.room2).unwrap().contents);
    let rooms = [room1.clone(), room2.clone()];

    use rand::Rng;
    let mut rng = rand::thread_rng();
    spawn_voxels(rng.gen(), &mut commands, &mut meshes, &mut materials,
                 &Some(image_assets.stone.clone()), &room1, &rooms);

    commands.spawn((
        Interactable,
        Item { item_type: ItemType::Potion },
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.05 })),
            material: materials.add(Color::rgb(1.0, 0.2, 0.2).into()),
            transform: Transform::from_xyz(1.5, 0.75, 1.5),
            ..default()
        }))
        .insert(Collider::cuboid(0.025, 0.025, 0.025))
        .insert(bevy_mod_outline::OutlineBundle {
            outline: bevy_mod_outline::OutlineVolume {
                colour: Color::WHITE,
                width: 5.0,
                ..default()
            },
            stencil: bevy_mod_outline::OutlineStencil {
                offset: 0.0,
                ..default()
            },
            ..default()
        });

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::try_from(shape::Icosphere {
                radius: 1.5,
                subdivisions: 3,
            }).unwrap()),
            material: materials.add(Color::rgb(1.0, 0.8, 0.8).into()),
            transform: Transform::from_xyz(1.5, 15.0, 1.5),
            ..default()
        },
        Collider::ball(1.5),
    ));

    use bevy_scene_hook::{SceneHook, HookedSceneBundle};
    let scene0 = asset_server.load("gltf/bottle.glb#Scene0");
    let mut bottle_transform = Transform::from_xyz(4.0, 1.0, 4.0)
        .with_scale(Vec3::splat(0.2));
    bottle_transform.rotate_y(2.0);
    commands.spawn((
        HookedSceneBundle {
            scene: SceneBundle {
                scene: scene0,
                transform: bottle_transform,
                ..Default::default()
            },
            hook: SceneHook::new(|entity, cmds| {
                let (Some(name), Some(children)) = (
                    entity.get::<Name>().map(|t| t.as_str()),
                    entity.get::<Children>(),
                ) else {
                    return;
                };
                if name == "Collider" {
                    cmds.insert(Visibility::Hidden);
                }
                let mut names = HashSet::new();
                names.insert("Sphere");
                names.insert("Blob1");
                names.insert("Blob2");
                names.insert("Blob3");
                names.insert("Blob4");
                if !names.contains(name) {
                    return;
                }
                cmds.commands()
                    .entity(*children.first().unwrap())
                    .insert(AddBloom { scale: 2.5 });
            }),
        },
    ));

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 16000.0,
            range: 500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 30.0, 4.0),
        ..default()
    });
    const HALF_SIZE: f32 = 10.0;
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 40.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(50.0, 50.0, 50.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        ..default()
    });
    // commands.insert_resource(AmbientLight {
    //     color: Color::WHITE,
    //     brightness: 0.03,
    // });
}

fn spawn_projectiles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    keyboard: Res<Input<KeyCode>>,
    player: Query<&GlobalTransform, With<RenderPlayer>>,
) {
    if keyboard.just_pressed(KeyCode::F) {
        let camera = player.single();
        let velocity = 0.1 * camera.forward();
        commands.spawn((
            Projectile { velocity },
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 0.05 })),
                material: materials.add(Color::rgb(1.0, 0.2, 0.2).into()),
                transform: camera.compute_transform(),
                ..default()
            }));
    }
}

fn respawn(
    mut rapier_context: ResMut<RapierContext>,
    mut logical_players: Query<(Entity, &mut Transform), With<LogicalPlayer>>,
) {
    for (entity, mut transform) in logical_players.iter_mut() {
        let new_pos = Vec3::new(0.0, 10.0, 0.0);
        transform.translation = new_pos;
        let iso = transform_to_iso(&transform,
                                   rapier_context.physics_scale());
        {
            let h = rapier_context.entity2body().get(&entity).unwrap().clone();
            rapier_context.bodies.get_mut(h).unwrap().set_position(iso, true);
        }
        {
            let h = rapier_context.entity2collider().get(&entity).unwrap().clone();
            rapier_context.colliders.get_mut(h).unwrap().set_position(iso);
        }
    }
}

fn respawn_falling(
    mut rapier_context: ResMut<RapierContext>,
    mut logical_players: Query<(Entity, &mut Transform), With<LogicalPlayer>>,
) {
    for (entity, mut transform) in logical_players.iter_mut() {
        if transform.translation.y < -20.0 {
            println!("Teleporting back to spawn");
            let new_pos = Vec3::new(0.0, 10.0, 0.0);
                transform.translation = new_pos;
            let iso = transform_to_iso(&transform,
                                       rapier_context.physics_scale());
            {
                let h = rapier_context.entity2body().get(&entity).unwrap().clone();
                rapier_context.bodies.get_mut(h).unwrap().set_position(iso, true);
            }
            {
                let h = rapier_context.entity2collider().get(&entity).unwrap().clone();
                rapier_context.colliders.get_mut(h).unwrap().set_position(iso);
            }
        }
    }
}

use rapier3d::math::{Isometry, Real};

pub fn transform_to_iso(transform: &Transform, physics_scale: Real) -> Isometry<Real> {
    Isometry::from_parts(
        (transform.translation / physics_scale).into(),
        transform.rotation.into(),
    )
}
