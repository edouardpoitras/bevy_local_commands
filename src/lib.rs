use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use std::io::prelude::*;
use std::io::BufReader;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::Mutex;

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
    pub pid: u32,
}

#[derive(Debug, Event)]
pub struct ShellCommandOutput {
    pub pid: u32,
    pub command: String,
    pub output: Vec<String>,
}

/// This will only trigger on stdout/stderr events for the process.
/// IE: 'sleep 9999' won't be killed cause no output is produced.
/// Work-arounds/fixes are being considered.
#[derive(Debug, Event)]
pub struct KillShellCommand(pub u32);

#[derive(Debug, Event)]
pub struct ShellCommandCompleted {
    pub success: bool,
    pub pid: u32,
    pub command: String,
    pub output_buffer: Vec<String>,
}

struct ActiveShellCommand {
    command: Command,
    process: Child,
    output_lines: Vec<String>,
    pid: u32,
}

#[derive(Default, Resource)]
struct ActiveShellCommands(Vec<Arc<Mutex<ActiveShellCommand>>>);

pub struct AdversityLocalCommandsPlugin;

impl Plugin for AdversityLocalCommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RunShellCommand>()
            .add_event::<ShellCommandStarted>()
            .add_event::<ShellCommandOutput>()
            .add_event::<KillShellCommand>()
            .add_event::<ShellCommandCompleted>()
            .init_resource::<ActiveShellCommands>()
            .add_systems(
                Update,
                (
                    handle_new_shell_commands,
                    handle_shell_command_output,
                    handle_kill_shell_command,
                    handle_completed_shell_commands,
                ),
            );
    }
}

fn handle_new_shell_commands(
    mut run_shell_command_event: EventReader<RunShellCommand>,
    mut shell_command_started_event: EventWriter<ShellCommandStarted>,
    mut active_shell_commands: ResMut<ActiveShellCommands>,
) {
    for run_shell_command in run_shell_command_event.iter() {
        // Assemble the command
        let mut cmd = Command::new(run_shell_command.program.clone());
        cmd.args(run_shell_command.arguments.clone())
            .stdout(Stdio::piped());

        let active_shell_command = spawn_shell_command(cmd);

        shell_command_started_event.send(ShellCommandStarted {
            command: format!("{:?}", active_shell_command.lock().unwrap().command), // TODO: Get rid of unwrap
            pid: active_shell_command.lock().unwrap().pid, // TODO: Get rid of unwrap
        });
        active_shell_commands.0.push(active_shell_command);
    }
}

fn handle_shell_command_output(
    mut active_shell_commands: ResMut<ActiveShellCommands>,
    mut shell_command_output: EventWriter<ShellCommandOutput>,
) {
    for active_shell_command in active_shell_commands.0.iter_mut() {
        let asc = active_shell_command.lock();
        if asc.is_ok() {
            let mut asc = asc.unwrap();

            // Empty the output buffer and send it as event
            let output: Vec<_> = asc.output_lines.drain(..).collect();

            shell_command_output.send(ShellCommandOutput {
                pid: asc.pid,
                command: format!("{:?}", asc.command),
                output,
            });
        }
    }
}

fn handle_kill_shell_command(
    mut active_shell_commands: ResMut<ActiveShellCommands>,
    mut kill_shell_commands: EventReader<KillShellCommand>,
) {
    for kill_shell_command in kill_shell_commands.iter() {
        let pid = kill_shell_command.0;
        let mut found = false;
        for active_shell_command in active_shell_commands.0.iter_mut() {
            let asc = active_shell_command.lock();
            if asc.is_ok() {
                let mut asc = asc.unwrap();
                if asc.pid == pid {
                    info!(
                        "Killing shell command (PID: {pid}, Command: {:?})",
                        asc.command
                    );
                    found = true;

                    // Kill the process
                    asc.process.kill().unwrap();
                    break;
                }
            }
        }
        if !found {
            warn!("Could not find and kill shell command (PID: {})", pid);
        }
    }
}

fn handle_completed_shell_commands(
    mut active_shell_commands: ResMut<ActiveShellCommands>,
    mut shell_command_completed_event: EventWriter<ShellCommandCompleted>,
    mut shell_command_output_events: EventWriter<ShellCommandOutput>,
) {
    // FIXME: Figure out how to detect/handle process completion
    todo!();
}

fn spawn_shell_command(mut cmd: Command) -> Arc<Mutex<ActiveShellCommand>> {
    // Start running the process
    let mut process = cmd.spawn().unwrap();
    let stdout = process.stdout.take();
    let pid = process.id();

    info!("Spawned command with pid {pid}: {cmd:?}");

    let active_shell_command = ActiveShellCommand {
        command: cmd,
        process,
        pid,
        output_lines: Vec::new(),
    };

    let active_shell_command = Arc::new(Mutex::new(active_shell_command));

    let asc_moved = active_shell_command.clone();
    let thread_pool = IoTaskPool::get();

    // Read stdout and write it to the output buffer
    if let Some(stdout) = stdout {
        // FIXME: Figure out what we need to do with the task
        let _task = thread_pool.spawn(async move {
            let mut reader = BufReader::new(stdout);

            let mut line = String::new();

            while let Ok(bytes) = reader.read_line(&mut line) {
                if bytes == 0 {
                    break;
                }

                if let Ok(mut asc) = asc_moved.lock() {
                    asc.output_lines.push(line.clone());
                }
            }
        });
    }

    active_shell_command
}
