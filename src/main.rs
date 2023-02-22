#![allow(dead_code)]
#![allow(deprecated)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_variables)]

use std::f32::consts::{PI, TAU};
use std::collections::HashMap;
use num_traits::float::FloatConst;
use bevy::prelude::*;
use bevy::window::WindowResized;
use bevy_rapier3d::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_inspector_egui::{quick::ResourceInspectorPlugin, quick::FilterQueryInspectorPlugin};
use crate::assets::GameState;
use crate::key_translator::TranslatedKey;
use crate::interact::{Interactable, Item};
use crate::outline::OutlineMaterial;
use crate::inventory::{Inventory, InventoryItem, ItemType};
use crate::projectile::Projectile;
use crate::enemy::spawn_enemy;
use crate::fps_controller::{
    FpsController, FpsControllerInput, LogicalPlayer, RenderPlayer
};
use crate::importable_shaders::ImportableShader;

pub mod outline;
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
pub mod enemy;
pub mod fps_controller;
pub mod room_loader;
pub mod circles;
pub mod spline;
pub mod shapes;
pub mod self_destruct;
pub mod importable_shaders;
pub mod explosion;
pub mod trail;

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
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(default_plugins)
        .insert_resource(RapierConfiguration::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        //.add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(bevy_scene_hook::HookPlugin)
        .add_plugin(bevy_egui::EguiPlugin)
        .add_plugin(crate::room_loader::TxtPlugin)
        .add_plugin(crate::fps_controller::FpsControllerPlugin)
        .add_plugin(crate::outline::OutlinePlugin)
        .add_plugin(crate::assets::AssetsPlugin)
        .add_plugin(crate::ui::UiPlugin)
        .add_plugin(crate::inventory::InventoryPlugin)
        .add_plugin(crate::interact::InteractPlugin)
        .add_plugin(crate::key_translator::KeyTranslatorPlugin)
        .add_plugin(crate::crt::CrtPlugin)
        .add_plugin(crate::projectile::ProjectilePlugin)
        .add_plugin(crate::enemy::EnemyPlugin)
        .add_plugin(crate::circles::CirclePlugin)
        .add_plugin(crate::self_destruct::SelfDestructPlugin)
        .add_plugin(crate::importable_shaders::ImportableShadersPlugin)
        .add_plugin(crate::explosion::ExplosionPlugin)
        .add_plugin(crate::trail::TrailPlugin)
        //.add_plugin(Sprite3dPlugin)
        //.add_plugin(crate::camera::PlayerPlugin)
        .add_startup_system(setup)
        .add_system(resize_camera_texture)
        .add_system(spawn_projectiles)
        .add_system_set(SystemSet::on_enter(GameState::Ready)
                        .with_system(spawn_level))
        .add_system_set(SystemSet::on_update(GameState::Ready)
                        .with_system(reload_level))
        //.add_system(movement)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.spawn(ImportableShader::new("animation"));

    commands.spawn((
        Collider::capsule(Vec3::Y * 0.125, Vec3::Y * 0.375, 0.125),
        ActiveEvents::COLLISION_EVENTS,
        Velocity::zero(),
        RigidBody::Dynamic,
        Sleeping::disabled(),
        LockedAxes::ROTATION_LOCKED,
        AdditionalMassProperties::Mass(1.0),
        GravityScale(0.0),
        Ccd { enabled: true },
        Transform::from_xyz(10.0, 10.0, 10.0),
        LogicalPlayer(0),
        FpsControllerInput {
            pitch: -TAU / 12.0,
            yaw: TAU * 5.0 / 8.0,
            ..default()
        },
        FpsController {
            walk_speed: 5.0,
            run_speed: 8.0,
            key_fly: KeyCode::Grave,
            enable_input: false,
            ..default()
        },
        Inventory::new(),
    ));

    use bevy::core_pipeline::bloom::BloomSettings;
    use bevy::render::view::RenderLayers;
    use bevy::core_pipeline::clear_color::ClearColorConfig;

    let post_processing_pass_layer = RenderLayers::layer((RenderLayers::TOTAL_LAYERS - 1) as u8);

    let render_target = images.add(make_camera_image(1.0));

    let mut camera = Camera::default();
    #[cfg(target_arch = "x86_64")]
    {
        camera.hdr = true;
    }

    commands.spawn((
        Camera3dBundle {
            camera_3d: Camera3d {
                clear_color: ClearColorConfig::Custom(Color::WHITE),
                ..default()
            },
            transform: Transform::from_xyz(10.0, 10.0, 10.0),
            camera: Camera {
                target: bevy::render::camera::RenderTarget::Image(render_target.clone()),
                ..camera.clone()
            },
            ..default()
        },
        RenderPlayer(0),
        BloomSettings::default(),
        UiCameraConfig { show_ui: false },
    ));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0 })),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(render_target.clone()),
                unlit: true,
                cull_mode: None,
                ..default()
            }),
            transform: Transform::IDENTITY
                .with_rotation(Quat::from_rotation_x(f32::PI() / 2.0))
                .with_translation(Vec3::new(300.0, 300.0, 299.0))
                .with_scale(Vec3::new(1.0, 1.0, -1.0))
                ,
            ..default()
        },
        CameraTexture(render_target),
        post_processing_pass_layer,
    ));

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(300.0, 300.0, 300.0),
            camera: Camera {
                priority: 1,
                ..camera.clone()
            },
            ..default()
        },
        post_processing_pass_layer,
    ));
}

#[derive(Component)]
struct CameraTexture(Handle<Image>);

fn make_camera_image(aspect_ratio: f32) -> Image {
    use bevy::render::camera::Camera;
    use bevy::render::render_resource::*;
    use bevy::render::texture::ImageSampler;

    let width = 512;
    let height = (width as f32 / aspect_ratio).round() as u32;

    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size: Extent3d { width, height, ..default() },
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
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
    mut camera_textures: Query<(&mut Transform, &CameraTexture)>,
) {
    let mut last_event = None;
    for e in resize_reader.iter() {
        last_event = Some(e);
    }
    if let Some(window_resized) = last_event {
        for (mut transform, camera_texture) in camera_textures.iter_mut() {
            let CameraTexture(handle) = camera_texture;
            let aspect_ratio = window_resized.width / window_resized.height;
            transform.scale.x = aspect_ratio;
            *images.get_mut(&handle).unwrap() = make_camera_image(aspect_ratio);
        }
    }
}

#[derive(Component)]
struct IsVoxel;

fn spawn_voxels(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    start_room: &crate::level::Room,
    rooms: &[crate::level::Room],
) {
    let pos_to_transform = |pos: bevy::math::IVec3| -> Transform {
        // Annoying hack because camera position is weird
        Transform::from_xyz(pos.x as f32 - 3.0,
                            pos.y as f32 - 0.2,
                            pos.z as f32 - 3.0)
    };
    let map = crate::level::Map::room_gluing(start_room, 20, rooms);
    let cube = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    let brown = materials.add(Color::rgb(0.8, 0.7, 0.6).into());
    for pos in map.voxels.bounding_box.iter() {
        if map.voxels.index(&pos).shape
            == crate::level::voxel::VoxelShape::Solid {
            commands.spawn(PbrBundle {
                mesh: cube.clone(),
                material: brown.clone(),
                transform: pos_to_transform(pos),
                ..default()
            })
                .insert(IsVoxel)
                .insert(Collider::cuboid(0.5, 0.5, 0.5));
        }
    }
    // Useful for debugging map generation
    if true {
        let room_box_corner = meshes.add(Mesh::from(shape::Cube { size: 1.75 }));
        let room_box_material = materials.add(Color::rgba(0.5, 0.0, 0.0, 0.3).into());
        for room_box in map.room_boxes {
            commands.spawn(PbrBundle {
                mesh: room_box_corner.clone(),
                material: room_box_material.clone(),
                transform: pos_to_transform(room_box.minimum),
                ..default()
            })
                .insert(IsVoxel);
            commands.spawn(PbrBundle {
                mesh: room_box_corner.clone(),
                material: room_box_material.clone(),
                transform: pos_to_transform(room_box.maximum),
                ..default()
            })
                .insert(IsVoxel);
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
                    .insert(IsVoxel);
            }
        }
    }
}

fn reload_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    rooms: Res<Assets<crate::room_loader::TextFile>>,
    room_assets: Res<crate::assets::RoomAssets>,
    keyboard: Res<Input<KeyCode>>,
    preexisting_voxels: Query<Entity, With<IsVoxel>>,
) {
    if keyboard.just_pressed(KeyCode::R) {
        for entity in preexisting_voxels.iter() {
            commands.entity(entity).despawn();
        }

        let room1 = crate::level::Room::parse(&rooms.get(&room_assets.room1).unwrap().contents);
        let room2 = crate::level::Room::parse(&rooms.get(&room_assets.room2).unwrap().contents);
        let rooms = [room1.clone(), room2.clone()];

        spawn_voxels(&mut commands, &mut meshes, &mut materials, &room1, &rooms);
    }
}

fn spawn_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut outlines: ResMut<Assets<OutlineMaterial>>,
    mut images: ResMut<Assets<Image>>,
    rooms: Res<Assets<crate::room_loader::TextFile>>,
    image_assets: Res<crate::assets::ImageAssets>,
    room_assets: Res<crate::assets::RoomAssets>,
) {
    let room1 = crate::level::Room::parse(&rooms.get(&room_assets.room1).unwrap().contents);
    let room2 = crate::level::Room::parse(&rooms.get(&room_assets.room2).unwrap().contents);
    let rooms = [room1.clone(), room2.clone()];

    spawn_voxels(&mut commands, &mut meshes, &mut materials, &room1, &rooms);

    spawn_enemy(&mut commands, &mut meshes, &mut materials,
                Vec3 { x: 1.0, y: 0.75, z: 1.5 });

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
        .insert(outlines.add(OutlineMaterial {
            width: 0.,
            color: Color::rgba(1.0, 1.0, 1.0, 1.0),
        }));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 1.5,
                subdivisions: 3,
            })),
            material: materials.add(Color::rgb(1.0, 0.8, 0.8).into()),
            transform: Transform::from_xyz(1.5, 15.0, 1.5),
            ..default()
        },
        Collider::ball(1.5),
    ));

    // use bevy_scene_hook::{SceneHook, HookedSceneBundle};
    // let scene0 = asset_server.load("gltf/vornak.glb#Scene0");
    // let mut vornak_transform = Transform::from_xyz(4.0, 1.0, 4.0)
    //     .with_scale(Vec3::splat(0.2));
    // vornak_transform.rotate_y(2.0);
    // commands.spawn((
    //     HookedSceneBundle {
    //         scene: SceneBundle {
    //             scene: scene0,
    //             transform: vornak_transform,
    //             ..Default::default()
    //         },
    //         hook: SceneHook::new(|entity, cmds| {
    //             println!("DEBUG: {:?}",
    //                      entity.get::<Name>().map(|t| t.as_str()));
    //         }),
    //     },
    // ));

    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 400.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
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
        commands.spawn_bundle((
            Projectile { velocity },
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 0.05 })),
                material: materials.add(Color::rgb(1.0, 0.2, 0.2).into()),
                transform: camera.compute_transform(),
                ..default()
            }));
    }
}
