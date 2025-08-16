use crate::prelude::*;
use bevy::camera::primitives::Aabb;
use bevy::camera::visibility::VisibilityRange;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::render::batching::NoAutomaticBatching;
use rand::prelude::IteratorRandom;
use rand::rng;

pub fn spawn_instanced_wind_affected(
    mut er_spawn: EventReader<SpawnProtoTypes<InstancedWindAffectedMaterial>>,
    mut cmd: Commands,
    prototype_assets: Res<Assets<ScatterAsset<InstancedWindAffectedMaterial>>>,
    q_chunks: Query<(&GlobalTransform, &ChunkLevel), (With<Chunk>, Without<Merging>)>,
    q_root: Query<(&LodConfig, Option<&ChunkLodConfig>), With<ScatterRoot>>,
) {
    for e in er_spawn.read() {
        debug!("Spawning instanced wind affected!");

        let instances = e
            .trigger
            .data
            .iter()
            .enumerate()
            .map(|(i, res)| InstanceData {
                position: res.translation,
                scale: res.scale.element_sum() / 3.0,
                // TODO expose
                color: LinearRgba::from(Color::hsla(78., 0.98, 0.5, 1.0)).to_f32_array(),
                index: i as u32,
            })
            .collect::<Vec<_>>();

        let mut chunk_gtf = Transform::default();
        let mut chunk_level = ChunkLevel::default();

        if let Some(chunk) = e.trigger.chunk {
            let Ok((gtf, level)) = q_chunks.get(chunk) else {
                continue;
            };

            chunk_gtf = gtf.compute_transform();
            chunk_level = level.clone();
        }

        let mut prototypes: Vec<ScatterAsset<InstancedWindAffectedMaterial>> = vec![];
        for item in e.items.iter() {
            let prototype = prototype_assets.get(&item.0);

            let Some(prototype) = prototype else {
                warn!("Couldn't get ScatterRoot!");
                return;
            };

            prototypes.push(prototype.clone());
        }

        let mut name_map =
            HashMap::<Name, Vec<&ScatterAsset<InstancedWindAffectedMaterial>>>::new();

        prototypes
            .iter()
            .filter(|x| e.trigger.chunk.is_none() || *x.lod_level == *chunk_level)
            .map(|x| (x.name.clone().unwrap_or(Name::new("")), x))
            .for_each(|(name, x)| {
                name_map
                    .get_mut(&name)
                    .map(|y| y.push(x))
                    .map(|_| x)
                    .or_else(|| name_map.insert(name, vec![x]).map(|_| x));
            });

        debug!(
            "Spawning {} instanced prototypes in level {chunk_level:?}.",
            prototypes.len()
        );

        let prototypes = name_map.values().choose(&mut rng());

        let Some(prototypes) = prototypes else {
            warn!("No prototypes in level {chunk_level:?}!");
            return;
        };

        let Ok((lod_config, chunk_lod_config)) = q_root.get(e.trigger.root) else {
            warn!("Couldn't get ScatterRoot!");
            return;
        };

        for prototype in prototypes {
            let mesh_handle = prototype.mesh().clone();
            let (mut min_point, mut max_point) = (Vec3::MAX, Vec3::MIN);

            let instances = instances
                .iter()
                .map(|instance| {
                    let mut instance = instance.clone();

                    instance.position = instance.position + chunk_gtf.translation;

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
                    InstanceMaterialData(instances),
                    NoAutomaticBatching,
                    WindAffected,
                    WindAffectedReady,
                ))
                .id();

            let lod_level = prototype.lod_level;

            let visibility_range = lod_config.get_visibility_range(lod_level);

            let chunk_center = chunk_gtf.translation;

            let local_min = min_point - chunk_center;
            let local_max = max_point - chunk_center;

            let local_aabb = Aabb::from_min_max(local_min, local_max);

            let parent = e.trigger.chunk.unwrap_or(e.trigger.layer);

            cmd.entity(entity).insert((
                Transform::default(),
                Visibility::Visible,
                Aabb::from(local_aabb),
                ChildOf(parent),
                visibility_range,
            ));
        }
    }
}
