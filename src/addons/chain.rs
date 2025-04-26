use crate::local_command::LocalCommand;
use crate::{Process, ProcessCompleted, ProcessError};
use bevy::prelude::*;
use std::iter::IntoIterator;

#[derive(Component)]
pub struct Chain {
    pub(crate) commands: Vec<LocalCommand>,
}

impl Chain {
    pub fn new(commands: impl IntoIterator<Item = LocalCommand>) -> Self {
        Self {
            commands: commands.into_iter().collect(),
        }
    }
}

#[derive(Debug, Event)]
pub struct ChainCompletedEvent {
    pub entity: Entity,
    pub success: bool,
}

pub fn chain_execution_system(
    mut commands: Commands,
    mut chain_query: Query<(Entity, &mut Chain)>,
    no_local_command: Query<(), Without<LocalCommand>>,
    mut process_completed_events: EventReader<ProcessCompleted>,
    mut process_error_events: EventReader<ProcessError>,
    mut chain_completed_events: EventWriter<ChainCompletedEvent>,
) {
    // Handle completed processes
    for event in process_completed_events.read() {
        if let Ok((entity, mut chain)) = chain_query.get_mut(event.entity) {
            if event.exit_status.success() {
                // If there are more commands in the chain, start the next one
                if !chain.commands.is_empty() {
                    let local_command = chain.commands.remove(0);
                    commands
                        .entity(entity)
                        .insert(local_command)
                        .remove::<Process>();
                } else {
                    // If all commands are completed successfully, remove components
                    commands
                        .entity(entity)
                        .remove::<(LocalCommand, Process, Chain)>();
                    chain_completed_events.write(ChainCompletedEvent {
                        entity,
                        success: true,
                    });
                }
            } else {
                // If the process was not successful, abandon the rest of the chain
                commands
                    .entity(entity)
                    .remove::<(LocalCommand, Process, Chain)>();
                chain_completed_events.write(ChainCompletedEvent {
                    entity,
                    success: false,
                });
            }
        }
    }
    // Also consider ProcessError events as completed processes
    for event in process_error_events.read() {
        if let Ok((entity, _)) = chain_query.get_mut(event.entity) {
            // Abandon the rest of the chain
            commands
                .entity(entity)
                .remove::<(LocalCommand, Process, Chain)>();
            chain_completed_events.write(ChainCompletedEvent {
                entity,
                success: false,
            });
        }
    }

    // Start the first command for new Chain components without LocalCommand
    for (entity, mut chain) in chain_query.iter_mut() {
        if !chain.commands.is_empty() && no_local_command.get(entity).is_ok() {
            let local_command = chain.commands.remove(0);
            commands.entity(entity).insert(local_command);
        }
    }
}
