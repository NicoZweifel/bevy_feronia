use bevy::math::Vec3;
use bevy::prelude::{BufferedEvent, Entity, Event};

#[derive(Event, BufferedEvent)]
pub struct SplitChunk(pub(crate) Entity);

#[derive(Event, BufferedEvent)]
pub struct MergeChunks {
    pub(crate) siblings: Vec<Entity>,
    pub(crate) parent_center: Vec3,
    pub(crate) parent_level: u32,
}
