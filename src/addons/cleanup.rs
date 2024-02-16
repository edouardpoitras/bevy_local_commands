use bevy::prelude::*;

use crate::{LocalCommand, Process, ProcessCompleted, Retry};

#[derive(Debug, Component)]
pub enum Cleanup {
    DespawnEntity,
    RemoveComponents,
}

/// Clean up any completed processes according to the Cleanup component.
///
/// Processes without the Cleanup component are ignored.
/// Takes into account the Retry component (will not perform cleanup until component is removed).
pub(crate) fn cleanup_completed_process(
    mut commands: Commands,
    query: Query<(&Cleanup, Option<&Retry>)>,
    mut process_completed_events: EventReader<ProcessCompleted>,
) {
    for process_completed_event in process_completed_events.read() {
        if let Ok((cleanup, option_retry)) = query.get(process_completed_event.entity) {
            // Don't cleanup a failed process if the Retry component is still attached to entity.
            if !process_completed_event.exit_status.success() && option_retry.is_some() {
                continue;
            }
            match cleanup {
                Cleanup::DespawnEntity => {
                    if let Some(mut entity_commands) =
                        commands.get_entity(process_completed_event.entity)
                    {
                        entity_commands.despawn();
                    }
                },
                Cleanup::RemoveComponents => {
                    if let Some(mut entity_commands) =
                        commands.get_entity(process_completed_event.entity)
                    {
                        entity_commands.remove::<(Process, Cleanup, LocalCommand)>();
                    }
                },
            }
        }
    }
}
