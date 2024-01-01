use bevy::prelude::*;
use bevy_local_commands::{BevyLocalCommandsPlugin, ProcessError, RunProcess};

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, BevyLocalCommandsPlugin))
        .add_systems(Startup, startup)
        .add_systems(Update, update)
        .run();
}

fn startup(mut shell_commands: EventWriter<RunProcess>) {
    shell_commands.send(RunProcess::new("commandthatdoesnotexist", vec!["-flags"]));
}

fn update(mut process_error: EventReader<ProcessError>) {
    for error in process_error.read() {
        println!(
            "Error running command ({}): {:?}",
            error.command, error.info
        );
        // Quit the app
        std::process::exit(0);
    }
}
