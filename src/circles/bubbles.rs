use crate::assets::FontAssets;
use crate::spline::PiecewiseLinearSpline;
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


#[derive(Component)]
pub struct BubblesCircle {
    start_time: Instant,
}

pub fn create_bubbles_circle(
    time: &Res<Time>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    bubbles_circle_materials: &mut ResMut<Assets<BubblesCircleMaterial>>,
    transform: &Transform,

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

    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(crate::shapes::TwoSided { size: bubbles_size })),
            material: bubbles_circle_materials.add(BubblesCircleMaterial {
                uniform: BubblesCircleMaterialUniform {
                    time: 0.0,
                    bubble_glyph_0: lookup_rect('∀'),
                    bubble_glyph_1: lookup_rect('∃'),
                    bubble_glyph_2: lookup_rect('⊔'),
                    bubble_glyph_3: lookup_rect('⊗'),
                    bubble_glyph_4: lookup_rect('⊸'),
                    bubble_glyph_5: lookup_rect('⋈'),
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
    bubbles_circles: Query<(Entity, &GlobalTransform, &Handle<BubblesCircleMaterial>, &BubblesCircle)>,
    mut bubbles_circle_materials: ResMut<Assets<BubblesCircleMaterial>>,
) {
    let time_scale = 0.5;
    let duration = 2.5;
    for (circle_entity, circle_gt, material, bubbles_circle) in bubbles_circles.iter() {
        if time.last_update().unwrap() >= bubbles_circle.start_time {
            let t = (time.last_update().unwrap() - bubbles_circle.start_time)
                .as_secs_f32() * time_scale;
            if t > duration {
                commands.entity(circle_entity).despawn();
                continue;
            }
            let m = &mut bubbles_circle_materials.get_mut(material).unwrap();
            m.uniform.time = t;
        }
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

#[derive(ShaderType, Debug, Clone)]
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
    bubble_glyph_0: GlyphRect,
    #[align(16)]
    bubble_glyph_1: GlyphRect,
    #[align(16)]
    bubble_glyph_2: GlyphRect,
    #[align(16)]
    bubble_glyph_3: GlyphRect,
    #[align(16)]
    bubble_glyph_4: GlyphRect,
    #[align(16)]
    bubble_glyph_5: GlyphRect,
}

impl Material for BubblesCircleMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/bubbles_circle.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
