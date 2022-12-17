pub mod outline;
//pub mod character;
pub mod magic;
pub mod level;
//pub mod camera;

// pub fn main() {
//     println!("Hello, World!");
// }














use std::f32::consts::TAU;
use std::collections::HashMap;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use bevy_rapier3d::prelude::*;
use bevy_fps_controller::controller::*;
use bevy_asset_loader::prelude::*;
use crate::level::Level;
use crate::outline::{OutlinePlugin, OutlineMaterial};

pub fn main() {
    App::new()
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Ready)
                .with_collection::<ImageAssets>())
        .add_state(GameState::Loading)
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .insert_resource(RapierConfiguration::default())
        .insert_resource(Inventory::new())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        //.add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(FpsControllerPlugin)
        .add_plugin(OutlinePlugin)
        //.add_plugin(Sprite3dPlugin)
        //.add_plugin(crate::camera::PlayerPlugin)
        .add_system(manage_cursor)
        .add_system(item_glow)
        .add_system(grab_item)
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::on_enter(GameState::Ready).with_system(setup_hud))
        // .add_system_set(
        //     SystemSet::on_update(GameState::Ready).with_system(hud_follow))
        //.add_system(movement)
        .run();
}

type InventoryPosition = (usize, usize);

#[derive(Clone)]
pub struct InventoryItem {
    item_type: ItemType,
    equipped: bool,
}

#[derive(Resource)]
pub struct Inventory {
    width: usize,
    height: usize,
    map: HashMap<InventoryPosition, InventoryItem>
}

impl Inventory {
    pub fn new() -> Self {
        Inventory {
            width: 16,
            height: 4,
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, item: &InventoryItem) {
        for x in 0 .. self.width {
            for y in 0 .. self.height {
                if !self.map.contains_key(&(x, y)) {
                    self.map.insert((x, y), item.clone());
                }
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum GameState { Loading, Ready }

#[derive(AssetCollection, Resource)]
pub struct ImageAssets {
    #[asset(path = "crosshair.png")]
    crosshair: Handle<Image>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ItemType {
    Potion,
    Staff,
    Book,
}

#[derive(Component)]
pub struct Item {
    selected: bool,
    item_type: ItemType,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut outlines: ResMut<Assets<OutlineMaterial>>,
) {
    let level = Level::from_png(&std::fs::File::open("./assets/level.png").unwrap());

    for y in 0 .. level.height {
        for x in 0 .. level.width {
            // walls
            if level.has_wall(x, y).unwrap() {
                commands.spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                    material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                    transform: Transform::from_xyz(x as f32, 1.0, y as f32),
                    ..default()
                })
                    .insert(Collider::cuboid(0.5, 0.5, 0.5));
            }
            // floors
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
        // Collider::cuboid(1.2, 1.2, 1.2),
        Collider::capsule(Vec3::Y * 0.125, Vec3::Y * 0.375, 0.125),
        ActiveEvents::COLLISION_EVENTS,
        Velocity::zero(),
        RigidBody::Dynamic,
        Sleeping::disabled(),
        LockedAxes::ROTATION_LOCKED,
        AdditionalMassProperties::Mass(1.0),
        GravityScale(0.0),
        Ccd { enabled: true }, // Prevent clipping when going fast
        Transform::from_xyz(10.0, 10.0, 10.0),
        LogicalPlayer(0),
        FpsControllerInput {
            pitch: -TAU / 12.0,
            yaw: TAU * 5.0 / 8.0,
            ..default()
        },
        FpsController { ..default() }
    ));
    commands.spawn((
        Camera3dBundle::default(),
        RenderPlayer(0),
    ));

    commands.spawn_bundle((
        Item { selected: false, item_type: ItemType::Potion },
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

    // light
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

pub fn setup_hud(
    mut commands: Commands,
    images: Res<ImageAssets>,
) {
    commands.spawn(
        ImageBundle {
            image: UiImage(images.crosshair.clone()),
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Percent(50.0),
                    bottom: Val::Percent(50.0),
                    ..default()
                },
                size: Size::new(Val::Px(16.0), Val::Px(16.0)),
                ..default()
            },
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

pub fn grab_item(
    mut commands: Commands,
    mouse: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    items: Query<(Entity, &Item)>,
    mut inventory: ResMut<Inventory>,
) {
    if mouse.just_pressed(MouseButton::Right) {
        for (entity, item) in items.iter() {
            if item.selected {
                commands.entity(entity).despawn();
                inventory.insert(&InventoryItem {
                    item_type: item.item_type.clone(),
                    equipped: false,
                });
            }
        }
    }
}

pub fn item_glow(
    // mut materials: ResMut<Assets<StandardMaterial>>,
    mut outlines: ResMut<Assets<OutlineMaterial>>,
    mut items: Query<(&mut Item, &GlobalTransform, &Collider, &Handle<OutlineMaterial>)>,
    player: Query<&GlobalTransform, With<RenderPlayer>>,
) {
    let camera_transform: &GlobalTransform = player.single();
    for (mut item, item_transform, item_collider, item_outline_material) in items.iter_mut() {
        let (_, item_rotation, item_translation) =
            item_transform.to_scale_rotation_translation();
        let collided = item_collider.cast_ray(
            item_translation,
            item_rotation,
            camera_transform.translation(),
            camera_transform.forward(),
            2.0,
            false,
        );
        outlines.get_mut(item_outline_material).unwrap().width =
            if collided.is_some() { 3.0 } else { 0.0 };
        (*item).selected = collided.is_some();
    }
}
