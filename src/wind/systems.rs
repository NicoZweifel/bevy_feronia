use crate::prelude::*;

use bevy_asset::Assets;
use bevy_ecs::prelude::*;
use bevy_image::*;
use bevy_render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_utils::default;
use noise::{NoiseFn, Perlin};

pub fn update_materials<T>(
    mut materials: ResMut<Assets<T>>,
    wind: Res<Wind>,
    mut scatter_assets: ResMut<Assets<ScatterAsset<T>>>,
    q_layer: Query<
        (WindOptionData, MaterialOptionData, &ScatterLayerOf),
        (With<ScatterLayer>, With<ScatterLayerType<T>>),
    >,
    q_root: Query<WindOptionData, With<ScatterRoot>>,
) where
    T: ScatterMaterial,
{
    for (_, asset) in scatter_assets.iter_mut() {
        if asset.properties.options.controlled {
            continue;
        };

        #[allow(deprecated)]
        let Some((wind_data, _material_options, root)) =
            asset.properties.layer.and_then(|x| q_layer.get(x).ok())
        else {
            dbg!("ScatterLayer not found!");
            continue;
        };

        let Ok(root_wind_data) = q_root.get(**root) else {
            dbg!("ScatterRoot not found!");
            continue;
        };

        let wind = wind.with(root_wind_data).with(wind_data);

        asset.properties.wind = wind;

        for part in &asset.parts {
            let Some(material) = materials.get_mut(&part.h_material) else {
                dbg!("Material not found!");
                continue;
            };

            // TODO update options
            /*
            let options = MaterialOptions::from(root_material_options)
                .with(material_options)
                .with_options(asset.properties.options)
                .with_quality(*asset.properties.lod, asset.properties.wind_affected);
             */

            T::update_material(material, wind, asset.properties.options);
        }
    }
}

pub(super) fn setup_wind_texture(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
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
