use crate::core::*;
use crate::prelude::*;
use bevy::asset::{Asset, Handle};
use bevy::camera::primitives::Aabb;
use bevy::mesh::Mesh;
use bevy::prelude::*;

#[cfg(feature = "avian")]
use avian3d::prelude::{Collider, RigidBody};

#[derive(Clone, Debug)]
pub struct ScatterAssetProperties {
    pub wind: Wind,
    pub options: MaterialOptions,
    pub aabb: Aabb,
    pub name: Option<Name>,
    pub lod: LevelOfDetail,
    pub layer: Entity,
    pub wind_affected: bool,
}

#[derive(Asset, Clone, TypePath)]
pub struct ScatterAsset<T = StandardMaterial>
where
    T: Asset + Clone,
{
    pub properties: ScatterAssetProperties,
    pub parts: Vec<ScatterAssetPart<T>>,

    #[cfg(feature = "avian")]
    pub rigid_body: Option<RigidBody>,
}

#[derive(Debug, Clone)]
pub struct ScatterAssetPart<T = StandardMaterial>
where
    T: Asset + Clone,
{
    pub transform: Transform,
    pub properties: ScatterAssetProperties,
    pub h_material: Handle<T>,
    pub h_mesh: Handle<Mesh>,

    #[cfg(feature = "avian")]
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

    pub fn wind_affected_bundle(&self, asset_handle: Handle<ScatterAsset<T>>) -> impl Bundle {
        (WindAffected, self.bundle(asset_handle.clone()))
    }

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
