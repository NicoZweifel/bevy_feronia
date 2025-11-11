use crate::prelude::*;
use bevy::pbr::Material;
use bevy::prelude::*;
use std::marker::PhantomData;

#[cfg(feature = "avian")]
use avian3d::prelude::RigidBody;

#[derive(Component, Clone)]
pub struct ScatterAssetCreationRequest<TOut, TIn>
where
    TIn: Material,
    TOut: ScatterMaterial<TIn>,
{
    pub properties: ScatterAssetProperties,

    pub parts: Vec<ScatterAssetPart<TIn>>,

    #[cfg(feature = "avian")]
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
        #[cfg(feature = "avian")]
        rigid_body: Option<RigidBody>,
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
