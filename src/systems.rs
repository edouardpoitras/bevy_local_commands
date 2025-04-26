use std::io::{self, prelude::*, BufReader, BufWriter};
use std::process::{Command, Stdio};

use bevy::{prelude::*, tasks::IoTaskPool};
use bevy_log::{error, info};

use crate::{
    LocalCommand, LocalCommandDone, LocalCommandState, Process, ProcessCompleted, ProcessError,
    ProcessErrorInfo, ProcessOutput, ProcessOutputBuffer,
};

/// A command is pending process creation.
///
/// This system will spawn the corresponding process if it is ready.
pub(crate) fn handle_new_command(
    mut commands: Commands,
    mut query: Query<(Entity, &mut LocalCommand), Without<Process>>,
    mut process_error_event: EventWriter<ProcessError>,
    time: Res<Time>,
) {
    for (entity, mut local_command) in query.iter_mut() {
        match &mut local_command.delay {
            Some(ref mut timer) if !timer.finished() => {
                timer.tick(time.delta());
            },
            _ => {
                local_command.delay = None;
                match spawn_process(&mut local_command.command) {
                    Ok(process) => {
                        commands.entity(entity).insert(process);
                        local_command.state = LocalCommandState::Running;
                    },
                    Err(_) => {
                        process_error_event.write(ProcessError {
                            entity,
                            info: ProcessErrorInfo::FailedToStart,
                        });
                        local_command.state = LocalCommandState::Error;
                    },
                }
            },
        }
    }
}

/// Periodically empty each processes' output buffer and send the new lines as [`ProcessOutputEvent`].
pub(crate) fn handle_process_output(
    query: Query<(Entity, &Process)>,
    mut process_output_event: EventWriter<ProcessOutput>,
) {
    for (entity, process) in query.iter() {
        if let Ok(mut buffer) = process.output_buffer.0.lock() {
            // Send the buffered output in the event while clearing the output buffer
            let mut output = String::new();
            std::mem::swap(&mut *buffer, &mut output);

            if !output.is_empty() {
                process_output_event.write(ProcessOutput { entity, output });
            }
        }
    }
}

/// Periodically check if any of the processes have finished.
///
/// For the completed processes, a [`ProcessCompleted`] event is produced.
pub(crate) fn handle_completed_process(
    mut query: Query<(Entity, &mut LocalCommand, &mut Process)>,
    mut process_completed_event: EventWriter<ProcessCompleted>,
) {
    for (entity, mut local_command, mut process) in query.iter_mut() {
        match local_command.state {
            // Transition state from LocalCommandState::Error to LocalCommandDone::Failed.
            // Retry addons should have already kicked in - unless the process failed to spawn.
            LocalCommandState::Error => {
                local_command.state = LocalCommandState::Done(LocalCommandDone::Failed);
                process_completed_event.write(ProcessCompleted {
                    entity,
                    exit_status: process.process.wait().unwrap(),
                });
                continue;
            },
            // If no cleanup addons is active, we don't want to keep checking this completed process.
            LocalCommandState::Done(_) => continue,
            _ => {},
        }

        // Deal with state management when process completes.
        if process.reader_task.is_finished() {
            let exit_status = process.process.wait().unwrap();
            match exit_status.code() {
                None => {
                    info!("Process with pid {} was killed", process.id());
                    local_command.state = LocalCommandState::Done(LocalCommandDone::Killed);
                    process_completed_event.write(ProcessCompleted {
                        entity,
                        exit_status,
                    });
                },
                Some(0) => {
                    info!("Process with pid {} exited with code 0", process.id());
                    local_command.state = LocalCommandState::Done(LocalCommandDone::Succeeded);
                    process_completed_event.write(ProcessCompleted {
                        entity,
                        exit_status,
                    });
                },
                Some(code) => {
                    error!(
                        "Process with pid {} exited with code {}",
                        process.id(),
                        code
                    );
                    // The next frame will transition the state from LocalCommandState::Error to
                    // LocalCommandDone::Failed if no retry addons have triggered.
                    local_command.state = LocalCommandState::Error;
                },
            }
        }
    }
}

pub(crate) fn spawn_process(command: &mut Command) -> io::Result<Process> {
    // Configure the stdio to be able to read the output and send input
    command.stdout(Stdio::piped());
    command.stdin(Stdio::piped());

    // Start running the process
    let mut process = command.spawn()?;
    let stdout = process.stdout.take().unwrap();
    let stdin = process.stdin.take().unwrap();
    let stdin_writer = BufWriter::new(stdin);
    let pid = process.id();

    info!("Spawned command with pid {pid}: {command:?}");

    let output_buffer = ProcessOutputBuffer::default();

    let moved_buffer = output_buffer.clone();
    let thread_pool = IoTaskPool::get();

    // Read stdout and write it to the output buffer
    let reader_task = thread_pool.spawn(async move {
        let mut reader = BufReader::new(stdout);

        let mut line = String::new();

        while let Ok(bytes) = reader.read_line(&mut line) {
            if bytes == 0 {
                break;
            }

            if let Ok(mut buffer) = moved_buffer.0.lock() {
                // Append the line to the buffer
                *buffer += &line;
                line.clear();
            }
        }
    });

    Ok(Process {
        process,
        output_buffer,
        reader_task,
        stdin_writer,
    })
}
