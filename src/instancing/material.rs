use crate::prelude::*;
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
    // Whether the material is controlled externally and isn't automatically updated by the Wind resource.
    pub controlled: bool,
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

impl<T, P> WindAffectable<T, P, StandardMaterial, InstancedWindAffectedMaterial>
    for InstancedWindAffectedMaterial
where
    T: Resource + ProtoTypes<InstancedWindAffectedMaterial, P>,
    P: ProtoType<InstancedWindAffectedMaterial> + Asset + Clone,
{
    fn create_material(
        _base: Option<StandardMaterial>,
        wind: Wind,
        noise_texture: Handle<Image>,
        controlled: bool,
    ) -> InstancedWindAffectedMaterial {
        InstancedWindAffectedMaterial {
            wind,
            noise_texture,
            controlled,
        }
    }

    fn update_material(mut materials: ResMut<Assets<InstancedWindAffectedMaterial>>, wind: Wind) {
        for (_, material) in materials.iter_mut().filter(|(_, x)| !x.controlled) {
            material.wind = wind.clone();
        }
    }

    fn component(material: Handle<InstancedWindAffectedMaterial>) -> impl Component {
        InstancedWindAffectedMeshMaterial(material)
    }
}

impl<'a> From<&'a InstancedWindAffectedMaterial> for WindUniform {
    fn from(material: &'a InstancedWindAffectedMaterial) -> Self {
        WindUniform::from(&material.wind)
    }
}
