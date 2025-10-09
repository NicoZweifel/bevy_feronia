use crate::core::*;
use crate::prelude::Wind;
use bevy::asset::{Asset, Handle};
use bevy::camera::primitives::Aabb;
use bevy::mesh::Mesh;
use bevy::prelude::*;

#[derive(Asset, TypePath, Clone, Debug)]
pub struct ScatterAsset<T>
where
    T: Asset + Clone,
{
    pub mesh: Handle<Mesh>,
    pub material: Handle<T>,
    pub wind: Wind,
    pub material_options: MaterialOptions,
    pub aabb: Aabb,
    pub lod_level: LevelOfDetail,
    pub name: Option<Name>,
    pub layer: Entity,
}

impl<T: Asset + Clone> ProtoType<T> for ScatterAsset<T> {
    fn mesh(&self) -> &Handle<Mesh> {
        &self.mesh
    }

    fn material(&self) -> &Handle<T> {
        &self.material
    }

    fn wind(&self) -> &Wind {
        &self.wind
    }

    fn aabb(&self) -> &Aabb {
        &self.aabb
    }

    fn lod(&self) -> &LevelOfDetail {
        &self.lod_level
    }

    fn material_options(&self) -> &MaterialOptions {
        &self.material_options
    }
}
