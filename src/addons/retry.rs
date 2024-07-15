use bevy::prelude::*;

use crate::{systems::spawn_process, LocalCommand, process::{Process, ProcessState}};

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
    mut query: Query<(Entity, &mut LocalCommand, &mut Process, &mut Retry)>,
    mut retry_events: EventWriter<RetryEvent>,
) {

    for (entity, mut local_command, mut process, mut retry) in query.iter_mut() {
        match process.state {
            ProcessState::Error => {
                match &mut *retry {
                    Retry::Attempts(retries) => {
                        if let Some(mut entity_commands) =
                            commands.get_entity(entity)
                        {
                            if *retries < 1 {
                                entity_commands.remove::<Retry>();
                                continue;
                            }

                            // Update the retry attempts
                            *retries -= 1;

                            // Spawn the process once again
                            match spawn_process(&mut local_command.command) {
                                Ok(new_process) => {
                                    *process = new_process;
                                    retry_events.send(RetryEvent {
                                        entity: entity,
                                        retries_left: *retries,
                                    });
                                },
                                Err(_) => {
                                    error!(
                                        "Failed to retry process: {:?} {:?}",
                                        local_command.get_program(),
                                        local_command.get_args()
                                    );
                                },
                            }
                        }
                    },
                }
            },
            _ => {},
        }
    }
}
