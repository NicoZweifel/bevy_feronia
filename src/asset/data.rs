use crate::core::{LevelOfDetail, MaterialOptionData};
use crate::prelude::{WindAffected, WindConfig, WindOptionData};
use bevy_camera::primitives::Aabb;
use bevy_ecs::prelude::*;
use bevy_ecs::query::QueryData;
use bevy_mesh::Mesh3d;
use bevy_pbr::{Material, MeshMaterial3d, StandardMaterial};
use bevy_transform::prelude::Transform;

#[cfg(feature = "avian")]
use avian3d::prelude::{Collider, RigidBody};

#[derive(QueryData)]
#[query_data()]
pub struct CollectableQueryData<T: Material = StandardMaterial> {
    pub entity: Entity,
    pub transform: &'static Transform,
    pub o_material: Option<&'static MeshMaterial3d<T>>,
    pub o_mesh: Option<&'static Mesh3d>,
    pub o_aabb: Option<&'static Aabb>,
    pub o_children: Option<&'static Children>,
    pub o_wind_config: Option<&'static WindConfig>,
    pub o_name: Option<&'static Name>,
    pub o_lod: Option<&'static LevelOfDetail>,
    pub o_wind_affected: Option<&'static WindAffected>,

    #[cfg(feature = "avian")]
    pub o_rigid_body: Option<&'static RigidBody>,
    #[cfg(feature = "avian")]
    pub o_collider: Option<&'static Collider>,

    pub material_options: MaterialOptionData,
    pub wind_data: WindOptionData<'static>,
}
