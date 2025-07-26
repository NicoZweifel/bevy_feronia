use bevy::prelude::*;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ChunkCenter;

#[derive(Component)]
pub struct CanSplit;

#[derive(Component)]
pub struct CanMerge;

#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct Chunk {
   /// The level of detail, e.g. (0=Low, 1=Medium, 2=High).
    pub level: u32,
    pub size: u32,
}
