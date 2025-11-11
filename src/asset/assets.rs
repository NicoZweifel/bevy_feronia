use crate::core::components::LevelOfDetail;
use crate::core::*;
use crate::prelude::*;
use bevy::asset::{Asset, Handle};
use bevy::camera::primitives::Aabb;
use bevy::mesh::Mesh;
use bevy::prelude::*;

#[cfg(all(feature = "avian"))]
use avian3d::prelude::RigidBody;

#[derive(Clone, Debug)]
pub struct ScatterAssetProperties {
    pub wind: Wind,
    pub options: MaterialOptions,
    pub mesh_handle: Handle<Mesh>,
    pub aabb: Aabb,
    pub name: Option<Name>,
    pub lod: LevelOfDetail,
    pub layer: Entity,
    pub wind_affected: bool,

    #[cfg(all(feature = "avian"))]
    pub rigid_body: Option<RigidBody>,
}

#[derive(Asset, TypePath, Clone, Debug)]
pub struct ScatterAsset<T = StandardMaterial>
where
    T: Asset + Clone,
{
    pub material: Handle<T>,
    pub properties: ScatterAssetProperties,
}

impl<T> ScatterAsset<T>
where
    T: Asset + Clone,
{
    pub fn new<TIn: Material, TOut: ScatterMaterial<TIn>>(
        material_handle: Handle<T>,
        request: &ScatterMaterialCreationRequest<TOut, TIn>,
    ) -> Self {
        Self {
            material: material_handle,
            properties: request.properties.clone(),
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
}

impl<T: Asset + Clone> ProtoType<T> for ScatterAsset<T> {
    fn mesh(&self) -> &Handle<Mesh> {
        &self.properties.mesh_handle
    }

    fn material(&self) -> &Handle<T> {
        &self.material
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
