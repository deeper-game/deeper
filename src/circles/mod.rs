use bevy::prelude::*;
use num_traits::float::FloatConst;

pub mod flesh;

pub struct CirclePlugin;

impl Plugin for CirclePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(MaterialPlugin::<flesh::FleshCircleMaterial>::default())
            .add_system(flesh::update_flesh_circles)
            .add_system(debug_circles);
    }
}

pub fn debug_circles(
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut flesh_circle_materials: ResMut<Assets<flesh::FleshCircleMaterial>>,
    camera: Query<&Transform, With<crate::fps_controller::RenderPlayer>>,
) {
    let cam = camera.single();
    if keyboard.just_pressed(KeyCode::Key1) {
        let fc_transform = cam
            .mul_transform(Transform::from_translation(Vec3::new(0.0, 0.0, -0.5)))
            .mul_transform(
                Transform::from_rotation(Quat::from_rotation_x(3.0 * f32::PI() / 2.0)));
        flesh::create_flesh_circle(
            &time, &mut commands, &mut meshes, &mut materials,
            &mut flesh_circle_materials, &fc_transform);
    }
}
