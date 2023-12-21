use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use bevy::tasks::Task;
use duct::cmd;
use duct::Expression;
use futures_lite::future;
use std::io::prelude::*;
use std::io::BufReader;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::SystemTime;

#[derive(Debug, Event)]
pub struct RunShellCommand {
    program: String,
    arguments: Vec<String>,
}

impl RunShellCommand {
    pub fn new<S: Into<String>>(program: S, arguments: Vec<S>) -> RunShellCommand {
        RunShellCommand { program: program.into(), arguments: arguments.into_iter().map(|s| s.into()).collect() }
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
}

struct ActiveShellCommand {
    command: String,
    task: Option<Task<bool>>,
    pid: u32,
    output_lines: Option<Vec<String>>,
    kill_requested: bool,
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

fn get_shell_command_string(run_shell_command: &RunShellCommand) -> String {
    format!(
        "{} {}",
        run_shell_command.program,
        &run_shell_command.arguments.join(" ")
    )
}

fn handle_new_shell_commands(
    mut run_shell_command_event: EventReader<RunShellCommand>,
    mut shell_command_started_event: EventWriter<ShellCommandStarted>,
    mut shell_command_completed_event: EventWriter<ShellCommandCompleted>,
    mut active_shell_commands: ResMut<ActiveShellCommands>,
) {
    for run_shell_command in run_shell_command_event.iter() {
        let command_string = get_shell_command_string(run_shell_command);
        let expression = cmd(&run_shell_command.program, &run_shell_command.arguments);
        let active_shell_command = spawn_shell_command(command_string.clone(), expression);
        if active_shell_command.is_none() {
            shell_command_completed_event.send(ShellCommandCompleted {
                pid: 0,
                command: command_string.clone(),
                success: false,
            });
            continue;
        }
        let active_shell_command = active_shell_command.unwrap();
        shell_command_started_event.send(ShellCommandStarted {
            command: command_string,
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
            if asc.output_lines.is_some() {
                let pid = asc.pid;
                let lines = asc.output_lines.take();
                shell_command_output.send(ShellCommandOutput { pid, output: lines.unwrap() });
            }
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
                    info!("Killing shell command (PID: {}, Command: {})", pid, &asc.command);
                    asc.kill_requested = true;
                    found = true;
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
) {
    for active_shell_command in active_shell_commands.0.iter_mut() {
        if let Ok(mut asc) = active_shell_command.lock() {
            let result = asc.task.as_mut().and_then(|task_result| {
                if task_result.is_finished() {
                    return future::block_on(future::poll_once(task_result));
                }
                None
            });
            if let Some(result) = result {
                info!(
                    "Command Completed (PID - {}, Success - {}): {}",
                    asc.pid, result, asc.command
                );
                shell_command_completed_event.send(ShellCommandCompleted {
                    success: result,
                    pid: asc.pid,
                    command: asc.command.clone(),
                });
            }
        }
    }
    active_shell_commands.0.retain_mut(|asc| {
        if let Ok(asc) = asc.lock() {
            if let Some(task) = &asc.task {
                if task.is_finished() || asc.task.is_none() {
                    return false;
                }
            }
        }
        true
    });
}

fn spawn_shell_command(command_string: String, expression: Expression) -> Option<Arc<Mutex<ActiveShellCommand>>> {
    let cs = command_string.clone();
    let mut result = false;
    let reader = expression.stderr_to_stdout().reader();
    if let Ok(reader_handle) = reader {
        let mut pid_value = None;
        if let Some(pid) = reader_handle.pids().first() {
            pid_value = Some(*pid);
        }
        if let Some(pid) = pid_value {
            info!("Spawned command with pid {}: {}", pid, &command_string);
            let active_shell_command = ActiveShellCommand {
                command: cs.clone(),
                task: None,
                pid,
                output_lines: None,
                kill_requested: false,
            };
            let active_shell_command = Arc::new(Mutex::new(active_shell_command));
            let asc_moved = active_shell_command.clone();
            let thread_pool = IoTaskPool::get();
            let task = thread_pool.spawn(async move {
                let mut lines = BufReader::new(&reader_handle).lines();
                let mut output_lines: Vec<String> = vec![];
                let mut time = SystemTime::now();
                loop {
                    // TODO: FIXME: This blocks the thread, which is why we can't kill until there's some output
                    // Tried using heim library to kill process but had dependency hell issues
                    let option_result_line = lines.next();
                    if let Some(result_line) = option_result_line {
                        if let Ok(line) = &result_line {
                            if let Ok(mut asc) = asc_moved.lock() {
                                output_lines.push(line.clone());
                                let time_elapsed = time.elapsed().unwrap_or_default();
                                // Only submit buffered lines and check for kill command every second
                                if time_elapsed > std::time::Duration::from_secs(1) {
                                    time = SystemTime::now();
                                    asc.output_lines = Some(output_lines.clone());
                                    output_lines.clear();
                                    // Check for kill command.
                                    if asc.kill_requested {
                                        if let Ok(_) = reader_handle.kill() {
                                            break;
                                        } else {
                                            error!("Failed to kill process PID {}", pid)
                                        }
                                    }
                                }
                            } else {
                                error!(
                                    "Failed to access active shell command for PID {}: {}",
                                    pid, command_string
                                );
                                warn!("We have probably lost command output lines");
                            }
                            continue;
                        } else {
                            error!("Command Exit Error (PID {}): {:?}", pid, result_line.as_ref().err());
                        }
                    } else {
                        info!("Command (PID {}) Completed: {}", pid, &command_string);
                        result = true;
                    }
                    break;
                }
                result
            });
            if let Ok(mut asc) = active_shell_command.lock() {
                asc.task = Some(task);
            } else {
                error!("Failed to add new active shell command: {}", &cs)
            }
            return Some(active_shell_command);
        } else {
            error!("Failed to get PID of shell command: {}", &command_string);
        }
    } else {
        warn!("Failed to spawn shell command: {}", &command_string);
    }
    None
}
