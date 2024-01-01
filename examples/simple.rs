use bevy::prelude::*;
use bevy_local_commands::{
    BevyLocalCommandsPlugin, ProcessCompleted, ProcessOutput, RunProcess,
};

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, BevyLocalCommandsPlugin))
        .add_systems(Startup, startup)
        .add_systems(Update, update)
        .run();
}

fn startup(mut shell_commands: EventWriter<RunProcess>) {
    if cfg!(windows) {
        shell_commands.send(RunProcess::new(
            "cmd",
            vec!["/C", "echo Sleeping for 1s && timeout 1 && echo Done"],
        ));
    } else if cfg!(unix) {
        shell_commands.send(RunProcess::new(
            "sh",
            vec!["-c", "echo Sleeping for 1s && sleep 1 && echo Done"],
        ));
    } else {
        println!("Could not choose appropriate command to run on current platform");
        std::process::exit(0);
    }
}

fn update(
    mut process_output_event: EventReader<ProcessOutput>,
    mut process_completed_event: EventReader<ProcessCompleted>,
) {
    for command_output in process_output_event.read() {
        for line in command_output.output.iter() {
            println!("Output Line ({}): {line}", command_output.pid);
        }
    }
    if !process_completed_event.is_empty() {
        let completed = process_completed_event.read().last().unwrap();
        println!(
            "Command completed (PID - {}, Success - {}): {}",
            completed.pid, completed.success, completed.command
        );
        // Quit the app
        std::process::exit(0);
    }
}
