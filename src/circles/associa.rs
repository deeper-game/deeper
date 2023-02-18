use crate::assets::FontAssets;
use crate::self_destruct::SelfDestructing;
use ab_glyph::Font as FontTrait;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{AsBindGroup, ShaderRef, ShaderType, PrimitiveTopology};
use bevy::text::*;
use bevy_rapier3d::prelude::{RapierContext, QueryFilter};
use bevy::utils::{Instant, Duration};
use num_traits::float::FloatConst;

#[derive(Component)]
pub struct AssociaCircle {
    start_time: Instant,
}

pub fn create_associa_circle(
    time: &Res<Time>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    associa_circle_materials: &mut ResMut<Assets<AssociaCircleMaterial>>,
    transform: &Transform,
) {
    let associa_size = 0.3;

    let mut vertices = Vec::new();
    let diamond_size = 0.71743893521;
    let tip_distance = 1.5;
    vertices.push(Mat3::IDENTITY.mul_vec3(Vec3::new(0.0, diamond_size, 1.0)));
    vertices.push(Mat3::IDENTITY.mul_vec3(Vec3::new(-diamond_size, 0.0, 1.0)));
    vertices.push(Mat3::IDENTITY.mul_vec3(Vec3::new(0.0, -diamond_size, 1.0)));
    vertices.push(Mat3::IDENTITY.mul_vec3(Vec3::new(diamond_size, 0.0, 1.0)));
    vertices.push(Mat3::from_rotation_y(std::f32::consts::TAU * (1.0/3.0)).mul_vec3(Vec3::new(0.0, diamond_size, 1.0)));
    vertices.push(Mat3::from_rotation_y(std::f32::consts::TAU * (1.0/3.0)).mul_vec3(Vec3::new(-diamond_size, 0.0, 1.0)));
    vertices.push(Mat3::from_rotation_y(std::f32::consts::TAU * (1.0/3.0)).mul_vec3(Vec3::new(0.0, -diamond_size, 1.0)));
    vertices.push(Mat3::from_rotation_y(std::f32::consts::TAU * (1.0/3.0)).mul_vec3(Vec3::new(diamond_size, 0.0, 1.0)));
    vertices.push(Mat3::from_rotation_y(std::f32::consts::TAU * (2.0/3.0)).mul_vec3(Vec3::new(0.0, diamond_size, 1.0)));
    vertices.push(Mat3::from_rotation_y(std::f32::consts::TAU * (2.0/3.0)).mul_vec3(Vec3::new(-diamond_size, 0.0, 1.0)));
    vertices.push(Mat3::from_rotation_y(std::f32::consts::TAU * (2.0/3.0)).mul_vec3(Vec3::new(0.0, -diamond_size, 1.0)));
    vertices.push(Mat3::from_rotation_y(std::f32::consts::TAU * (2.0/3.0)).mul_vec3(Vec3::new(diamond_size, 0.0, 1.0)));
    vertices.push(Vec3::new(0.0, tip_distance, 0.0));
    vertices.push(Vec3::new(0.0, -tip_distance, 0.0));

    for i in

    let edges = vec![
        0, 1,
        1, 2,
        2, 3,
        3, 0,

        4, 5,
        5, 6,
        6, 7,
        7, 4,

        8, 9,
        9, 10,
        10, 11,
        11, 8,

        0, 12,
        2, 13,
        4, 12,
        6, 13,
        8, 12,
        10, 13,

        3, 5,
        7, 9,
        11, 1,
    ];

    let mut mesh = Mesh::new(PrimitiveTopology::LineList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.set_indices(Some(bevy::render::mesh::Indices::U32(edges)));

    commands.spawn((
        MaterialMeshBundle {
            //mesh: meshes.add(Mesh::from(crate::shapes::TwoSided { size: associa_size })),
            mesh: meshes.add(mesh),
            material: associa_circle_materials.add(AssociaCircleMaterial {
                uniform: AssociaCircleMaterialUniform {
                    time: 0.0,
                },
            }),
            transform: transform.clone(),
            ..default()
        },
        bevy::pbr::NotShadowCaster,
        AssociaCircle {
            start_time: time.last_update().unwrap() + Duration::from_millis(100),
        },
    ));
}

pub fn update_associa_circles(
    mut commands: Commands,
    time: Res<Time>,
    associa_circles: Query<(Entity, &Handle<AssociaCircleMaterial>, &AssociaCircle)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut associa_circle_materials: ResMut<Assets<AssociaCircleMaterial>>,
) {
    let circle_duration = 20.0;
    for (circle_entity, material, associa_circle) in associa_circles.iter() {
        if time.last_update().unwrap() >= associa_circle.start_time {
            let t = (time.last_update().unwrap() - associa_circle.start_time)
                .as_secs_f32() / circle_duration;
            if t > 1.0 {
                commands.entity(circle_entity).despawn();
                continue;
            }
            let m = &mut associa_circle_materials.get_mut(material).unwrap();
            m.uniform.time = t;
        }
    }
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "f2b9f9d7-b7fa-40a7-8294-5f4e54bfec8d"]
pub struct AssociaCircleMaterial {
    #[uniform(0)]
    uniform: AssociaCircleMaterialUniform,
}

#[derive(ShaderType, Debug, Clone)]
struct AssociaCircleMaterialUniform {
    #[align(16)]
    time: f32,
}

impl Material for AssociaCircleMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/associa_circle.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
