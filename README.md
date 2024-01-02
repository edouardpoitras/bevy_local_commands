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
    mut shell_commands: EventWriter<RunProcess>,
) {
    shell_commands.send(RunProcess::new("bash", vec!["-c", "sleep 1 && echo slept"]));
}
```

**See commands started and kill running commands:**

```rust
fn kill_started_command(
    mut process_started: EventReader<ProcessStarted>,
    mut kill_process: EventWriter<KillProcess>,
) {
    if let (Some(process_started)) = process_started.iter().last() {
        warn!("Sending kill command for {}", process_started.pid);
        kill_process.send(KillProcess(process_started.pid));
    }
}
```

**Receive command output:**

```rust
fn get_command_output(mut process_output_event: EventReader<ProcessOutput>) {
    for output in process_output_event.iter() {
        info!("Command PID: {}", output.pid);
        for line in output.output.iter() {
            info!("Line Output: {}", line);
        }
    }
}
```

**See commands completed:**

```rust
fn get_completed(mut process_completed: EventReader<ProcessCompleted>) {
    for completed in process_completed.iter() {
        info!("Command completed (PID - {}, Success - {}): {}", completed.pid, completed.success, completed.command);
    }
}
```

## Todo

- [ ] Mac testing (not sure if it works yet)


## Bevy Compatilibity

|bevy|bevy_local_commands|
|---|---|
|0.12|0.2|
|0.11|0.1|
