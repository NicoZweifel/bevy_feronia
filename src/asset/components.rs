use crate::prelude::*;
use bevy::asset::Handle;
use bevy::pbr::Material;
use bevy::prelude::*;
use std::marker::PhantomData;

// Decouples the "read" phase (collection) with the "write" phase (processing).
#[derive(Component, Clone)]
pub struct ScatterMaterialCreationRequest<TOut, TIn>
where
    TIn: Material,
    TOut: ScatterMaterial<TIn>,
{
    pub source_material_handle: Option<Handle<TIn>>,
    pub properties: ScatterAssetProperties,
    pub _phantom: PhantomData<TOut>,
}

impl<TOut, TIn> ScatterMaterialCreationRequest<TOut, TIn>
where
    TIn: Material,
    TOut: ScatterMaterial<TIn>,
{
    pub fn new(
        source_material_handle: Option<Handle<TIn>>,
        properties: ScatterAssetProperties,
    ) -> Self {
        Self {
            source_material_handle,
            properties,
            _phantom: PhantomData,
        }
    }
}
