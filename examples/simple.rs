use bevy::prelude::*;
use bevy_local_commands::{BevyLocalCommandsPlugin, LocalCommand, ProcessCompleted, ProcessOutput};

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, BevyLocalCommandsPlugin))
        .add_systems(Startup, startup)
        .add_systems(Update, update)
        .run();
}

fn startup(mut commands: Commands) {
    // Choose the command based on the OS
    #[cfg(not(windows))]
    let cmd = LocalCommand::new("sh").args(["-c", "echo Sleeping for 1s && sleep 1 && echo Done"]);
    #[cfg(windows)]
    let cmd = LocalCommand::new("powershell").args(["echo 'Sleeping for 1s'; sleep 1; echo Done"]);

    let id = commands.spawn(cmd).id();
    println!("Spawned the command as entity {id:?}");
}

fn update(
    mut process_output_event: EventReader<ProcessOutput>,
    mut process_completed_event: EventReader<ProcessCompleted>,
) {
    for process_output in process_output_event.read() {
        for line in process_output.lines() {
            println!("Output Line ({:?}): {line}", process_output.entity);
        }
    }
    if let Some(process_completed) = process_completed_event.read().last() {
        println!(
            "Command {:?} completed (Success - {})",
            process_completed.entity,
            process_completed.exit_status.success()
        );
        // Quit the app
        std::process::exit(0);
    }
}
