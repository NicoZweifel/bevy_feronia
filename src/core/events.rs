use crate::core::SpawnTrigger;
use crate::prelude::ScatterItemAsset;
use bevy::asset::Asset;
use bevy::prelude::{Event, Message};

/// Event used to trigger the spawning of a batch of prototypes.
#[derive(Event, Message, Debug, Clone)]
pub struct SpawnProtoTypes<T>
where
    T: Asset + Clone,
{
    /// A list of asset definitions to be scattered.
    pub items: Vec<ScatterItemAsset<T>>,
    /// Where and why the spawn was triggered.
    pub trigger: SpawnTrigger,
}

impl<T> SpawnProtoTypes<T>
where
    T: Asset + Clone,
{
    pub fn new(items: Vec<ScatterItemAsset<T>>, trigger: SpawnTrigger) -> Self {
        Self { items, trigger }
    }

    pub fn with_items(mut self, items: Vec<ScatterItemAsset<T>>) -> Self {
        self.items = items;
        self
    }
}

impl<T> From<SpawnTrigger> for SpawnProtoTypes<T>
where
    T: Asset + Clone,
{
    fn from(value: SpawnTrigger) -> Self {
        Self::new(Vec::new(), value)
    }
}
