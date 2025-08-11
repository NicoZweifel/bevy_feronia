use crate::asset::assets::ScatterAsset;
use crate::prelude::*;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use rand::prelude::{IndexedRandom, IteratorRandom};

#[derive(Resource, Deref, DerefMut, Clone, Debug)]
pub struct ScatterAssets<T>(pub Vec<Handle<ScatterAsset<T>>>)
where
    T: Asset + Clone;

impl<T> Default for ScatterAssets<T>
where
    T: Asset + Clone,
{
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T: Asset + Clone> ProtoTypes<T, ScatterAsset<T>> for ScatterAssets<T> {
    fn choose(
        &self,
        scatter_items: &Vec<ScatterItemType<T>>,
    ) -> Option<HashMap<LodLevel, Handle<ScatterAsset<T>>>> {
        let mut rng = rand::rng();

        info!("Scattering Assets! {:?}", scatter_items.len());

        let items = scatter_items.iter().map(|x| match x {
            ScatterItemType::Handle(x) => Some(x.clone()),
            _ => self.0.choose(&mut rng).map(|x| x.clone()),
        });

        items
            .choose(&mut rand::rng())?
            .map(|x| HashMap::from([(LodLevel(0), x)]))
    }
}

#[derive(Resource, Deref, DerefMut, Clone, Debug)]
pub struct ScatterAssetsNameMap<T>(HashMap<Name, HashMap<LodLevel, Handle<ScatterAsset<T>>>>)
where
    T: Asset + Clone;

impl<T> Default for ScatterAssetsNameMap<T>
where
    T: Asset + Clone,
{
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl<T: Asset + Clone> ProtoTypes<T, ScatterAsset<T>> for ScatterAssetsNameMap<T> {
    fn choose(
        &self,
        scatter_items: &Vec<ScatterItemType<T>>,
    ) -> Option<HashMap<LodLevel, Handle<ScatterAsset<T>>>> {
        let mut rng = rand::rng();

        info!("Scattering Assets! {:?}", scatter_items.len());

        let scatter_type = scatter_items.choose(&mut rng)?;
        match scatter_type {
            ScatterItemType::Handle(x) => Some(HashMap::from([(LodLevel(0), x.clone())])),
            ScatterItemType::Name(x) => {
                info!("Scattering Asset {x}!");
                self.get(x)
                    .map_or(self.0.values().choose(&mut rng).map(|x| x.clone()), |x| {
                        Some(x.clone())
                    })
            }
        }
    }
}
