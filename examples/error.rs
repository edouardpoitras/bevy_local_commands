use std::process::Command;

use bevy::prelude::*;
use bevy_local_commands::{BevyLocalCommandsPlugin, LocalCommand, ProcessError};

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, BevyLocalCommandsPlugin))
        .add_systems(Startup, startup)
        .add_systems(Update, update)
        .run();
}

fn startup(mut commands: Commands) {
    commands.spawn(LocalCommand::new(Command::new("commandthatdoesnotexist")));
}

fn update(mut process_error: EventReader<ProcessError>) {
    for error in process_error.read() {
        println!(
            "Error running command ({:?}): {:?}",
            error.entity, error.info
        );
        // Quit the app
        std::process::exit(0);
    }
}
