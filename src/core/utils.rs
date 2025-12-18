use bevy_ecs::prelude::*;

pub fn despawn(cmd: &mut Commands, iter: impl IntoIterator<Item = Entity>) {
    for e in iter.into_iter() {
        cmd.entity(e).despawn();
    }
}
