use crate::self_destruct::SelfDestructing;
use bevy::prelude::*;
use bevy::utils::{Instant, Duration};

pub struct TrailPlugin;

impl Plugin for TrailPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_trails);
    }
}

#[derive(Component)]
pub struct TrailGenerating {
    pub material: Handle<StandardMaterial>,
    pub fade_duration: Duration,
    pub previous_time: Instant,
    pub previous_position: Vec3,
}

fn trail_line_segment(
    meshes: &mut ResMut<Assets<Mesh>>,
    material: Handle<StandardMaterial>,
    start: Vec3,
    end: Vec3,
    radius: f32,
) -> PbrBundle {
    let center = (end + start) / 2.0;
    let length = (end - start).length();
    let mesh = meshes.add(Mesh::from(
        shape::Box::new(radius, radius, length)));
    let transform = Transform::from_translation(center)
        .looking_at(end, Vec3::NEG_Y);
    PbrBundle { mesh, material, transform, ..default() }
}

fn update_trails(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    time: Res<Time>,
    mut trail_generators: Query<(&mut TrailGenerating, &Transform)>,
) {
    for (mut trail_generator, transform) in trail_generators.iter_mut() {
        if (transform.translation - trail_generator.previous_position).length() > 0.1 {
            commands.spawn((
                trail_line_segment(
                    &mut meshes, trail_generator.material.clone(),
                    trail_generator.previous_position,
                    transform.translation,
                    0.001),
                SelfDestructing::new(trail_generator.fade_duration),
                bevy::pbr::NotShadowCaster,
            ));
            trail_generator.previous_time = time.last_update().unwrap();
            trail_generator.previous_position = transform.translation;
        }
    }
}
