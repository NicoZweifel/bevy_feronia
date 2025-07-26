use bevy::prelude::{BufferedEvent, Entity, Event};

#[derive(Event, BufferedEvent)]
pub struct SplitChunk(pub(crate) Entity);

impl SplitChunk {
    pub fn get(&self) -> Entity{
       self.0
    }
}

#[derive(Event, BufferedEvent)]
pub struct MergeChunks {
    pub(crate) siblings: Vec<Entity>,
}

#[derive(Event,BufferedEvent)]
pub struct MergeCheck {
    pub parent: Entity,
    pub children: Vec<Entity>,
}
