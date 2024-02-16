use bevy::prelude::*;

use crate::LocalCommand;

#[derive(Component)]
pub struct Delay {
    pub seconds: f32,
}

pub(crate) fn handle_new_delayed_command(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Delay), With<LocalCommand>>,
    time: Res<Time>,
) {
    for (entity, mut delay) in query.iter_mut() {
        delay.seconds = delay.seconds - time.delta_seconds();
        if delay.seconds <= 0.0 {
            if let Some(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.remove::<Delay>();
            }
        }
    }
}
