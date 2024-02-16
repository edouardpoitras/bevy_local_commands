use bevy::prelude::*;
use bevy_local_commands::{
    BevyLocalCommandsPlugin, Cleanup, LocalCommand, ProcessCompleted, Retry,
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
    query: Query<(&LocalCommand, &Retry)>,
) {
    if let Some(process_completed) = process_completed_event.read().last() {
        if let Ok((local_command, retry)) = query.get(process_completed.entity) {
            println!(
                "Command {:?} {:?} completed (Success - {})",
                local_command.get_program(),
                local_command.get_args(),
                process_completed.exit_status.success()
            );
            println!("Retries remaining: {:?}", retry);
        } else {
            println!("Can't find entity anymore, exiting");
            std::process::exit(0);
        }
    }
}
