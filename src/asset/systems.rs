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
    q_roots: Query<(Entity, &ScatterRoot, MaterialOptionData), Without<ScatterRootProcessed>>,
    q_layers: Query<
        (&Children, MaterialOptionData, WindData),
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
) where
    TIn: Material,
    TOut: ScatterMaterial<TIn> + Asset + Clone,
{
    for (root, children, root_material_data) in &q_roots {
        debug!(
            "Queueing ScatterAsset creation requests in root {:?}...",
            root
        );

        for layer in children.iter() {
            let mut wind = wind.clone();
            let Ok((scatter_items, material_option_data, wind_data)) = q_layers.get(layer) else {
                continue;
            };

            wind = wind.with(wind_data);
            let options = MaterialOptions::from(root_material_data).with(material_option_data);

            for item in scatter_items {
                queue_requests_recursive::<TOut, TIn>(
                    layer, *item, &mut cmd, &wind, &options, None, None, &q_collect,
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
    current_name: Option<Name>,
    current_lod_level: Option<LevelOfDetail>,
    q_children: &Query<
        CollectableQueryData<TIn>,
        (
            Without<ScatterLayerChildProcessed>,
            Without<ScatterMaterialCreationRequest<TOut, TIn>>,
        ),
    >,
) -> bool
where
    TIn: Material,
    TOut: ScatterMaterial<TIn> + Asset + Clone,
{
    let Ok((
        entity,
        material,
        mesh,
        aabb,
        children,
        wind_component,
        name,
        lod,
        wind_affected,
        material_option_data,
        wind_data,
    )) = q_children.get(entity)
    else {
        return false;
    };

    let (mut wind, controlled) = wind_component
        .and_then(|x| x.wind_override.clone().map(|x| (x.clone(), true)))
        .unwrap_or_else(|| ((*wind).clone(), false));

    wind = wind.with(wind_data);

    let lod = lod.map_or(current_lod_level.unwrap_or_default(), |x| *x);

    let name = current_name.map_or(name.cloned(), Some);

    let hue = (entity.index() * 30) as f32 % 360.0;
    let debug_color = Color::hsl(hue, 1.0, 0.5);

    let mut options = options
        .with(material_option_data)
        .with_debug_color(debug_color);

    if !controlled {
        options = options.with_quality(*lod, options.wind_affected);
    };

    let mut has_children_with_materials = false;
    if let Some(children) = children {
        for child in children {
            let found_material = queue_requests_recursive::<TOut, TIn>(
                layer,
                *child,
                cmd,
                &wind,
                &options,
                name.clone(),
                Some(lod),
                q_children,
            );

            if found_material {
                has_children_with_materials = true;
            }
        }
    }

    if has_children_with_materials {
        cmd.entity(entity).insert(ScatterLayerChildProcessed);
    }

    let (Some(material), Some(mesh), Some(aabb)) = (material, mesh, aabb) else {
        return has_children_with_materials;
    };

    cmd.entity(entity)
        .insert(ScatterMaterialCreationRequest::<TOut, TIn>::new(
            material.0.clone(),
            ScatterAssetProperties {
                wind,
                options,
                mesh_handle: mesh.0.clone(),
                aabb: *aabb,
                name,
                lod_level: lod,
                layer,
                wind_affected: wind_affected.is_some(),
            },
        ));

    true
}

pub fn process_distinct_material_requests<TOut, TIn>(
    mut cmd: Commands,
    mut requests_query: Query<(Entity, &mut ScatterMaterialCreationRequest<TOut, TIn>)>,
    materials_in: Res<Assets<TIn>>,
    mut materials_out: ResMut<Assets<TOut>>,
    wind_noise_texture: Res<WindTexture>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut prototype_assets: ResMut<Assets<ScatterAsset<TOut>>>,
) where
    TIn: Material,
    TOut: ScatterMaterial<TIn> + Asset + Clone,
{
    for (entity, mut request) in &mut requests_query {
        let source_material = materials_in.get(&request.source_material_handle);

        let new_material = TOut::create_material(
            source_material.cloned(),
            wind_noise_texture.0.clone(),
            &request.properties,
        );
        let material_handle = materials_out.add(new_material);

        let mesh = meshes
            .get(&request.properties.mesh_handle)
            .cloned()
            .unwrap();
        let mesh_handle = meshes.add(mesh);
        let mesh_aabb = meshes.get(&mesh_handle).unwrap().compute_aabb().unwrap();
        request.properties.aabb = mesh_aabb;

        let asset = ScatterAsset::new(material_handle, &request);

        let asset_handle = prototype_assets.add(asset.clone());

        cmd.entity(entity)
            .remove::<ScatterMaterialCreationRequest<TOut, TIn>>()
            .remove::<MeshMaterial3d<TIn>>();

        insert_bundle::<TOut>(&mut cmd, entity, asset_handle, asset, &request.properties);
    }
}

pub fn process_same_type_material_requests<T>(
    mut cmd: Commands,
    mut requests: Query<(Entity, &mut ScatterMaterialCreationRequest<T, T>)>,
    mut materials: ResMut<Assets<T>>,
    wind_noise_texture: Res<WindTexture>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut prototype_assets: ResMut<Assets<ScatterAsset<T>>>,
) where
    T: ScatterMaterial<T> + Material + Clone,
{
    for (entity, mut request) in &mut requests {
        let source_material = materials.get(&request.source_material_handle).cloned();

        let new_material = T::create_material(
            source_material,
            wind_noise_texture.0.clone(),
            &request.properties,
        );
        let material_handle = materials.add(new_material);

        let mesh = meshes
            .get(&request.properties.mesh_handle)
            .cloned()
            .unwrap();
        let mesh_handle = meshes.add(mesh);
        let mesh_aabb = meshes.get(&mesh_handle).unwrap().compute_aabb().unwrap();

        request.properties.aabb = mesh_aabb;

        let asset = ScatterAsset::new(material_handle, &request);

        let asset_handle = prototype_assets.add(asset.clone());

        cmd.entity(entity)
            .remove::<ScatterMaterialCreationRequest<T, T>>();

        insert_bundle::<T>(&mut cmd, entity, asset_handle, asset, &request.properties);
    }
}

fn insert_bundle<'a, T>(
    cmd: &'a mut Commands,
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
