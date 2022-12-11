//pub mod character;
pub mod magic;
pub mod level;
//pub mod camera;

// pub fn main() {
//     println!("Hello, World!");
// }














use std::f32::consts::TAU;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use bevy_rapier3d::prelude::*;
use bevy_fps_controller::controller::*;
use crate::level::Level;

pub struct HelloPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(RapierConfiguration::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(FpsControllerPlugin)
        //.add_plugin(crate::camera::PlayerPlugin)
        .add_startup_system(setup)
        .add_system(manage_cursor)
        //.add_system(movement)
        .run();
}

#[derive(Component)]
pub struct Player;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let level = Level::from_png(&std::fs::File::open("./level.png").unwrap());

    for y in 0 .. level.height {
        for x in 0 .. level.width {
            // walls
            if level.has_wall(x, y).unwrap() {
                commands.spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                    material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                    transform: Transform::from_xyz(x as f32, 1.0, y as f32),
                    ..default()
                })
                    .insert(Collider::cuboid(0.5, 0.5, 0.5))
                    .insert_bundle(TransformBundle::from(
                        Transform::from_xyz(x as f32, 1.0, y as f32)));
            }
            // floors
            if level.has_floor(x, y).unwrap() {
                commands.spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                    material: materials.add(Color::rgb(0.0, 0.7, 0.6).into()),
                    transform: Transform::from_xyz(x as f32, 0.0, y as f32),
                    ..default()
                })
                    .insert(Collider::cuboid(0.5, 0.5, 0.5))
                    .insert_bundle(TransformBundle::from(
                        Transform::from_xyz(x as f32, 0.0, y as f32)));
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
