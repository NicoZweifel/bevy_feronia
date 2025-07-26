use bevy::prelude::*;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ChunkCenter;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CanSplit;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CanMerge;

#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct Chunk {
    /// The level of detail, i.e. (0=Low, 1=Medium, 2=High).
    pub level: u32,
    pub size: u32,
}
