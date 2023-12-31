# Bevy Local Commands

[![Bevy Local Commands](https://github.com/edouardpoitras/bevy_local_commands/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/edouardpoitras/bevy_local_commands/actions/workflows/rust.yml)
[![Latest version](https://img.shields.io/crates/v/bevy_local_commands.svg)](https://crates.io/crates/bevy_local_commands)
[![Documentation](https://docs.rs/bevy_local_commands/badge.svg)](https://docs.rs/bevy_local_commands)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)
![Apache](https://img.shields.io/badge/license-Apache-blue.svg)

Bevy plugin to manage local shell commands.

## Usage

**Add the plugin:**

```rust
// ...
.add_plugins(BevyLocalCommandsPlugin)
// ...
```

**Run shell commands:**

```rust
fn run_command(mut commands: Commands) {
    let mut cmd = std::process::Command::new("bash");
    cmd.args(["-c", "sleep 1 && echo slept"]);

    commands.spawn(LocalCommand::new(cmd));
}
```

**See commands started and kill running commands:**

```rust
fn kill_started_command(mut active_processes: Query<&mut Process>) {
    for mut process in active_processes.iter_mut() {
        warn!("Killing process {}", process.id());
        process.kill().unwrap();
    }
}
```

**Receive command output:**

```rust
fn get_command_output(mut process_output_event: EventReader<ProcessOutput>) {
    for output in process_output_event.read() {
        info!("Output for command {:?}", output.entity);

        for line in output.output.iter() {
            info!("Line Output: {}", line);
        }
    }
}
```

**See commands completed:**

```rust
fn get_completed(mut process_completed_event: EventReader<ProcessCompleted>) {
    for completed in process_completed_event.read() {
        info!("Command completed (Entity - {}, Success - {})", completed.entity, completed.success);
    }
}
```

## Todo

- [ ] Mac testing (not sure if it works yet)

## Bevy Compatilibity

| bevy | bevy_local_commands |
| ---- | ------------------- |
| 0.12 | 0.3                 |
| 0.11 | 0.1                 |
