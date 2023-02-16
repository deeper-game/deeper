use crate::assets::FontAssets;
use crate::self_destruct::SelfDestructing;
use crate::spline::{Frame, PiecewiseLinearSpline, CubicBezierCurve};
use ab_glyph::Font as FontTrait;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{
    AsBindGroup, ShaderRef, ShaderType
};
use bevy::text::*;
use bevy_rapier3d::prelude::{RapierContext, QueryFilter};
use bevy::utils::{Instant, Duration};
use num_traits::float::FloatConst;

const NUM_BUBBLES: usize = 6;

#[derive(Component)]
pub struct Missile {
    start_time: Instant,
    position_curve: CubicBezierCurve<Vec3>,
    frame_curve: PiecewiseLinearSpline<Frame>,
    seed: f32,
    explosion_settings: ExplosionSettings,
}

#[derive(Component)]
pub struct TrailGenerating {
    material: Handle<StandardMaterial>,
    fade_duration: Duration,
    previous_time: Instant,
    previous_position: Vec3,
}

#[derive(Component)]
pub struct BubblesCircle {
    start_time: Instant,
}

#[derive(Clone)]
pub struct ExplosionSettings {
    mesh: Handle<Mesh>,
}

#[derive(Component)]
pub struct Explosion;

pub fn create_missile(
    commands: &mut Commands,
    rapier_context: &Res<RapierContext>,
    transform: &Transform,
    mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
    explosion_settings: &ExplosionSettings,
    start_time: &Instant,
    seed: f32,
) -> Option<Entity> {
    let collision_result = rapier_context.cast_ray_and_get_normal(
        transform.translation,
        transform.forward(),
        f32::MAX,
        true,
        QueryFilter::new(),
    );

    if !collision_result.is_some() {
        return None;
    }

    let (_, ray_intersection) = collision_result.unwrap();

    // TODO(taktoa): scale this with the distance to the target
    let start_stiffness = 1.0;
    let end_stiffness = 1.0;

    let bezier = CubicBezierCurve {
        p0: transform.translation,
        p1: transform.translation + start_stiffness * transform.forward(),
        p2: ray_intersection.point + end_stiffness * ray_intersection.normal,
        p3: ray_intersection.point,
    };

    let frames: PiecewiseLinearSpline<Frame> = {
        let mut ts = Vec::new();
        let mut bezier_samples = Vec::new();
        let mut bezier_tangents = Vec::new();
        let resolution = 1000;
        for i in 0 .. resolution + 1 {
            let t = (i as f32) / (resolution as f32);
            ts.push(t);
            bezier_samples.push(bezier.interpolate(t));
            bezier_tangents.push(bezier.derivative(t).normalize());
        }
        let initial_frame = Frame {
            forward: bezier_tangents[0],
            up: transform.up(),
            right: transform.right(),
        };

        let frames = crate::spline::rotation_minimizing_frames(
            &initial_frame, &bezier_samples, &bezier_tangents);

        PiecewiseLinearSpline::new(
            &ts.iter().cloned().zip(frames.into_iter())
                .collect::<Vec<(f32, Frame)>>())
    };

    Some(commands.spawn((
        PbrBundle {
            mesh: mesh.clone(),
            material: material.clone(),
            transform: transform.clone(),
            visibility: Visibility::INVISIBLE,
            ..default()
        },
        Missile {
            start_time: start_time.clone(),
            position_curve: bezier,
            frame_curve: frames,
            seed: seed,
            explosion_settings: explosion_settings.clone(),
        },
    )).id())
}

pub fn create_bubbles_circle(
    time: &Res<Time>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    bubbles_circle_materials: &mut ResMut<Assets<BubblesCircleMaterial>>,
    transform: &Transform,

    rapier_context: &Res<RapierContext>,

    font_assets: &Res<FontAssets>,
    fonts: &Res<Assets<Font>>,
    font_atlas_sets: &Res<Assets<FontAtlasSet>>,
    texture_atlases: &Res<Assets<TextureAtlas>>,
) {
    let bubbles_size = 0.3;

    let font_handle: &Handle<Font> = &font_assets.dejavu_sans;
    let font: &Font = fonts.get(&font_assets.dejavu_sans).unwrap();
    let font_atlas_set: &FontAtlasSet =
        font_atlas_sets.get(&font_handle.cast_weak::<FontAtlasSet>()).unwrap();
    let font_atlas: &FontAtlas = &font_atlas_set.iter().next().unwrap().1[0];
    let font_texture_atlas: &TextureAtlas =
        texture_atlases.get(&font_atlas.texture_atlas).unwrap();

    let number_of_missiles = 20;

    let missile_material = materials.add(StandardMaterial {
        base_color: Color::rgb(1.0, 1.0, 1.0),
        unlit: true,
        ..default()
    });

    let missile_mesh = meshes.add(Mesh::from(shape::Icosphere {
        radius: 0.005,
        subdivisions: 1,
    }));

    let explosion_settings = ExplosionSettings {
        mesh: meshes.add(Mesh::from(shape::Icosphere {
            radius: 0.06,
            subdivisions: 1,
        })),
    };

    let trail_material = materials.add(StandardMaterial {
        base_color: Color::rgba(1.0, 1.0, 1.0, 0.4),
        unlit: true,
        ..default()
    });

    for i in 0 .. number_of_missiles {
        let fraction = (i as f32) / (number_of_missiles as f32);
        let distance = fraction * bubbles_size / 2.0;
        let theta = fraction * 16.1803398875 * std::f32::consts::TAU;
        let offset = Vec2::new(
            distance * f32::cos(theta),
            distance * f32::sin(theta));

        let mut xform = transform
            .mul_transform(
                Transform::from_rotation(
                    Quat::from_rotation_x(-3.0 * f32::PI() / 2.0)));

        let transformed_offset =
            xform.right() * offset.x + xform.up() * offset.y;

        xform.translation += transformed_offset;

        use rand::Rng;
        let mut rng = rand::thread_rng();
        let seed = (100000.0 * rng.gen::<f32>()) + (1000 * i) as f32;
        let start_time =
            time.last_update().unwrap() + Duration::from_millis(100);

        if let Some(missile_entity) = create_missile(
            commands, rapier_context, &xform, &missile_mesh,
            &missile_material, &explosion_settings, &start_time, seed
        ) {
            commands.entity(missile_entity).insert(TrailGenerating {
                material: trail_material.clone(),
                fade_duration: Duration::from_millis(500),
                previous_time: time.last_update().unwrap(),
                previous_position: xform.translation,
            });
        } else {
            return;
        }
    }

    let lookup_rect = |character: char| -> GlyphRect {
        let glyph_id = font.font.glyph_id(character);
        let offset = SubpixelOffset::from(ab_glyph::Point { x: 0.0, y: 0.0 });
        let glyph_index = font_atlas.get_glyph_index(glyph_id, offset).unwrap();
        let size = font_texture_atlas.size;
        let rect = font_texture_atlas.textures[glyph_index];
        assert_eq!(size.x, size.y);
        GlyphRect {
            min_x: rect.min.x / size.x,
            min_y: rect.min.y / size.y,
            max_x: rect.max.x / size.x,
            max_y: rect.max.y / size.y,
        }
    };

    use rand::seq::SliceRandom;
    let mut symbols = "∀∃∅∧∨⊔⊓⊏⊑⊗⊕⊖⊛⊸⋈⋉≡⊤⊥⊦⊧".chars().collect::<Vec<char>>();
    let mut rng = rand::thread_rng();
    symbols.shuffle(&mut rng);
    assert!(symbols.len() >= NUM_BUBBLES);

    let mut rects = [GlyphRect::default(); NUM_BUBBLES];
    for i in 0 .. NUM_BUBBLES {
        rects[i] = lookup_rect(symbols[i]);
    }

    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(crate::shapes::TwoSided { size: bubbles_size })),
            material: bubbles_circle_materials.add(BubblesCircleMaterial {
                uniform: BubblesCircleMaterialUniform {
                    time: 0.0,
                    bubble_glyphs: rects,
                },
                font_texture_atlas: Some(font_texture_atlas.texture.clone()),
            }),
            transform: transform.clone(),
            ..default()
        },
        bevy::pbr::NotShadowCaster,
        BubblesCircle {
            start_time: time.last_update().unwrap() + Duration::from_millis(100),
        },
    ));
}

pub fn update_bubbles_circles(
    rapier_context: Res<RapierContext>,
    mut commands: Commands,
    time: Res<Time>,
    mut missiles: Query<(Entity, &mut Visibility, &mut Transform, &Missile)>,
    bubbles_circles: Query<(Entity, &GlobalTransform, &Handle<BubblesCircleMaterial>, &BubblesCircle)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut bubbles_circle_materials: ResMut<Assets<BubblesCircleMaterial>>,
) {
    let circle_duration = 2.5;
    let missile_duration = 1.0;
    for (missile_entity, mut visibility, mut transform, missile) in missiles.iter_mut() {
        if time.last_update().unwrap() >= missile.start_time {
            let t = (time.last_update().unwrap() - missile.start_time)
                .as_secs_f32();
            if t > circle_duration {
                *visibility = Visibility::VISIBLE;
            }
            if t > circle_duration + missile_duration {
                commands.entity(missile_entity).despawn();
                commands.spawn((
                    PbrBundle {
                        mesh: missile.explosion_settings.mesh.clone(),
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
                continue;
            }
            let td = f32::clamp((t - circle_duration) / missile_duration, 0.0, 1.0);
            let envelope = 1.0 - ((1.0 - (2.0 * td)) * (1.0 - (2.0 * td)));
            let wiggliness = 0.15;
            let mut noise_x = noisy_bevy::simplex_noise_2d_seeded(
                Vec2::new(2.0 * td, 0.0),
                missile.seed);
            noise_x *= envelope * wiggliness;
            let mut noise_y = noisy_bevy::simplex_noise_2d_seeded(
                Vec2::new(0.0, 2.0 * td),
                missile.seed);
            noise_y *= envelope * wiggliness;
            let frame = missile.frame_curve.interpolate(td);
            transform.translation =
                missile.position_curve.interpolate(td)
                + (frame.right * noise_x) + (frame.up * noise_y);
            transform.rotation = Quat::from_mat3(&Mat3 {
                x_axis: frame.right,
                y_axis: frame.up,
                z_axis: -frame.forward,
            });
        }
    }
    for (circle_entity, circle_gt, material, bubbles_circle) in bubbles_circles.iter() {
        if time.last_update().unwrap() >= bubbles_circle.start_time {
            let t = (time.last_update().unwrap() - bubbles_circle.start_time)
                .as_secs_f32() / circle_duration;
            if t > 1.0 {
                commands.entity(circle_entity).despawn();
                continue;
            }
            let m = &mut bubbles_circle_materials.get_mut(material).unwrap();
            m.uniform.time = t;
        }
    }
}

pub fn trail_line_segment(
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

pub fn update_trails(
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
            ));
            trail_generator.previous_time = time.last_update().unwrap();
            trail_generator.previous_position = transform.translation;
        }
    }
}

pub fn update_explosions(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut explosions: Query<(&mut Transform, &Handle<StandardMaterial>), With<Explosion>>,
) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let colors = [
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

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "dba4e055-be56-4dae-b803-bf0b5b9f459c"]
pub struct BubblesCircleMaterial {
    #[uniform(0)]
    uniform: BubblesCircleMaterialUniform,
    #[texture(1)]
    #[sampler(2)]
    font_texture_atlas: Option<Handle<Image>>,
}

#[derive(ShaderType, Debug, Clone, Copy, Default)]
struct GlyphRect {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
}

#[derive(ShaderType, Debug, Clone)]
struct BubblesCircleMaterialUniform {
    #[align(16)]
    time: f32,
    #[align(16)]
    bubble_glyphs: [GlyphRect; 8],
}

impl Material for BubblesCircleMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/bubbles_circle.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
