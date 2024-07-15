use bevy::prelude::*;
use bevy_local_commands::{
    BevyLocalCommandsPlugin, Cleanup, LocalCommand, ProcessCompleted, Retry, RetryEvent,
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
    let cmd = LocalCommand::new("sh").args(["-c", "echo Sleeping for 1s && sleep 1 && INVALID "]);
    #[cfg(windows)]
    let cmd = LocalCommand::new("cmd").args(["/C", "echo Sleeping for 1s && timeout 1 && INVALID"]);

    let id = commands
        .spawn((cmd, Retry::Attempts(3), Cleanup::DespawnEntity))
        .id();
    println!("Spawned the command as temporary entity {id:?} with 3 retries");
}

fn update(
    mut process_completed_event: EventReader<ProcessCompleted>,
    mut retry_events: EventReader<RetryEvent>,
) {
    for retry_event in retry_events.read() {
        println!("Retry event triggered: {:?}", retry_event);
    }
    for process_completed in process_completed_event.read() {
        println!("{:?}", process_completed);
        std::process::exit(0);
    }
}
