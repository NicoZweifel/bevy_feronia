use std::marker::PhantomData;

use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use noise::{NoiseFn, Perlin};

mod extension;
pub mod prelude;
use prelude::*;

pub struct WindPlugin<M: Material, W: WindAffectable<M, W> + Material> {
    pub _marker: PhantomData<(M, W)>,
}

impl<M: Material, W: WindAffectable<M, W> + Material> Default for WindPlugin<M, W> {
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<M: Material, W: WindAffectable<M, W> + Material> Plugin for WindPlugin<M, W> {
    fn build(&self, app: &mut App) {
        app.init_resource::<Wind>()
            .register_type::<Wind>()
            .init_resource::<WindAffectedTypes<W>>()
            .add_systems(Startup, setup_wind_texture)
            .add_systems(
                Update,
                (setup_wind_affected::<M, W>, update_materials::<M, W>),
            );
    }
}

fn create_material<M: Material, W: WindAffectable<M, W> + Material>(
    cmd: &mut Commands,
    materials: &mut ResMut<Assets<M>>,
    extended_materials: &mut ResMut<Assets<W>>,
    (entity, material, mesh): (Entity, &MeshMaterial3d<M>, &Mesh3d),
    wind_noise_texture: &Res<WindTexture>,
    wind: &Res<Wind>,
) -> WindAffectedType<W> {
    let base = materials.get(material).unwrap();

    let new_material = W::create_material(
        (*base).clone(),
        (*wind).clone(),
        wind_noise_texture.0.clone(),
    );

    let material = extended_materials.add(new_material);

    cmd.entity(entity)
        .remove::<MeshMaterial3d<StandardMaterial>>()
        .insert((MeshMaterial3d(material.clone()), WindAffectedReady));

    WindAffectedType {
        mesh: mesh.0.clone(),
        material,
        wind: (*wind).clone(),
    }
}

fn update_materials<M: Material, W: WindAffectable<M, W> + Material>(
    materials: ResMut<Assets<W>>,
    wind: Res<Wind>,
) {
    W::update_material(materials, wind.clone());
}

fn setup_wind_affected<M: Material, W: WindAffectable<M, W> + Material>(
    mut cmd: Commands,
    q: Query<
        (Entity, &MeshMaterial3d<M>, &Mesh3d),
        (With<WindAffected>, Without<WindAffectedReady>),
    >,
    mut materials: ResMut<Assets<M>>,
    mut extended_materials: ResMut<Assets<W>>,
    mut types: ResMut<WindAffectedTypes<W>>,
    wind_noise_texture: Res<WindTexture>,
    wind: Res<Wind>,
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
                )
            })
            .collect(),
    );
}

fn setup_wind_texture(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
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
