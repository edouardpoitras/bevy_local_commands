use bevy::prelude::*;
use bevy_local_commands::{
    BevyLocalCommandsPlugin, LocalCommand, ProcessCompleted, Retry, RetryEvent,
};

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
    query: Query<&LocalCommand, With<Retry>>,
    mut retry_events: EventReader<RetryEvent>,
) {
    if let Some(process_completed) = process_completed_event.read().last() {
        if let Ok(local_command) = query.get(process_completed.entity) {
            println!(
                "Command {:?} {:?} completed (Success - {})",
                local_command.get_program(),
                local_command.get_args(),
                process_completed.exit_status.success()
            );
        } else {
            println!("Retry component removed from entity, exiting");
            std::process::exit(0);
        }
    }
    for retry_event in retry_events.read() {
        println!("Retry event triggered: {:?}", retry_event);
    }
}
