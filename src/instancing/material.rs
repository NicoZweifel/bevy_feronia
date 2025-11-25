use crate::prelude::*;

use bevy_asset::{Asset, AssetId, Handle};
use bevy_camera::primitives::Aabb;
use bevy_color::ColorToComponents;
use bevy_ecs::prelude::*;
use bevy_ecs::query::QueryItem;
use bevy_ecs::system::{SystemParamItem, lifetimeless::SRes};
use bevy_image::Image;
use bevy_reflect::TypePath;
use bevy_render::extract_component::ExtractComponent;
use bevy_render::render_asset::{PrepareAssetError, RenderAsset};
use bevy_render::render_resource::{AsBindGroup, AsBindGroupError, BindGroup};
use bevy_render::renderer::RenderDevice;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
#[uniform(50, WindUniform)]
pub struct InstancedWindAffectedMaterial {
    pub wind: Wind,
    pub aabb: Aabb,
    pub options: MaterialOptions,
    #[texture(51)]
    #[sampler(52)]
    pub noise_texture: Handle<Image>,
}

impl InstancedWindAffectedMaterial {
    pub fn new(properties: &ScatterAssetProperties, noise_texture: Handle<Image>) -> Self {
        Self {
            wind: properties.wind,
            aabb: properties.aabb,
            options: properties.options,
            noise_texture,
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct InstancedWindAffectedMeshMaterial(pub Handle<InstancedWindAffectedMaterial>);

impl ExtractComponent for InstancedWindAffectedMeshMaterial {
    type QueryData = &'static InstancedWindAffectedMeshMaterial;
    type QueryFilter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, '_, Self::QueryData>) -> Option<Self> {
        Some(item.clone())
    }
}

pub struct PreparedInstancedWindAffectedMaterial {
    pub bind_group: BindGroup,
}

impl RenderAsset for PreparedInstancedWindAffectedMaterial {
    type SourceAsset = InstancedWindAffectedMaterial;
    type Param = (
        SRes<RenderDevice>,
        <InstancedWindAffectedMaterial as AsBindGroup>::Param,
    );

    fn prepare_asset(
        source_asset: Self::SourceAsset,
        _asset_id: AssetId<Self::SourceAsset>,
        (render_device, param): &mut SystemParamItem<Self::Param>,
        _previous_asset: Option<&Self>,
    ) -> Result<Self, PrepareAssetError<Self::SourceAsset>> {
        match source_asset.as_bind_group(
            &InstancedWindAffectedMaterial::bind_group_layout(render_device),
            render_device,
            param,
        ) {
            Ok(x) => Ok(PreparedInstancedWindAffectedMaterial {
                bind_group: x.bind_group,
            }),
            Err(AsBindGroupError::RetryNextUpdate) => {
                Err(PrepareAssetError::RetryNextUpdate(source_asset))
            }
            Err(other) => Err(PrepareAssetError::AsBindGroupError(other)),
        }
    }
}

impl<'a> From<&'a InstancedWindAffectedMaterial> for WindUniform {
    fn from(material: &'a InstancedWindAffectedMaterial) -> Self {
        WindUniform::from(&material.wind)
            .with_edge_correction_factor(material.options.edge_correction_factor)
            .with_aabb(&material.aabb)
            .with_debug_color(material.options.debug_color.to_linear().to_vec4())
    }
}
