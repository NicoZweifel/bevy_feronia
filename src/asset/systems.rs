use crate::core::components::LevelOfDetail;
use crate::prelude::*;
use bevy::camera::primitives::{Aabb, MeshAabb};
use bevy::ecs::query::QueryData;
use bevy::prelude::*;

use crate::scatter::utils::combine_aabbs;
#[cfg(feature = "avian")]
use avian3d::prelude::{Collider, RigidBody};

#[derive(QueryData)]
#[query_data()]
pub struct CollectableQueryData<'w, T: Material> {
    pub entity: Entity,
    pub transform: &'w Transform,
    pub o_material: Option<&'w MeshMaterial3d<T>>,
    pub o_mesh: Option<&'w Mesh3d>,
    pub o_aabb: Option<&'w Aabb>,
    pub o_children: Option<&'w Children>,
    pub o_wind_config: Option<&'w WindConfig>,
    pub o_name: Option<&'w Name>,
    pub o_lod: Option<&'w LevelOfDetail>,
    pub o_wind_affected: Option<&'w WindAffected>,

    #[cfg(feature = "avian")]
    pub o_rigid_body: Option<&'w RigidBody>,
    #[cfg(feature = "avian")]
    pub o_collider: Option<&'w Collider>,

    pub material_options: MaterialOptionData<'w>,
    pub wind_data: WindData<'w>,
}

/// Internal enum to signal the result of the recursive asset search.
enum AssetSearchResult {
    /// Found no parts or roots.
    None,
    /// Found a renderable part (mesh+material).
    Part,
    /// Found an asset root.
    Root,
}

/// System that initiates the asset collection "read" phase.
///
/// Iterates all [`ScatterRoot`] and [`ScatterLayer`] entities and begins the
/// recursive search for logical asset roots within them.
pub fn queue_asset_creation_requests<TOut, TIn>(
    mut cmd: Commands,
    q_roots: Query<(Entity, &ScatterRoot), Without<ScatterRootProcessed>>,
    q_layers: Query<
        (Entity, &Children, MaterialOptionData, WindData),
        (
            With<ScatterLayer>,
            Without<ScatterLayerProcessed>,
            With<ScatterLayerType<TOut, TIn>>,
        ),
    >,
    q_items: Query<
        CollectableQueryData<TIn>,
        (
            Without<ScatterLayerChildProcessed>,
            Without<ScatterAssetCreationRequest<TOut, TIn>>,
        ),
    >,
    wind: Res<Wind>,
    mut meshes: ResMut<Assets<Mesh>>,
) where
    TIn: Material,
    TOut: ScatterMaterial<TIn>,
{
    let iter = q_roots
        .iter()
        .flat_map(|(root, root_children)| {
            debug!(
                "Queueing ScatterAsset creation requests in root {:?}...",
                root
            );
            root_children.iter()
        })
        .filter_map(|layer_entity| q_layers.get(layer_entity).ok())
        .flat_map(
            |(layer_entity, layer_children, material_option_data, wind_data)| {
                layer_children.iter().map(move |item_entity| {
                    (layer_entity, item_entity, material_option_data, wind_data)
                })
            },
        );

    for (layer, item_entity, material_option_data, wind_data) in iter {
        let Ok(item) = q_items.get(item_entity) else {
            continue;
        };

        let wind = (*wind).with(wind_data).with(item.wind_data);
        let options = MaterialOptions::from(material_option_data).with(item.material_options);
        let lod = item.o_lod.cloned().unwrap_or_default();
        let name = item.o_name.cloned();

        let result = queue_asset_creation_requests_recursive::<TOut, TIn>(
            layer,
            item_entity,
            &mut cmd,
            &wind,
            &options,
            name.clone(),
            Some(lod),
            &q_items,
            &mut meshes,
        );

        let AssetSearchResult::Part = result else {
            continue;
        };

        collect_and_queue_request::<TOut, TIn>(
            layer,
            item_entity,
            &mut cmd,
            &wind,
            &options,
            name,
            lod,
            #[cfg(feature = "avian")]
            item.o_rigid_body.cloned(),
            &q_items,
            &mut meshes,
        );
    }
}

/// Recursively searches for "asset roots" and spawns creation requests.
///
/// An asset root is defined as:
/// 1.  An entity with a `RigidBody`, if the `avian` feature is enabled.
/// 2.  The parent entity of the first `Part` (mesh+material).
///
/// Returns an [`AssetSearchResult`] to signal to the parent what was found.
fn queue_asset_creation_requests_recursive<TOut, TIn>(
    layer: Entity,
    entity: Entity,
    cmd: &mut Commands,
    wind: &Wind,
    options: &MaterialOptions,
    o_current_name: Option<Name>,
    o_current_lod: Option<LevelOfDetail>,
    q_items: &Query<
        CollectableQueryData<TIn>,
        (
            Without<ScatterLayerChildProcessed>,
            Without<ScatterAssetCreationRequest<TOut, TIn>>,
        ),
    >,
    meshes: &mut ResMut<Assets<Mesh>>,
) -> AssetSearchResult
where
    TIn: Material,
    TOut: ScatterMaterial<TIn>,
{
    let Ok(item) = q_items.get(entity) else {
        return AssetSearchResult::None;
    };

    let wind = item
        .o_wind_config
        .and_then(|x| x.wind_override)
        .unwrap_or(*wind)
        .with(item.wind_data);

    let lod = item.o_lod.map_or(o_current_lod.unwrap_or_default(), |x| *x);
    let name = o_current_name.map_or(item.o_name.cloned(), Some);
    let options = options.with(item.material_options);

    #[cfg(feature = "avian")]
    let is_physics_root = item.o_rigid_body.is_some();
    #[cfg(not(feature = "avian"))]
    let is_physics_root = false;

    if is_physics_root {
        collect_and_queue_request::<TOut, TIn>(
            layer,
            entity,
            cmd,
            &wind,
            &options,
            name,
            lod,
            #[cfg(feature = "avian")]
            item.o_rigid_body.cloned(),
            q_items,
            meshes,
        );
        return AssetSearchResult::Root;
    }

    let has_mesh_and_material = item.o_mesh.is_some() && item.o_material.is_some();

    let mut found_part = false;
    let mut found_root = false;

    if let Some(children) = item.o_children {
        for child in children.iter() {
            match queue_asset_creation_requests_recursive::<TOut, TIn>(
                layer,
                child,
                cmd,
                &wind,
                &options,
                name.clone(),
                Some(lod),
                q_items,
                meshes,
            ) {
                AssetSearchResult::None => {}
                AssetSearchResult::Part => found_part = true,
                AssetSearchResult::Root => found_root = true,
            }
        }
    }

    if found_part && !found_root {
        collect_and_queue_request::<TOut, TIn>(
            layer,
            entity,
            cmd,
            &wind,
            &options,
            name,
            lod,
            #[cfg(feature = "avian")]
            None,
            q_items,
            meshes,
        );
        return AssetSearchResult::Root;
    }

    if has_mesh_and_material {
        return AssetSearchResult::Part;
    }

    if found_root {
        cmd.entity(entity).insert(ScatterLayerChildProcessed);
        return AssetSearchResult::Root;
    }

    cmd.entity(entity).insert(ScatterLayerChildProcessed);

    AssetSearchResult::None
}

/// Collects all parts and queues the [`ScatterAssetCreationRequest`].
///
/// This is called *after* a root entity has been identified. It calculates
/// the final properties (like the combined AABB) and inserts the request component.
fn collect_and_queue_request<TOut, TIn>(
    layer: Entity,
    root_entity: Entity,
    cmd: &mut Commands,
    root_wind: &Wind,
    o_root: &MaterialOptions,
    o_root_name: Option<Name>,
    root_lod: LevelOfDetail,
    #[cfg(feature = "avian")] o_root_rigid_body: Option<RigidBody>,
    q_items: &Query<
        CollectableQueryData<TIn>,
        (
            Without<ScatterLayerChildProcessed>,
            Without<ScatterAssetCreationRequest<TOut, TIn>>,
        ),
    >,
    meshes: &mut ResMut<Assets<Mesh>>,
) where
    TIn: Material,
    TOut: ScatterMaterial<TIn>,
{
    let all_parts = collect_parts_recursive_internal(
        layer,
        root_entity,
        cmd,
        root_wind,
        o_root,
        o_root_name.clone(),
        Some(root_lod),
        q_items,
        meshes,
    );

    if all_parts.is_empty() {
        return;
    }

    let mut union_aabb = all_parts[0].properties.aabb;
    for part in &all_parts[1..] {
        union_aabb = combine_aabbs(&union_aabb, &part.properties.aabb);
    }

    let any_wind_affected =
        o_root.wind_affected || all_parts.iter().any(|part| part.properties.wind_affected);

    let global_properties = ScatterAssetProperties {
        wind: *root_wind,
        options: *o_root,
        aabb: union_aabb,
        name: o_root_name,
        lod: root_lod,
        layer,
        wind_affected: any_wind_affected,
    };

    let request = ScatterAssetCreationRequest::<TOut, TIn>::new(
        global_properties,
        all_parts,
        #[cfg(feature = "avian")]
        o_root_rigid_body,
    );

    cmd.entity(root_entity).insert(request);
}

/// Internal helper to recursively gather all `ScatterAssetPart`s for a root.
///
/// This function *must* mark all visited entities with `ScatterLayerChildProcessed`
/// to prevent them from being re-searched.
fn collect_parts_recursive_internal<TOut, TIn>(
    layer: Entity,
    entity: Entity,
    cmd: &mut Commands,
    wind: &Wind,
    options: &MaterialOptions,
    o_current_name: Option<Name>,
    o_current_lod: Option<LevelOfDetail>,
    q_items: &Query<
        CollectableQueryData<TIn>,
        (
            Without<ScatterLayerChildProcessed>,
            Without<ScatterAssetCreationRequest<TOut, TIn>>,
        ),
    >,
    meshes: &mut ResMut<Assets<Mesh>>,
) -> Vec<ScatterAssetPart<TIn>>
where
    TOut: ScatterMaterial<TIn>,
    TIn: Material,
{
    let Ok(item) = q_items.get(entity) else {
        return vec![];
    };

    cmd.entity(entity).insert(ScatterLayerChildProcessed);

    let wind = item
        .o_wind_config
        .and_then(|x| x.wind_override)
        .unwrap_or(*wind)
        .with(item.wind_data);

    let lod = item.o_lod.map_or(o_current_lod.unwrap_or_default(), |x| *x);
    let name = o_current_name.map_or(item.o_name.cloned(), Some);

    // TODO https://github.com/NicoZweifel/bevy_feronia/issues/16
    let hue = (item.entity.index() * 30) as f32 % 360.0;
    let debug_color = Color::hsl(hue, 1.0, 0.5);
    let options = options
        .with(item.material_options)
        .with_debug_color(debug_color);

    let mut all_parts = item
        .o_children
        .map(|children| children.iter())
        .unwrap_or_default()
        .flat_map(|child| {
            collect_parts_recursive_internal(
                layer,
                child,
                cmd,
                &wind,
                &options,
                name.clone(),
                Some(lod),
                q_items,
                meshes,
            )
        })
        .collect::<Vec<_>>();

    if let (Some(mesh), Some(material)) = (item.o_mesh, item.o_material) {
        let aabb = item.o_aabb.cloned().unwrap_or_else(|| {
            meshes
                .get(&mesh.0)
                .and_then(|x| x.compute_aabb())
                .unwrap_or_default()
        });

        let part_properties = ScatterAssetProperties {
            wind,
            options,
            aabb,
            name,
            lod,
            layer,
            wind_affected: options.wind_affected || item.o_wind_affected.is_some(),
        };

        let part = ScatterAssetPart::new(
            material.0.clone(),
            mesh.0.clone(),
            *item.transform,
            part_properties,
            #[cfg(feature = "avian")]
            item.o_collider.cloned(),
        );
        all_parts.push(part);
    }

    all_parts
}

/// System that processes asset requests where `TIn` and `TOut` are different.
///
/// Creates a *new* `TOut` material for each asset part.
pub fn process_distinct_material_requests<TOut, TIn>(
    mut cmd: Commands,
    q_requests: Query<(Entity, &ScatterAssetCreationRequest<TOut, TIn>)>,
    materials_in: Res<Assets<TIn>>,
    mut materials_out: ResMut<Assets<TOut>>,
    wind_noise_texture: Res<WindTexture>,
    mut prototype_assets: ResMut<Assets<ScatterAsset<TOut>>>,
) where
    TIn: Material,
    TOut: ScatterMaterial<TIn>,
{
    for (
        entity,
        ScatterAssetCreationRequest {
            parts,
            properties,
            #[cfg(feature = "avian")]
            o_rigid_body,
            ..
        },
    ) in &q_requests
    {
        let parts = parts
            .iter()
            .map(
                |ScatterAssetPart {
                     h_material,
                     h_mesh,
                     transform,
                     properties,
                     #[cfg(feature = "avian")]
                     collider,
                 }| {
                    let source_material = materials_in.get(h_material);

                    let material = TOut::create_material(
                        source_material.cloned(),
                        wind_noise_texture.0.clone(),
                        properties,
                    );

                    let h_material = materials_out.add(material);

                    ScatterAssetPart {
                        transform: *transform,
                        properties: properties.clone(),
                        h_material,
                        h_mesh: h_mesh.clone(),
                        #[cfg(feature = "avian")]
                        collider: collider.clone(),
                    }
                },
            )
            .collect::<Vec<_>>();

        let asset = ScatterAsset::new(
            parts.clone(),
            properties.clone(),
            #[cfg(feature = "avian")]
            *o_rigid_body,
        );
        let h_scatter_asset = prototype_assets.add(asset.clone());

        cmd.entity(entity)
            .remove::<ScatterAssetCreationRequest<TOut, TIn>>();

        for part in parts {
            part.insert_bundle(&mut cmd, entity, h_scatter_asset.clone());
        }
    }
}

/// System that processes asset requests where `TIn` and `TOut` are the same.
///
/// Creates a *new* `T` material for each asset part.
///
/// TODO: Reconsider if a new material needs to be created here or if we can always re-use the base.
pub fn process_same_type_material_requests<T>(
    mut cmd: Commands,
    requests: Query<(Entity, &ScatterAssetCreationRequest<T, T>)>,
    mut materials: ResMut<Assets<T>>,
    wind_noise_texture: Res<WindTexture>,
    mut scatter_assets: ResMut<Assets<ScatterAsset<T>>>,
) where
    T: ScatterMaterial<T> + Material,
{
    for (
        entity,
        ScatterAssetCreationRequest {
            parts,
            properties,
            #[cfg(feature = "avian")]
            o_rigid_body,
            ..
        },
    ) in &requests
    {
        let parts = parts
            .iter()
            .map(|part| {
                let source_material = materials.get(&part.h_material);

                let material = T::create_material(
                    source_material.cloned(),
                    wind_noise_texture.0.clone(),
                    properties,
                );

                let h_material = materials.add(material);

                ScatterAssetPart::<T> {
                    h_material,
                    ..part.clone()
                }
            })
            .collect::<Vec<_>>();

        let asset = ScatterAsset::new(
            parts.clone(),
            properties.clone(),
            #[cfg(feature = "avian")]
            *o_rigid_body,
        );

        let h_scatter_asset = scatter_assets.add(asset.clone());

        cmd.entity(entity)
            .remove::<ScatterAssetCreationRequest<T, T>>();

        for part in parts {
            part.insert_bundle(&mut cmd, entity, h_scatter_asset.clone());
        }
    }
}
