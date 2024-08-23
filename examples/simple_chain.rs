use bevy::prelude::*;
use bevy_local_commands::{
    BevyLocalCommandsPlugin, Chain, LocalCommand, ProcessCompleted, ProcessOutput,
};

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, BevyLocalCommandsPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(mut commands: Commands) {
    // Create a chain of commands
    #[cfg(not(windows))]
    let chain = Chain::new(vec![
        LocalCommand::new("sh").args(["-c", "echo 'First command' && sleep 1"]),
        LocalCommand::new("sh").args(["-c", "echo 'Second command' && sleep 1"]),
        LocalCommand::new("sh").args(["-c", "echo 'Third command' && sleep 1"]),
    ]);
    #[cfg(windows)]
    let chain = Chain::new(vec![
        LocalCommand::new("powershell").args(["echo 'First command' && sleep 1"]),
        LocalCommand::new("powershell").args(["echo 'Second command' && sleep 1"]),
        LocalCommand::new("powershell").args(["echo 'Third command' && sleep 1"]),
    ]);

    // Spawn an entity with the Chain component
    let id = commands.spawn(chain).id();
    println!("Spawned the chain as entity {id:?}");
}

fn update(
    mut process_output_event: EventReader<ProcessOutput>,
    mut process_completed_event: EventReader<ProcessCompleted>,
    chain_query: Query<(), With<Chain>>,
) {
    for process_output in process_output_event.read() {
        for line in process_output.lines() {
            println!("Output Line ({:?}): {line}", process_output.entity);
        }
    }

    for process_completed in process_completed_event.read() {
        println!(
            "Command {:?} completed (Success - {})",
            process_completed.entity,
            process_completed.exit_status.success()
        );
    }

    // Check if there are no more Chain components (all chains completed)
    if chain_query.is_empty() {
        println!("All chain commands completed. Exiting the app.");
        std::process::exit(0);
    }
}
