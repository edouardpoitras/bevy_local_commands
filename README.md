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
    commands.spawn(LocalCommand::new("bash").args(["-c", "sleep 1 && echo slept"]));
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

        for line in output.lines() {
            info!("Line Output: {}", line);
        }
    }
}
```

**Send command input:**

```rust
fn send_command_input(
    mut process_output_event: EventReader<ProcessOutput>,
    mut active_processes: Query<&mut Process>,
) {
    for output in process_output_event.read() {
        for line in output.lines() {
            if line.ends_with("Prompt String: ") {
                let mut process = active_processes.get_mut(output.entity).unwrap();
                process.println("Text to send").expect("Failed to write to process");
            }
        }
    }
}
```

**See commands completed:**

```rust
fn get_completed(mut process_completed_event: EventReader<ProcessCompleted>) {
    for completed in process_completed_event.read() {
        info!(
            "Command completed (Entity - {}, Success - {})",
            completed.entity,
            completed.exit_status.success()
        );
    }
}
```

**Retry and cleanup behavior:**

```rust
fn retries_and_cleanup_on_completion(mut commands: Commands) {
    commands.spawn((
        LocalCommand::new("bash").args(["-c", "sleep 1 && invalid-command --that=fails"]),
        // Attempt the command 3 times before giving up
        // NOTE: The Retry component will be removed from the entity when no retries are left
        Retry::Attempts(3)
        // Cleanup::DespawnEntity will despawn the entity upon process completion.
        // Cleanup::RemoveComponents will remove this crate's components upon process completion.
        Cleanup::DespawnEntity
    ));
}
```

## Todo

- [ ] Mac testing (not sure if it works yet)

## Bevy Compatilibity

| bevy | bevy_local_commands |
| ---- | ------------------- |
| 0.14 | 0.6                 |
| 0.13 | 0.5                 |
| 0.12 | 0.4                 |
| 0.11 | 0.1                 |
