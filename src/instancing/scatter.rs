use crate::prelude::*;
use bevy_eidolon::prelude::*;

use bevy_asset::Handle;
use bevy_camera::primitives::Aabb;
use bevy_ecs::prelude::*;
use bevy_image::Image;
use bevy_math::{EulerRot, Vec3};
use bevy_mesh::Mesh3d;
use bevy_pbr::StandardMaterial;
use bevy_platform::collections::{HashMap, HashSet, hash_map::Entry};
use bevy_render::batching::NoAutomaticBatching;
use bevy_utils::default;

use bevy_camera::visibility::NoAutoAabb;
use rand::prelude::IndexedRandom;
use rand::{RngCore, SeedableRng};
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
    ) -> Self {
        Self::new(properties, noise_texture)
    }

    fn update_material(
        material: &mut Self,
        current_wind: Wind,
        previous_wind: Wind,
        options: ScatterMaterialOptions,
    ) {
        if material.current != current_wind
            || material.previous != previous_wind
            || material.options != options
        {
            material.current = current_wind;
            material.previous = previous_wind;
            material.options = options;
        }
    }

    fn component(material: Handle<Self>) -> impl Component {
        InstancedMeshMaterial(material)
    }

    fn spawn(cmd: &mut Commands, request: SpawnRequest<Self>) {
        let names = request.get_sorted_names();
        let name_set: HashSet<&Name> = names
            .iter()
            .filter_map(|name| {
                let group = request.name_map.get(*name)?;
                let min_lod = group
                    .iter()
                    .map(|h| *h.asset.properties.lod)
                    .min()
                    .unwrap_or_default();

                group
                    .iter()
                    .any(|h| h.is_lod(request.is_chunked, min_lod))
                    .then_some(*name)
            })
            .collect();

        if name_set.is_empty() {
            return;
        }

        let mut groups: HashMap<&Name, GroupData> = HashMap::with_capacity(names.len());
        let count = request.event.trigger.data.len();

        for (i, res) in request.event.trigger.data.iter().enumerate() {
            let name = if name_set.len() == 1 {
                *name_set.iter().next().unwrap()
            } else {
                let mut rng = Pcg64::seed_from_u64(res.seed);
                match names.choose(&mut rng) {
                    Some(n) if name_set.contains(n) => n,
                    _ => continue,
                }
            };

            let mut rng = Pcg64::seed_from_u64(res.seed);

            let position = res.transform.translation;
            let (rotation, ..) = res.transform.rotation.to_euler(EulerRot::YXZ);
            let scale = res.transform.scale.x;
            let rnd_height = rng.next_u32() & 0xFFFF;
            let rnd_bend = rng.next_u32() & 0xFFFF;
            let packed_seed = (rnd_bend << 16) | rnd_height;
            let instance = InstanceData {
                position,
                scale,
                rotation,
                index: i as u32,
                seed: packed_seed,
                ..default()
            };

            match groups.entry(name) {
                Entry::Occupied(mut e) => {
                    let g = e.get_mut();
                    g.instances.push(instance);
                    g.min_pos = g.min_pos.min(position);
                    g.max_pos = g.max_pos.max(position);
                    g.max_scale = g.max_scale.max(scale);
                }
                Entry::Vacant(e) => {
                    let capacity = count / name_set.len();
                    let mut instances = Vec::with_capacity(capacity.max(16));
                    instances.push(instance);

                    e.insert(GroupData {
                        instances,
                        min_pos: position,
                        max_pos: position,
                        max_scale: scale,
                    });
                }
            };
        }

        for (name, group_data) in groups {
            let base_instances = Arc::new(group_data.instances);

            for handle_asset in request.prototypes_from_name_iter(name) {
                let asset = &handle_asset.asset;
                let half_extents =
                    Vec3::from(asset.properties.aabb.half_extents * group_data.max_scale);

                let center = Vec3::from(asset.properties.aabb.center * group_data.max_scale);
                let min = group_data.min_pos + center - half_extents;
                let max = group_data.max_pos + center + half_extents;
                let aabb = Aabb::from_min_max(min, max);
                let visibility_range = request
                    .lod_config
                    .get_visibility_range(asset.properties.lod)
                    .as_vec4();

                for part in asset.parts.iter() {
                    let instances = base_instances.clone();
                    let entity = cmd
                        .spawn((
                            Self::component(part.material().clone()),
                            Mesh3d(part.mesh().clone()),
                            InstanceMaterialData {
                                visibility_range,
                                instances,
                                color: asset
                                    .properties
                                    .options
                                    .color
                                    .base_color
                                    .unwrap_or_default()
                                    .to_linear(),
                            },
                            NoAutomaticBatching,
                            ScatteredInstance(request.event.trigger.layer),
                            ScatteredAsset(handle_asset.handle.clone()),
                            part.properties.options.render_layers.clone(),
                        ))
                        .id();

                    cmd.entity(entity).insert((
                        part.transform,
                        aabb,
                        // Required since bevy 0.18 if adding Aabb manually.
                        NoAutoAabb,
                        ChildOf(request.parent),
                    ));

                    if asset.properties.wind_affected {
                        cmd.entity(entity).insert(WindAffected);
                    }

                    if asset.properties.options.general.gpu_cull {
                        cmd.entity(entity).insert(GpuCullCompute);
                    }
                }
            }
        }
    }
}
