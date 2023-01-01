use std::f32::consts::{PI, TAU};
use std::collections::HashMap;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use bevy_rapier3d::prelude::*;
use bevy_fps_controller::controller::*;
use bevy_asset_loader::prelude::*;
use bevy_inspector_egui::{InspectorPlugin, widgets::InspectorQuery};
use crate::level::Level;
use crate::outline::{OutlinePlugin, OutlineMaterial};

pub mod outline;
pub mod magic;
pub mod level;

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
        .insert_resource(ItemSelected { entity: None })
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        //.add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(FpsControllerPlugin)
        .add_plugin(OutlinePlugin)
        .add_plugin(InspectorPlugin::<InspectorQuery<Entity, (With<Style>, Without<Parent>)>>::new())
        //.add_plugin(Sprite3dPlugin)
        //.add_plugin(crate::camera::PlayerPlugin)
        .add_system(manage_cursor)
        .add_system(item_glow)
        .add_system(grab_item)
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::on_enter(GameState::Ready).with_system(setup_hud))
        .add_system_set(
            SystemSet::on_enter(GameState::Ready).with_system(show_inventory))
        .add_system_set(
            SystemSet::on_update(GameState::Ready).with_system(update_inventory))
        // .add_system_set(
        //     SystemSet::on_update(GameState::Ready).with_system(hud_follow))
        //.add_system(movement)
        .run();
}

type InventoryPosition = (usize, usize);

#[derive(Clone, Debug)]
pub struct InventoryItem {
    item_type: ItemType,
    equipped: bool,
}

#[derive(Component, Debug)]
pub struct Inventory {
    width: usize,
    height: usize,
    map: HashMap<InventoryPosition, InventoryItem>,
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
                    return;
                }
            }
        }
    }
}

#[derive(Component)]
pub struct InventorySlot {
    position: InventoryPosition,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum GameState { Loading, Ready }

#[derive(AssetCollection, Resource)]
pub struct ImageAssets {
    #[asset(path = "empty.png")]
    empty: Handle<Image>,
    #[asset(path = "crosshair.png")]
    crosshair: Handle<Image>,
    #[asset(path = "coin.png")]
    coin: Handle<Image>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ItemType {
    Potion,
    Staff,
    Book,
}

impl ItemType {
    pub fn icon(&self, image_assets: &ImageAssets) -> Handle<Image> {
        match *self {
            ItemType::Potion => image_assets.coin.clone(),
            ItemType::Staff => image_assets.coin.clone(),
            ItemType::Book => image_assets.coin.clone(),
        }
    }
}

#[derive(Component)]
pub struct Item {
    item_type: ItemType,
}

#[derive(Resource)]
pub struct ItemSelected {
    entity: Option<Entity>,
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
        FpsController { ..default() },
        Inventory::new(),
    ));
    commands.spawn((
        Camera3dBundle::default(),
        RenderPlayer(0),
    ));

    commands.spawn_bundle((
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

pub fn setup_hud(
    mut commands: Commands,
    images: Res<ImageAssets>,
) {
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                image: UiImage(images.crosshair.clone()),
                style: Style {
                    size: Size::new(Val::Px(32.0), Val::Px(32.0)),
                    ..default()
                },
                ..default()
            });
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
    item_selected: Res<ItemSelected>,
    items: Query<&Item>,
    mut inventories: Query<&mut Inventory, With<LogicalPlayer>>,
) {
    let mut inventory = inventories.single_mut();
    if mouse.just_pressed(MouseButton::Right) {
        if let Some(entity) = item_selected.entity {
            let item_type = items.get(entity).unwrap().item_type.clone();
            commands.entity(entity).despawn();
            inventory.insert(&InventoryItem { item_type, equipped: false });
        }
    }
}

pub fn update_inventory(
    images: Res<ImageAssets>,
    mut inventory_slots: Query<(&InventorySlot, &mut UiImage)>,
    mut inventories: Query<&mut Inventory, With<LogicalPlayer>>,
) {
    let mut inventory = inventories.single_mut();
    for (slot, mut image) in inventory_slots.iter_mut() {
        if inventory.map.contains_key(&slot.position) {
            *image = UiImage(inventory.map[&slot.position]
                             .item_type.icon(&images));
        } else {
            *image = UiImage(images.empty.clone());
        }
    }
}

pub fn show_inventory(
    images: Res<ImageAssets>,
    mut commands: Commands,
) {
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(NodeBundle {
                style: Style {
                    size: Size {
                        width: Val::Px(512.0),
                        height: Val::Px(128.0),
                    },
                    flex_wrap: FlexWrap::Wrap,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: Color::rgb(0.65, 0.65, 0.65).into(),
                ..default()
            }).with_children(|parent| {
                for i in 0 .. 16 {
                    for j in 0 .. 4 {
                        parent.spawn(NodeBundle {
                            style: Style {
                                margin: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            background_color: Color::rgb(0.4, 0.4, 0.4).into(),
                            ..default()
                        }).with_children(|parent| {
                            parent.spawn(ImageBundle {
                                style: Style {
                                    size: Size {
                                        width: Val::Px(28.0),
                                        height: Val::Px(28.0),
                                    },
                                    ..default()
                                },
                                image: UiImage(images.empty.clone()),
                                ..default()
                            })
                                .insert(InventorySlot { position: (i, j) });
                        });
                    }
                }
            });
        });


    // commands
    //     .spawn(NodeBundle {
    //         style: Style {
    //             position_type: PositionType::Absolute,
    //             size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
    //             align_items: AlignItems::Stretch,
    //             justify_content: JustifyContent::Center,
    //             ..default()
    //         },
    //         ..default()
    //     })
    //     .with_children(|parent| {
    //         parent.spawn(NodeBundle {
    //             style: Style {
    //                 size: Size {
    //                     width: Val::Percent(70.0),
    //                     height: Val::Auto,
    //                 },
    //                 aspect_ratio: Some(4.0),
    //                 ..default()
    //             },
    //             background_color: Color::rgb(0.65, 0.65, 0.65).into(),
    //             ..default()
    //         });
    //     });
}

pub fn item_glow(
    rapier_context: Res<RapierContext>,
    mut item_selected: ResMut<ItemSelected>,
    mut outlines: ResMut<Assets<OutlineMaterial>>,
    mut items: Query<(&mut Item, &Handle<OutlineMaterial>),
                     (With<GlobalTransform>, With<Collider>)>,
    player: Query<&GlobalTransform, With<RenderPlayer>>,
) {
    let camera: &GlobalTransform = player.single();
    if let Some((entity, toi)) = rapier_context.cast_ray(
        camera.translation(), camera.forward(), 2.0, false,
        QueryFilter::exclude_dynamic(),
    ) {
        if item_selected.entity != Some(entity) {
            if let Some(e) = item_selected.entity {
                if let Ok((_, old_material)) = items.get_mut(e) {
                    outlines.get_mut(old_material).unwrap().width = 0.0;
                }
            }
            item_selected.entity = None;

            if let Ok((_, new_material)) = items.get(entity) {
                outlines.get_mut(new_material).unwrap().width = 3.0;
                item_selected.entity = Some(entity);
            }
        }
    } else {
        if let Some(e) = item_selected.entity {
            if let Ok((_, old_material)) = items.get(e) {
                outlines.get_mut(old_material).unwrap().width = 0.0;
            }
        }
        item_selected.entity = None;
    }
}
