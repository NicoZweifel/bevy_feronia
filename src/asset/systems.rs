use crate::prelude::*;
use bevy::prelude::*;
use bevy::render::primitives::MeshAabb;

pub fn collect_assets<TIn, TOut>(
    mut cmd: Commands,
    q_roots: Query<(Entity, &ScatterRoot), Without<ScatterRootProcessed>>,
    q_layers: Query<
        &Children,
        (
            With<ScatterLayer>,
            Without<ScatterLayerProcessed>,
            With<ScatterLayerType<TIn, TOut>>,
        ),
    >,
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
    wind_noise_texture: Res<WindTexture>,
    wind: Res<Wind>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut prototype_assets: ResMut<Assets<ScatterAsset<TOut>>>,
) where
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
{
    for (root, children) in &q_roots {
        debug!("Collecting ScatterAssets in root {:?}...", root);

        for layer in children.iter() {
            let Ok(scatter_items) = q_layers.get(layer) else {
                continue;
            };

            let result = scatter_items
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
                debug!("Found {} assets in layer {:?}", result.len(), layer);
                cmd.entity(layer).insert(ScatterLayerProcessed);
            }
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
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
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

    debug!(
        "Adding asset {:?} lod_level {:?}",
        asset.name, asset.lod_level
    );

    let asset_handle = prototype_assets.add(asset);

    cmd.entity(entity)
        .remove::<MeshMaterial3d<TIn>>()
        .insert((WindAffectedRegistered(asset_handle.clone()), WindAffected));

    cmd.spawn((
        ScatterItem,
        ScatterItemAsset::<TOut>(asset_handle.clone()),
        lod_level,
        ChildOf(layer),
        ScatterItemOf(layer),
    ));

    types.push(asset_handle);

    types
}
