use crate::prelude::*;
use bevy::camera::primitives::Aabb;
use bevy::ecs::system::{SystemParamItem, lifetimeless::SRes};
use bevy::platform::collections::HashMap;
use bevy::render::batching::NoAutomaticBatching;
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
use rand::SeedableRng;
use rand::prelude::IndexedRandom;
use rand_pcg::Pcg64;

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

impl ScatterMaterial for InstancedWindAffectedMaterial {
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

    fn spawn(
        mut cmd: Commands,
        mut mr_spawn: MessageReader<SpawnProtoTypes<InstancedWindAffectedMaterial>>,
        prototype_assets: Res<Assets<ScatterAsset<InstancedWindAffectedMaterial>>>,
        q_chunks: Query<(&GlobalTransform, &ChunkLevel), (With<Chunk>, Without<Merging>)>,
        q_root: Query<&LodConfig, With<ScatterRoot>>,
        q_layers: Query<(), With<ScatterChunked>>,
    ) {
        for event in mr_spawn.read() {
            debug!("Spawning instanced wind affected!");

            let (chunk_gtf, chunk_level) = event
                .trigger
                .chunk
                .map(|x| q_chunks.get(x).ok())
                .flatten()
                .map(|(gtf, level)| (*gtf, level.clone()))
                .unwrap_or_default();

            let is_chunked =
                event.trigger.chunk.is_some() && q_layers.get(event.trigger.layer).is_ok();

            let prototypes: Vec<_> = event
                .items
                .iter()
                .filter_map(|h| prototype_assets.get(&**h))
                .collect();

            let mut name_map: HashMap<Name, Vec<&ScatterAsset<InstancedWindAffectedMaterial>>> =
                HashMap::new();

            prototypes.iter().for_each(|p| {
                let name = p.name.clone().unwrap_or_else(|| Name::new(""));
                name_map.entry(name).or_default().push(*p);
            });

            if name_map.is_empty() {
                continue;
            }

            let mut sorted_names: Vec<&Name> = name_map.keys().collect();
            sorted_names.sort();

            let mut instance_groups: HashMap<Name, Vec<InstanceData>> = HashMap::new();

            for (i, res) in event.trigger.data.iter().enumerate() {
                let mut rng = Pcg64::seed_from_u64(res.seed);
                let Some(chosen_name) = sorted_names.choose(&mut rng) else {
                    continue;
                };

                let target_lod_level = if is_chunked {
                    *chunk_level
                } else {
                    name_map
                        .get(*chosen_name)
                        .and_then(|group| group.iter().map(|p| *p.lod_level).min())
                        .unwrap_or_default()
                };

                if name_map
                    .get(*chosen_name)
                    .and_then(|g| g.iter().find(|p| *p.lod_level == target_lod_level))
                    .is_some()
                {
                    let instance_data = InstanceData {
                        position: res.transform.translation,
                        scale: res.transform.scale.element_sum() / 3.0,
                        index: i as u32,
                        ..default()
                    };

                    instance_groups
                        .entry((*chosen_name).clone())
                        .or_default()
                        .push(instance_data);
                }
            }

            let Ok(lod_config) = q_root.get(event.trigger.root) else {
                warn!("Couldn't get ScatterRoot!");
                continue;
            };

            for (name, instances) in instance_groups {
                let target_lod = if is_chunked {
                    *chunk_level
                } else {
                    name_map
                        .get(&name)
                        .unwrap()
                        .iter()
                        .map(|p| *p.lod_level)
                        .min()
                        .unwrap_or_default()
                };

                let prototype = name_map
                    .get(&name)
                    .unwrap()
                    .iter()
                    .find(|p| *p.lod_level == target_lod)
                    .unwrap();

                let mesh_handle = prototype.mesh().clone();
                let (mut min_point, mut max_point) = (Vec3::MAX, Vec3::MIN);

                let visibility_range = lod_config.get_visibility_range(prototype.lod_level);

                let instances_with_offset = instances
                    .iter()
                    .map(|instance| {
                        let mut instance = *instance;
                        instance.position += chunk_gtf.translation();

                        let instance_min =
                            instance.position + Vec3::from(prototype.aabb().min() * instance.scale);
                        let instance_max =
                            instance.position + Vec3::from(prototype.aabb().max() * instance.scale);
                        min_point = min_point.min(instance_min);
                        max_point = max_point.max(instance_max);

                        instance
                    })
                    .collect::<Vec<_>>();

                let entity = cmd
                    .spawn((
                        InstancedWindAffectedMeshMaterial(prototype.material().clone()),
                        Mesh3d(mesh_handle),
                        InstanceMaterialData {
                            color: LinearRgba::from(Color::hsla(78., 0.98, 0.5, 1.0))
                                .to_f32_array(),
                            visibility_range: [
                                visibility_range.start_margin.start,
                                visibility_range.start_margin.end,
                                visibility_range.end_margin.start,
                                visibility_range.end_margin.end,
                            ],
                            instances: instances_with_offset,
                        },
                        NoAutomaticBatching,
                        WindAffected,
                        WindAffectedReady,
                        ScatteredInstance(event.trigger.layer),
                    ))
                    .id();

                let local_aabb = Aabb::from_min_max(
                    min_point - chunk_gtf.translation(),
                    max_point - chunk_gtf.translation(),
                );
                let parent = event.trigger.chunk.unwrap_or(event.trigger.layer);

                cmd.entity(entity).insert((
                    Transform::default(),
                    Visibility::Visible,
                    local_aabb,
                    ChildOf(parent),
                ));
            }
        }
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
