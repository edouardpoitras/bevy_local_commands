use bevy::prelude::*;

use crate::{LocalCommand, Process, ProcessCompleted};

#[derive(Debug, Component)]
pub enum Retry {
    None,
    Attempts(usize),
}

/// Retry failed processes according to the Retry component.
///
/// Processes without the Retry component are ignored, same as Retry::None.
pub(crate) fn retry_failed_process(
    mut commands: Commands,
    query: Query<(&LocalCommand, &Retry)>,
    mut process_completed_events: EventReader<ProcessCompleted>,
) {
    for process_completed_event in process_completed_events.read() {
        if process_completed_event.exit_status.success() {
            continue;
        }
        if let Ok((current_local_command, retry)) = query.get(process_completed_event.entity) {
            match retry {
                Retry::Attempts(retries) => {
                    if *retries < 1 {
                        continue;
                    }
                    if let Some(mut entity_commands) =
                        commands.get_entity(process_completed_event.entity)
                    {
                        // Remove both the Process and LocalCommand
                        entity_commands.remove::<(Process, LocalCommand)>();

                        // Create new LocalCommand
                        let mut new_local_command =
                            LocalCommand::new(current_local_command.get_program());

                        // Match current working directory
                        new_local_command.command.current_dir(
                            current_local_command
                                .get_current_dir()
                                .unwrap_or(std::path::Path::new(".")),
                        );

                        // Match arguments
                        new_local_command
                            .command
                            .args(current_local_command.get_args());

                        // Match environment variables
                        for (key, option_value) in current_local_command.get_envs() {
                            if let Some(value) = option_value {
                                new_local_command.command.env(key, value);
                            }
                        }

                        // Re-add the LocalCommand to trigger Added<LocalCommand> event for Process creation
                        entity_commands.insert((new_local_command, Retry::Attempts(retries - 1)));
                    }
                },
                Retry::None => {},
            }
        }
    }
}
