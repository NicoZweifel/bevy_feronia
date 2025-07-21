use crate::prelude::*;
use bevy::asset::{Asset, Assets};
use bevy::image::{Image, ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::mesh::{Mesh, Mesh3d};
use bevy::pbr::{Material, MeshMaterial3d};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use noise::{NoiseFn, Perlin};

fn create_material<M: Material, W: WindAffectable<M, W> + Asset>(
    cmd: &mut Commands,
    materials: &mut ResMut<Assets<M>>,
    extended_materials: &mut ResMut<Assets<W>>,
    (entity, material, mesh): (Entity, &MeshMaterial3d<M>, &Mesh3d),
    wind_noise_texture: &Res<WindTexture>,
    wind: &Res<Wind>,
    meshes: &mut ResMut<Assets<Mesh>>,
) -> WindAffectedType<W> {
    let new_material = W::create_material(
        (*materials.get(material).unwrap()).clone(),
        (*wind).clone(),
        wind_noise_texture.0.clone(),
    );

    let material = extended_materials.add(new_material);
    let mesh = meshes.get(mesh).cloned().unwrap();
    let mesh = meshes.add(mesh.clone());

    cmd.entity(entity).despawn();
    /*
    cmd.entity(entity)
        .remove::<MeshMaterial3d<StandardMaterial>>()
        .insert(W::create_material_component(material.clone()));
     */

    WindAffectedType {
        mesh,
        material,
        wind: (*wind).clone(),
    }
}

pub fn update_materials<M: Material, W: WindAffectable<M, W> + Asset>(
    materials: ResMut<Assets<W>>,
    wind: Res<Wind>,
) {
    W::update_material(materials, wind.clone());
}

pub fn setup_wind_affected<M: Material, W: WindAffectable<M, W> + Asset>(
    mut cmd: Commands,
    q: Query<(Entity,&MeshMaterial3d<M>, &Mesh3d), (With<WindAffected>, Without<WindAffectedReady>)>,
    mut materials: ResMut<Assets<M>>,
    mut extended_materials: ResMut<Assets<W>>,
    mut types: ResMut<WindAffectedTypes<W>>,
    wind_noise_texture: Res<WindTexture>,
    wind: Res<Wind>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
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

pub fn setup_wind_texture(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let texture_size = 512;
    let mut image_buffer = Vec::with_capacity((texture_size * texture_size) as usize);

    let perlin = Perlin::new(1);

    for y in 0..texture_size {
        for x in 0..texture_size {
            let sample_scale = 5.0;
            let point = [
                x as f64 / texture_size as f64 * sample_scale,
                y as f64 / texture_size as f64 * sample_scale,
            ];

            let noise_value = perlin.get(point);

            let byte = ((noise_value * 0.5 + 0.5) * 255.0) as u8;
            image_buffer.push(byte);
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
        TextureFormat::R8Unorm,
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
