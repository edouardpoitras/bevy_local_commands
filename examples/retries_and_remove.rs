use bevy::prelude::*;
use bevy_local_commands::{
    BevyLocalCommandsPlugin, Cleanup, LocalCommand, Process, ProcessCompleted, Retry, RetryEvent,
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
        .spawn((cmd, Retry::Attempts(3), Cleanup::RemoveComponents))
        .id();
    println!("Spawned the command as temporary entity {id:?} with 3 retries");
}

fn update(
    mut process_completed_event: EventReader<ProcessCompleted>,
    mut retry_events: EventReader<RetryEvent>,
    query: Query<(
        Entity,
        Option<&LocalCommand>,
        Option<&Process>,
        Option<&Retry>,
        Option<&Cleanup>,
    )>,
) {
    for retry_event in retry_events.read() {
        println!("Retry event triggered: {:?}", retry_event);
        let components = query.get(retry_event.entity).unwrap();
        assert!(components.1.is_some());
        assert!(components.2.is_none());
        assert!(components.3.is_some());
        assert!(components.4.is_some());
    }
    for process_completed in process_completed_event.read() {
        println!("{:?}", process_completed);
        let components = query.get(process_completed.entity).unwrap();
        assert!(components.1.is_none());
        assert!(components.2.is_none());
        assert!(components.3.is_none());
        assert!(components.4.is_none());
        std::process::exit(0);
    }
}
