use bevy::prelude::*;

pub struct AddBloomPlugin;

impl Plugin for AddBloomPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(add_bloom);
    }
}

#[derive(Clone, Debug, Component)]
pub struct AddBloom {
    pub scale: f32,
}

fn add_bloom(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut to_add: Query<(Entity, &AddBloom, &mut Handle<StandardMaterial>)>,
) {
    let mut entities = Vec::new();
    for (entity, add_bloom, mut material_handle) in to_add.iter_mut() {
        entities.push(entity);
        let mut material = materials.get(&material_handle).unwrap().clone();
        material.emissive *= add_bloom.scale;
        *material_handle = materials.add(material);
    }
    for entity in entities {
        commands.entity(entity).remove::<AddBloom>();
    }
}
