use bevy::prelude::*;

pub struct ImportableShadersPlugin;

impl Plugin for ImportableShadersPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(load_shaders);
    }
}

#[derive(Component)]
pub struct ImportableShader {
    name: String,
    shader: Option<Handle<Shader>>,
}

impl ImportableShader {
    pub fn new(name: &str) -> Self {
        ImportableShader {
            name: name.to_string(),
            shader: None,
        }
    }
}

#[derive(Component)]
struct Loaded;

fn load_shaders(
    mut commands: Commands,
    mut asset_server: ResMut<AssetServer>,
    mut shaders: ResMut<Assets<Shader>>,
    mut importables: Query<(Entity, &mut ImportableShader), Without<Loaded>>,
) {
    for (entity, mut importable) in importables.iter_mut() {
        if importable.shader.is_none() {
            importable.shader = Some(asset_server.load::<Shader, _>(
                format!("shaders/{}.wgsl", importable.name)));
        }
        let shader_handle = importable.shader.clone().unwrap();
        if let Some(mut shader) = shaders.get_mut(&shader_handle) {
            shader.set_import_path(format!("deeper::{}", importable.name));
            commands.entity(entity).insert(Loaded);
        }
    }
}
