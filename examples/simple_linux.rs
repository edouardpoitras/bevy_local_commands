use bevy::prelude::*;
use bevy_local_commands::{
    BevyLocalCommandsPlugin, ProcessCompleted, ProcessOutputEvent, RunProcess,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, BevyLocalCommandsPlugin))
        .add_systems(Startup, startup)
        .add_systems(Update, update)
        .run();
}

fn startup(mut shell_commands: EventWriter<RunProcess>) {
    shell_commands.send(RunProcess::new(
        "bash",
        vec!["-c", "echo Sleeping for 1s && sleep 1 && echo Done"],
    ));
}

fn update(
    mut shell_command_output: EventReader<ProcessOutputEvent>,
    mut shell_command_completed: EventReader<ProcessCompleted>,
) {
    for command_output in shell_command_output.read() {
        for line in command_output.output.iter() {
            info!("Output Line ({}): {line}", command_output.pid);
        }
    }
    if !shell_command_completed.is_empty() {
        let completed = shell_command_completed.read().last().unwrap();
        info!(
            "Command completed (PID - {}, Success - {}): {}",
            completed.pid, completed.success, completed.command
        );
    }
}
