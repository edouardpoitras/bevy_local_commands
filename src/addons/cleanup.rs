use bevy::prelude::*;

use crate::{LocalCommand, LocalCommandState, Chain, Delay, Retry, process::Process};

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
    query: Query<(Entity, &LocalCommand, &Cleanup)>,
) {
    for (entity, local_command, cleanup) in query.iter() {
        match local_command.state {
            LocalCommandState::Done(_) => {
                match cleanup {
                    Cleanup::DespawnEntity => {
                        if let Some(mut entity_commands) = commands.get_entity(entity) {
                            entity_commands.despawn();
                        }
                    },
                    Cleanup::RemoveComponents => {
                        if let Some(mut entity_commands) = commands.get_entity(entity) {
                            entity_commands.remove::<(Process, Chain, Delay, Retry, Cleanup, LocalCommand)>();
                        }
                    },
                }
            },
            _ => {},
        }
    }
}
