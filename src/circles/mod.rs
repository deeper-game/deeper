use crate::assets::{GameState, FontAssets};
use crate::fps_controller::RenderPlayer;
use crate::netcode::Peer;
use bevy::prelude::*;
use bevy::text::*;
use bevy_rapier3d::prelude::RapierContext;
use num_traits::float::FloatConst;

pub mod flesh;
pub mod bubbles;
pub mod associa;

pub struct CirclePlugin;

impl Plugin for CirclePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(MaterialPlugin::<flesh::FleshCircleMaterial>::default())
            .add_plugin(MaterialPlugin::<bubbles::BubblesCircleMaterial>::default())
            .add_system(load_font_atlas.in_schedule(OnEnter(GameState::Ready)))
            .add_system(flesh::update_flesh_circles.run_if(in_state(GameState::Ready)))
            .add_system(bubbles::update_bubbles_circles.run_if(in_state(GameState::Ready)))
            .add_system(associa::update_associa_circles.run_if(in_state(GameState::Ready)))
            .add_system(debug_circles.run_if(in_state(GameState::Ready)));
    }
}

pub fn load_font_atlas(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
) {
    let mut text_bundle = TextBundle::from_section(
        "∀∃∅∧∨⊔⊓⊏⊑⊗⊕⊖⊛⊸⋈⋉≡⊤⊥⊦⊧",
        TextStyle {
            font: font_assets.dejavu_sans.clone(),
            font_size: 60.0,
            color: Color::WHITE,
        },
    );
    text_bundle.visibility = Visibility::Hidden;
    commands.spawn(text_bundle);
}

pub fn debug_circles(
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut flesh_circle_materials: ResMut<Assets<flesh::FleshCircleMaterial>>,
    mut bubbles_circle_materials: ResMut<Assets<bubbles::BubblesCircleMaterial>>,
    mut associa_circle_materials: ResMut<Assets<associa::AssociaCircleMaterial>>,
    camera: Query<&Transform, (With<RenderPlayer>, Without<Peer>)>,

    rapier_context: Res<RapierContext>,

    font_assets: Res<FontAssets>,
    fonts: Res<Assets<Font>>,
    font_atlas_sets: Res<Assets<FontAtlasSet>>,
    texture_atlases: Res<Assets<TextureAtlas>>,
) {
    let cam = camera.single();
    let transform = cam
        .mul_transform(Transform::from_translation(Vec3::new(0.0, 0.0, -0.5)))
        .mul_transform(
            Transform::from_rotation(Quat::from_rotation_x(3.0 * f32::PI() / 2.0)));
    if keyboard.just_pressed(KeyCode::Key1) {
        flesh::create_flesh_circle(
            &time, &mut commands, &mut meshes, &mut materials,
            &mut flesh_circle_materials, &transform);
    }
    if keyboard.just_pressed(KeyCode::Key2) {
        bubbles::create_bubbles_circle(
            &time, &mut commands, &mut meshes, &mut materials,
            &mut bubbles_circle_materials, &transform,
            &rapier_context,
            &font_assets, &fonts, &font_atlas_sets, &texture_atlases);
    }
    if keyboard.just_pressed(KeyCode::Key3) {
        associa::create_associa_circle(
            &time, &mut commands, &mut meshes, &mut materials,
            &mut associa_circle_materials, &transform);
    }
}
