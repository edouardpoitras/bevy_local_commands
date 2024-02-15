use bevy::prelude::*;

use crate::{LocalCommand, Process, ProcessCompleted};

#[derive(Debug, Component)]
pub enum Cleanup {
    None,
    DespawnEntity,
    RemoveComponent,
}

/// Clean up any completed processes according to the Cleanup component.
///
/// Processes without the Cleanup component are ignored, same as Cleanup::None.
pub(crate) fn cleanup_completed_process(
    mut commands: Commands,
    query: Query<&Cleanup>,
    mut process_completed_events: EventReader<ProcessCompleted>,
) {
    for process_completed_event in process_completed_events.read() {
        if let Ok(cleanup) = query.get(process_completed_event.entity) {
            match cleanup {
                Cleanup::DespawnEntity => {
                    if let Some(mut entity_commands) =
                        commands.get_entity(process_completed_event.entity)
                    {
                        entity_commands.despawn();
                    }
                },
                Cleanup::RemoveComponent => {
                    if let Some(mut entity_commands) =
                        commands.get_entity(process_completed_event.entity)
                    {
                        entity_commands.remove::<(Process, Cleanup, LocalCommand)>();
                    }
                },
                Cleanup::None => {},
            }
        }
    }
}
