use crate::prelude::*;
use crate::scatter::utils::combine_aabbs;
use bevy_asset::Assets;
use bevy_camera::primitives::MeshAabb;
use bevy_ecs::prelude::*;
use bevy_mesh::{Mesh, Mesh3d};
use bevy_platform::collections::{HashMap, HashSet};

#[cfg(feature = "tracing")]
use tracing::{debug, error, trace, warn};

pub fn backend(world: &mut World) {
    let Some(item_backend) = world.get_resource::<ScatterAssetBackend>() else {
        #[cfg(feature = "tracing")]
        error!("No AssetItemBackend found!");
        return;
    };

    let processed_entities = world
        .run_system_with(**item_backend, ())
        .into_iter()
        .flatten()
        .flatten()
        .map(|AssetItem { entity, item_of }| {
            let mut cmd: Commands = world.commands();
            cmd.entity(entity).insert(item_of.clone());

            item_of
        })
        .fold(HashSet::new(), |mut acc, item_of| {
            acc.insert(item_of.root);
            acc.insert(item_of.item);
            acc.insert(item_of.layer);
            acc
        });

    let mut cmd: Commands = world.commands();

    for e in processed_entities {
        cmd.entity(e).remove::<NeedsAssetCollection>();
    }
}

pub fn insert_parts<T: ScatterMaterial>(
    mut cmd: Commands,
    q_items: Query<(Entity, &AssetItemOf), Without<ScatterAssetPart>>,
    q_data: Query<(&ChildOf, CollectableQueryData), (Without<ScatterLayerChildProcessed>,)>,
    q_layers: Query<
        (Entity, MaterialOptionData, WindOptionData),
        (With<ScatterLayer>, With<ScatterLayerType<T>>),
    >,
    wind: Res<Wind>,
    meshes: ResMut<Assets<Mesh>>,
) {
    for ScatterAssetPartEntity { entity, part } in q_items
        .into_iter()
        .map(|x| AssetItem::from(x))
        .filter_map(|AssetItem { entity, item_of }| {
            let (child_of, scene_root_data) = q_data
                .get(item_of.root)
                .map_err(|_| {
                    #[cfg(feature = "tracing")]
                    warn!("Could not get AssetItem root {}, skipping.", item_of.root);
                })
                .ok()?;

            let layer = child_of.parent();
            let (_, layer_material_option_data, layer_wind_data) = q_layers
                .get(layer)
                .map_err(|_| {
                    #[cfg(feature = "tracing")]
                    trace!(
                        "Multiple ScatterLayerTypes in use, skipping Layer {}.",
                        layer
                    );
                })
                .ok()?;

            let (child_of, child_data) = q_data
                .get(entity)
                .map_err(|_| {
                    #[cfg(feature = "tracing")]
                    warn!(
                        "Asset part {:?} is not a processable scatter asset part, skipping.",
                        entity
                    );
                })
                .ok()?;

            let (_, item_root_data) = q_data
                .get(item_of.item)
                .map_err(|_| {
                    #[cfg(feature = "tracing")]
                    warn!("Could not get AssetItem {}, skipping.", item_of.item);
                })
                .ok()?;

            let (_, parent_data) = q_data
                .get(child_of.parent())
                .map_err(|_| {
                    #[cfg(feature = "tracing")]
                    warn!(
                        "Could not get AssetItem parent {}, skipping.",
                        child_of.parent()
                    );
                })
                .ok()?;

            let o_mesh = child_data.o_mesh?;
            let aabb = child_data.o_aabb.cloned().unwrap_or_else(|| {
                meshes
                    .get(o_mesh)
                    .and_then(|mesh| mesh.compute_aabb())
                    .unwrap_or_default()
            });

            ScatterAssetPartEntity::try_from_data(
                entity,
                item_of,
                *wind,
                layer_wind_data,
                scene_root_data,
                item_root_data,
                parent_data,
                child_data,
                layer_material_option_data,
                aabb,
            )
        })
    {
        cmd.entity(entity).insert(part);
    }
}

pub fn insert_requests<T: ScatterMaterial>(
    mut cmd: Commands,
    q_parts: Query<
        (Entity, &ScatterAssetPart, &AssetItemOf),
        Without<ScatterAssetCreationRequest<T>>,
    >,
    q_data: Query<(&ChildOf, CollectableQueryData), (Without<ScatterLayerChildProcessed>,)>,
    q_layers: Query<
        (Entity, MaterialOptionData, WindOptionData),
        (With<ScatterLayer>, With<ScatterLayerType<T>>),
    >,
    wind: Res<Wind>,
) {
    let processed_scene_roots = q_parts
        .iter()
        .fold(
            HashMap::<AssetItemOf, Vec<ScatterAssetPartEntity>>::new(),
            |mut acc, (entity, part, item_of)| {
                acc.entry(item_of.clone())
                    .or_default()
                    .push(ScatterAssetPartEntity::new(entity, part.clone()));
                acc
            },
        )
        .into_iter()
        .filter_map(|(item_of, entity_parts)| {
            #[cfg(feature = "tracing")]
            debug!(
                "Collecting ScatterAssetPart {:?}: {:?} {:?}",
                item_of,
                entity_parts.len(),
                entity_parts[0].part.properties.lod
            );

            let AssetItemOf {
                root: scene_root, ..
            } = &item_of;

            let (child_of, scene_root_data) = q_data
                .get(*scene_root)
                .map_err(|_| {
                    #[cfg(feature = "tracing")]
                    debug!(
                        "Scene asset {:?} is not a processable scatter asset, skipping.",
                        scene_root
                    );
                })
                .ok()?;

            let layer = child_of.parent();

            let (_, layer_material_option_data, layer_wind_data) = q_layers
                .get(layer)
                .map_err(|_| {
                    #[cfg(feature = "tracing")]
                    trace!(
                        "Multiple ScatterLayerTypes in use, skipping Layer {}.",
                        layer
                    );
                })
                .ok()?;

            let wind = (*wind)
                .with(layer_wind_data)
                .with(scene_root_data.wind_data);

            let options = ScatterMaterialOptions::from(layer_material_option_data)
                .with(scene_root_data.material_options);

            let (part_entities, parts) = entity_parts.iter().fold(
                (Vec::<Entity>::new(), Vec::<ScatterAssetPart>::new()),
                |(mut part_entities, mut parts), p| {
                    part_entities.push(p.entity);
                    parts.push(p.part.clone());

                    (part_entities, parts)
                },
            );

            let mut union_aabb = parts[0].properties.aabb;
            for part in &parts[1..] {
                union_aabb = combine_aabbs(&union_aabb, &part.properties.aabb);
            }

            Some((
                item_of.clone(),
                ScatterAssetCreationRequest::<T>::from_data(item_of, entity_parts, wind, options),
                part_entities,
            ))
        })
        .map(|(item_of, request, part_entities)| {
            cmd.entity(item_of.item).insert(request);

            for part_entity in part_entities {
                cmd.entity(part_entity)
                    .remove::<ScatterAssetPart>()
                    .remove::<AssetItemOf>()
                    .remove::<Mesh3d>();
            }

            item_of
        })
        .fold(HashSet::new(), |mut acc, item_of| {
            acc.insert((item_of.root, item_of.name));
            acc
        });

    for (scene_root, _name) in processed_scene_roots {
        #[cfg(feature = "tracing")]
        debug!(
            "Processed ScatterLayerChild {} {scene_root}",
            _name.unwrap_or_default()
        );
        cmd.entity(scene_root).insert(ScatterLayerChildProcessed);
    }
}
