use crate::prelude::*;
use bevy::camera::primitives::{Aabb, MeshAabb};
use bevy::prelude::*;

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
    q_collect: Query<
        (
            Entity,
            Option<&MeshMaterial3d<TIn>>,
            Option<&Mesh3d>,
            Option<&Aabb>,
            Option<&Children>,
            Option<&WindConfig>,
            Option<&Name>,
            Option<&LevelOfDetail>,
            Option<&WindAffected>,
        ),
        Without<ScatterLayerChildProcessed>,
    >,
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
                .flat_map(|x| {
                    collect_assets_recursive::<TIn, TOut>(
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
                .collect::<Vec<_>>();

            if result.is_empty() {
                continue;
            };

            debug!("Found {} assets in layer {:?}", result.len(), layer);
        }
    }
}

fn collect_assets_recursive<TIn, TOut>(
    layer: Entity,
    entity: Entity,
    cmd: &mut Commands,
    materials: &mut ResMut<Assets<TIn>>,
    extended_materials: &mut ResMut<Assets<TOut>>,
    wind_noise_texture: &Res<WindTexture>,
    wind: &Res<Wind>,
    current_name: Option<Name>,
    current_lod_level: Option<LevelOfDetail>,
    scatter_assets: &mut ResMut<Assets<ScatterAsset<TOut>>>,
    meshes: &mut ResMut<Assets<Mesh>>,
    q_children: &Query<
        (
            Entity,
            Option<&MeshMaterial3d<TIn>>,
            Option<&Mesh3d>,
            Option<&Aabb>,
            Option<&Children>,
            Option<&WindConfig>,
            Option<&Name>,
            Option<&LevelOfDetail>,
            Option<&WindAffected>,
        ),
        Without<ScatterLayerChildProcessed>,
    >,
) -> Vec<Handle<ScatterAsset<TOut>>>
where
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
{
    let mut types: Vec<Handle<ScatterAsset<TOut>>> = Vec::new();

    // TODO only add displacement/wind affected materials if wind affected
    let Ok((entity, material, mesh, aabb, children, wind_component, name, lod, _wind_affected)) =
        q_children.get(entity)
    else {
        return types;
    };

    let (final_wind, controlled) = wind_component
        .and_then(|x| x.wind_override.clone().map(|x| (x.clone(), true)))
        .unwrap_or_else(|| ((*wind).clone(), false));

    let lod = lod.map_or(current_lod_level.unwrap_or_default(), |x| *x);

    let name = current_name.map_or(name.cloned(), Some);

    if let Some(children) = children {
        for child in children.iter() {
            types.append(&mut collect_assets_recursive::<TIn, TOut>(
                layer,
                child,
                cmd,
                materials,
                extended_materials,
                wind_noise_texture,
                wind,
                name.clone(),
                Some(lod),
                scatter_assets,
                meshes,
                q_children,
            ));
        }
    }

    if !types.is_empty() {
        cmd.entity(entity).insert(ScatterLayerChildProcessed);
    }

    let Some(material) = material else {
        return types;
    };

    let Some(mesh) = mesh else {
        return types;
    };

    let Some(aabb) = aabb else { return types };

    let hue = (entity.index() * 30) as f32 % 360.0;
    let unique_color = Color::hsl(hue, 1.0, 0.5);

    let new_material = TOut::create_material(
        Some(materials.get(material).unwrap().clone()),
        final_wind.clone(),
        wind_noise_texture.0.clone(),
        controlled,
        *aabb,
        unique_color,
        // TODO: expose with setting
        true,
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
        lod_level: lod,
    };

    debug!(
        "Adding asset {:?} lod_level {:?}",
        asset.name, asset.lod_level
    );

    let asset_handle = scatter_assets.add(asset);

    cmd.entity(entity).remove::<MeshMaterial3d<TIn>>().insert((
        // TODO only do this and ignore scatter item logic (some assets might not ever be scattered and just need to be affected by wind).
        WindAffectedRegistered(asset_handle.clone()),
        WindAffected,
        ScatterItem,
        ScatterItemAsset::<TOut>(asset_handle.clone()),
        lod,
        ChildOf(layer),
        ScatterItemOf(layer),
        ScatterLayerChildProcessed,
    ));

    types.push(asset_handle);

    types
}
