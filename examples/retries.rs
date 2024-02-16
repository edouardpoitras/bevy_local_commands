use bevy::prelude::*;
use bevy_local_commands::{BevyLocalCommandsPlugin, LocalCommand, ProcessCompleted, Retry};

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
    let cmd =
        LocalCommand::new("sh").args(["-c", "echo Sleeping for 1s && sleep 1 && THIS SHOULD FAIL"]);
    #[cfg(windows)]
    let cmd = LocalCommand::new("cmd").args([
        "/C",
        "echo Sleeping for 1s && timeout 1 && THIS SHOULD FAIL",
    ]);

    let id = commands.spawn((cmd, Retry::Attempts(3))).id();
    println!("Spawned the command as entity {id:?} with 3 retries");
}

fn update(
    mut process_completed_event: EventReader<ProcessCompleted>,
    query: Query<(&LocalCommand, &Retry)>,
) {
    if let Some(process_completed) = process_completed_event.read().last() {
        let (local_command, retry) = query.get(process_completed.entity).unwrap();
        println!(
            "Command {:?} {:?} completed (Success - {})",
            local_command.get_program(),
            local_command.get_args(),
            process_completed.exit_status.success()
        );
        println!("Retries remaining: {:?}", retry);
        if let Retry::Attempts(0) = retry {
            println!("No retries left, exiting");
            std::process::exit(0);
        }
    }
}
