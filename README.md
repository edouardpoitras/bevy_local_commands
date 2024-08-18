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

**Retries:**

```rust
fn retries(mut commands: Commands) {
    commands.spawn((
        LocalCommand::new("bash").args(["-c", "sleep 1 && invalid-command --that=fails"]),
        Retry::Attempts(3) // Attempt the command 3 times before giving up
    ));
}
```

**Cleanup:**

```rust
fn cleanup_on_completion(mut commands: Commands) {
    commands.spawn((
        LocalCommand::new("bash").args(["-c", "sleep 1"]),
        Cleanup::DespawnEntity // Will despawn the entity upon process completion
        // Cleanup::RemoveComponents // Will remove only this crate's components upon process completion
    ));
}
```

**Delay:**

```rust
fn delay_process_start(mut commands: Commands) {
    commands.spawn((
        LocalCommand::new("bash").args(["-c", "sleep 1"]),
        Delay::Fixed(Duration::from_secs(2)), // Start the process after a 2s delay (applies to each retry)
    ));
}
```

**Chaining:**

```rust
fn chain_multiple_commands(mut commands: Commands) {
    commands.spawn((
        Chain::new(vec![
            LocalCommand::new("sh").args(["-c", "echo 'First command'"]),
            LocalCommand::new("sh").args(["-c", "echo 'Second command'"]),
            LocalCommand::new("sh").args(["-c", "echo 'Third command'"]),
        ]),
        Retry::Attempts(2), // Retry applies to any link in the chain
        Delay::Fixed(Duration::from_secs(3)), // Wait 3s between retries and chain commands
        Cleanup::RemoveComponents // Remove Chain, Retry, Delay, and Cleanup components upon completion
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
