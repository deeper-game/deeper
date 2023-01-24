use bevy::prelude::*;
use bevy_rapier3d::prelude::{
    RapierContext, Collider, QueryFilter, RayIntersection
};
use crate::fps_controller::LogicalPlayer;
use crate::projectile::ProjectileImpact;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(take_damage)
            .add_system(move_enemies);
    }
}

#[derive(Component)]
pub struct Health {
    pub amount: f32
}

#[derive(Component)]
pub struct Enemy;

pub fn spawn_enemy(
    mut commands: &mut Commands,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
) {
    commands.spawn_bundle((
        Enemy,
        Health { amount: 1.0 },
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.25 })),
            material: materials.add(Color::rgb(0.5, 0.8, 0.5).into()),
            transform: Transform::from_xyz(
                position.x,
                position.y,
                position.z),
            ..default()
        }))
        .insert(Collider::cuboid(0.125, 0.125, 0.125));
}

fn take_damage(
    mut commands: Commands,
    mut hit_events: EventReader<ProjectileImpact>,
    mut enemies: Query<&mut Health, With<Enemy>>,
) {
    for hit in hit_events.iter() {
        if let Ok(mut health) = enemies.get_mut(hit.hit_entity) {
            (*health).amount -= 0.1;
            if (*health).amount < 0.0 {
                commands.entity(hit.hit_entity).despawn();
                println!("Killed baddy!");
            }
        }
    }
}

fn move_enemies (
) {
}
