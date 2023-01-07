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
use crate::inventory::{Inventory, InventoryItem, ItemType};

pub mod outline;
pub mod magic;
pub mod level;
pub mod ui;
pub mod assets;
pub mod inventory;
pub mod editor;

pub fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .insert_resource(RapierConfiguration::default())
        .insert_resource(Selected { entity: None })
        .insert_resource(ScreenActivated { entity: None })
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        //.add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(FpsControllerPlugin)
        .add_plugin(OutlinePlugin)
        .add_plugin(crate::assets::AssetsPlugin)
        .add_plugin(crate::ui::UiPlugin)
        .add_plugin(crate::inventory::InventoryPlugin)
        //.add_plugin(Sprite3dPlugin)
        //.add_plugin(crate::camera::PlayerPlugin)
        .add_system(manage_cursor)
        .add_system(interaction_glow)
        .add_system(interact)
        .add_system(run_editor)
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

#[derive(Resource)]
pub struct ScreenActivated {
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

    {
        use bevy::render::render_resource::*;

        let mut image = Image::new_fill(
            Extent3d { width: 1360, height: 768, depth_or_array_layers: 1 },
            TextureDimension::D2,
            &[255u8, 0u8, 255u8, 255u8],
            TextureFormat::Bgra8UnormSrgb);

        let material_handle = materials.add(StandardMaterial {
            base_color_texture: Some(images.add(image)),
            // reflectance: 0.02,
            unlit: false,
            ..default()
        });

        commands.spawn_bundle((
            crate::editor::Screen::new(editor::Editor::new()),
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Plane { size: 0.25 })),
                material: material_handle,
                transform: Transform::from_xyz(1.0, 0.75, 1.0),
                    //.looking_at(Vec3::new(1.5, 1.5, 1.5), Vec3::new(0.0, 1.0, 0.0)),
                ..default()
            },
            Interactable,
            Collider::cuboid(0.125, 0.01, 0.125),
            outlines.add(OutlineMaterial {
                width: 0.,
                color: Color::rgba(1.0, 1.0, 1.0, 1.0),
            }),
        ));

    }

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

fn keycode_to_letter(keycode: KeyCode, shift: bool) -> Option<char> {
    if !shift {
        match keycode {
            KeyCode::Key1 => Some('1'),
            KeyCode::Key2 => Some('2'),
            KeyCode::Key3 => Some('3'),
            KeyCode::Key4 => Some('4'),
            KeyCode::Key5 => Some('5'),
            KeyCode::Key6 => Some('6'),
            KeyCode::Key7 => Some('7'),
            KeyCode::Key8 => Some('8'),
            KeyCode::Key9 => Some('9'),
            KeyCode::Key0 => Some('0'),
            KeyCode::A => Some('a'),
            KeyCode::B => Some('b'),
            KeyCode::C => Some('c'),
            KeyCode::D => Some('d'),
            KeyCode::E => Some('e'),
            KeyCode::F => Some('f'),
            KeyCode::G => Some('g'),
            KeyCode::H => Some('h'),
            KeyCode::I => Some('i'),
            KeyCode::J => Some('j'),
            KeyCode::K => Some('k'),
            KeyCode::L => Some('l'),
            KeyCode::M => Some('m'),
            KeyCode::N => Some('n'),
            KeyCode::O => Some('o'),
            KeyCode::P => Some('p'),
            KeyCode::Q => Some('q'),
            KeyCode::R => Some('r'),
            KeyCode::S => Some('s'),
            KeyCode::T => Some('t'),
            KeyCode::U => Some('u'),
            KeyCode::V => Some('v'),
            KeyCode::W => Some('w'),
            KeyCode::X => Some('x'),
            KeyCode::Y => Some('y'),
            KeyCode::Z => Some('z'),
            KeyCode::Space => Some(' '),
            KeyCode::Return => Some('\n'),
            KeyCode::Tab => Some('\t'),
            KeyCode::Comma => Some(','),
            KeyCode::Period => Some('.'),
            KeyCode::Apostrophe => Some('\''),
            KeyCode::Equals => Some('='),
            KeyCode::Minus => Some('-'),
            KeyCode::Slash => Some('/'),
            KeyCode::Backslash => Some('\\'),
            KeyCode::Grave => Some('`'),
            KeyCode::Semicolon => Some(';'),
            KeyCode::Colon => Some(':'),
            KeyCode::LBracket => Some('['),
            KeyCode::RBracket => Some(']'),
            _ => None,
        }
    } else {
        match keycode {
            KeyCode::Key1 => Some('!'),
            KeyCode::Key2 => Some('@'),
            KeyCode::Key3 => Some('#'),
            KeyCode::Key4 => Some('$'),
            KeyCode::Key5 => Some('%'),
            KeyCode::Key6 => Some('^'),
            KeyCode::Key7 => Some('&'),
            KeyCode::Key8 => Some('*'),
            KeyCode::Key9 => Some('('),
            KeyCode::Key0 => Some(')'),
            KeyCode::A => Some('A'),
            KeyCode::B => Some('B'),
            KeyCode::C => Some('C'),
            KeyCode::D => Some('D'),
            KeyCode::E => Some('E'),
            KeyCode::F => Some('F'),
            KeyCode::G => Some('G'),
            KeyCode::H => Some('H'),
            KeyCode::I => Some('I'),
            KeyCode::J => Some('J'),
            KeyCode::K => Some('K'),
            KeyCode::L => Some('L'),
            KeyCode::M => Some('M'),
            KeyCode::N => Some('N'),
            KeyCode::O => Some('O'),
            KeyCode::P => Some('P'),
            KeyCode::Q => Some('Q'),
            KeyCode::R => Some('R'),
            KeyCode::S => Some('S'),
            KeyCode::T => Some('T'),
            KeyCode::U => Some('U'),
            KeyCode::V => Some('V'),
            KeyCode::W => Some('W'),
            KeyCode::X => Some('X'),
            KeyCode::Y => Some('Y'),
            KeyCode::Z => Some('Z'),
            KeyCode::Space => Some(' '),
            KeyCode::Return => Some('\n'),
            KeyCode::Comma => Some('<'),
            KeyCode::Period => Some('>'),
            KeyCode::Apostrophe => Some('"'),
            KeyCode::Equals => Some('+'),
            KeyCode::Minus => Some('_'),
            KeyCode::Slash => Some('?'),
            KeyCode::Backslash => Some('|'),
            KeyCode::Grave => Some('~'),
            KeyCode::Semicolon => Some(':'),
            KeyCode::Colon => Some(':'),
            KeyCode::LBracket => Some('{'),
            KeyCode::RBracket => Some('}'),
            _ => None,
        }
    }
}

fn keycode_to_key(
    keycode: KeyCode,
    input: &Input<KeyCode>
) -> Option<termion::event::Key> {
    use termion::event::Key;

    let is_control =
        input.pressed(KeyCode::LControl) || input.pressed(KeyCode::RControl);
    let is_alt =
        input.pressed(KeyCode::LAlt) || input.pressed(KeyCode::RAlt);
    let is_shift =
        input.pressed(KeyCode::LShift) || input.pressed(KeyCode::RShift);

    if is_control && !is_alt {
        return Some(Key::Ctrl(keycode_to_letter(keycode, is_shift)?));
    }
    if !is_control && is_alt {
        return Some(Key::Alt(keycode_to_letter(keycode, is_shift)?));
    }
    if is_control && is_alt {
        return None;
    }
    if let Some(character) = keycode_to_letter(keycode, is_shift) {
        return Some(Key::Char(character));
    }

    match keycode {
        KeyCode::Back            => Some(Key::Backspace),
        KeyCode::Left            => Some(Key::Left),
        KeyCode::Right           => Some(Key::Right),
        KeyCode::Up              => Some(Key::Up),
        KeyCode::Down            => Some(Key::Down),
        KeyCode::Home            => Some(Key::Home),
        KeyCode::End             => Some(Key::End),
        KeyCode::PageUp          => Some(Key::PageUp),
        KeyCode::PageDown        => Some(Key::PageDown),
        KeyCode::Tab if is_shift => Some(Key::BackTab),
        KeyCode::Delete          => Some(Key::Delete),
        KeyCode::Insert          => Some(Key::Insert),
        KeyCode::Escape          => Some(Key::Esc),
        _                        => None,
    }
}

pub fn run_editor(
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut screens: Query<(&mut crate::editor::Screen, &Handle<StandardMaterial>)>,
    screen_activated: Res<ScreenActivated>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if let Some(entity) = screen_activated.entity {
        let (mut screen, material_handle) = screens.get_mut(entity).unwrap();

        let mut needs_rerender = false;

        for keycode in keyboard_input.get_just_pressed() {
            if let Some(key) = keycode_to_key(*keycode, &keyboard_input) {
                screen.editor.process_keypress(key);
                needs_rerender = true;
            }
        }

        if !needs_rerender {
            return;
        }

        screen.editor.refresh_screen();

        let rasterized = screen.editor.rasterize().unwrap();

        let image_handle =
            materials.get_mut(material_handle).unwrap()
            .base_color_texture.clone().unwrap();
        let image: &mut Image = images.get_mut(&image_handle).unwrap();

        {
            let mut index = 0;
            for y in 0 .. rasterized.height {
                for x in 0 .. rasterized.width {
                    let [a, r, g, b] =
                        rasterized.get((rasterized.width - 1) - x, y).to_le_bytes();
                    image.data[index + 0] = b;
                    image.data[index + 1] = g;
                    image.data[index + 2] = r;
                    image.data[index + 3] = a;
                    index += 4;
                }
            }
        }

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

pub fn interact(
    mut commands: Commands,
    mouse: Res<Input<MouseButton>>,
    selected: Res<Selected>,
    items: Query<&Item>,
    mut inventories: Query<&mut Inventory, With<LogicalPlayer>>,
    screens: Query<&crate::editor::Screen>,
    mut screen_activated: ResMut<ScreenActivated>,
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
