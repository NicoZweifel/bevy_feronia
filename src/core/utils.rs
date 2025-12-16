use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Commands;

pub fn despawn(cmd: &mut Commands, iter: impl IntoIterator<Item = Entity>) {
    for e in iter.into_iter() {
        cmd.entity(e).despawn();
    }
}
