use bevy_derive::Deref;
use bevy_ecs::prelude::*;
use bevy_math::Vec3;

#[derive(Event, Message, Deref)]
pub struct SplitChunk(pub(crate) Entity);

impl SplitChunk {
    pub fn get(&self) -> Entity {
        self.0
    }
}

#[derive(Event, Message)]
pub struct MergeChunks {
    pub parent: Entity,
    pub children: Vec<Entity>,
}

impl MergeChunks {
    pub fn new(parent: Entity, children: Vec<Entity>) -> Self {
        Self { parent, children }
    }
}

impl From<(Entity, Vec<Entity>)> for MergeChunks {
    fn from((parent, children): (Entity, Vec<Entity>)) -> Self {
        Self::new(parent, children)
    }
}

#[derive(Event, Message, Clone)]
pub struct MergeCheck {
    pub parent: Entity,
    pub children: Vec<Entity>,
    pub parent_translation: Vec3,
    pub merge_distance: f32,
    pub center: Vec3,
}

impl MergeCheck {
    pub fn new(parent: Entity, children: Vec<Entity>) -> Self {
        Self {
            parent,
            children,
            parent_translation: Vec3::ZERO,
            merge_distance: 0.0,
            center: Vec3::ZERO,
        }
    }

    pub fn with_parent_translation(mut self, parent_translation: impl Into<Vec3>) -> Self {
        self.parent_translation = parent_translation.into();
        self
    }

    pub fn with_merge_distance(mut self, merge_distance: impl Into<f32>) -> Self {
        self.merge_distance = merge_distance.into();
        self
    }

    pub fn with_center(mut self, center: impl Into<Vec3>) -> Self {
        self.center = center.into();
        self
    }

    pub fn check(&self) -> bool {
        let distance = self.center.distance(self.parent_translation);
        self.merge_distance < distance
    }
}
