use crate::prelude::*;
use crate::scatter::utils::combine_aabbs;
use bevy_asset::Assets;
use bevy_camera::primitives::MeshAabb;
use bevy_ecs::prelude::*;
use bevy_mesh::{Mesh, Mesh3d};
use bevy_platform::collections::{HashMap, HashSet};

#[cfg(feature = "trace")]
use tracing::{debug, error, trace, warn};

pub fn backend(world: &mut World) {
    let Some(item_backend) = world.get_resource::<ScatterAssetBackend>() else {
        #[cfg(feature = "trace")]
        error!("No AssetItemBackend found!");
        return;
    };

    let processed_entities = world
        .run_system_with(**item_backend, ())
        .into_iter()
        .flatten()
        .flatten()
        .map(
            |AssetPart {
                 entity,
                 part_of: item_of,
             }| {
                let mut cmd: Commands = world.commands();
                cmd.entity(entity).insert(item_of.clone());

                item_of
            },
        )
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
    q_items: Query<(Entity, &AssetPartOf), Without<ScatterAssetPart>>,
    q_data: Query<(&ChildOf, CollectableQueryData), (Without<ScatterLayerChildProcessed>,)>,
    q_layers: Query<
        (Entity, MaterialOptionData, WindOptionData),
        (With<ScatterLayer>, With<ScatterLayerType<T>>),
    >,
    global_wind: Res<GlobalWind>,
    meshes: ResMut<Assets<Mesh>>,
) {
    let wind = global_wind.current;
    for ScatterAssetPartEntity { entity, part } in
        q_items.into_iter().map(AssetPart::from).filter_map(
            |AssetPart {
                 entity,
                 part_of: item_of,
             }| {
                let (child_of, scene_root_data) = q_data
                    .get(item_of.root)
                    .inspect_err(|_| {
                        #[cfg(feature = "trace")]
                        error!(
                            "Could not get AssetItem {} root {}, skipping.",
                            item_of.root, entity
                        );
                    })
                    .ok()?;

                let layer = child_of.parent();
                let (_, layer_material_option_data, layer_wind_data) = q_layers
                    .get(layer)
                    .inspect_err(|_| {
                        #[cfg(feature = "trace")]
                        trace!(
                            "Multiple ScatterLayerTypes in use, skipping Layer {}.",
                            layer
                        );
                    })
                    .ok()?;

                let (child_of, child_data) = q_data
                    .get(entity)
                    .inspect_err(|_| {
                        #[cfg(feature = "trace")]
                        warn!(
                            "Asset part {:?} is not a processable scatter asset part, skipping.",
                            entity
                        );
                    })
                    .ok()?;

                let (_, item_root_data) = q_data
                    .get(item_of.item)
                    .inspect_err(|_| {
                        #[cfg(feature = "trace")]
                        warn!("Could not get AssetItem {}, skipping.", item_of.item);
                    })
                    .ok()?;

                let (_, parent_data) = q_data
                    .get(child_of.parent())
                    .inspect_err(|_| {
                        #[cfg(feature = "trace")]
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
                    wind,
                    layer_wind_data,
                    scene_root_data,
                    item_root_data,
                    parent_data,
                    child_data,
                    layer_material_option_data,
                    aabb,
                )
            },
        )
    {
        cmd.entity(entity).insert(part);
    }
}

pub fn insert_requests<T: ScatterMaterial>(
    mut cmd: Commands,
    q_parts: Query<
        (Entity, &ScatterAssetPart, &AssetPartOf),
        Without<ScatterAssetCreationRequest<T>>,
    >,
    q_data: Query<(&ChildOf, CollectableQueryData), (Without<ScatterLayerChildProcessed>,)>,
    q_layers: Query<CollectableQueryData, (With<ScatterLayer>, With<ScatterLayerType<T>>)>,
    global_wind: Res<GlobalWind>,
) {
    let wind = global_wind.current;
    let processed_scene_roots = q_parts
        .iter()
        .fold(
            HashMap::<AssetPartOf, Vec<ScatterAssetPartEntity>>::new(),
            |mut acc, (entity, part, item_of)| {
                acc.entry(item_of.clone())
                    .or_default()
                    .push(ScatterAssetPartEntity::new(entity, part.clone()));
                acc
            },
        )
        .into_iter()
        .filter_map(|(item_of, entity_parts)| {
            #[cfg(feature = "trace")]
            debug!(
                "Collecting ScatterAssetPart {:?}: {:?} {:?}",
                item_of,
                entity_parts.len(),
                entity_parts[0].part.properties.lod
            );

            let AssetPartOf {
                root: scene_root, ..
            } = &item_of;

            let (child_of, scene_root_data) = q_data
                .get(*scene_root)
                .inspect_err(|_| {
                    #[cfg(feature = "trace")]
                    debug!(
                        "Scene asset {:?} is not a processable scatter asset, skipping.",
                        scene_root
                    );
                })
                .ok()?;

            let layer = child_of.parent();

            let layer_data = q_layers
                .get(layer)
                .inspect_err(|_| {
                    #[cfg(feature = "trace")]
                    trace!(
                        "Multiple ScatterLayerTypes in use, skipping Layer {}.",
                        layer
                    );
                })
                .ok()?;

            let wind = wind
                .multiply(layer_data.wind_data)
                .multiply(scene_root_data.wind_data);

            let options = ScatterMaterialOptions::from(layer_data.material_options)
                .with(scene_root_data.material_options);

            let (part_entities, parts) = entity_parts.iter().fold(
                (Vec::<Entity>::new(), Vec::<ScatterAssetPart>::new()),
                |(mut part_entities, mut parts), p| {
                    part_entities.push(p.entity);
                    parts.push(p.part.clone());

                    (part_entities, parts)
                },
            );

            let mut union_aabb = parts[0].properties.aabb.transform(&parts[0].transform);
            for part in parts.iter().skip(1) {
                let transformed = part.properties.aabb.transform(&part.transform);
                union_aabb = combine_aabbs(&union_aabb, &transformed);
            }

            Some((
                item_of.clone(),
                ScatterAssetCreationRequest::<T>::from_data(
                    item_of,
                    entity_parts,
                    wind,
                    options,
                    #[cfg(feature = "avian")]
                    scene_root_data
                        .o_rigid_body
                        .or(layer_data.o_rigid_body)
                        .cloned(),
                ),
                part_entities,
            ))
        })
        .map(|(item_of, request, part_entities)| {
            cmd.entity(item_of.item).insert(request);

            for part_entity in part_entities {
                cmd.entity(part_entity)
                    .remove::<ScatterAssetPart>()
                    .remove::<AssetPartOf>()
                    .remove::<Mesh3d>();
            }

            item_of
        })
        .fold(HashSet::new(), |mut acc, item_of| {
            acc.insert((item_of.root, item_of.name));
            acc
        });

    for (scene_root, _name) in processed_scene_roots {
        #[cfg(feature = "trace")]
        debug!(
            "Processed ScatterLayerChild {} {scene_root}",
            _name.unwrap_or_default()
        );
        cmd.entity(scene_root).insert(ScatterLayerChildProcessed);
    }
}
