use bevy::prelude::*;
use bevy::utils::Duration;

pub struct SelfDestructPlugin;

impl Plugin for SelfDestructPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(execute_self_destruct);
    }
}

#[derive(Component)]
pub struct SelfDestructing {
    timer: Timer,
}

impl SelfDestructing {
    pub fn new(duration: Duration) -> SelfDestructing {
        SelfDestructing {
            timer: Timer::new(duration, TimerMode::Once),
        }
    }
}

fn execute_self_destruct(
    mut commands: Commands,
    time: Res<Time>,
    mut all: Query<(Entity, &mut SelfDestructing)>,
) {
    for (entity, mut sd) in all.iter_mut() {
        sd.timer.tick(time.delta());
        if sd.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}
