use bevy::prelude::*;

#[derive(Event, Message, Deref)]
pub struct SplitChunk(pub(crate) Entity);

impl SplitChunk {
    pub fn get(&self) -> Entity {
        self.0
    }
}

#[derive(Event, Message)]
pub struct MergeChunks {
    pub children: Vec<Entity>,
    pub parent: Entity,
}

#[derive(Event, Message)]
pub struct MergeCheck {
    pub parent: Entity,
    pub children: Vec<Entity>,
}
