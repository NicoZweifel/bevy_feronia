use crate::prelude::*;
use bevy::asset::{Asset, Assets};
use bevy::image::{Image, ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::mesh::{Mesh, Mesh3d};
use bevy::pbr::{Material, MeshMaterial3d};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use noise::{NoiseFn, Perlin};

fn create_material<M, W>(
    cmd: &mut Commands,
    materials: &mut ResMut<Assets<M>>,
    extended_materials: &mut ResMut<Assets<W>>,
    (entity, material, mesh, wind_component): (
        Entity,
        &MeshMaterial3d<M>,
        &Mesh3d,
        Option<&WindConfig>,
    ),
    wind_noise_texture: &Res<WindTexture>,
    wind: &Res<Wind>,
    meshes: &mut ResMut<Assets<Mesh>>,
) -> WindAffectedType<W>
where
    M: Material,
    W: WindAffectable<M, W> + Asset + Clone,
{
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

    let new_material = W::create_material(
        (*materials.get(material).unwrap()).clone(),
        final_wind.clone(),
        wind_noise_texture.0.clone(),
        wind_config.wind_override.is_some(),
    );

    let material = extended_materials.add(new_material);
    let mesh = meshes.get(mesh).cloned().unwrap();
    let mesh = meshes.add(mesh.clone());

    let wind_type = WindAffectedType {
        mesh,
        material,
        wind: final_wind,
    };

    // NOTE: replacing materials and scattering new bindless ones with batch_spawn will cause a but with instance_index
    // TODO
    cmd.entity(entity).despawn();
   /* cmd.entity(entity)
        .remove::<MeshMaterial3d<M>>()
        .insert(WindAffectedRegistered(wind_type.clone()));*/

    wind_type
}

pub fn update_materials<M, W>(
    materials: ResMut<Assets<W>>,
    wind: Res<Wind>,
) where
    M: Material,
    W: WindAffectable<M, W> + Asset
{
    W::update_material(materials, wind.clone());
}

pub fn collect_types<M, W>(
    mut cmd: Commands,
    q: Query<
        (Entity, &MeshMaterial3d<M>, &Mesh3d, Option<&WindConfig>),
        (With<WindAffected>, Without<WindAffectedRegistered<W>>),
    >,
    mut materials: ResMut<Assets<M>>,
    mut extended_materials: ResMut<Assets<W>>,
    mut types: ResMut<WindAffectedTypes<W>>,
    wind_noise_texture: Res<WindTexture>,
    wind: Res<Wind>,
    mut meshes: ResMut<Assets<Mesh>>,
) where
    M: Material,
    W: WindAffectable<M, W> + Asset + Clone
{
    types.values.append(
        &mut q
            .iter()
            .map(|x| {
                create_material::<M, W>(
                    &mut cmd,
                    &mut materials,
                    &mut extended_materials,
                    x,
                    &wind_noise_texture,
                    &wind,
                    &mut meshes,
                )
            })
            .collect(),
    );
}

pub fn insert_material<M, W>(
    mut cmd: Commands,
    q: Query<
        (Entity,&WindAffectedRegistered<W>),
        Without<WindAffectedReady>,
    >,
) where
    M: Material,
    W: WindAffectable<M, W> + Asset + Clone
{
    for (entity,wind_affected) in &q{

        cmd.entity(entity)
            .insert((W::component(wind_affected.get().material), WindAffectedReady));
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

            // Sample both noise patterns
            let macro_noise_value = macro_perlin.get([point[0] * macro_sample_scale, point[1] * macro_sample_scale]);
            let micro_noise_value = micro_perlin.get([point[0] * micro_sample_scale, point[1] * micro_sample_scale]);

            // Normalize to the 0.0-1.0 range and convert to u8
            let macro_byte = ((macro_noise_value * 0.5 + 0.5) * 255.0) as u8;
            let micro_byte = ((micro_noise_value * 0.5 + 0.5) * 255.0) as u8;

            // Push the bytes into the buffer: R, G, B, A
            image_buffer.push(macro_byte); // R channel for macro noise
            image_buffer.push(micro_byte); // G channel for micro noise
            image_buffer.push(0);          // B channel is unused
            image_buffer.push(255);        // A channel is unused
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
