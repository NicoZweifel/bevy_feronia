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

    fn spawn(cmd: &mut Commands, request: SpawnRequest<InstancedWindAffectedMaterial>) {
        let instance_groups: HashMap<Name, Vec<InstanceData>> =
            request.event.trigger.data.iter().enumerate().fold(
                HashMap::new(),
                |mut map, (i, res)| {
                    let mut rng = Pcg64::seed_from_u64(res.seed);
                    let Some(name) = request.names.choose(&mut rng) else {
                        return map;
                    };

                    let min_lod = request
                        .name_map
                        .get(name)
                        .and_then(|group| {
                            group
                                .iter()
                                .map(|ScatterHandleAsset { asset, .. }| *asset.properties.lod)
                                .min()
                        })
                        .unwrap_or_default();

                    if request
                        .name_map
                        .get(name)
                        .and_then(|g| {
                            g.iter().find(|handle_asset| {
                                handle_asset.is_lod(request.is_chunked, min_lod)
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

                        map.entry((*name).clone()).or_default().push(instance_data);
                    }

                    map
                },
            );

        for (ScatterHandleAsset { handle, asset }, instances) in
            instance_groups.iter().flat_map(|(name, instances)| {
                request
                    .prototypes_from_name_iter(name)
                    .map(move |handle_asset| (handle_asset, instances))
            })
        {
            let (mut min_point, mut max_point) = (Vec3::MAX, Vec3::MIN);

            let visibility_range = request
                .lod_config
                .get_visibility_range(asset.properties.lod);

            let instances_with_offset = instances
                .iter()
                .map(|instance| {
                    let mut instance = *instance;
                    instance.position += request.chunk_gtf_translation;

                    let instance_min = instance.position
                        + Vec3::from(asset.properties.aabb.min() * instance.scale);
                    let instance_max = instance.position
                        + Vec3::from(asset.properties.aabb.max() * instance.scale);

                    min_point = min_point.min(instance_min);
                    max_point = max_point.max(instance_max);

                    instance
                })
                .collect::<Vec<_>>();

            let local_aabb = Aabb::from_min_max(
                min_point - request.chunk_gtf_translation,
                max_point - request.chunk_gtf_translation,
            );

            let parent = request
                .event
                .trigger
                .chunk
                .unwrap_or(request.event.trigger.layer);

            for part in &asset.parts {
                let mesh_handle = part.mesh().clone();
                let entity = cmd
                    .spawn((
                        InstancedWindAffectedMeshMaterial(part.material().clone()),
                        Mesh3d(mesh_handle),
                        InstanceMaterialData {
                            color: LinearRgba::from(
                                asset
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
                            instances: instances_with_offset
                                .clone()
                                .into_iter()
                                .map(|mut instance| {
                                    instance.position += part.transform.translation;
                                    instance.scale *= part.transform.scale.element_sum() / 3.0;
                                    instance
                                })
                                .collect(),
                            static_bend_strength: asset.properties.options.static_bend_strength,
                            curve_factor: asset.properties.options.curve_factor,
                        },
                        NoAutomaticBatching,
                        WindAffected,
                        ScatteredInstance(request.event.trigger.layer),
                        ScatteredAsset(handle.clone()),
                    ))
                    .id();

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
