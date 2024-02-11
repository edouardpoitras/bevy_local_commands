use std::io::{self, prelude::*, BufReader, BufWriter};
use std::process::{Command, Stdio};

use bevy::{prelude::*, tasks::IoTaskPool};

use crate::{
    LocalCommand, Process, ProcessCompleted, ProcessError, ProcessErrorInfo, ProcessOutput,
    ProcessOutputBuffer,
};

/// A new command has been added.
///
/// This system will spawn the corresponding process and track the process output.
pub(crate) fn handle_new_command(
    mut commands: Commands,
    mut query: Query<(Entity, &mut LocalCommand), (Added<LocalCommand>, Without<Process>)>,
    mut process_error_event: EventWriter<ProcessError>,
) {
    for (entity, mut local_command) in query.iter_mut() {
        match spawn_process(&mut local_command.command) {
            Ok(process) => {
                commands.entity(entity).insert(process);
            },
            Err(_) => {
                process_error_event.send(ProcessError {
                    entity,
                    info: ProcessErrorInfo::FailedToStart,
                });
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
                process_output_event.send(ProcessOutput { entity, output });
            }
        }
    }
}

/// Periodically check if any of the processes have finished.
///
/// For the completed processes, a [`ProcessCompleted`] event is produced and the entities despawned.
pub(crate) fn handle_completed_process(
    mut commands: Commands,
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

            // The process is finished, despawn the entity
            if let Some(mut entity_cmd) = commands.get_entity(entity) {
                entity_cmd.despawn()
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
