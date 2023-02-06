use bevy::prelude::*;

pub mod flesh;

pub struct CirclePlugin;

impl Plugin for CirclePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(MaterialPlugin::<flesh::FleshCircleMaterial>::default())
            .add_startup_system(flesh::create_flesh_circle)
            .add_system(flesh::update_flesh_circles);
    }
}
