use crate::prelude::*;
use bevy::asset::{Asset, Assets};
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::pbr::{Material, MeshMaterial3d};
use bevy::prelude::*;
use bevy::render::primitives::MeshAabb;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use noise::{NoiseFn, Perlin};

pub fn update_materials<B, T>(materials: ResMut<Assets<T>>, wind: Res<Wind>)
where
    B: Material,
    T: WindAffectable<B, T, WindAffectedTypes<T>, WindAffectedType<T>> + Asset + Clone,
{
    T::update_material(materials, wind.clone());
}

pub fn collect_types<B, T>(
    mut cmd: Commands,
    q_affected: Query<
        (
            Entity,
            Option<&MeshMaterial3d<B>>,
            Option<&Mesh3d>,
            Option<&Children>,
            Option<&WindConfig>,
        ),
        (With<WindAffected>, Without<WindAffectedRegistered<T>>),
    >,
    q_children: Query<(
        Entity,
        Option<&MeshMaterial3d<B>>,
        Option<&Mesh3d>,
        Option<&Children>,
        Option<&WindConfig>,
    )>,
    mut materials: ResMut<Assets<B>>,
    mut extended_materials: ResMut<Assets<T>>,
    mut types: ResMut<WindAffectedTypes<T>>,
    wind_noise_texture: Res<WindTexture>,
    wind: Res<Wind>,
    mut meshes: ResMut<Assets<Mesh>>,
) where
    B: Material,
    T: WindAffectable<B, T, WindAffectedTypes<T>, WindAffectedType<T>> + Asset + Clone,
{
    types.values.append(
        &mut q_affected
            .iter()
            .map(|x| {
                collect_types_recursive::<B, T>(
                    &mut cmd,
                    &mut materials,
                    &mut extended_materials,
                    x,
                    &wind_noise_texture,
                    &wind,
                    &mut meshes,
                    &q_children,
                )
            })
            .flatten()
            .collect(),
    );
}

pub fn insert_material<B, T>(
    mut cmd: Commands,
    q: Query<(Entity, &WindAffectedRegistered<T>), Without<WindAffectedReady>>,
) where
    B: Material,
    T: WindAffectable<B, T, WindAffectedTypes<T>, WindAffectedType<T>> + Asset + Clone,
{
    for (entity, wind_affected) in &q {
        cmd.entity(entity).insert((
            T::component(wind_affected.get().material),
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

fn collect_types_recursive<B, T>(
    cmd: &mut Commands,
    materials: &mut ResMut<Assets<B>>,
    extended_materials: &mut ResMut<Assets<T>>,
    (entity, material, mesh, children, wind_component): (
        Entity,
        Option<&MeshMaterial3d<B>>,
        Option<&Mesh3d>,
        Option<&Children>,
        Option<&WindConfig>,
    ),
    wind_noise_texture: &Res<WindTexture>,
    wind: &Res<Wind>,
    meshes: &mut ResMut<Assets<Mesh>>,
    q_children: &Query<(
        Entity,
        Option<&MeshMaterial3d<B>>,
        Option<&Mesh3d>,
        Option<&Children>,
        Option<&WindConfig>,
    )>,
) -> Vec<WindAffectedType<T>>
where
    B: Material,
    T: WindAffectable<B, T, WindAffectedTypes<T>, WindAffectedType<T>> + Asset + Clone,
{
    let mut types: Vec<WindAffectedType<T>> = Vec::new();

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

    if let Some(children) = children {
        for child in children.iter() {
            let Ok(x) = q_children.get(child) else {
                continue;
            };

            types.append(&mut collect_types_recursive::<B, T>(
                cmd,
                materials,
                extended_materials,
                x,
                wind_noise_texture,
                wind,
                meshes,
                &q_children,
            ));
        }
    }

    let Some(material) = material else {
        return types;
    };

    let Some(mesh) = mesh else { return types };

    let new_material = T::create_material(
        Some(materials.get(material).unwrap().clone()),
        final_wind.clone(),
        wind_noise_texture.0.clone(),
        wind_config.wind_override.is_some(),
    );

    let material = extended_materials.add(new_material);
    let mesh = meshes.get(mesh).cloned().unwrap();
    let mesh = meshes.add(mesh.clone());
    let mesh_aabb = meshes.get(&mesh).unwrap().compute_aabb().unwrap();

    let wind_type = WindAffectedType {
        mesh,
        material,
        wind: final_wind,
        aabb: mesh_aabb,
    };

    cmd.entity(entity)
        .remove::<MeshMaterial3d<B>>()
        .insert(WindAffectedRegistered(wind_type.clone()));

    types.push(wind_type);
    types
}
