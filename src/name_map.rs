use bevy::prelude::*;

#[derive(Resource)]
struct Console {
}

pub struct ConsolePlugin;

impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ConsoleConfiguration {
                // override config here
                ..Default::default()
            })
            .add_console_command::<SpawnGuiCommand, _>(spawn_gui)
            .add_console_command::<SetGuiCommand, _>(set_gui);
    }
}
