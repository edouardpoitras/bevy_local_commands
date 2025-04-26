use bevy::prelude::*;

use crate::{process::Process, LocalCommand, LocalCommandState};

#[derive(Debug, Component)]
pub enum Retry {
    Attempts(usize),
}

#[derive(Debug, Event)]
pub struct RetryEvent {
    pub entity: Entity,
    pub retries_left: usize,
}

/// Retry failed processes according to the Retry component.
///
/// Processes without the Retry component are ignored.
/// The Retry component is removed from the entity when retries are done.
pub(crate) fn retry_failed_process(
    mut commands: Commands,
    mut query: Query<(Entity, &mut LocalCommand, &mut Retry), With<Process>>,
    mut retry_events: EventWriter<RetryEvent>,
) {
    for (entity, mut local_command, mut retry) in query.iter_mut() {
        if local_command.state == LocalCommandState::Error {
            match &mut *retry {
                Retry::Attempts(retries) => {
                    if let Ok(mut entity_commands) = commands.get_entity(entity) {
                        if *retries < 1 {
                            entity_commands.remove::<Retry>();
                            continue;
                        }

                        // Update the retry attempts
                        *retries -= 1;

                        // Spawn the process once again
                        commands.entity(entity).remove::<Process>();
                        local_command.delay = None;
                        local_command.state = LocalCommandState::Ready;
                        retry_events.write(RetryEvent {
                            entity,
                            retries_left: *retries,
                        });
                    }
                },
            }
        }
    }
}
