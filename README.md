# Bevy Local Commands

[![Bevy Local Commands](https://github.com/edouardpoitras/bevy_local_commands/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/edouardpoitras/bevy_local_commands/actions/workflows/rust.yml)
[![Latest version](https://img.shields.io/crates/v/bevy_local_commands.svg)](https://crates.io/crates/bevy_local_commands)
[![Documentation](https://docs.rs/bevy_local_commands/badge.svg)](https://docs.rs/bevy_local_commands)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)
![Apache](https://img.shields.io/badge/license-Apache-blue.svg)

Bevy plugin that exposes events that can be used to execute simple shell commands.

## Usage

**Add the plugin:**

```rust
// ...
.add_plugins(BevyLocalCommandsPlugin)
// ...
```

**Run shell commands:**

```rust
fn run_command(
    mut shell_commands: EventWriter<RunShellCommand>,
) {
    shell_commands.send(RunShellCommand::new("bash", vec!["-c", "sleep 1 && echo slept"]));
}
```

**See commands started and kill running commands:**

```rust
fn kill_started_command(
    mut shell_command_started: EventReader<ShellCommandStarted>,
    mut kill_commands: EventWriter<KillShellCommand>,
) {
    if let (Some(shell_command_started)) = shell_command_started.iter().last() {
        warn!("Sending kill command for {}", shell_command_started.pid);
        kill_shell_commands.send(KillShellCommand(shell_command_started.pid));
    }
}
```

Note: Current limitation - kill will only trigger when the command generates output.

**Receive command output:**

```rust
fn get_command_output(mut shell_command_output: EventReader<ShellCommandOutput>) {
    for command_output in shell_command_output.iter() {
        info!("Command PID: {}", command_output.pid);
        for line in  command_output.output.iter() {
            info!("Line Output: {}", line);
        }
    }
}
```

**See commands completed:**

```rust
fn get_completed(mut shell_command_completed: EventReader<ShellCommandCompleted>) {
    for completed in shell_command_completed.iter() {
        info!("Command completed (PID - {}, Success - {}): {}", completed.pid, completed.success, completed.command);
    }
}
```

## Todo

- [ ] Better way to kill commands that are still running
- [ ] Windows/Mac testing (not sure if it works yet)


## Bevy Compatilibity

|bevy|bevy_local_commands|
|---|---|
|0.12|0.2|
|0.11|0.1|