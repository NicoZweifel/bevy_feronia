use crate::prelude::*;
use bevy_eidolon::prelude::*;

use bevy_asset::Handle;
use bevy_camera::primitives::Aabb;
use bevy_ecs::prelude::*;
use bevy_image::Image;
use bevy_math::{EulerRot, Vec3, Vec4};
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
        current_wind: Wind,
        previous_wind: Wind,
        options: ScatterMaterialOptions,
    ) {
        material.current= current_wind;
        material.previous = previous_wind;
        material.options = options;
    }

    fn component(material: Handle<InstancedWindAffectedMaterial>) -> impl Component {
        InstancedMeshMaterial(material)
    }

    fn spawn(cmd: &mut Commands, request: SpawnRequest<InstancedWindAffectedMaterial>) {
        let names = request
            .get_sorted_names()
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();

        let name_set: HashSet<Name> = names
            .iter()
            .filter_map(|name| Some((name, request.name_map.get(name)?)))
            .fold(HashSet::new(), |mut acc, (name, group)| {
                let min_lod = group
                    .iter()
                    .map(|h| *h.asset.properties.lod)
                    .min()
                    .unwrap_or_default();

                if group.iter().any(|h| h.is_lod(request.is_chunked, min_lod)) {
                    acc.insert(name.clone());
                }

                acc
            });

        let groups: HashMap<Name, GroupData> = request
            .event
            .trigger
            .data
            .iter()
            .filter_map(|res| {
                (name_set.len() == 1)
                    .then(|| name_set.iter().next().map(|x| (x, res)))
                    .flatten();

                let mut rng = Pcg64::seed_from_u64(res.seed);

                let name = names.choose(&mut rng)?;

                name_set.contains(name).then_some((name, res))
            })
            .enumerate()
            .fold(HashMap::new(), |mut acc, (i, (name, res))| {
                let position = res.transform.translation;
                let (rotation, ..) = res.transform.rotation.to_euler(EulerRot::YXZ);
                let scale = res.transform.scale.x;

                let instance = InstanceData {
                    position,
                    scale,
                    rotation,
                    index: i as u32,
                    ..default()
                };

                match acc.entry(name.clone()) {
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
                };

                acc
            });

        for (name, group_data) in groups {
            let base_instances = Arc::new(group_data.instances);

            for handle_asset in request.prototypes_from_name_iter(&name) {
                let asset = &handle_asset.asset;

                let half_extents =
                    Vec3::from(asset.properties.aabb.half_extents * group_data.max_scale);
                let center_offset = Vec3::from(asset.properties.aabb.center * group_data.max_scale);

                let local_min = group_data.min_pos + center_offset - half_extents;
                let local_max = group_data.max_pos + center_offset + half_extents;

                let local_aabb = Aabb::from_min_max(local_min, local_max);

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
                                    inst.scale *= part.transform.scale.x;
                                    let (rotation, ..) =
                                        part.transform.rotation.to_euler(EulerRot::YXZ);
                                    inst.rotation += rotation;
                                    println!("{}", inst.rotation);
                                    inst
                                })
                                .collect(),
                        )
                    };

                    let entity = cmd
                        .spawn((
                            InstancedMeshMaterial(part.material().clone()),
                            Mesh3d(part.mesh().clone()),
                            InstanceMaterialData {
                                visibility_range: Vec4::new(
                                    visibility_range.start_margin.start,
                                    visibility_range.start_margin.end,
                                    visibility_range.end_margin.start,
                                    visibility_range.end_margin.end,
                                ),
                                instances,
                                color: default(),
                            },
                            NoAutomaticBatching,
                            ScatteredInstance(request.event.trigger.layer),
                            ScatteredAsset(handle_asset.handle.clone()),
                        ))
                        .id();

                    cmd.entity(entity).insert((
                        Transform::default(),
                        local_aabb,
                        ChildOf(request.parent),
                    ));

                    if asset.properties.wind_affected {
                        cmd.entity(entity).insert(WindAffected);
                    }

                    if asset.properties.options.gpu_cull {
                        cmd.entity(entity).insert(GpuCullCompute);
                    }
                }
            }
        }
    }
}
