use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_local_commands::{
    ActiveProcessMap, BevyLocalCommandsPlugin, KillProcess, ProcessCompleted, ProcessOutputEvent,
    RunProcess,
};

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, BevyLocalCommandsPlugin))
        .add_systems(Startup, startup)
        .add_systems(Update, update)
        // Kill the command after 6s
        .add_systems(Update, kill.run_if(on_timer(Duration::from_secs(6))))
        .run();
}

fn startup(mut shell_commands: EventWriter<RunProcess>) {
    if cfg!(windows) {
        shell_commands.send(RunProcess::new(
            "cmd",
            vec!["/C", "echo Sleeping for 10s && timeout 10 && echo Done"],
        ));
    } else if cfg!(unix) {
        shell_commands.send(RunProcess::new(
            "sh",
            vec!["-c", "echo Sleeping for 10s && sleep 10 && echo Done"],
        ));
    } else {
        println!("Could not choose appropriate command to run on current platform");
        std::process::exit(0);
    }
}

fn kill(active_processes: Res<ActiveProcessMap>, mut kill_process_event: EventWriter<KillProcess>) {
    for &pid in active_processes.0.keys() {
        println!("Killing {pid}");
        kill_process_event.send(KillProcess(pid));
    }
}

fn update(
    mut process_output_event: EventReader<ProcessOutputEvent>,
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