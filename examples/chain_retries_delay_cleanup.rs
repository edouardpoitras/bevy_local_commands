use std::time::Duration;

use bevy::prelude::*;
use bevy_local_commands::{
    BevyLocalCommandsPlugin, Chain, Cleanup, Delay, LocalCommand, ProcessCompleted, ProcessOutput,
    Retry,
};

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, BevyLocalCommandsPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn a entity with all addons
    let id = commands
        .spawn((
            Chain::new(vec![
                LocalCommand::new("sh").args(["-c", "echo 'First command'"]),
                LocalCommand::new("sh").args(["-c", "echo 'Second command'"]),
                LocalCommand::new("sh").args(["-c", "echo 'Third command'"]),
            ]),
            Retry::Attempts(2),
            Delay::Fixed(Duration::from_secs(3)),
            Cleanup::RemoveComponents,
        ))
        .id();
    println!("Spawned as entity {id:?}");
}

fn update(
    mut process_output_event: EventReader<ProcessOutput>,
    mut process_completed_event: EventReader<ProcessCompleted>,
    command_query: Query<(), (With<Chain>, With<Retry>, With<Delay>, With<Cleanup>)>,
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

    // Check if our entity is done running chain commands
    if command_query.is_empty() {
        println!("All commands completed - cleanup performed. Exiting the app.");
        std::process::exit(0);
    }
}
