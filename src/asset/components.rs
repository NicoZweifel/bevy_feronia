use crate::prelude::*;
use bevy_ecs::prelude::*;
use std::marker::PhantomData;

use crate::scatter::utils::combine_aabbs;
#[cfg(feature = "avian")]
use avian3d::prelude::RigidBody;
use bevy_pbr::StandardMaterial;

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
    pub parts: Vec<ScatterAssetPart<StandardMaterial>>,
    #[cfg(feature = "avian")]
    /// The physics body found at the asset root, if any.
    pub o_rigid_body: Option<RigidBody>,

    /// The [`ScatterLayer`] this request was created for.
    pub layer: Entity,

    pub _phantom_data: PhantomData<T>,
}

impl<T> ScatterAssetCreationRequest<T>
where
    T: ScatterMaterial,
{
    pub fn new(
        properties: ScatterAssetProperties,
        parts: Vec<ScatterAssetPart<StandardMaterial>>,
        layer: Entity,
        #[cfg(feature = "avian")] rigid_body: Option<RigidBody>,
    ) -> Self {
        Self {
            properties,
            parts,
            layer,
            #[cfg(feature = "avian")]
            o_rigid_body: rigid_body,
            _phantom_data: Default::default(),
        }
    }

    pub fn from_data(
        item_of: AssetItemOf,
        entity_parts: Vec<ScatterAssetPartEntity<StandardMaterial>>,
        wind: Wind,
        options: MaterialOptions,
    ) -> ScatterAssetCreationRequest<T> {
        let parts: Vec<ScatterAssetPart> =
            entity_parts.into_iter().map(|p| p.part.clone()).collect();

        let mut union_aabb = parts[0].properties.aabb;
        for part in &parts[1..] {
            union_aabb = combine_aabbs(&union_aabb, &part.properties.aabb);
        }

        let any_wind_affected = parts.iter().any(|part| part.properties.wind_affected);

        ScatterAssetCreationRequest::<T>::new(
            ScatterAssetProperties {
                wind,
                options,
                aabb: union_aabb,
                name: item_of.name.clone(),
                lod: parts
                    .iter()
                    .map(|part| part.properties.lod)
                    .min()
                    .unwrap_or_default(),
                layer: Some(item_of.layer),
                wind_affected: any_wind_affected,
            },
            parts,
            item_of.layer,
            #[cfg(feature = "avian")]
            scene_root_data.o_rigid_body.cloned(),
        )
    }
}
