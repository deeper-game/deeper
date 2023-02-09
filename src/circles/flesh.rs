use crate::spline::PiecewiseLinearSpline;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy_rapier3d::prelude::{RapierContext, QueryFilter};
use num_traits::float::FloatConst;
use std::time::Duration;

#[derive(Component)]
pub struct Laser;

#[derive(Component)]
pub struct FleshCircle {
    start_time: std::time::Instant,
    lasers: Vec<Entity>,
}

const LASER_CUBE_SIZE: f32 = 0.01;

pub fn create_flesh_circle(
    time: &Res<Time>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    flesh_circle_materials: &mut ResMut<Assets<FleshCircleMaterial>>,
    transform: &Transform,
) {
    let laser_material = materials.add(StandardMaterial {
        base_color: Color::Rgba {
            red: 0.871,
            green: 1.0,
            blue: 0.905,
            alpha: 1.0,
        },
        unlit: true,
        ..default()
    });
    let flesh_size = 0.1;
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut lasers = Vec::new();
    let number_of_lasers = 5;
    for i in 0 .. number_of_lasers {
        let r_squared: f32 =
            rng.sample::<f32, _>(rand::distributions::Open01)
            * f32::powi((flesh_size / 2.0) * 0.75, 2);
        let theta: f32 =
            rng.sample::<f32, _>(rand::distributions::Open01) * 2.0 * f32::PI();
        let x = f32::sqrt(r_squared) * f32::cos(theta);
        let z = f32::sqrt(r_squared) * f32::sin(theta);
        let entity_commands = commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(
                    shape::Cube { size: LASER_CUBE_SIZE }
                )),
                material: laser_material.clone(),
                transform: Transform::from_xyz(x, 0.01, z)
                    .with_scale(Vec3::ZERO),
                ..default()
            },
            bevy::pbr::NotShadowCaster,
            Laser,
        ));
        lasers.push(entity_commands.id());
    }
    let flesh = commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(crate::shapes::TwoSided { size: flesh_size })),
            material: flesh_circle_materials.add(FleshCircleMaterial {
                resolution: 75.0,
                radius: 0.0,
                border: 0.0,
                flesh_time: 0.0,
                alpha: 0.6,
            }),
            transform: transform.clone(),
            ..default()
        },
        bevy::pbr::NotShadowCaster,
        FleshCircle {
            start_time: time.last_update().unwrap() + Duration::from_millis(100),
            lasers: lasers.clone(),
        },
    )).id();
    commands.entity(flesh).push_children(&lasers);
}

pub fn update_flesh_circles(
    rapier_context: Res<RapierContext>,
    mut commands: Commands,
    time: Res<Time>,
    flesh_circles: Query<(Entity, &GlobalTransform, &Handle<FleshCircleMaterial>, &FleshCircle)>,
    mut lasers: Query<&mut Transform, With<Laser>>,
    mut flesh_circle_materials: ResMut<Assets<FleshCircleMaterial>>,
) {
    let time_scale = 0.5;
    let duration = 2.5;
    let radius_spline = PiecewiseLinearSpline::new(&[
        (0.0, 0.0),
        (0.4, 0.4),
        (0.9, 0.4),
        (1.0, 0.0),
    ]);
    let length_spline = PiecewiseLinearSpline::new(&[
        (0.0, 0.0),
        (0.775, 0.0),
        (0.8, 1.0),
        (0.83, 0.0),
        (0.9, 0.0),
        (1.0, 0.0),
    ]);
    for (circle_entity, circle_gt, material, flesh_circle) in flesh_circles.iter() {
        let t = (time.last_update().unwrap() - flesh_circle.start_time)
            .as_secs_f32() * time_scale;
        if t >= 0.0 {
            if t > duration {
                for laser in &flesh_circle.lasers {
                    commands.entity(laser.clone()).despawn();
                }
                commands.entity(circle_entity).despawn();
                continue;
            }
            let m = &mut flesh_circle_materials.get_mut(material).unwrap();
            m.flesh_time = t;
            m.radius = radius_spline.interpolate(t / duration);
            m.border = 0.025 * m.radius;
            for laser in &flesh_circle.lasers {
                let laser_transform =
                    &mut lasers.get_mut(laser.clone()).unwrap();

                let mut original_laser_transform = Transform::IDENTITY;
                original_laser_transform.translation = laser_transform.translation;
                original_laser_transform.translation.y = 0.0;
                let original_laser_gt =
                    circle_gt.mul_transform(original_laser_transform);
                let original_laser_y = original_laser_gt.up();
                let collision_result = rapier_context.cast_ray(
                    original_laser_gt.translation(),
                    original_laser_gt.up(),
                    f32::MAX,
                    true,
                    QueryFilter::new(),
                );
                let max_length =
                    2.0 * collision_result.map(|(_, toi)| toi).unwrap_or(1000.0);

                let current_length =
                    max_length * length_spline.interpolate(t / duration);
                let laser_distance = {
                    let mut p = laser_transform.translation.clone();
                    p.y = 0.0;
                    p.length()
                };
                let laser_scale = if m.radius > laser_distance {
                    m.radius
                } else {
                    0.0
                };
                laser_transform.translation.y = (current_length / 4.0) + 0.01;
                laser_transform.scale.x = laser_scale;
                laser_transform.scale.y = current_length / (2.0 * LASER_CUBE_SIZE);
                laser_transform.scale.z = laser_scale;
                laser_transform.rotation *= Quat::from_rotation_y(3.0);
            }
        }
    }
}


// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "9ce1803d-8107-48c0-8125-dbd4e838d03a"]
pub struct FleshCircleMaterial {
    #[uniform(0)]
    resolution: f32,
    #[uniform(0)]
    radius: f32,
    #[uniform(0)]
    border: f32,
    #[uniform(0)]
    flesh_time: f32,
    #[uniform(0)]
    alpha: f32,
}

impl Material for FleshCircleMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/flesh_circle.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
