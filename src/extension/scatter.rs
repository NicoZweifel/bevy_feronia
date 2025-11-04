use crate::core::events::SpawnProtoTypes;
use crate::prelude::*;
use bevy::pbr::ExtendedMaterial;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
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
        ScatterLayerType::<ExtendedWindAffectedMaterial>::default(),
    )
}

impl ScatterMaterial for ExtendedWindAffectedMaterial {
    fn create_material(
        base: Option<StandardMaterial>,
        noise_texture: Handle<Image>,
        properties: &ScatterAssetProperties,
    ) -> ExtendedWindAffectedMaterial {
        ExtendedMaterial {
            base: base.unwrap_or_default(),
            extension: WindAffectedExtension::new(properties, noise_texture),
        }
    }

    fn update_material(
        material: &mut ExtendedWindAffectedMaterial,
        wind: Wind,
        options: MaterialOptions,
    ) {
        let ext = &mut material.extension;
        ext.wind = wind;
        ext.options = options;
    }

    fn component(material: Handle<ExtendedWindAffectedMaterial>) -> impl Component {
        MeshMaterial3d(material)
    }

    fn spawn(
        mut cmd: Commands,
        mut mr_spawn: MessageReader<SpawnProtoTypes<ExtendedWindAffectedMaterial>>,
        prototype_assets: Res<Assets<ScatterAsset<ExtendedWindAffectedMaterial>>>,
        q_chunks: Query<(&GlobalTransform, &ChunkLevel), (With<Chunk>, Without<Merging>)>,
        q_root: Query<&LodConfig, With<ScatterRoot>>,
        q_layers: Query<(), With<ScatterChunked>>,
    ) {
        for event in mr_spawn.read() {
            debug!("Spawning standard!");

            let chunk_level = event
                .trigger
                .chunk
                .and_then(|x| q_chunks.get(x).map(|(_, lvl)| lvl).ok())
                .cloned()
                .unwrap_or_default();

            let is_chunked =
                event.trigger.chunk.is_some() && q_layers.get(event.trigger.layer).is_ok();

            let prototypes: Vec<_> = event
                .items
                .iter()
                .filter_map(|h| prototype_assets.get(&**h))
                .collect();

            let mut name_map: HashMap<Name, Vec<&ScatterAsset<_>>> = HashMap::new();

            prototypes.iter().for_each(|p| {
                let name = p.properties.name.clone().unwrap_or_else(|| Name::new(""));
                name_map.entry(name).or_default().push(*p);
            });

            if name_map.is_empty() {
                continue;
            }

            let mut sorted_names: Vec<&Name> = name_map.keys().collect();
            sorted_names.sort();

            let parent = event.trigger.chunk.unwrap_or(event.trigger.layer);

            let Ok(lod_config) = q_root.get(event.trigger.root) else {
                warn!("Couldn't get ScatterRoot!");
                continue;
            };

            cmd.spawn_batch(
                event
                    .trigger
                    .data
                    .iter()
                    .flat_map(|res| {
                        let mut rng = Pcg64::seed_from_u64(res.seed);

                        let Some(chosen_name) = sorted_names.choose(&mut rng) else {
                            return vec![];
                        };

                        let Some(prototypes_to_spawn) = name_map.get(*chosen_name) else {
                            return vec![];
                        };

                        prototypes_to_spawn
                            .iter()
                            .filter(|p| {
                                if is_chunked {
                                    *p.properties.lod == *chunk_level
                                } else {
                                    *p.properties.lod >= *chunk_level
                                }
                            })
                            .map(move |prototype| {
                                let visibility_range =
                                    lod_config.get_visibility_range(prototype.properties.lod);
                                (
                                    res.transform,
                                    Mesh3d(prototype.mesh().clone()),
                                    MeshMaterial3d(prototype.material().clone()),
                                    WindAffected,
                                    WindAffectedReady,
                                    ChildOf(parent),
                                    visibility_range,
                                    ScatteredInstance(event.trigger.layer),
                                )
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>(),
            );
        }
    }
}
