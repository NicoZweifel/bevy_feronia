use crate::prelude::*;
use bevy::asset::{Asset, Assets};
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::pbr::{Material, MeshMaterial3d};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::render::primitives::MeshAabb;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use noise::{NoiseFn, Perlin};

pub fn update_materials<TIn, TOut>(materials: ResMut<Assets<TOut>>, wind: Res<Wind>)
where
    TIn: Material,
    TOut: WindAffectable<ScatterAssets<TOut>, ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
{
    TOut::update_material(materials, wind.clone());
}

pub fn update_name_map<T: Asset + Clone + std::fmt::Debug>(
    types: Res<ScatterAssets<T>>,
    mut name_map: ResMut<ScatterAssetsNameMap<T>>,
    assets: Res<Assets<ScatterAsset<T>>>,
) {
    name_map.clear();

    info!("Updating ScatterAssets name map...");

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

pub fn collect_types<TIn, TOut>(
    mut cmd: Commands,
    q_collect: Query<
        (
            Entity,
            Option<&MeshMaterial3d<TIn>>,
            Option<&Mesh3d>,
            Option<&Children>,
            Option<&WindConfig>,
            Option<&Name>,
            Option<&LodLevel>,
        ),
        (
            With<WindAffected>,
            Without<WindAffectedRegistered<TOut>>,
            Without<WindAffectedReady>,
        ),
    >,
    q_children: Query<
        (
            Entity,
            Option<&MeshMaterial3d<TIn>>,
            Option<&Mesh3d>,
            Option<&Children>,
            Option<&WindConfig>,
            Option<&Name>,
            Option<&LodLevel>,
        ),
        (
            Without<WindAffectedRegistered<TOut>>,
            Without<WindAffectedReady>,
        ),
    >,
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
    if q_collect.iter().peekable().peek().is_none() {
        return;
    };

    info!("Collecting ScatterAssets...");

    (**types).append(
        &mut q_collect
            .iter()
            .map(|x| {
                collect_types_recursive::<TIn, TOut>(
                    x.0,
                    &mut cmd,
                    &mut materials,
                    &mut extended_materials,
                    x,
                    &wind_noise_texture,
                    &wind,
                    None,
                    None,
                    &mut prototype_assets,
                    &mut meshes,
                    &q_children,
                )
            })
            .flatten()
            .collect(),
    );
}

pub fn insert_material<TIn, TOut>(
    mut cmd: Commands,
    q: Query<(Entity, &WindAffectedRegistered<TOut>), Without<WindAffectedReady>>,
) where
    TIn: Material,
    TOut: WindAffectable<ScatterAssets<TOut>, ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
{
    for (entity, wind_affected) in &q {
        info!("Replacing Material with WindAffected material...");

        cmd.entity(entity).insert((
            TOut::component(wind_affected.get().material),
            WindAffectedReady,
        ));
    }
}

pub fn setup_wind_texture(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let texture_size = 512;
    let mut image_buffer = Vec::with_capacity((texture_size * texture_size * 4) as usize);

    let macro_perlin = Perlin::new(1);
    let micro_perlin = Perlin::new(2);

    for y in 0..texture_size {
        for x in 0..texture_size {
            let macro_sample_scale = 5.0;
            let micro_sample_scale = 20.0;
            let point = [
                x as f64 / texture_size as f64,
                y as f64 / texture_size as f64,
            ];

            let macro_noise_value =
                macro_perlin.get([point[0] * macro_sample_scale, point[1] * macro_sample_scale]);
            let micro_noise_value =
                micro_perlin.get([point[0] * micro_sample_scale, point[1] * micro_sample_scale]);

            let macro_byte = ((macro_noise_value * 0.5 + 0.5) * 255.0) as u8;
            let micro_byte = ((micro_noise_value * 0.5 + 0.5) * 255.0) as u8;

            image_buffer.push(macro_byte); // R channel for macro noise
            image_buffer.push(micro_byte); // G channel for micro noise
            image_buffer.push(0); // B channel is unused
            image_buffer.push(255); // A channel is unused
        }
    }

    let mut wind_image = Image::new(
        Extent3d {
            width: texture_size,
            height: texture_size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        image_buffer,
        TextureFormat::Rgba8Unorm,
        default(),
    );

    let sampler_descriptor = ImageSampler::Descriptor(ImageSamplerDescriptor {
        label: Some("Wind Noise Sampler".into()),
        address_mode_u: ImageAddressMode::MirrorRepeat,
        address_mode_v: ImageAddressMode::MirrorRepeat,
        address_mode_w: ImageAddressMode::MirrorRepeat,
        ..default()
    });

    wind_image.sampler = sampler_descriptor;

    let handle = images.add(wind_image);

    commands.insert_resource(WindTexture(handle));
}

fn collect_types_recursive<TIn, TOut>(
    root: Entity,
    cmd: &mut Commands,
    materials: &mut ResMut<Assets<TIn>>,
    extended_materials: &mut ResMut<Assets<TOut>>,
    (entity, material, mesh, children, wind_component, name, lod_level): (
        Entity,
        Option<&MeshMaterial3d<TIn>>,
        Option<&Mesh3d>,
        Option<&Children>,
        Option<&WindConfig>,
        Option<&Name>,
        Option<&LodLevel>,
    ),
    wind_noise_texture: &Res<WindTexture>,
    wind: &Res<Wind>,
    current_name: Option<Name>,
    current_lod_level: Option<LodLevel>,
    prototype_assets: &mut ResMut<Assets<ScatterAsset<TOut>>>,
    meshes: &mut ResMut<Assets<Mesh>>,
    q_children: &Query<
        (
            Entity,
            Option<&MeshMaterial3d<TIn>>,
            Option<&Mesh3d>,
            Option<&Children>,
            Option<&WindConfig>,
            Option<&Name>,
            Option<&LodLevel>,
        ),
        (
            Without<WindAffectedRegistered<TOut>>,
            Without<WindAffectedReady>,
        ),
    >,
) -> Vec<Handle<ScatterAsset<TOut>>>
where
    TIn: Material,
    TOut: WindAffectable<ScatterAssets<TOut>, ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
{
    let mut types: Vec<Handle<ScatterAsset<TOut>>> = Vec::new();

    let wind_config = if let Some(wind_component) = wind_component {
        wind_component
    } else {
        &WindConfig::default()
    };

    let final_wind = if let Some(wind) = &wind_config.wind_override {
        wind.clone()
    } else {
        (*wind).clone()
    };

    let lod_level = if let Some(lod_level) = lod_level {
        lod_level.clone()
    } else {
        current_lod_level.unwrap_or_else(|| LodLevel::default())
    };

    let name = if let Some(name) = current_name {
        Some(name.clone())
    } else {
        name.map(|x| x.clone())
    };

    if let Some(children) = children {
        for child in children.iter() {
            let Ok(x) = q_children.get(child) else {
                continue;
            };

            types.append(&mut collect_types_recursive::<TIn, TOut>(
                root,
                cmd,
                materials,
                extended_materials,
                x,
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
        wind_config.wind_override.is_some(),
    );

    let material = extended_materials.add(new_material);
    let mesh = meshes.get(mesh).cloned().unwrap();
    let mesh = meshes.add(mesh.clone());
    let mesh_aabb = meshes.get(&mesh).unwrap().compute_aabb().unwrap();

    let asset = ScatterAsset {
        mesh,
        material,
        wind: final_wind,
        aabb: mesh_aabb,
        name,
        lod_level,
    };

    cmd.entity(entity).remove::<MeshMaterial3d<TIn>>().insert((
        WindAffectedRegistered(asset.clone()),
        WindAffected,
        WindAffectedReady,
    ));

    cmd.entity(root).insert(WindAffectedReady);

    types.push(prototype_assets.add(asset));

    types
}
