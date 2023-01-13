use std::f32::consts::{PI, TAU};
use std::collections::HashMap;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use bevy_rapier3d::prelude::*;
use bevy_fps_controller::controller::*;
use bevy_asset_loader::prelude::*;
use bevy_inspector_egui::{InspectorPlugin, widgets::InspectorQuery};
use crate::key_translator::TranslatedKey;
use crate::level::Level;
use crate::outline::{OutlinePlugin, OutlineMaterial};
use crate::inventory::{Inventory, InventoryItem, ItemType};

pub mod outline;
pub mod key_translator;
pub mod magic;
pub mod level;
pub mod ui;
pub mod assets;
pub mod inventory;
pub mod editor;
pub mod crt;

pub fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .insert_resource(RapierConfiguration::default())
        .insert_resource(Selected { entity: None })
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        //.add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(FpsControllerPlugin)
        .add_plugin(OutlinePlugin)
        .add_plugin(crate::assets::AssetsPlugin)
        .add_plugin(crate::ui::UiPlugin)
        .add_plugin(crate::inventory::InventoryPlugin)
        .add_plugin(crate::key_translator::KeyTranslatorPlugin)
        .add_plugin(crate::crt::CrtPlugin)
        //.add_plugin(Sprite3dPlugin)
        //.add_plugin(crate::camera::PlayerPlugin)
        .add_system(manage_cursor)
        .add_system(interaction_glow)
        .add_system(interact)
        .add_startup_system(setup)
        //.add_system(movement)
        .run();
}

#[derive(Component)]
pub struct Interactable;

#[derive(Component)]
pub struct Item {
    item_type: ItemType,
}

#[derive(Resource)]
pub struct Selected {
    entity: Option<Entity>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut outlines: ResMut<Assets<OutlineMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let level = Level::from_png(
        &std::fs::File::open("./assets/level.png").unwrap());

    for y in 0 .. level.height {
        for x in 0 .. level.width {
            if level.has_wall(x, y).unwrap() {
                commands.spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                    material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                    transform: Transform::from_xyz(x as f32, 1.0, y as f32),
                    ..default()
                })
                    .insert(Collider::cuboid(0.5, 0.5, 0.5));
            }
            if level.has_floor(x, y).unwrap() {
                commands.spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                    material: materials.add(Color::rgb(0.0, 0.7, 0.6).into()),
                    transform: Transform::from_xyz(x as f32, 0.0, y as f32),
                    ..default()
                })
                    .insert(Collider::cuboid(0.5, 0.5, 0.5));
            }
        }
    }

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
        FpsController { ..default() },
        Inventory::new(),
    ));
    commands.spawn((
        Camera3dBundle::default(),
        RenderPlayer(0),
    ));

    commands.spawn_bundle((
        Interactable,
        Item { item_type: ItemType::Potion },
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.05 })),
            material: materials.add(Color::rgb(1.0, 0.2, 0.2).into()),
            transform: Transform::from_xyz(1.5, 0.55, 1.5),
            ..default()
        }))
        .insert(Collider::cuboid(0.025, 0.025, 0.025))
        .insert(outlines.add(OutlineMaterial {
            width: 0.,
            color: Color::rgba(1.0, 1.0, 1.0, 1.0),
        }));

    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
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

pub fn interact(
    mut commands: Commands,
    mouse: Res<Input<MouseButton>>,
    selected: Res<Selected>,
    items: Query<&Item>,
    mut inventories: Query<&mut Inventory, With<LogicalPlayer>>,
    screens: Query<&crate::editor::Screen>,
    mut screen_activated: ResMut<crate::crt::ScreenActivated>,
    mut controllers: Query<&mut FpsController>,
) {
    let mut inventory = inventories.single_mut();
    if mouse.just_pressed(MouseButton::Right) {
        if let Some(entity) = selected.entity {
            if let Ok(item) = items.get(entity) {
                let item_type = item.item_type.clone();
                commands.entity(entity).despawn();
                inventory.insert(&InventoryItem { item_type, equipped: false });
            }
            if screens.get(entity).is_ok() {
                screen_activated.entity = Some(entity);
                for mut controller in &mut controllers {
                    controller.enable_input = false;
                }
            }
        }
    }
}

pub fn interaction_glow(
    rapier_context: Res<RapierContext>,
    mut selected: ResMut<Selected>,
    mut outlines: ResMut<Assets<OutlineMaterial>>,
    mut interactables: Query<&Handle<OutlineMaterial>,
                             (With<GlobalTransform>, With<Collider>, With<Interactable>)>,
    player: Query<&GlobalTransform, With<RenderPlayer>>,
) {
    let camera: &GlobalTransform = player.single();
    if let Some((entity, toi)) = rapier_context.cast_ray(
        camera.translation(), camera.forward(), 2.0, false,
        QueryFilter::exclude_dynamic(),
    ) {
        if selected.entity != Some(entity) {
            if let Some(e) = selected.entity {
                if let Ok(old_material) = interactables.get_mut(e) {
                    outlines.get_mut(old_material).unwrap().width = 0.0;
                }
            }
            selected.entity = None;

            if let Ok(new_material) = interactables.get(entity) {
                outlines.get_mut(new_material).unwrap().width = 3.0;
                selected.entity = Some(entity);
            }
        }
    } else {
        if let Some(e) = selected.entity {
            if let Ok(old_material) = interactables.get(e) {
                outlines.get_mut(old_material).unwrap().width = 0.0;
            }
        }
        selected.entity = None;
    }
}
