use crate::prelude::*;
use bevy::pbr::Material;
use bevy::prelude::*;
use std::marker::PhantomData;

#[cfg(feature = "avian")]
use avian3d::prelude::RigidBody;

/// A [Component] acting as a request to create a [`ScatterAsset`].
///
/// This decouples the "read" phase (scene traversal) and the "write" phase
/// ([`ScatterAsset`] processing).
///
/// Systems like [`queue_asset_creation_requests`] create these,
/// and systems like [`process_distinct_material_requests`] consume them.
#[derive(Component, Clone)]
pub struct ScatterAssetCreationRequest<TOut, TIn>
where
    TIn: Material,
    TOut: ScatterMaterial<TIn>,
{
    /// Global properties for the asset, collected from the root entity.
    pub properties: ScatterAssetProperties,
    /// The list of all parts found, using the *original* `TIn` material.
    pub parts: Vec<ScatterAssetPart<TIn>>,
    #[cfg(feature = "avian")]
    /// The physics body found at the asset root, if any.
    pub o_rigid_body: Option<RigidBody>,

    pub _phantom: PhantomData<TOut>,
}

impl<TOut, TIn> ScatterAssetCreationRequest<TOut, TIn>
where
    TIn: Material,
    TOut: ScatterMaterial<TIn>,
{
    pub fn new(
        properties: ScatterAssetProperties,
        parts: Vec<ScatterAssetPart<TIn>>,
        #[cfg(feature = "avian")] rigid_body: Option<RigidBody>,
    ) -> Self {
        Self {
            properties,
            parts,
            #[cfg(feature = "avian")]
            o_rigid_body: rigid_body,
            _phantom: default(),
        }
    }
}
