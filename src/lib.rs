use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use bevy::tasks::Task;
use bevy::utils::HashMap;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::Mutex;

/// The ID of a process.
type Pid = u32;

#[derive(Debug, Event)]
pub struct RunProcess {
    program: String,
    arguments: Vec<String>,
}

impl RunProcess {
    pub fn new<S: Into<String>>(program: S, arguments: Vec<S>) -> RunProcess {
        RunProcess {
            program: program.into(),
            arguments: arguments.into_iter().map(|s| s.into()).collect(),
        }
    }
}

#[derive(Debug, Event)]
pub struct ProcessStarted {
    pub command: String,
    pub pid: Pid,
}

#[derive(Debug, Event)]
pub struct ProcessOutput {
    pub pid: Pid,
    pub command: String,
    pub output: Vec<String>,
}

/// This will only trigger on stdout/stderr events for the process.
/// IE: 'sleep 9999' won't be killed cause no output is produced.
/// Work-arounds/fixes are being considered.
#[derive(Debug, Event)]
pub struct KillProcess(pub Pid);

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ProcessErrorInfo {
    FailedToStart,
}

#[derive(Debug, PartialEq, Eq, Clone, Event)]
pub struct ProcessError {
    pub command: String,
    pub info: ProcessErrorInfo,
}

#[derive(Debug, Event)]
pub struct ProcessCompleted {
    pub success: bool,
    pub pid: u32,
    pub command: String,
}

/// The lines written to the standard output by a given process.
#[derive(Debug, Default, Clone)]
struct ProcessOutputBuffer(Arc<Mutex<Vec<String>>>);

#[derive(Debug)]
pub struct ActiveProcess {
    command: Command,
    process: Child,
    task: Task<()>,
    output_buffer: ProcessOutputBuffer,
}

#[derive(Default, Resource)]
pub struct ActiveProcessMap(pub HashMap<Pid, ActiveProcess>);

pub struct BevyLocalCommandsPlugin;

impl Plugin for BevyLocalCommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RunProcess>()
            .add_event::<ProcessStarted>()
            .add_event::<ProcessOutput>()
            .add_event::<KillProcess>()
            .add_event::<ProcessCompleted>()
            .add_event::<ProcessError>()
            .init_resource::<ActiveProcessMap>()
            .add_systems(
                Update,
                (
                    handle_new_process,
                    handle_kill_process,
                    handle_process_output,
                    handle_completed_process,
                )
                    .chain(),
            );
    }
}

fn handle_new_process(
    mut run_process_event: EventReader<RunProcess>,
    mut process_started_event: EventWriter<ProcessStarted>,
    mut process_error_event: EventWriter<ProcessError>,
    mut active_process_map: ResMut<ActiveProcessMap>,
) {
    for run_shell_command in run_process_event.read() {
        // Assemble the command
        let mut cmd = Command::new(run_shell_command.program.clone());
        cmd.args(run_shell_command.arguments.clone())
            .stdout(Stdio::piped());

        let command = format!("{cmd:?}");

        let Ok(active_process) = spawn_process(cmd) else {
            process_error_event.send(ProcessError {
                command,
                info: ProcessErrorInfo::FailedToStart,
            });
            continue;
        };

        let pid = active_process.process.id();

        process_started_event.send(ProcessStarted { command, pid });

        active_process_map.0.insert(pid, active_process);
    }
}

fn handle_process_output(
    mut active_process_map: ResMut<ActiveProcessMap>,
    mut process_output_event: EventWriter<ProcessOutput>,
) {
    for (&pid, active_process) in active_process_map.0.iter_mut() {
        if let Ok(mut output_buffer) = active_process.output_buffer.0.lock() {
            // Send the buffered output in the event while clearing the output buffer
            let mut output = Vec::<String>::new();
            std::mem::swap(&mut *output_buffer, &mut output);

            if !output.is_empty() {
                process_output_event.send(ProcessOutput {
                    pid,
                    command: format!("{:?}", active_process.command),
                    output,
                });
            }
        }
    }
}

fn handle_kill_process(
    mut active_process_map: ResMut<ActiveProcessMap>,
    mut kill_process_event: EventReader<KillProcess>,
) {
    for kill_shell_command in kill_process_event.read() {
        let pid = kill_shell_command.0;

        if let Some(active_process) = active_process_map.0.get_mut(&pid) {
            // Kill the process
            // TODO: Handle unwrap
            active_process.process.kill().unwrap();
        } else {
            warn!("Could not find and kill shell command (PID: {})", pid);
        }
    }
}

fn handle_completed_process(
    mut active_process_map: ResMut<ActiveProcessMap>,
    mut process_completed_event: EventWriter<ProcessCompleted>,
) {
    // Remember which processes completed so we can remove them from the map
    let mut completed_processes = Vec::new();

    for (&pid, active_process) in active_process_map.0.iter_mut() {
        if active_process.task.is_finished() {
            let exit_status = active_process.process.wait().unwrap();
            process_completed_event.send(ProcessCompleted {
                command: format!("{:?}", active_process.command),
                pid,
                success: exit_status.success(),
            });

            completed_processes.push(pid);
        }
    }

    // Clean up process map
    for pid in completed_processes {
        active_process_map.0.remove(&pid);
    }
}

fn spawn_process(mut command: Command) -> io::Result<ActiveProcess> {
    // Start running the process
    let mut process = command.spawn()?;
    let stdout = process.stdout.take().unwrap();
    let pid = process.id();

    info!("Spawned command with pid {pid}: {command:?}");

    let output_buffer = ProcessOutputBuffer::default();

    let moved_buffer = output_buffer.clone();
    let thread_pool = IoTaskPool::get();

    // Read stdout and write it to the output buffer
    let task = thread_pool.spawn(async move {
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

    Ok(ActiveProcess {
        command,
        process,
        output_buffer,
        task,
    })
}
