use crate::prelude::events::SpawnScatterAssets;
use crate::prelude::*;

use bevy_ecs::message::MessageWriter;
use bevy_ecs::system::ResMut;

/// A simple system that processes one event per frame by popping it of the [`SpawnScatterAssetsEventQueue<T>`].
/// In large worlds with lots of chunks, this prevents frame drops and stutter when multiple chunks with many instances are spawned at the same time.
///
/// This is more or less a simple Band-Aid fix, and more tracing and optimization has to be done,
/// e.g., optimizing CPU scatter tasks and adding a scatter compute pipeline.
pub fn process_scatter_queue<T>(
    mut queue: ResMut<SpawnScatterAssetsEventQueue<T>>,
    mut mw_spawn: MessageWriter<SpawnScatterAssets<T>>,
) where
    T: ScatterMaterial,
{
    let Some(event) = queue.pop_front() else {
        return;
    };

    mw_spawn.write(event);
}
