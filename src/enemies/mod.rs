use bevy::prelude::*;

pub mod halo;

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(crate::enemies::halo::HaloPlugin)
            .add_system_set(
                SystemSet::on_enter(crate::assets::GameState::Ready)
                    .with_system(spawn_guys));
    }
}

fn spawn_guys(
    mut commands: Commands,
) {
    commands.spawn((
        crate::enemies::halo::SpawnHalo {
            transform: Transform::from_translation(Vec3::new(5.0, 5.0, 5.0)),
        },
    ));
}
