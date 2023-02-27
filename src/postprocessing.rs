use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{
    AsBindGroup, ShaderRef, ShaderType
};

pub struct PostprocessingPlugin;

impl Plugin for PostprocessingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(MaterialPlugin::<PostprocessingMaterial>::default());
    }
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone, Default)]
#[uuid = "a490b695-f79f-4606-baa6-1e57fc96998b"]
pub struct PostprocessingMaterial {
    #[uniform(0)]
    pub uniform: PostprocessingMaterialUniform,
    #[texture(1)]
    #[sampler(2)]
    pub input: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    pub palette: Handle<Image>,
}

#[derive(ShaderType, Debug, Clone, Default)]
pub struct PostprocessingMaterialUniform {
    #[align(16)]
    pub placeholder: Vec4,
}

impl Material for PostprocessingMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/postprocessing.wgsl".into()
    }
}
