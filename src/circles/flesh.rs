use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use num_traits::float::FloatConst;

#[derive(Component)]
pub struct Laser;

#[derive(Component)]
pub struct FleshCircle {
    start_time: std::time::Instant,
    lasers: Vec<Entity>,
}

pub fn create_flesh_circle(
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut flesh_circle_materials: ResMut<Assets<FleshCircleMaterial>>,
) {
    let laser_material = materials.add(StandardMaterial {
        base_color: Color::Rgba {
            red: 0.871,
            green: 1.0,
            blue: 0.905,
            alpha: 1.0,
        },
        ..default()
    });
    let flesh_size = 0.25;
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut lasers = Vec::new();
    let number_of_lasers = 20;
    for i in 0 .. number_of_lasers {
        let r_squared: f32 =
            rng.sample::<f32, _>(rand::distributions::Open01)
            * f32::powi(flesh_size / 4.0, 2);
        let theta: f32 =
            rng.sample::<f32, _>(rand::distributions::Open01) * 2.0 * f32::PI();
        let x = f32::sqrt(r_squared) * f32::cos(theta);
        let z = f32::sqrt(r_squared) * f32::sin(theta);
        let entity_commands = commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 0.01 })),
                material: laser_material.clone(),
                transform: Transform::from_xyz(x, 0.0, z),
                ..default()
            },
            bevy::pbr::NotShadowCaster,
            Laser,
        ));
        lasers.push(entity_commands.id());
    }
    let flesh = commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: flesh_size })),
            material: flesh_circle_materials.add(FleshCircleMaterial {
                resolution: 75.0,
                radius: 0.4,
                border: 0.01,
                flesh_time: 0.0,
                alpha: 0.75,
            }),
            transform: Transform::from_xyz(3.0, 0.75, 3.0)
                .looking_at(Vec3::new(1.0, -0.5, 1.0), Vec3::new(0.0, 1.0, 0.0)),
            ..default()
        },
        bevy::pbr::NotShadowCaster,
        FleshCircle {
            start_time: time.last_update().unwrap(),
            lasers: lasers.clone(),
        },
    )).id();
    commands.entity(flesh).push_children(&lasers);
}

pub fn update_flesh_circles(
    time: Res<Time>,
    flesh_circles: Query<(&Handle<FleshCircleMaterial>, &FleshCircle)>,
    mut lasers: Query<&mut Transform, With<Laser>>,
    mut flesh_circle_materials: ResMut<Assets<FleshCircleMaterial>>,
) {
    for (material, flesh_circle) in flesh_circles.iter() {
        let t = time.elapsed_seconds_wrapped();
        *(&mut flesh_circle_materials.get_mut(material).unwrap().flesh_time) = t;
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
