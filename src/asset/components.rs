use crate::prelude::*;
use crate::scatter::utils::combine_aabbs;

use bevy_ecs::prelude::*;
use bevy_pbr::StandardMaterial;

use std::marker::PhantomData;

#[cfg(feature = "avian")]
use avian3d::prelude::RigidBody;

/// A [Component] acting as a request to create a [`ScatterAsset`]. These are created by the backend.
///
/// This decouples the "read" phase (scene traversal) and the "write" phase.
/// ([`ScatterAsset`] creation) and is mostly because of a limitation where we cannot have access
/// to the same material Assets twice (duplicate resource access),
/// e.g., when creating a ScatterAsset with a [`StandardMaterial`] from a [`StandardMaterial`].
///
/// Systems like [`queue_asset_creation_requests`] create these,
/// and systems like [`process_distinct_material_requests`] consume them.
#[derive(Component, Clone)]
pub struct ScatterAssetCreationRequest<T = StandardMaterial>
where
    T: ScatterMaterial,
{
    /// Global properties for the asset, collected from the root entity.
    pub properties: ScatterAssetProperties,
    /// The list of all parts found, using the *original* material.
    pub parts: Vec<ScatterAssetPartEntity<StandardMaterial>>,
    #[cfg(feature = "avian")]
    /// The physics body found at the asset root, if any.
    pub o_rigid_body: Option<RigidBody>,

    /// The [`ScatterLayer`] this request was created for.
    pub layer: Entity,

    pub _marker: PhantomData<T>,
}

impl<T> ScatterAssetCreationRequest<T>
where
    T: ScatterMaterial,
{
    pub fn new(
        properties: ScatterAssetProperties,
        parts: Vec<ScatterAssetPartEntity<StandardMaterial>>,
        layer: Entity,
        #[cfg(feature = "avian")] rigid_body: Option<RigidBody>,
    ) -> Self {
        Self {
            properties,
            parts,
            layer,
            #[cfg(feature = "avian")]
            o_rigid_body: rigid_body,
            _marker: Default::default(),
        }
    }

    pub fn from_data(
        item_of: AssetPartOf,
        entity_parts: Vec<ScatterAssetPartEntity<StandardMaterial>>,
        wind: Wind,
        options: ScatterMaterialOptions,
        #[cfg(feature = "avian")] rigid_body: Option<RigidBody>,
    ) -> ScatterAssetCreationRequest<T> {
        let parts: Vec<ScatterAssetPartEntity> = entity_parts.into_iter().collect();

        let mut union_aabb = parts[0].part.properties.aabb;
        for ScatterAssetPartEntity { part, .. } in &parts[1..] {
            // TODO transforms
            union_aabb = combine_aabbs(&union_aabb, &part.properties.aabb);
        }

        let wind_affected = parts
            .iter()
            .any(|ScatterAssetPartEntity { part, .. }| part.properties.wind_affected);

        ScatterAssetCreationRequest::<T>::new(
            ScatterAssetProperties {
                wind,
                options,
                aabb: union_aabb,
                name: item_of.name.clone(),
                lod: parts
                    .iter()
                    .map(|ScatterAssetPartEntity { part, .. }| part.properties.lod)
                    .min()
                    .unwrap_or_default(),
                #[allow(deprecated)]
                wind_affected,
            },
            parts,
            item_of.layer,
            #[cfg(feature = "avian")]
            rigid_body,
        )
    }
}
