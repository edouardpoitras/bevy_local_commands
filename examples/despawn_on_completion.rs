use bevy::prelude::*;
use bevy_local_commands::{BevyLocalCommandsPlugin, Cleanup, LocalCommand, ProcessCompleted};

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
    let cmd =
        LocalCommand::new("cmd").args(["/C", "echo Sleeping for 1s && timeout 1 && echo Done"]);

    let id = commands.spawn((cmd, Cleanup::DespawnEntity)).id();
    println!("Spawned the command as entity {id:?}");
}

fn update(
    mut process_completed_event: EventReader<ProcessCompleted>,
    query: Query<Entity>,
    mut entity: Local<Option<Entity>>,
) {
    if let Some(process_completed) = process_completed_event.read().last() {
        *entity = Some(process_completed.entity);
        println!(
            "Command {:?} completed (Success - {})",
            process_completed.entity,
            process_completed.exit_status.success()
        );
    }
    if entity.is_some() && query.get(entity.unwrap()).is_err() {
        // Entity no longer exists, quit the app.
        println!("Entity no longer exists, exiting");
        std::process::exit(0);
    }
}
