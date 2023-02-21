use crate::self_destruct::SelfDestructing;
use bevy::prelude::*;
use bevy::utils::{Instant, Duration};

pub struct ExplosionPlugin;

impl Plugin for ExplosionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_explosions);
    }
}

#[derive(Component)]
pub struct Explosion;

#[derive(Clone)]
pub struct ExplosionSettings {
    pub mesh: Handle<Mesh>,
}

pub fn create_explosion(commands: &mut Commands,
                        materials: &mut Assets<StandardMaterial>,
                        settings: &ExplosionSettings,
                        transform: &Transform) {
    commands.spawn((
        PbrBundle {
            mesh: settings.mesh.clone(),
            material: materials.add(StandardMaterial {
                unlit: true,
                ..default()
            }),
            transform: transform.clone(),
            ..default()
        },
        Explosion,
        SelfDestructing::new(Duration::from_millis(500)),
    ));
}

fn update_explosions(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut explosions: Query<(&mut Transform, &Handle<StandardMaterial>), With<Explosion>>,
) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut colors = [
        Color::ORANGE,
        Color::ORANGE_RED,
        Color::RED,
        Color::ORANGE,
        Color::ORANGE_RED,
        Color::RED,
        Color::ORANGE,
        Color::ORANGE_RED,
        Color::RED,
        Color::WHITE,
        Color::GRAY,
    ];
    for mut color in colors.iter_mut() {
        let [r, g, b, a] = color.clone().as_linear_rgba_f32();
        let explosion_bloom = 3.0;
        *color = Color::rgba_linear(explosion_bloom * r,
                                    explosion_bloom * g,
                                    explosion_bloom * b,
                                    a);
    }
    let color_dist = rand::distributions::Slice::new(&colors).unwrap();
    for (mut transform, material) in explosions.iter_mut() {
        transform.translation += 0.01 * Vec3::new(
            rng.gen::<f32>() - 0.5,
            rng.gen::<f32>() - 0.5,
            rng.gen::<f32>() - 0.5,
        );
        transform.scale = Vec3::ONE * (1.0 - 0.25 + 0.5 * rng.gen::<f32>());
        materials.get_mut(material).unwrap().base_color =
            *rng.sample(&color_dist);
    }
}
