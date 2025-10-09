use crate::prelude::*;
use crate::scatter::utils::select_consistent_prototype;
use bevy::camera::primitives::Aabb;
use bevy::prelude::*;
use bevy::render::batching::NoAutomaticBatching;

pub fn spawn_instanced_wind_affected(
    mut mr_spawn: MessageReader<SpawnProtoTypes<InstancedWindAffectedMaterial>>,
    mut cmd: Commands,
    prototype_assets: Res<Assets<ScatterAsset<InstancedWindAffectedMaterial>>>,
    q_chunks: Query<(&GlobalTransform, &ChunkLevel), (With<Chunk>, Without<Merging>)>,
    q_root: Query<&LodConfig, With<ScatterRoot>>,
) {
    for event in mr_spawn.read() {
        debug!("Spawning instanced wind affected!");

        let instances = event
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

        if let Some(chunk) = event.trigger.chunk {
            let Ok((gtf, level)) = q_chunks.get(chunk) else {
                continue;
            };

            chunk_gtf = gtf.compute_transform();
            chunk_level = level.clone();
        }

        let Some((chosen_name, prototype)) = select_consistent_prototype(
            &event.items,
            event.trigger.seed,
            &prototype_assets,
            &chunk_level,
            event.trigger.chunk.is_some(),
        ) else {
            debug!(
                "No suitable instanced prototype found for LOD {:?}. Skipping.",
                chunk_level
            );
            continue;
        };

        let Ok(lod_config) = q_root.get(event.trigger.root) else {
            warn!("Couldn't get ScatterRoot!");
            continue;
        };

        debug!(
            "Spawning instanced prototype '{}' in level {:?}",
            chosen_name, chunk_level
        );

        let mesh_handle = prototype.mesh().clone();
        let (mut min_point, mut max_point) = (Vec3::MAX, Vec3::MIN);

        let instances = instances
            .iter()
            .map(|instance| {
                let mut instance = *instance;

                instance.position += chunk_gtf.translation;

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

        let parent = event.trigger.chunk.unwrap_or(event.trigger.layer);

        cmd.entity(entity).insert((
            Transform::default(),
            Visibility::Visible,
            local_aabb,
            ChildOf(parent),
            visibility_range,
        ));
    }
}
