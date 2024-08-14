use bevy::prelude::*;
use std::time::Duration;

use crate::{process::Process, LocalCommand, LocalCommandState};

#[derive(Debug, Component)]
pub enum Delay {
    Fixed(Duration),
}

/// Apply delay settings to entities with LocalCommand + Delay components that have yet to be processed.
///
/// State of LocalCommandState::Ready is required for the delay to be applied.
/// This system should run before the handle_new_command system.
pub(crate) fn apply_delay(mut query: Query<(&mut LocalCommand, &Delay), Without<Process>>) {
    for (mut local_command, delay) in query.iter_mut() {
        if local_command.state == LocalCommandState::Ready && local_command.delay.is_none() {
            match delay {
                Delay::Fixed(duration) => {
                    local_command.delay =
                        Some(Timer::from_seconds(duration.as_secs_f32(), TimerMode::Once));
                },
            }
        }
    }
}
