use bevy::prelude::*;

use crate::{LocalCommand, process::{Process, ProcessState}};

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
    query: Query<(Entity, &Process, &Cleanup)>,
) {
    for (entity, process, cleanup) in query.iter() {
        match process.state {
            ProcessState::Done(_) => {
                match cleanup {
                    Cleanup::DespawnEntity => {
                        if let Some(mut entity_commands) = commands.get_entity(entity) {
                            entity_commands.despawn();
                        }
                    },
                    Cleanup::RemoveComponents => {
                        if let Some(mut entity_commands) = commands.get_entity(entity) {
                            entity_commands.remove::<(Process, Cleanup, LocalCommand)>();
                        }
                    },
                }
            },
            _ => {},
        }
    }
}
