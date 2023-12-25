use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use bevy::tasks::Task;
use bevy::utils::HashMap;
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
pub struct RunShellCommand {
    program: String,
    arguments: Vec<String>,
}

impl RunShellCommand {
    pub fn new<S: Into<String>>(program: S, arguments: Vec<S>) -> RunShellCommand {
        RunShellCommand {
            program: program.into(),
            arguments: arguments.into_iter().map(|s| s.into()).collect(),
        }
    }
}

#[derive(Debug, Event)]
pub struct ShellCommandStarted {
    pub command: String,
    pub pid: Pid,
}

#[derive(Debug, Event)]
pub struct ShellCommandOutput {
    pub pid: Pid,
    pub command: String,
    pub output: Vec<String>,
}

/// This will only trigger on stdout/stderr events for the process.
/// IE: 'sleep 9999' won't be killed cause no output is produced.
/// Work-arounds/fixes are being considered.
#[derive(Debug, Event)]
pub struct KillShellCommand(pub Pid);

#[derive(Debug, Event)]
pub struct ShellCommandCompleted {
    pub success: bool,
    pub pid: u32,
    pub command: String,
}

/// The lines written to the standard output by a given process.
#[derive(Debug, Default, Clone)]
struct ProcessOutputBuffer(Arc<Mutex<Vec<String>>>);

#[derive(Debug)]
struct ActiveProcess {
    command: Command,
    process: Child,
    task: Task<()>,
    output_buffer: ProcessOutputBuffer,
}

#[derive(Default, Resource)]
struct ActiveProcessMap(HashMap<Pid, ActiveProcess>);

pub struct BevyLocalCommandsPlugin;

impl Plugin for BevyLocalCommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RunShellCommand>()
            .add_event::<ShellCommandStarted>()
            .add_event::<ShellCommandOutput>()
            .add_event::<KillShellCommand>()
            .add_event::<ShellCommandCompleted>()
            .init_resource::<ActiveProcessMap>()
            .add_systems(
                Update,
                (
                    handle_new_process,
                    handle_shell_command_output,
                    handle_kill_process,
                    handle_completed_shell_commands,
                )
                    .chain(),
            );
    }
}

fn handle_new_process(
    mut run_shell_command_event: EventReader<RunShellCommand>,
    mut shell_command_started_event: EventWriter<ShellCommandStarted>,
    mut active_process_map: ResMut<ActiveProcessMap>,
) {
    for run_shell_command in run_shell_command_event.read() {
        // Assemble the command
        let mut cmd = Command::new(run_shell_command.program.clone());
        cmd.args(run_shell_command.arguments.clone())
            .stdout(Stdio::piped());

        let active_process = spawn_process(cmd);
        let pid = active_process.process.id();

        shell_command_started_event.send(ShellCommandStarted {
            command: format!("{:?}", active_process.command),
            pid,
        });

        active_process_map.0.insert(pid, active_process);
    }
}

fn handle_shell_command_output(
    mut active_process_map: ResMut<ActiveProcessMap>,
    mut shell_command_output: EventWriter<ShellCommandOutput>,
) {
    for (&pid, active_process) in active_process_map.0.iter_mut() {
        if let Ok(mut output_buffer) = active_process.output_buffer.0.lock() {
            // Send the buffered output in the event while clearing the output buffer
            let mut output = Vec::<String>::new();
            std::mem::swap(&mut *output_buffer, &mut output);

            if !output.is_empty() {
                shell_command_output.send(ShellCommandOutput {
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
    mut kill_shell_commands: EventReader<KillShellCommand>,
) {
    for kill_shell_command in kill_shell_commands.read() {
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

fn handle_completed_shell_commands(
    mut active_process_map: ResMut<ActiveProcessMap>,
    mut shell_command_completed_event: EventWriter<ShellCommandCompleted>,
) {
    // Remember which processes completed so we can remove them from the map
    let mut completed_processes = Vec::new();

    for (&pid, active_process) in active_process_map.0.iter_mut() {
        if active_process.task.is_finished() {
            let exit_status = active_process.process.wait().unwrap();
            shell_command_completed_event.send(ShellCommandCompleted {
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

fn spawn_process(mut command: Command) -> ActiveProcess {
    // Start running the process
    let mut process = command.spawn().unwrap();
    let stdout = process.stdout.take().unwrap();
    let pid = process.id();

    info!("Spawned command with pid {pid}: {command:?}");

    let output_buffer = ProcessOutputBuffer::default();

    let output_buffer_moved = output_buffer.clone();
    let thread_pool = IoTaskPool::get();

    // Read stdout and write it to the output buffer
    let task = thread_pool.spawn(async move {
        let mut reader = BufReader::new(stdout);

        let mut line = String::new();

        while let Ok(bytes) = reader.read_line(&mut line) {
            if bytes == 0 {
                break;
            }

            if let Ok(mut output_buffer) = output_buffer_moved.0.lock() {
                output_buffer.push(line.clone());
            }
        }
    });

    ActiveProcess {
        command,
        process,
        output_buffer,
        task,
    }
}
