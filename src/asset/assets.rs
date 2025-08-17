use crate::core::{LevelOfDetail, ProtoType};
use crate::prelude::Wind;
use bevy::asset::{Asset, Handle};
use bevy::camera::primitives::Aabb;
use bevy::mesh::Mesh;
use bevy::prelude::{Name, TypePath};

#[derive(Asset, TypePath, Clone, Debug)]
pub struct ScatterAsset<T>
where
    T: Asset + Clone,
{
    pub mesh: Handle<Mesh>,
    pub material: Handle<T>,
    pub wind: Option<Wind>,
    pub aabb: Aabb,
    pub lod_level: LevelOfDetail,
    pub name: Option<Name>,
}

impl<T: Asset + Clone> ProtoType<T> for ScatterAsset<T> {
    fn mesh(&self) -> &Handle<Mesh> {
        &self.mesh
    }

    fn material(&self) -> &Handle<T> {
        &self.material
    }

    fn wind(&self) -> Option<&Wind> {
        self.wind.as_ref()
    }

    fn aabb(&self) -> &Aabb {
        &self.aabb
    }

    fn lod(&self) -> &LevelOfDetail {
        &self.lod_level
    }
}
