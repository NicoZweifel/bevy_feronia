use bevy_ecs::prelude::*;
use bevy_ecs::query::QueryFilter;
use bevy_reflect::Reflect;

#[derive(Component, Default)]
pub struct NeedsAssetCollection;

#[derive(Clone, Debug)]
pub struct AssetItem {
    pub entity: Entity,
    pub item_of: AssetItemOf,
}

impl AssetItem {
    pub fn new(entity: Entity, item_of: AssetItemOf) -> Self {
        Self { entity, item_of }
    }
}

impl From<(Entity, &AssetItemOf)> for AssetItem {
    fn from((entity, item_of): (Entity, &AssetItemOf)) -> Self {
        Self::new(entity, item_of.clone())
    }
}

#[derive(Reflect, Eq, PartialEq, Hash, Clone, Debug, Component)]
#[reflect(Component, Clone, Debug, PartialEq, Hash)]
pub struct AssetItemOf {
    pub item: Entity,
    pub root: Entity,
    pub layer: Entity,
    pub name: Option<Name>,
}

impl AssetItemOf {
    pub fn new(item: Entity, root: Entity, layer: Entity) -> Self {
        Self {
            item,
            root,
            layer,
            name: None,
        }
    }

    /// Uses root item name, then parent name, otherwise child name.
    pub fn with_name_from_queries<F: QueryFilter, P: QueryFilter>(
        mut self,
        entity: Entity,
        q_name: &Query<&Name, F>,
        q_parent: &Query<&ChildOf, P>,
    ) -> Self {
        self.name = q_name
            .get(self.item)
            .or_else(|_| {
                q_parent
                    .get(entity)
                    .map(|x| q_name.get(x.parent()))
                    .flatten()
            })
            .or_else(|_| q_name.get(entity))
            .ok()
            .or(self.name.as_ref())
            .cloned();

        self
    }
}
