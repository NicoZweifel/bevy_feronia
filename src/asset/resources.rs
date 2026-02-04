use crate::prelude::*;
use bevy_asset::AssetId;
use bevy_ecs::prelude::*;
use bevy_platform::collections::HashMap;
use bevy_reflect::prelude::Reflect;
use std::marker::PhantomData;

/// Manages references between [`ScatterAsset`]s and the layers they belong to.
///
/// This allows assets to be independent of specific ECS world states while still maintaining
/// the ability to look up their origin.
///
/// This might have to change when Assets become entities in bevy TODO
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct ScatterAssetManager<T: ScatterMaterialAsset> {
    /// Maps the [`AssetId`] of a [`ScatterAsset`] to the [`Entity`] of the layer it belongs to.
    pub asset_to_layer: HashMap<AssetId<ScatterAsset<T>>, Entity>,
    /// Maps the [`AssetId`] of a [`ScatterAsset`] and the parts to the entities they belong to.
    pub asset_to_entity: HashMap<AssetId<ScatterAsset<T>>, (Entity, Vec<Entity>)>,
    #[reflect(ignore)]
    _marker: PhantomData<T>,
}

impl<T: ScatterMaterialAsset> Default for ScatterAssetManager<T> {
    fn default() -> Self {
        Self {
            asset_to_layer: HashMap::new(),
            asset_to_entity: HashMap::new(),
            _marker: PhantomData,
        }
    }
}
