use crate::core::SpawnTrigger;
use crate::prelude::{ScatterAsset, ScatterHandleAsset, ScatterItemAsset};
use bevy::asset::{Asset, Assets, Handle};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

/// Event used to trigger the spawning of a batch of `[ScatterAssets]`.
#[derive(Event, Message, Debug, Clone)]
pub struct SpawnScatterAssets<T>
where
    T: Asset + Clone,
{
    /// A list of asset definitions to be scattered.
    pub items: Vec<ScatterItemAsset<T>>,
    /// Contains the computed scatter results (`data`) and contextual information.
    pub trigger: SpawnTrigger,
}

impl<T> SpawnScatterAssets<T>
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

    pub fn create_name_map<'w>(
        &self,
        prototype_assets: &'w Res<Assets<ScatterAsset<T>>>,
    ) -> HashMap<Name, Vec<ScatterHandleAsset<'w, T>>> {
        let mut name_map: HashMap<Name, Vec<ScatterHandleAsset<'w, T>>> = HashMap::new();

        for (handle, asset) in self
            .items
            .iter()
            .filter_map(|h| prototype_assets.get(&**h).map(|p| ((**h).clone(), p)))
        {
            let name = asset.properties.name.clone().unwrap_or_else(|| {
                warn!("ScatteringAsset {:?} has no name!", handle);

                Name::new("")
            });
            name_map
                .entry(name)
                .or_default()
                .push(ScatterHandleAsset { handle, asset });
        }

        name_map
    }
}

impl<T> From<SpawnTrigger> for SpawnScatterAssets<T>
where
    T: Asset + Clone,
{
    fn from(value: SpawnTrigger) -> Self {
        Self::new(Vec::new(), value)
    }
}
