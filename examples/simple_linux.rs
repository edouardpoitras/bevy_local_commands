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
    mut process_output_event: EventReader<ProcessOutputEvent>,
    mut process_completed_event: EventReader<ProcessCompleted>,
) {
    for command_output in process_output_event.read() {
        for line in command_output.output.iter() {
            info!("Output Line ({}): {line}", command_output.pid);
        }
    }
    if !process_completed_event.is_empty() {
        let completed = process_completed_event.read().last().unwrap();
        info!(
            "Command completed (PID - {}, Success - {}): {}",
            completed.pid, completed.success, completed.command
        );
    }
}
