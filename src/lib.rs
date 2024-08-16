use std::process::ExitStatus;
use std::str::Lines;
use std::sync::Arc;
use std::sync::Mutex;

use bevy::prelude::*;

mod addons;
mod local_command;
mod process;
mod systems;

pub use addons::chain::{Chain, ChainCompletedEvent};
pub use addons::cleanup::Cleanup;
pub use addons::delay::Delay;
pub use addons::retry::{Retry, RetryEvent};
pub use local_command::{LocalCommand, LocalCommandDone, LocalCommandState};
pub use process::Process;

/// The ID of a process.
pub type Pid = u32;

#[derive(Debug, Event)]
pub struct ProcessOutput {
    pub entity: Entity,
    /// The output generated in the last frame.
    ///
    /// Has a trailing newline character.
    output: String,
}

impl ProcessOutput {
    /// The whole output string generated by the program.
    ///
    /// Does not include the final terminating newline.
    pub fn all(&self) -> &str {
        self.output.trim_end_matches('\n')
    }

    /// An iterator over the lines of the output, as string slices.
    pub fn lines(&self) -> Lines<'_> {
        self.all().lines()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ProcessErrorInfo {
    FailedToStart,
}

#[derive(Debug, PartialEq, Eq, Clone, Event)]
pub struct ProcessError {
    pub entity: Entity,
    pub info: ProcessErrorInfo,
}

#[derive(Debug, Event)]
pub struct ProcessCompleted {
    pub entity: Entity,
    pub exit_status: ExitStatus,
}

/// The lines written to the standard output by a given process.
#[derive(Debug, Default, Clone)]
struct ProcessOutputBuffer(Arc<Mutex<String>>);

pub struct BevyLocalCommandsPlugin;

impl Plugin for BevyLocalCommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ProcessOutput>()
            .add_event::<ProcessCompleted>()
            .add_event::<ProcessError>()
            .add_event::<RetryEvent>()
            .add_event::<ChainCompletedEvent>()
            .add_systems(PreUpdate, addons::delay::apply_delay)
            .add_systems(
                Update,
                (
                    systems::handle_new_command,
                    systems::handle_process_output,
                    systems::handle_completed_process,
                    addons::cleanup::cleanup_completed_process,
                    addons::retry::retry_failed_process,
                    addons::chain::chain_execution_system,
                )
                    .chain(),
            );
    }
}
