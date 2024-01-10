use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use bevy::tasks::Task;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::BufWriter;
use std::process::Child;
use std::process::ChildStdin;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::Mutex;

/// The ID of a process.
pub type Pid = u32;

#[derive(Debug, Event)]
pub struct ProcessOutput {
    pub entity: Entity,
    pub output: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ProcessErrorInfo {
    FailedToStart,
}

#[derive(Debug, PartialEq, Eq, Clone, Event)]
pub struct ProcessError {
    pub entity: Entity,
    pub info: ProcessErrorInfo,
}

#[derive(Debug, Event)]
pub struct ProcessCompleted {
    pub entity: Entity,
    pub success: bool,
}

/// The lines written to the standard output by a given process.
#[derive(Debug, Default, Clone)]
struct ProcessOutputBuffer(Arc<Mutex<Vec<String>>>);

#[derive(Debug, Component)]
pub struct LocalCommand {
    pub command: Command,
}

impl LocalCommand {
    pub fn new(command: Command) -> Self {
        Self { command }
    }
}

#[derive(Debug, Component)]
pub struct Process {
    process: Child,
    reader_task: Task<()>,
    output_buffer: ProcessOutputBuffer,
    stdin_writer: BufWriter<ChildStdin>,
}

impl Process {
    pub fn id(&self) -> Pid {
        self.process.id()
    }

    pub fn kill(&mut self) -> io::Result<()> {
        self.process.kill()
    }

    /// Write a string to the process stdin.
    ///
    /// See [`Process::println`] for a version which adds a newline (`\n`) to the end of the string.
    ///
    /// Here's how you can write "Hello world!" to a process that has just been started:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_local_commands::Process;
    /// fn provide_input(mut query: Query<&mut Process, Added<Process>>) {
    ///     for mut process in query.iter_mut() {
    ///         process.print("Hello world!").unwrap();
    ///     }
    /// }
    /// ```
    ///
    /// If you want more control, you can also use the `write!` macro.
    /// Just keep in mind that this might not flush the input buffer directly,
    /// so your process might receive the output later.
    /// You can also manually flush the buffer when you have written all your input.
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_local_commands::Process;
    /// // To use the macro, we need to import `Write`
    /// use std::io::Write;
    ///
    /// fn provide_input(mut query: Query<&mut Process, Added<Process>>) {
    ///     for mut process in query.iter_mut() {
    ///         let name = "world";
    ///         write!(&mut process, "Hello {name}!").unwrap();
    ///         // Make sure that the process receives the input
    ///         process.flush().unwrap();
    ///     }
    /// }
    /// ```
    pub fn print(&mut self, input: &str) -> Result<(), io::Error> {
        self.write_all(input.as_bytes())?;
        self.flush()
    }

    /// Write a string, terminated by a newline (`\n`) to the process stdin.
    ///
    /// See [`Process::print`] for a version without the newline.
    ///
    /// Here's how you can write "Hello world!" to a process that has just been started:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_local_commands::Process;
    /// fn provide_input(mut query: Query<&mut Process, Added<Process>>) {
    ///     for mut process in query.iter_mut() {
    ///         process.println("Hello world!").unwrap();
    ///     }
    /// }
    /// ```
    ///
    /// If you want more control, you can also use the `writeln!` macro.
    /// Just keep in mind that this might not flush the input buffer directly,
    /// so your process might receive the output later.
    /// You can also manually flush the buffer when you have written all your input.
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_local_commands::Process;
    /// // To use the macro, we need to import `Write`
    /// use std::io::Write;
    ///
    /// fn provide_input(mut query: Query<&mut Process, Added<Process>>) {
    ///     for mut process in query.iter_mut() {
    ///         let name = "world";
    ///         writeln!(&mut process, "Hello {name}!").unwrap();
    ///         // Make sure that the process receives the input
    ///         process.flush().unwrap();
    ///     }
    /// }
    /// ```
    pub fn println(&mut self, input: &str) -> Result<(), io::Error> {
        self.write_all(input.as_bytes())?;
        self.write_all(b"\n")?;
        self.flush()
    }
}

impl Write for Process {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stdin_writer.write(buf)
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.stdin_writer.write_all(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stdin_writer.flush()
    }
}

pub struct BevyLocalCommandsPlugin;

impl Plugin for BevyLocalCommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ProcessOutput>()
            .add_event::<ProcessCompleted>()
            .add_event::<ProcessError>()
            .add_systems(
                Update,
                (
                    handle_new_command,
                    handle_process_output,
                    handle_completed_process,
                )
                    .chain(),
            );
    }
}

/// A new command has been added.
///
/// This system will spawn the corresponding process and track the process output.
fn handle_new_command(
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
fn handle_process_output(
    query: Query<(Entity, &Process)>,
    mut process_output_event: EventWriter<ProcessOutput>,
) {
    for (entity, process) in query.iter() {
        if let Ok(mut output_buffer) = process.output_buffer.0.lock() {
            // Send the buffered output in the event while clearing the output buffer
            let mut output = Vec::<String>::new();
            std::mem::swap(&mut *output_buffer, &mut output);

            if !output.is_empty() {
                process_output_event.send(ProcessOutput { entity, output });
            }
        }
    }
}

/// Periodically check if any of the processes have finished.
///
/// For the completed processes, a [`ProcessCompleted`] event is produced and the entities despawned.
fn handle_completed_process(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Process)>,
    mut process_completed_event: EventWriter<ProcessCompleted>,
) {
    for (entity, mut process) in query.iter_mut() {
        if process.reader_task.is_finished() {
            let exit_status = process.process.wait().unwrap();

            process_completed_event.send(ProcessCompleted {
                entity,
                success: exit_status.success(),
            });

            // The process is finished, despawn the entity
            if let Some(mut entity_cmd) = commands.get_entity(entity) {
                entity_cmd.despawn()
            }
        }
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
                // The line includes the terminating new line, but we already have all lines separated
                buffer.push(line.trim_end_matches('\n').to_string());
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
