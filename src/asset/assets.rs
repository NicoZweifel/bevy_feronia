use crate::core::*;
use crate::prelude::*;
use bevy_asset::*;
use bevy_camera::primitives::Aabb;
use bevy_ecs::prelude::*;
use bevy_mesh::Mesh;
use bevy_pbr::{Material, StandardMaterial};
use bevy_reflect::prelude::{Reflect, ReflectDefault};
use bevy_transform::prelude::Transform;
use std::fmt::Debug;

#[cfg(feature = "avian")]
use avian3d::prelude::RigidBody;

/// Shared properties for a [`ScatterAsset`] and its [`ScatterAssetPart`]s.
#[derive(Clone, Debug, Reflect, Default)]
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
    /// TODO move out of here, find way to update materials without it, e.g. a HashNap resource
    /// Deprecated
    #[deprecated]
    pub layer: Option<Entity>,
    /// Whether wind affects this asset/part.
    pub wind_affected: bool,
}

/// An [`Asset`] that represents a combined, multipart scatterable object.
///
/// Holds the final, processed asset data, including the list of
/// all its component parts and their final material handles.
#[derive(Asset, Clone, Reflect, Default, Debug)]
#[reflect(Default, Debug, Clone)]
pub struct ScatterAsset<T = StandardMaterial>
where
    T: ScatterMaterialAsset,
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
/// This typically represents one mesh+material pair from the hierarchy.
#[derive(Clone, Default, Reflect, Debug, Component)]
#[reflect(Default, Clone, Debug, Component)]
pub struct ScatterAssetPart<T = StandardMaterial>
where
    T: ScatterMaterialAsset,
{
    /// The local transform of this part relative to the asset root.
    pub transform: Transform,
    /// Properties specific to this part (inherited/modified during the collection phase).
    pub properties: ScatterAssetProperties,
    /// Handle to the material (`T`) for this part.
    pub h_material: Handle<T>,
    /// Handle to the mesh for this part.
    pub h_mesh: Handle<Mesh>,

    /// Optional name for the asset that this part belongs to.
    ///
    /// This can also be the name of the part itself if the parent had no name.
    /// Typically, this shouldn't happen unless there is exactly 1 part.
    pub name: Option<Name>,
}

#[derive(Clone, Debug, Reflect)]
#[reflect(Clone, Debug)]
pub struct ScatterAssetPartEntity<T: ScatterMaterialAsset = StandardMaterial> {
    pub entity: Entity,
    pub part: ScatterAssetPart<T>,
}

impl<T: ScatterMaterialAsset> ScatterAssetPartEntity<T> {
    pub fn new(entity: Entity, part: ScatterAssetPart<T>) -> Self {
        Self { entity, part }
    }
}

impl<T: ScatterMaterialAsset + Material> ScatterAssetPartEntity<T> {
    pub fn try_from_data(
        entity: Entity,
        item_of: AssetItemOf,
        wind: Wind,
        layer_wind_data: WindOptionData,
        scene_root_data: CollectableQueryDataItem<T>,
        item_root_data: CollectableQueryDataItem<T>,
        parent_data: CollectableQueryDataItem<T>,
        child_data: CollectableQueryDataItem<T>,
        layer_material_option_data: MaterialOptionDataItem,
        aabb: Aabb,
    ) -> Option<Self> {
        let AssetItemOf { layer, .. } = item_of;

        let wind = wind
            .with(layer_wind_data)
            .with(scene_root_data.wind_data)
            .with(child_data.wind_data);

        let options = MaterialOptions::from(layer_material_option_data)
            .with(scene_root_data.material_options)
            .with(child_data.material_options);

        let mesh = child_data.o_mesh?;
        let material = child_data.o_material?;

        // Check LOD on the child itself, then its parent, then the item root and finally the scene root.
        let lod = child_data
            .o_lod
            .or(parent_data.o_lod)
            .or(item_root_data.o_lod)
            .or(scene_root_data.o_lod)
            .cloned()
            .unwrap_or_default();

        let part_properties = ScatterAssetProperties {
            wind,    // TODO: Inherit this?
            options, // TODO: Inherit this?
            aabb,
            name: item_of.name.clone(),
            layer: Some(layer),
            lod,
            wind_affected: options.wind_affected
                || layer_material_option_data.wind_affected.is_some(),
        };

        Some(ScatterAssetPartEntity {
            entity,
            part: ScatterAssetPart::new(
                item_of.name.clone(),
                material.0.clone(),
                mesh.0.clone(),
                *child_data.transform,
                part_properties,
            ),
        })
    }
}

impl<T> ScatterAsset<T>
where
    T: ScatterMaterialAsset,
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
    T: ScatterMaterialAsset,
{
    pub fn new(
        name: Option<Name>,
        h_material: Handle<T>,
        h_mesh: Handle<Mesh>,
        transform: Transform,
        properties: ScatterAssetProperties,
    ) -> Self {
        Self {
            name,
            transform,
            h_mesh,
            h_material,
            properties,
        }
    }

    /// Returns the standard bundle for this asset part.
    pub fn bundle(&self, asset_handle: Handle<ScatterAsset<T>>, layer: Entity) -> impl Bundle {
        (
            ScatterItem,
            ScatterItemAsset::<T>(asset_handle.clone()),
            self.properties.lod,
            ChildOf(layer),
            ScatterItemOf(layer),
            ScatterLayerChildProcessed,
        )
    }

    /// Returns the wind-affected bundle for this asset part.
    pub fn wind_affected_bundle(
        &self,
        asset_handle: Handle<ScatterAsset<T>>,
        layer: Entity,
    ) -> impl Bundle {
        (WindAffected, self.bundle(asset_handle.clone(), layer))
    }

    /// Inserts the correct bundle (wind-affected or normal) onto the entity.
    pub fn insert_bundle(
        &self,
        cmd: &mut Commands,
        entity: Entity,
        asset_handle: Handle<ScatterAsset<T>>,
        layer: Entity,
    ) {
        if self.properties.wind_affected {
            cmd.entity(entity)
                .insert(self.wind_affected_bundle(asset_handle, layer));
        } else {
            cmd.entity(entity).insert(self.bundle(asset_handle, layer));
        }
    }
}

impl ScatterAssetPart<StandardMaterial> {
    pub fn into_scatter_material_part<T: ScatterMaterial + Debug>(
        self,
        materials_in: &Assets<StandardMaterial>,
        materials_out: &mut Assets<T>,
        wind_noise_texture: &WindTexture,
    ) -> ScatterAssetPart<T> {
        let ScatterAssetPart {
            h_material,
            h_mesh,
            transform,
            properties,
            name,
        } = self;

        let mut source_material = materials_in.get(&h_material).cloned();

        if properties.options.unlit {
            source_material = source_material.map(|mut m| {
                m.unlit = true;
                m
            })
        }

        let material =
            T::create_material(source_material, wind_noise_texture.0.clone(), &properties);

        let h_material = materials_out.add(material);

        ScatterAssetPart {
            transform,
            properties: properties.clone(),
            h_material,
            h_mesh: h_mesh.clone(),
            name: name.clone(),
        }
    }
}

impl<T: ScatterMaterialAsset> ProtoType<T> for ScatterAssetPart<T> {
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
