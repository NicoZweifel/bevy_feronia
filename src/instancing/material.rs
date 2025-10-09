use crate::prelude::*;
use bevy::camera::primitives::Aabb;
use bevy::ecs::system::{SystemParamItem, lifetimeless::SRes};
use bevy::{
    ecs::query::QueryItem,
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_asset::{PrepareAssetError, RenderAsset},
        render_resource::*,
        renderer::RenderDevice,
    },
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[uniform(50, WindUniform)]
pub struct InstancedWindAffectedMaterial {
    pub wind: Wind,
    pub aabb: Aabb,
    pub options: MaterialOptions,

    #[texture(51)]
    #[sampler(52)]
    pub noise_texture: Handle<Image>,
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

pub(crate) struct PreparedInstancedWindAffectedMaterial {
    pub(crate) bind_group: BindGroup,
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

impl<T> WindAffectable<T, StandardMaterial, InstancedWindAffectedMaterial>
    for InstancedWindAffectedMaterial
where
    T: ProtoType<InstancedWindAffectedMaterial> + Asset + Clone,
{
    fn create_material(
        _base: Option<StandardMaterial>,
        wind: Wind,
        noise_texture: Handle<Image>,
        aabb: Aabb,
        options: MaterialOptions,
    ) -> InstancedWindAffectedMaterial {
        InstancedWindAffectedMaterial {
            wind,
            noise_texture,
            aabb,
            options,
        }
    }

    fn update_material(
        material: &mut InstancedWindAffectedMaterial,
        wind: Wind,
        options: MaterialOptions,
    ) {
        material.wind = wind;
        material.options = options;
    }

    fn component(material: Handle<InstancedWindAffectedMaterial>) -> impl Component {
        InstancedWindAffectedMeshMaterial(material)
    }
}

impl<'a> From<&'a InstancedWindAffectedMaterial> for WindUniform {
    fn from(material: &'a InstancedWindAffectedMaterial) -> Self {
        WindUniform::from(&material.wind)
            .with_lod_threshold(material.options.lod_threshold)
            .with_curve_factor(material.options.curve_factor)
            .with_edge_correction_factor(material.options.edge_correction_factor)
            .with_aabb(&material.aabb)
            .with_debug_color(material.options.debug_color.to_linear().to_vec4())
    }
}
