use std::time::Duration;

use bevy::prelude::*;
use bevy_local_commands::{
    BevyLocalCommandsPlugin, Chain, ChainCompletedEvent, Delay, LocalCommand, ProcessCompleted,
    ProcessOutput, Retry, RetryEvent,
};

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, BevyLocalCommandsPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn an entity with the relevant components
    let id = commands
        .spawn((
            Chain::new(vec![
                LocalCommand::new("sh").args(["-c", "echo 'First command'"]),
                LocalCommand::new("sh").args(["-c", "exit 1"]), // Failure
                LocalCommand::new("sh").args(["-c", "echo 'Third command'"]),
            ]),
            Retry::Attempts(2),
            Delay::Fixed(Duration::from_secs(2)),
        ))
        .id();
    println!("Spawned the chain as entity {id:?}");
}

fn update(
    mut process_output_event: EventReader<ProcessOutput>,
    mut process_completed_event: EventReader<ProcessCompleted>,
    mut process_chain_completed_event: EventReader<ChainCompletedEvent>,
    mut process_retry_event: EventReader<RetryEvent>,
    chain_query: Query<(), With<Chain>>,
) {
    for process_output in process_output_event.read() {
        for line in process_output.lines() {
            println!("Output Line ({:?}): {line}", process_output.entity);
        }
    }

    for process_retry in process_retry_event.read() {
        println!(
            "Command for entity {:?} failed, retrying ({} left)",
            process_retry.entity, process_retry.retries_left,
        );
    }

    for process_completed in process_completed_event.read() {
        println!(
            "Command {:?} completed (Success - {})",
            process_completed.entity,
            process_completed.exit_status.success()
        );
    }

    for process_chain_completed in process_chain_completed_event.read() {
        println!(
            "Chain of commands completed for entity {} (Success - {})",
            process_chain_completed.entity, process_chain_completed.success,
        );
    }

    // Check if there is no more Chain component
    if chain_query.is_empty() {
        println!("Chain commands done. Exiting the app.");
        std::process::exit(0);
    }
}
