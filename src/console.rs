use bevy::prelude::*;
use bevy_console::{ConsoleConfiguration, ConsoleCommand, AddConsoleCommand};

#[derive(Resource)]
struct Console {
}

pub struct ConsolePlugin;

impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(bevy_console::ConsolePlugin)
            .insert_resource(ConsoleConfiguration {
                // override config here
                ..Default::default()
            })
            .add_console_command::<SpawnGuiCommand, _>(spawn_gui)
            .add_console_command::<SetGuiCommand, _>(set_gui);
    }
}



#[derive(ConsoleCommand)]
#[console_command(name = "spawn_gui")]
struct SpawnGuiCommand {
    name: String,
}

fn spawn_box(
    mut commands: Commands,
    mut log: ConsoleCommand<SpawnGuiCommand>,
) {

}

#[derive(ConsoleCommand)]
#[console_command(name = "set_gui")]
struct SetGuiCommand {
    name: String,
}

fn set_gui(
    mut log: ConsoleCommand<SpawnGuiCommand>,
) {
}
