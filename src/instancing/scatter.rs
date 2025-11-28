use crate::prelude::*;
use bevy_asset::Handle;
use bevy_camera::{primitives::Aabb, visibility::Visibility};
use bevy_color::{Color, LinearRgba};
use bevy_ecs::prelude::*;
use bevy_image::Image;
use bevy_math::{Vec3, Vec4};
use bevy_mesh::Mesh3d;
use bevy_pbr::StandardMaterial;
use bevy_platform::collections::{HashMap, HashSet, hash_map::Entry};
use bevy_render::batching::NoAutomaticBatching;
use bevy_transform::prelude::Transform;
use bevy_utils::default;
use rand::SeedableRng;
use rand::prelude::IndexedRandom;
use rand_pcg::Pcg64;
use std::borrow::Cow;
use std::sync::Arc;

pub fn scatter_layer(name: impl Into<Cow<'static, str>>) -> impl Bundle
where
{
    (
        Name::new(name),
        ScatterLayer::default(),
        ScatterLayerType::<InstancedWindAffectedMaterial>::default(),
    )
}

struct GroupData {
    instances: Vec<InstanceData>,
    min_pos: Vec3,
    max_pos: Vec3,
    max_scale: f32,
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
        options: ScatterMaterialOptions,
    ) {
        material.wind = wind;
        material.options = options;
    }

    fn component(material: Handle<InstancedWindAffectedMaterial>) -> impl Component {
        InstancedWindAffectedMeshMaterial(material)
    }

    fn spawn(cmd: &mut Commands, request: SpawnRequest<InstancedWindAffectedMaterial>) {
        let mut valid_names: HashSet<Name> = HashSet::default();

        for name in &request.names {
            let Some(group) = request.name_map.get(name) else {
                continue;
            };

            let min_lod = group
                .iter()
                .map(|h| *h.asset.properties.lod)
                .min()
                .unwrap_or_default();

            if group.iter().any(|h| h.is_lod(request.is_chunked, min_lod)) {
                valid_names.insert(name.clone());
            }
        }

        let mut groups: HashMap<Name, GroupData> = HashMap::new();

        for (i, res) in request.event.trigger.data.iter().enumerate() {
            let mut rng = Pcg64::seed_from_u64(res.seed);

            let Some(name) = request.names.choose(&mut rng) else {
                continue;
            };

            if !valid_names.contains(name) {
                continue;
            }

            let position = res.transform.translation + request.chunk_gtf_translation;
            let scale = res.transform.scale.element_sum() / 3.0;

            let instance = InstanceData {
                position,
                scale,
                index: i as u32,
                ..default()
            };

            match groups.entry((*name).clone()) {
                Entry::Occupied(mut e) => {
                    let g = e.get_mut();
                    g.instances.push(instance);
                    g.min_pos = g.min_pos.min(position);
                    g.max_pos = g.max_pos.max(position);
                    g.max_scale = g.max_scale.max(scale);
                }
                Entry::Vacant(e) => {
                    e.insert(GroupData {
                        instances: vec![instance],
                        min_pos: position,
                        max_pos: position,
                        max_scale: scale,
                    });
                }
            }
        }

        for (name, group_data) in groups {
            for handle_asset in request.prototypes_from_name_iter(&name) {
                let asset = &handle_asset.asset;

                let half_extents =
                    Vec3::from(asset.properties.aabb.half_extents * group_data.max_scale);
                let center_offset = Vec3::from(asset.properties.aabb.center * group_data.max_scale);

                let world_min = group_data.min_pos + center_offset - half_extents;
                let world_max = group_data.max_pos + center_offset + half_extents;

                let local_aabb = Aabb::from_min_max(
                    world_min - request.chunk_gtf_translation,
                    world_max - request.chunk_gtf_translation,
                );

                let base_instances = Arc::new(group_data.instances.clone());

                let visibility_range = request
                    .lod_config
                    .get_visibility_range(asset.properties.lod);

                for part in asset.parts.iter() {
                    let instances = if part.transform == Transform::default() {
                        base_instances.clone()
                    } else {
                        Arc::new(
                            base_instances
                                .iter()
                                .map(|original| {
                                    let mut inst = *original;
                                    inst.position += part.transform.translation * inst.scale;
                                    inst.scale *= part.transform.scale.element_sum() / 3.0;
                                    inst
                                })
                                .collect(),
                        )
                    };

                    let entity = cmd
                        .spawn((
                            InstancedWindAffectedMeshMaterial(part.material().clone()),
                            Mesh3d(part.mesh().clone()),
                            InstanceMaterialData {
                                specular_power: asset.properties.options.specular_power,
                                specular_strength: asset.properties.options.specular_strength,
                                translucency: asset.properties.options.translucency,
                                top_color: LinearRgba::from(
                                    asset
                                        .properties
                                        .options
                                        .top_color
                                        .unwrap_or(Color::hsla(106., 0.37, 0.37, 1.0)),
                                ),
                                bottom_color: LinearRgba::from(
                                    asset
                                        .properties
                                        .options
                                        .bottom_color
                                        .unwrap_or(Color::hsla(105., 0.54, 0.37, 1.0)),
                                ),
                                visibility_range: Vec4::new(
                                    visibility_range.start_margin.start,
                                    visibility_range.start_margin.end,
                                    visibility_range.end_margin.start,
                                    visibility_range.end_margin.end,
                                ),
                                instances,
                                static_bend_strength: asset.properties.options.static_bend_strength,
                                curve_factor: asset.properties.options.curve_factor,
                            },
                            NoAutomaticBatching,
                            ScatteredInstance(request.event.trigger.layer),
                            ScatteredAsset(handle_asset.handle.clone()),
                        ))
                        .id();

                    cmd.entity(entity).insert((
                        Transform::default(),
                        Visibility::Visible,
                        local_aabb,
                        ChildOf(request.parent),
                    ));

                    if asset.properties.wind_affected {
                        cmd.entity(entity).insert(WindAffected);
                    }

                    if asset.properties.options.gpu_cull {
                        cmd.entity(entity).insert(GpuCull);
                    }
                }
            }
        }
    }
}
