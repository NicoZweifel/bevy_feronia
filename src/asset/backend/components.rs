use bevy_ecs::prelude::*;
use bevy_ecs::query::QueryFilter;
use bevy_reflect::Reflect;

#[derive(Component, Default)]
pub struct NeedsAssetCollection;

#[derive(Clone, Debug)]
pub struct AssetPart {
    pub entity: Entity,
    pub item_of: AssetPartOf,
}

impl AssetPart {
    pub fn new(entity: Entity, item_of: AssetPartOf) -> Self {
        Self { entity, item_of }
    }
}

impl From<(Entity, &AssetPartOf)> for AssetPart {
    fn from((entity, item_of): (Entity, &AssetPartOf)) -> Self {
        Self::new(entity, item_of.clone())
    }
}

#[derive(Reflect, Eq, PartialEq, Hash, Clone, Debug, Component)]
#[reflect(Component, Clone, Debug, PartialEq, Hash)]
pub struct AssetPartOf {
    /// The entity that this part belongs to.
    pub item: Entity,
    /// The root entity of the layer this part belongs to.
    pub root: Entity,
    /// The entity of the layer this part belongs to.
    pub layer: Entity,
    pub name: Option<Name>,
}

impl AssetPartOf {
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
            .or_else(|_| q_parent.get(entity).and_then(|x| q_name.get(x.parent())))
            .or_else(|_| q_name.get(entity))
            .ok()
            .or(self.name.as_ref())
            .cloned();

        self
    }
}
