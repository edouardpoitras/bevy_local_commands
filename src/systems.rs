use std::io::{self, prelude::*, BufReader, BufWriter};
use std::process::{Command, Stdio};

use bevy::{prelude::*, tasks::IoTaskPool};

use crate::addons::delay::Delay;
use crate::{
    LocalCommand, Process, ProcessCompleted, ProcessError, ProcessErrorInfo, ProcessOutput, ProcessOutputBuffer
};

/// A new command has been added.
///
/// This system will handle spawn processes and setup process output tracking.
pub(crate) fn handle_new_command(
    mut commands: Commands,
    mut added_query: Query<(Entity, &mut LocalCommand), (Added<LocalCommand>, Without<Process>, Without<Delay>)>,
    mut process_error_event: EventWriter<ProcessError>,
) {
    // Spawn process for recently added LocalCommands.
    for (entity, mut local_command) in added_query.iter_mut() {
        spawn_process_and_handle_error(&mut commands, entity, &mut local_command, &mut process_error_event);
    }
}

/// A delayed command is ready to spawn.
///
/// This system will handle spawning previously delayed processes and setup process output tracking.
pub(crate) fn handle_delayed_command(
    mut commands: Commands,
    mut query: Query<&mut LocalCommand>,
    mut delayed_entities: RemovedComponents<Delay>,
    mut process_error_event: EventWriter<ProcessError>,
) {
    // Spawn process for previous delayed LocalCommands.
    for entity in delayed_entities.read() {
        if let Ok(mut local_command) = query.get_mut(entity) {
            spawn_process_and_handle_error(&mut commands, entity, &mut local_command, &mut process_error_event);
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
                process_output_event.send(ProcessOutput { entity, output });
            }
        }
    }
}

/// Periodically check if any of the processes have finished.
///
/// For the completed processes, a [`ProcessCompleted`] event is produced.
pub(crate) fn handle_completed_process(
    mut query: Query<(Entity, &mut Process)>,
    mut process_completed_event: EventWriter<ProcessCompleted>,
) {
    for (entity, mut process) in query.iter_mut() {
        if process.reader_task.is_finished() {
            let exit_status = process.process.wait().unwrap();

            process_completed_event.send(ProcessCompleted {
                entity,
                exit_status,
            });
        }
    }
}

fn spawn_process_and_handle_error(
    commands: &mut Commands,
    entity: Entity,
    local_command: &mut LocalCommand,
    process_error_event: &mut EventWriter<ProcessError>
) {
    match spawn_process(&mut local_command.command) {
        Ok(process) => {
            commands.entity(entity).insert(process);
        },
        Err(_) => {
            process_error_event.send(ProcessError {
                entity: entity,
                info: ProcessErrorInfo::FailedToStart,
            });
        },
    }
}

fn spawn_process(command: &mut Command) -> io::Result<Process> {
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
