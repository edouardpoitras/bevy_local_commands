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
    pub output_buffer: Vec<String>,
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

pub struct AdversityLocalCommandsPlugin;

impl Plugin for AdversityLocalCommandsPlugin {
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
                ),
            );
    }
}

fn handle_new_process(
    mut run_shell_command_event: EventReader<RunShellCommand>,
    mut shell_command_started_event: EventWriter<ShellCommandStarted>,
    mut active_process_map: ResMut<ActiveProcessMap>,
) {
    for run_shell_command in run_shell_command_event.iter() {
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
            // Empty the output buffer and send it as event
            let output: Vec<_> = output_buffer.drain(..).collect();

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
    for kill_shell_command in kill_shell_commands.iter() {
        let pid = kill_shell_command.0;

        if let Some(active_process) = active_process_map.0.get_mut(&pid) {
            // Kill the process
            // TODO: Handle unwrap
            active_process.process.kill().unwrap();
        } else {
            warn!("Could not find and kill shell command (PID: {})", pid);
        }

        // The process is killed, we can remove it
        active_process_map.0.remove(&pid);
    }
}

fn handle_completed_shell_commands(
    mut active_shell_commands: ResMut<ActiveProcessMap>,
    mut shell_command_completed_event: EventWriter<ShellCommandCompleted>,
    mut shell_command_output_events: EventWriter<ShellCommandOutput>,
) {
    // FIXME: Figure out how to detect/handle process completion
    todo!();
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
