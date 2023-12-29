use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_local_commands::{
    ActiveProcessMap, BevyLocalCommandsPlugin, KillProcess, ProcessCompleted, ProcessOutputEvent,
    RunProcess,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, BevyLocalCommandsPlugin))
        .add_systems(Startup, startup)
        .add_systems(Update, update)
        // Kill the command after 6s
        .add_systems(Update, kill.run_if(on_timer(Duration::from_secs(6))))
        .run();
}

fn startup(mut shell_commands: EventWriter<RunProcess>) {
    shell_commands.send(RunProcess::new(
        "bash",
        vec!["-c", "echo Sleeping for 10s && sleep 10 && echo Done"],
    ));
}

fn kill(active_processes: Res<ActiveProcessMap>, mut kill_shell_command: EventWriter<KillProcess>) {
    for &pid in active_processes.0.keys() {
        info!("Killing {pid}");
        kill_shell_command.send(KillProcess(pid));
    }
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
