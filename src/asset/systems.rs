use crate::prelude::*;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::render::primitives::MeshAabb;

pub fn sync_asset_name_map<T: Asset + Clone + std::fmt::Debug>(
    types: Res<ScatterAssets<T>>,
    mut name_map: ResMut<ScatterAssetsNameMap<T>>,
    assets: Res<Assets<ScatterAsset<T>>>,
) {
    // TODO invalidate instead of syncing
    name_map.clear();

    info!("Syncing ScatterAssets name map...");

    types
        .iter()
        .filter_map(|handle| {
            assets
                .get(handle)
                .map(|scatter_asset| (scatter_asset, handle))
        })
        .filter_map(|(asset, handle)| {
            asset
                .name
                .clone()
                .map(|name| (name, asset.lod_level, handle.clone()))
        })
        .filter_map(|(name, lvl, handle)| {
            info!("Adding {:?} to name map", (&name, lvl));
            name_map
                .get_mut(&name)
                .map(|x| x.insert(lvl, handle.clone()))
                .map(|_| (name.clone(), lvl))
                .or_else(|| {
                    // Note: returns None even though insertion was successful
                    name_map.insert(name.clone(), HashMap::from([(lvl, handle.clone())]));
                    Some((name.clone(), lvl))
                })
        })
        .filter(|_| true)
        .for_each(|x| {
            info!("Added {:?} to name map", x);
        });
}

pub fn collect_assets<TIn, TOut>(
    mut cmd: Commands,
    q_roots: Query<(Entity, &ScatterRoot), Without<ScatterRootReady>>,
    q_layers: Query<&Children, (With<ScatterLayer>, Without<ScatterLayerProcessed>)>,
    q_collect: Query<(
        Entity,
        Option<&MeshMaterial3d<TIn>>,
        Option<&Mesh3d>,
        Option<&Children>,
        Option<&WindConfig>,
        Option<&Name>,
        Option<&LodLevel>,
        Option<&WindAffected>,
    )>,
    mut materials: ResMut<Assets<TIn>>,
    mut extended_materials: ResMut<Assets<TOut>>,
    mut types: ResMut<ScatterAssets<TOut>>,
    wind_noise_texture: Res<WindTexture>,
    wind: Res<Wind>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut prototype_assets: ResMut<Assets<ScatterAsset<TOut>>>,
) where
    TIn: Material,
    TOut: WindAffectable<ScatterAssets<TOut>, ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
{
    for (root, children) in &q_roots {
        info!("Collecting ScatterAssets in root {:?}...", root);

        for layer in children.iter() {
            let Ok(scatter_items) = q_layers.get(layer) else {
                continue;
            };

            let mut result = scatter_items
                .iter()
                .map(|x| {
                    collect_assets_recursive::<TIn, TOut>(
                        root,
                        layer,
                        x,
                        &mut cmd,
                        &mut materials,
                        &mut extended_materials,
                        &wind_noise_texture,
                        &wind,
                        None,
                        None,
                        &mut prototype_assets,
                        &mut meshes,
                        &q_collect,
                    )
                })
                .flatten()
                .collect::<Vec<_>>();

            if result.len() > 0 {
                cmd.entity(layer).insert(ScatterLayerProcessed);
                cmd.entity(root).insert(ScatterRootReady);
            }

            (**types).append(&mut result);
        }
    }
}

fn collect_assets_recursive<TIn, TOut>(
    root: Entity,
    layer: Entity,
    entity: Entity,
    cmd: &mut Commands,
    materials: &mut ResMut<Assets<TIn>>,
    extended_materials: &mut ResMut<Assets<TOut>>,
    wind_noise_texture: &Res<WindTexture>,
    wind: &Res<Wind>,
    current_name: Option<Name>,
    current_lod_level: Option<LodLevel>,
    prototype_assets: &mut ResMut<Assets<ScatterAsset<TOut>>>,
    meshes: &mut ResMut<Assets<Mesh>>,
    q_children: &Query<(
        Entity,
        Option<&MeshMaterial3d<TIn>>,
        Option<&Mesh3d>,
        Option<&Children>,
        Option<&WindConfig>,
        Option<&Name>,
        Option<&LodLevel>,
        Option<&WindAffected>,
    )>,
) -> Vec<Handle<ScatterAsset<TOut>>>
where
    TIn: Material,
    TOut: WindAffectable<ScatterAssets<TOut>, ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
{
    let mut types: Vec<Handle<ScatterAsset<TOut>>> = Vec::new();

    let Ok((entity, material, mesh, children, wind_component, name, lod_level, wind_affected)) =
        q_children.get(entity)
    else {
        return types;
    };

    let (final_wind, controlled) = wind_component
        .map(|x| x.wind_override.clone().map(|x| (x.clone(), true)))
        .flatten()
        .unwrap_or_else(|| ((*wind).clone(), false));

    let lod_level = lod_level.map_or(
        current_lod_level.unwrap_or_else(|| LodLevel::default()),
        |x| x.clone(),
    );

    let name = current_name.map_or_else(|| name.map(|x| x.clone()), |x| Some(x.clone()));

    if let Some(children) = children {
        for child in children.iter() {
            let Ok(x) = q_children.get(child) else {
                continue;
            };

            types.append(&mut collect_assets_recursive::<TIn, TOut>(
                root,
                layer,
                child,
                cmd,
                materials,
                extended_materials,
                wind_noise_texture,
                wind,
                name.clone(),
                Some(lod_level),
                prototype_assets,
                meshes,
                &q_children,
            ));
        }
    }

    let Some(material) = material else {
        return types;
    };

    let Some(mesh) = mesh else {
        return types;
    };

    let new_material = TOut::create_material(
        Some(materials.get(material).unwrap().clone()),
        final_wind.clone(),
        wind_noise_texture.0.clone(),
        controlled,
    );

    let material = extended_materials.add(new_material);
    let mesh = meshes.get(mesh).cloned().unwrap();
    let mesh = meshes.add(mesh.clone());
    let mesh_aabb = meshes.get(&mesh).unwrap().compute_aabb().unwrap();

    let asset = ScatterAsset {
        mesh,
        material,
        wind: Some(final_wind),
        aabb: mesh_aabb,
        name: name.clone(),
        lod_level,
    };

    let asset_handle = prototype_assets.add(asset);

    cmd.entity(entity)
        .remove::<MeshMaterial3d<TIn>>()
        .insert((WindAffectedRegistered(asset_handle.clone()), WindAffected));

    let scatter_item = cmd
        .spawn((
            ScatterItem,
            name.clone().map_or_else(
                || ScatterItemType::<TOut>::Handle(asset_handle.clone()),
                |x| ScatterItemType::<TOut>::Name(x),
            ),
            ChildOf(layer),
            ScatterItemOf(layer),
        ))
        .id();

    if let Some(name) = name {
        cmd.entity(scatter_item).insert(name);
    }

    types.push(asset_handle);

    types
}
