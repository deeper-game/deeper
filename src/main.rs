#![allow(dead_code)]
#![allow(deprecated)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_variables)]

use std::f32::consts::{PI, TAU};
use std::collections::HashMap;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use bevy_rapier3d::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_inspector_egui::{quick::ResourceInspectorPlugin, quick::FilterQueryInspectorPlugin};
use crate::assets::GameState;
use crate::key_translator::TranslatedKey;
use crate::interact::{Interactable, Item};
use crate::level::Level;
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
        .add_system(manage_cursor)
        .add_system(spawn_projectiles)
        .add_system_set(SystemSet::on_enter(GameState::Ready)
                        .with_system(spawn_level))
        //.add_system(movement)
        .run();
}

fn setup(
    mut commands: Commands,
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
            ..default()
        },
        Inventory::new(),
    ));

    use bevy::core_pipeline::bloom::BloomSettings;

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(10.0, 10.0, 10.0),
            #[cfg(target_arch = "x86_64")]
            camera: Camera {
                hdr: true,
                ..default()
            },
            ..default()
        },
        RenderPlayer(0),
        BloomSettings::default(),
    ));
}

fn spawn_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut outlines: ResMut<Assets<OutlineMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut rooms: ResMut<Assets<crate::room_loader::TextFile>>,
    image_assets: Res<crate::assets::ImageAssets>,
    room_assets: Res<crate::assets::RoomAssets>,
) {
    let room1 = crate::level::Room::parse(&rooms.get(&room_assets.room1).unwrap().contents);
    let room2 = crate::level::Room::parse(&rooms.get(&room_assets.room2).unwrap().contents);
    let map = crate::level::room_gluing(&room1.clone(), 0, &[room1, room2]);

    for pos in map.voxels.bounding_box.iter() {
        if map.voxels.index(&pos).shape == crate::level::VoxelShape::Solid {
            commands.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                // Annoying hack because camera position is weird
                transform: Transform::from_xyz(pos.x as f32 - 3.0,
                                               pos.y as f32 - 0.2,
                                               pos.z as f32 - 3.0),
                ..default()
            })
                .insert(Collider::cuboid(0.5, 0.5, 0.5));
        }
    }

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

pub fn manage_cursor(
    mut windows: ResMut<Windows>,
    btn: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
    mut controllers: Query<&mut FpsController>,
) {
    let window = windows.get_primary_mut().unwrap();
    if btn.just_pressed(MouseButton::Left) {
        window.set_cursor_grab_mode(CursorGrabMode::Locked);
        window.set_cursor_visibility(false);
        for mut controller in &mut controllers {
            controller.enable_input = true;
        }
    }
    if key.just_pressed(KeyCode::Escape) {
        window.set_cursor_grab_mode(CursorGrabMode::None);
        window.set_cursor_visibility(true);
        for mut controller in &mut controllers {
            controller.enable_input = false;
        }
    }
}
