use crate::core::events::SpawnProtoTypes;
use crate::prelude::*;
use bevy::camera::primitives::Aabb;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::render::batching::NoAutomaticBatching;
use rand::SeedableRng;
use rand::prelude::IndexedRandom;
use rand_pcg::Pcg64;
use std::borrow::Cow;

pub fn scatter_layer(name: impl Into<Cow<'static, str>>) -> impl Bundle
where
{
    (
        Name::new(name),
        ScatterLayer::default(),
        ScatterLayerType::<InstancedWindAffectedMaterial>::default(),
    )
}

impl ScatterMaterial for InstancedWindAffectedMaterial {
    fn create_material(
        _base: Option<StandardMaterial>,
        noise_texture: Handle<Image>,
        properties: &ScatterAssetProperties,
    ) -> InstancedWindAffectedMaterial {
        InstancedWindAffectedMaterial::new(properties, noise_texture)
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
                .and_then(|x| q_chunks.get(x).ok())
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
                let name = p.properties.name.clone().unwrap_or_else(|| Name::new(""));
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

                let min_lod = name_map
                    .get(*chosen_name)
                    .and_then(|group| group.iter().map(|p| *p.properties.lod).min())
                    .unwrap_or_default();

                if name_map
                    .get(*chosen_name)
                    .and_then(|g| {
                        g.iter().find(|p| {
                            if is_chunked {
                                *p.properties.lod == min_lod
                            } else {
                                *p.properties.lod >= min_lod
                            }
                        })
                    })
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
                let target_lod = *chunk_level;

                let prototypes = name_map.get(&name).unwrap().iter().filter(|p| {
                    if is_chunked {
                        *p.properties.lod == target_lod
                    } else {
                        *p.properties.lod >= target_lod
                    }
                });

                for prototype in prototypes {
                    let mesh_handle = prototype.mesh().clone();
                    let (mut min_point, mut max_point) = (Vec3::MAX, Vec3::MIN);

                    let visibility_range =
                        lod_config.get_visibility_range(prototype.properties.lod);

                    let instances_with_offset = instances
                        .iter()
                        .map(|instance| {
                            let mut instance = *instance;
                            instance.position += chunk_gtf.translation();

                            let instance_min = instance.position
                                + Vec3::from(prototype.aabb().min() * instance.scale);
                            let instance_max = instance.position
                                + Vec3::from(prototype.aabb().max() * instance.scale);

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
                                color: LinearRgba::from(
                                    prototype
                                        .properties
                                        .options
                                        .color
                                        .unwrap_or(Color::hsla(106., 0.37, 0.37, 1.0)),
                                )
                                .to_f32_array(),
                                visibility_range: [
                                    visibility_range.start_margin.start,
                                    visibility_range.start_margin.end,
                                    visibility_range.end_margin.start,
                                    visibility_range.end_margin.end,
                                ],
                                instances: instances_with_offset,
                                static_bend_strength: prototype
                                    .properties
                                    .options
                                    .static_bend_strength,
                                curve_factor: prototype.properties.options.curve_factor,
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
}
