use bevy::prelude::*;
use crate::assets::GameState;

pub mod halo;

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(crate::enemies::halo::HaloPlugin)
            .add_system(spawn_guys.in_schedule(OnEnter(GameState::Ready)));
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
