use crate::prelude::*;
use crate::scatter::events::ScatterResults;
use bevy::camera::primitives::Aabb;
use bevy::camera::visibility::VisibilityRange;
use bevy::ecs::system::{SystemParamItem, lifetimeless::SRes};
use bevy::render::batching::NoAutomaticBatching;
use bevy::{
    asset::*,
    ecs::query::QueryItem,
    pbr::StandardMaterial,
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_asset::{PrepareAssetError, RenderAsset},
        render_resource::*,
        renderer::RenderDevice,
    },
};
use rand::prelude::IndexedRandom;

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

impl
    WindAffectable<
        StandardMaterial,
        InstancedWindAffectedMaterial,
        WindAffectedTypes<InstancedWindAffectedMaterial>,
        WindAffectedType<InstancedWindAffectedMaterial>,
    > for InstancedWindAffectedMaterial
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

    fn spawn(
        mut cmd: Commands,
        trigger: On<ScatterResults>,
        prototypes: &WindAffectedTypes<InstancedWindAffectedMaterial>,
        q_chunks: Query<(&GlobalTransform, &ChunkOf, &ChunkLevel), With<Chunk>>,
        q_chunk_config: Query<(&ChunkLodConfig, &Aabb), With<ChunkRoot>>,
    ) {
        let mut rng = rand::rng();
        let prototype = prototypes.values().choose(&mut rng).unwrap();

        let instances = trigger
            .results
            .iter()
            .enumerate()
            .map(|(i, res)| InstanceData {
                position: res.global_transform.translation,
                scale: res.global_transform.scale.element_sum() / 3.0,
                color: LinearRgba::from(Color::hsla(78., 0.98, 0.5, 1.0)).to_f32_array(),
                index: i as u32,
            })
            .collect::<Vec<_>>();

        let mesh_handle = prototype.mesh.clone();
        let (mut min_point, mut max_point) = (Vec3::MAX, Vec3::MIN);

        for instance in &instances {
            let instance_min =
                instance.position + Vec3::from(prototype.aabb.min() * instance.scale);
            let instance_max =
                instance.position + Vec3::from(prototype.aabb.max() * instance.scale);
            min_point = min_point.min(instance_min);
            max_point = max_point.max(instance_max);
        }

        let entity = cmd
            .spawn((
                InstancedWindAffectedMaterial::component(prototype.material.clone()),
                Mesh3d(mesh_handle),
                InstanceMaterialData(instances),
                NoAutomaticBatching,
                WindAffected,
                WindAffectedReady,
            ))
            .id();

        let (chunk_gtf, chunk_root, chunk_level) = match trigger.chunk {
            None => (Transform::default(), None, &ChunkLevel::default()),
            Some(x) => match q_chunks.get(x) {
                Ok((chunk_gtf, chunk_root, chunk_level)) => {
                    (chunk_gtf.compute_transform(), Some(chunk_root), chunk_level)
                }
                Err(_) => (Transform::default(), None, &ChunkLevel::default()),
            },
        };

        let chunk_config = match chunk_root {
            None => None,
            Some(x) => Some(q_chunk_config.get(**x).unwrap()),
        };

        let lod_level = **chunk_level as usize;
        let current_lod_config = match chunk_config {
            None => &LodConfig::default(),
            Some((x, _)) => &(**x)[lod_level],
        };

        let current_lod_dist = current_lod_config.distance;

        // TODO expose
        const FADE_BAND: f32 = 2.0;

        if let Some((chunk_config, aabb)) = chunk_config {
            let start_margin = if lod_level == 0 {
                0.0..0.0
            } else {
                let prev_lod_dist = (**chunk_config)[lod_level - 1].distance;
                prev_lod_dist - FADE_BAND..prev_lod_dist
            };

            let end_margin = if lod_level as u32 == chunk_config.get_max_lod_level() {
                f32::MAX..f32::MAX
            } else {
                current_lod_dist - FADE_BAND..current_lod_dist
            };

            let chunk_center = chunk_gtf.translation;

            let local_min = min_point - chunk_center;
            let local_max = max_point - chunk_center;

            let local_aabb = Aabb::from_min_max(local_min, local_max);

            cmd.entity(entity).insert((
                Aabb::from(local_aabb),
                VisibilityRange {
                    start_margin,
                    end_margin,
                    use_aabb: false,
                },
                ChildOf(trigger.chunk.unwrap()),
            ));
        } else {
            cmd.entity(entity).insert(ChildOf(trigger.target()));
        };
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
