use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{process::Process, LocalCommand, LocalCommandState};

#[derive(Debug, Component)]
pub struct Cleanup<T: Component> {
    data: PhantomData<T>
}

/// Clean up any completed processes according to the Cleanup component.
///
/// Processes without the Cleanup component are ignored.
pub(crate) fn cleanup_completed_process<T: Component>(
    mut commands: Commands,
    query: Query<(Entity, &LocalCommand), With<Cleanup<T>>>,
) {
    for (entity, local_command) in query.iter() {
        if let LocalCommandState::Done(_) = local_command.state {
            if let Some(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.remove::<(LocalCommand, Process, T, Cleanup<T>)>();
            }
        }
    }
}
