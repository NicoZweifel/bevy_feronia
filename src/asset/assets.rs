use crate::core::*;
use crate::prelude::*;
use bevy::asset::{Asset, Handle};
use bevy::camera::primitives::Aabb;
use bevy::mesh::Mesh;
use bevy::prelude::*;

#[cfg(feature = "avian")]
use avian3d::prelude::{Collider, RigidBody};

/// Shared properties for a [`ScatterAsset`] and its [`ScatterAssetPart`]s.
#[derive(Clone, Debug)]
pub struct ScatterAssetProperties {
    /// Wind properties applied to this asset/part.
    pub wind: Wind,
    /// Material properties applied to this asset/part.
    pub options: MaterialOptions,
    /// The local [`Aabb`] of this specific asset/part.
    pub aabb: Aabb,
    /// The inherited name.
    pub name: Option<Name>,
    /// The inherited [`LevelOfDetail`].
    pub lod: LevelOfDetail,
    /// The [`Entity`] of the layer this asset belongs to.
    pub layer: Entity,
    /// Whether wind affects this asset/part.
    pub wind_affected: bool,
}

/// An [`Asset`] that represents a combined, multipart scatterable object.
///
/// Holds the final, processed asset data, including the list of
/// all its component parts and their final material handles.
#[derive(Asset, Clone, TypePath)]
pub struct ScatterAsset<T = StandardMaterial>
where
    T: Asset + Clone,
{
    /// Global properties that apply to the asset as a whole.
    pub properties: ScatterAssetProperties,
    /// The list of individual, renderable parts that make up this asset.
    pub parts: Vec<ScatterAssetPart<T>>,
    #[cfg(feature = "avian")]
    /// Optional physics body for the entire asset.
    pub rigid_body: Option<RigidBody>,
}

/// A single, renderable part of a [`ScatterAsset`].
///
/// This typically represents one mesh+material pair from the original hierarchy.
#[derive(Debug, Clone)]
pub struct ScatterAssetPart<T = StandardMaterial>
where
    T: Asset + Clone,
{
    /// The local transform of this part relative to the asset root.
    pub transform: Transform,
    /// Properties specific to this part (inherited/modified during the collection phase).
    pub properties: ScatterAssetProperties,
    /// Handle to the final, processed material (`TOut`).
    pub h_material: Handle<T>,
    /// Handle to the mesh for this part.
    pub h_mesh: Handle<Mesh>,
    #[cfg(feature = "avian")]
    /// Optional physics collider for this specific part.
    pub collider: Option<Collider>,
}

impl<T> ScatterAsset<T>
where
    T: Asset + Clone,
{
    pub fn new(
        parts: Vec<ScatterAssetPart<T>>,
        properties: ScatterAssetProperties,
        #[cfg(feature = "avian")] rigid_body: Option<RigidBody>,
    ) -> Self {
        Self {
            properties,
            parts,
            #[cfg(feature = "avian")]
            rigid_body,
        }
    }
}

impl<T> ScatterAssetPart<T>
where
    T: Asset + Clone,
{
    pub fn new(
        h_material: Handle<T>,
        h_mesh: Handle<Mesh>,
        transform: Transform,
        properties: ScatterAssetProperties,
        #[cfg(feature = "avian")] collider: Option<Collider>,
    ) -> Self {
        Self {
            transform,
            h_mesh,
            h_material,
            properties,
            #[cfg(feature = "avian")]
            collider,
        }
    }

    /// Returns the standard bundle for this asset part.
    pub fn bundle(&self, asset_handle: Handle<ScatterAsset<T>>) -> impl Bundle {
        (
            ScatterItem,
            ScatterItemAsset::<T>(asset_handle.clone()),
            self.properties.lod,
            ChildOf(self.properties.layer),
            ScatterItemOf(self.properties.layer),
            Visibility::Hidden,
            ScatterLayerChildProcessed,
        )
    }

    /// Returns the wind-affected bundle for this asset part.
    pub fn wind_affected_bundle(&self, asset_handle: Handle<ScatterAsset<T>>) -> impl Bundle {
        (WindAffected, self.bundle(asset_handle.clone()))
    }

    /// Inserts the correct bundle (wind-affected or normal) onto the entity.
    pub fn insert_bundle(
        self,
        cmd: &mut Commands,
        entity: Entity,
        asset_handle: Handle<ScatterAsset<T>>,
    ) {
        if self.properties.wind_affected {
            cmd.entity(entity)
                .insert(self.wind_affected_bundle(asset_handle));
        } else {
            cmd.entity(entity).insert(self.bundle(asset_handle));
        }
    }
}

impl<T: Asset + Clone> ProtoType<T> for ScatterAssetPart<T> {
    fn mesh(&self) -> &Handle<Mesh> {
        &self.h_mesh
    }

    fn material(&self) -> &Handle<T> {
        &self.h_material
    }

    fn wind(&self) -> &Wind {
        &self.properties.wind
    }

    fn aabb(&self) -> &Aabb {
        &self.properties.aabb
    }

    fn lod(&self) -> &LevelOfDetail {
        &self.properties.lod
    }

    fn material_options(&self) -> &MaterialOptions {
        &self.properties.options
    }
}
