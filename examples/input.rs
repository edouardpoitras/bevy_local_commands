use bevy::prelude::*;
use bevy_local_commands::{
    BevyLocalCommandsPlugin, LocalCommand, Process, ProcessCompleted, ProcessOutput,
};

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, BevyLocalCommandsPlugin))
        .add_systems(Startup, startup)
        .add_systems(Update, update)
        .run();
}

fn startup(mut commands: Commands) {
    // Choose the command based on the OS
    #[cfg(not(windows))]
    let cmd =
        LocalCommand::new("sh").args(["-c", "echo 'Enter Name:' && read NAME && echo Hello $NAME"]);
    #[cfg(windows)]
    let cmd = LocalCommand::new("powershell")
        .args(["$name = Read-Host 'Enter Name'; echo \"Name Entered: $name\""]);
    let id = commands.spawn(cmd).id();
    println!("Spawned the command as entity {id:?}");
}

fn update(
    mut process_output_event: EventReader<ProcessOutput>,
    mut process_completed_event: EventReader<ProcessCompleted>,
    mut active_processes: Query<&mut Process>,
) {
    for process_output in process_output_event.read() {
        for line in process_output.lines() {
            println!("{line}");
        }
    }
    if let Ok(mut process) = active_processes.get_single_mut() {
        process.println("Bevy").unwrap_or_default();
    }
    if let Some(process_completed) = process_completed_event.read().last() {
        println!(
            "Command {:?} completed (Success - {})",
            process_completed.entity,
            process_completed.exit_status.success()
        );
        // Quit the app
        std::process::exit(0);
    }
}
