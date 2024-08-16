use bevy::prelude::*;

use crate::{process::Process, Chain, Delay, LocalCommand, LocalCommandState, Retry};

#[derive(Debug, Component)]
pub enum Cleanup {
    DespawnEntity,
    RemoveComponents,
}

/// Clean up any completed processes according to the Cleanup component.
///
/// Processes without the Cleanup component are ignored.
pub(crate) fn cleanup_completed_process(
    mut commands: Commands,
    query: Query<(Entity, &LocalCommand, &Cleanup, Option<&Chain>)>,
) {
    for (entity, local_command, cleanup, option_chain) in query.iter() {
        // Cleanup does not work well with Chains (it tries to cleanup after every command)
        // For now, skip cleanup if Chain of commands is not empty yet
        if let Some(chain) = option_chain {
            if !chain.commands.is_empty() {
                continue;
            }
        }
        if let LocalCommandState::Done(_) = local_command.state {
            match cleanup {
                Cleanup::DespawnEntity => {
                    if let Some(mut entity_commands) = commands.get_entity(entity) {
                        entity_commands.despawn();
                    }
                },
                Cleanup::RemoveComponents => {
                    if let Some(mut entity_commands) = commands.get_entity(entity) {
                        entity_commands
                            .remove::<(Process, Chain, Delay, Retry, Cleanup, LocalCommand)>();
                    }
                },
            }
        }
    }
}
