use bevy::prelude::*;
use bevy_rapier3d::prelude::{
    RapierContext, Collider, QueryFilter, RayIntersection
};
use crate::fps_controller::LogicalPlayer;

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<ProjectileImpact>()
            .add_system(move_projectiles);
    }
}

#[derive(Component)]
pub struct Projectile {
    pub velocity: Vec3
}

impl Projectile {
    pub fn velocity(&self) -> Vec3 {
        self.velocity
    }

    pub fn speed(&self) -> f32 {
        f32::sqrt(self.velocity.dot(self.velocity))
    }
}

#[derive(Clone, Debug)]
pub struct ProjectileImpact {
    pub projectile_entity: Entity,
    pub hit_entity: Entity,
    pub intersection: RayIntersection,
}

fn move_projectiles(
    rapier_context: Res<RapierContext>,
    mut projectiles: Query<(Entity, &mut GlobalTransform, &Projectile)>,
    player: Query<Entity, With<LogicalPlayer>>,
    mut hit_events: EventWriter<ProjectileImpact>,
) {
    let Ok(player_entity) = player.get_single() else { return; };
    for (projectile_entity, mut pose, projectile) in projectiles.iter_mut() {
        *pose =
            GlobalTransform::IDENTITY
            .mul_transform(Transform::from_translation(projectile.velocity())
                           .mul_transform(pose.compute_transform()));
        if let Some((hit_entity, intersection)) =
            rapier_context.cast_ray_and_get_normal(
                pose.translation(), projectile.velocity(), 1.0, false,
                QueryFilter::default().exclude_rigid_body(player_entity))
        {
            let impact = ProjectileImpact {
                projectile_entity,
                hit_entity,
                intersection,
            };
            println!("Projectile impact: {:?}", impact);
            hit_events.send(impact);
        }
    }
}
