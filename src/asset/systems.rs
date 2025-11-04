use crate::core::components::LevelOfDetail;
use crate::prelude::*;
use bevy::camera::primitives::{Aabb, MeshAabb};
use bevy::prelude::*;

pub type CollectableQueryData<'w, T> = (
    Entity,
    Option<&'w MeshMaterial3d<T>>,
    Option<&'w Mesh3d>,
    Option<&'w Aabb>,
    Option<&'w Children>,
    Option<&'w WindConfig>,
    Option<&'w Name>,
    Option<&'w LevelOfDetail>,
    Option<&'w WindAffected>,
    MaterialOptionData<'w>,
    WindData<'w>,
);

pub fn queue_material_creation_requests<TOut, TIn>(
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
    q_collect: Query<
        CollectableQueryData<TIn>,
        (
            Without<ScatterLayerChildProcessed>,
            Without<ScatterMaterialCreationRequest<TOut, TIn>>,
        ),
    >,
    wind: Res<Wind>,
    mut meshes: ResMut<Assets<Mesh>>,
) where
    TIn: Material,
    TOut: ScatterMaterial<TIn>,
{
    for (root, children) in &q_roots {
        debug!(
            "Queueing ScatterAsset creation requests in root {:?}...",
            root
        );

        for (layer, scatter_items, material_option_data, wind_data) in
            children.iter().filter_map(|layer| q_layers.get(layer).ok())
        {
            let mut wind = *wind;
            wind = wind.with(wind_data);

            let options = MaterialOptions::from(material_option_data);

            for item in scatter_items {
                queue_requests_recursive::<TOut, TIn>(
                    layer,
                    *item,
                    &mut cmd,
                    &wind,
                    &options,
                    None,
                    None,
                    &q_collect,
                    &mut meshes,
                );
            }
        }
    }
}

fn queue_requests_recursive<TOut, TIn>(
    layer: Entity,
    entity: Entity,
    cmd: &mut Commands,
    wind: &Wind,
    options: &MaterialOptions,
    o_current_name: Option<Name>,
    o_current_lod: Option<LevelOfDetail>,
    q_children: &Query<
        CollectableQueryData<TIn>,
        (
            Without<ScatterLayerChildProcessed>,
            Without<ScatterMaterialCreationRequest<TOut, TIn>>,
        ),
    >,
    meshes: &mut ResMut<Assets<Mesh>>,
) -> bool
where
    TIn: Material,
    TOut: ScatterMaterial<TIn>,
{
    let Ok((
        entity,
        o_material,
        o_mesh,
        o_aabb,
        o_children,
        o_wind,
        o_name,
        o_lod,
        o_wind_affected,
        material_option_data,
        wind_data,
    )) = q_children.get(entity)
    else {
        return false;
    };

    let mut wind = o_wind.and_then(|x| x.wind_override).unwrap_or(*wind);

    wind = wind.with(wind_data);

    let lod = o_lod.map_or(o_current_lod.unwrap_or_default(), |x| *x);
    let name = o_current_name.map_or(o_name.cloned(), Some);

    // TODO expose in some way
    let hue = (entity.index() * 30) as f32 % 360.0;
    let debug_color = Color::hsl(hue, 1.0, 0.5);

    let options = options
        .with(material_option_data)
        .with_debug_color(debug_color);

    #[allow(clippy::unnecessary_fold)]
    let has_children_with_materials = o_children
        .map(|children| children.iter())
        .unwrap_or_default()
        .map(|child| {
            queue_requests_recursive::<TOut, TIn>(
                layer,
                child,
                cmd,
                &wind,
                &options,
                name.clone(),
                Some(lod),
                q_children,
                meshes,
            )
        })
        .fold(false, |acc, has_material| acc || has_material);

    if has_children_with_materials {
        cmd.entity(entity).insert(ScatterLayerChildProcessed);
    }

    let Some(mesh) = o_mesh else {
        // TODO allow/create adapter/backends logic to allow more than just mesh
        return has_children_with_materials;
    };

    let aabb = o_aabb.cloned().unwrap_or_else(|| {
        meshes
            .get(&mesh.0)
            .and_then(|x| x.compute_aabb())
            .unwrap_or_default()
    });

    let request = ScatterMaterialCreationRequest::<TOut, TIn>::new(
        o_material.map(|x| x.0.clone()),
        ScatterAssetProperties {
            wind,
            options,
            mesh_handle: mesh.0.clone(),
            aabb,
            name,
            lod,
            layer,
            wind_affected: o_wind_affected.is_some(),
        },
    );

    cmd.entity(entity).insert(request);

    true
}

pub fn process_distinct_material_requests<TOut, TIn>(
    mut cmd: Commands,
    requests_query: Query<(Entity, &ScatterMaterialCreationRequest<TOut, TIn>)>,
    materials_in: Res<Assets<TIn>>,
    mut materials_out: ResMut<Assets<TOut>>,
    wind_noise_texture: Res<WindTexture>,
    mut prototype_assets: ResMut<Assets<ScatterAsset<TOut>>>,
) where
    TIn: Material,
    TOut: ScatterMaterial<TIn>,
{
    for (entity, request) in &requests_query {
        let source_material = request
            .source_material_handle
            .clone()
            .and_then(|x| materials_in.get(&x));

        let new_material = TOut::create_material(
            source_material.cloned(),
            wind_noise_texture.0.clone(),
            &request.properties,
        );

        let material_handle = materials_out.add(new_material);
        let asset = ScatterAsset::new(material_handle, request);
        let asset_handle = prototype_assets.add(asset.clone());

        cmd.entity(entity)
            .remove::<ScatterMaterialCreationRequest<TOut, TIn>>()
            .remove::<MeshMaterial3d<TIn>>();

        insert_bundle::<TOut>(&mut cmd, entity, asset_handle, asset, &request.properties);
    }
}

pub fn process_same_type_material_requests<T>(
    mut cmd: Commands,
    requests: Query<(Entity, &ScatterMaterialCreationRequest<T, T>)>,
    mut materials: ResMut<Assets<T>>,
    wind_noise_texture: Res<WindTexture>,
    mut prototype_assets: ResMut<Assets<ScatterAsset<T>>>,
) where
    T: ScatterMaterial<T> + Material + Clone,
{
    for (entity, request) in &requests {
        let source_material = request
            .source_material_handle
            .clone()
            .and_then(|x| materials.get(&x));

        let new_material = T::create_material(
            source_material.cloned(),
            wind_noise_texture.0.clone(),
            &request.properties,
        );

        let material_handle = materials.add(new_material);
        let asset = ScatterAsset::new(material_handle, request);
        let asset_handle = prototype_assets.add(asset.clone());

        cmd.entity(entity)
            .remove::<ScatterMaterialCreationRequest<T, T>>();

        insert_bundle::<T>(&mut cmd, entity, asset_handle, asset, &request.properties);
    }
}

fn insert_bundle<T>(
    cmd: &mut Commands,
    entity: Entity,
    asset_handle: Handle<ScatterAsset<T>>,
    asset: ScatterAsset<T>,
    properties: &ScatterAssetProperties,
) where
    T: Asset + Clone,
{
    if properties.wind_affected {
        cmd.entity(entity)
            .insert(asset.wind_affected_bundle(asset_handle));
    } else {
        cmd.entity(entity).insert(asset.bundle(asset_handle));
    }
}
