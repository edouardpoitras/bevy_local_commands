use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_local_commands::{
    BevyLocalCommandsPlugin, LocalCommand, Process, ProcessCompleted, ProcessOutput,
};

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, BevyLocalCommandsPlugin))
        .add_systems(Startup, startup)
        .add_systems(Update, update)
        // Kill the command after 2s
        .add_systems(Update, kill.run_if(on_timer(Duration::from_secs(2))))
        .run();
}

fn startup(mut commands: Commands) {
    // Choose the command based on the OS
    #[cfg(not(windows))]
    let cmd = LocalCommand::new("sh").args([
        "-c",
        "echo Sleeping for 4s && sleep 4 && echo This should not print or execute && sleep 100",
    ]);
    #[cfg(windows)]
    let cmd = LocalCommand::new("cmd").args([
        "/C",
        "echo Sleeping for 4s && timeout 4 && echo This should not print or execute && timeout 100",
    ]);

    let id = commands.spawn(cmd).id();
    println!("Spawned the command as entity {id:?}")
}

fn kill(mut active_processes: Query<(Entity, &mut Process)>) {
    for (entity, mut process) in active_processes.iter_mut() {
        println!("Killing {entity:?}");
        process.kill().unwrap();
    }
}

fn update(
    mut process_output_event: EventReader<ProcessOutput>,
    mut process_completed_event: EventReader<ProcessCompleted>,
) {
    for process_output in process_output_event.read() {
        for line in process_output.output.iter() {
            println!("Output Line ({:?}): {line}", process_output.entity);
        }
    }
    if let Some(completed) = process_completed_event.read().last() {
        println!(
            "Command {:?} completed (Success - {})",
            completed.entity,
            completed.exit_status.success()
        );
        // Quit the app
        std::process::exit(0);
    }
}
