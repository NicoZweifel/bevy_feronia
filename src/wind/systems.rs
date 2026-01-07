use crate::prelude::*;

use bevy_asset::Assets;
use bevy_ecs::prelude::*;
use bevy_image::*;
use bevy_render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_utils::default;
use noise::{NoiseFn, Perlin};

pub fn update_materials<T>(
    mut materials: ResMut<Assets<T>>,
    global_wind: Res<GlobalWind>,
    mut scatter_assets: ResMut<Assets<ScatterAsset<T>>>,
    q_layer: Query<
        (WindOptionData, MaterialOptionData, &ScatterLayerOf),
        (With<ScatterLayer>, With<ScatterLayerType<T>>),
    >,
    q_root: Query<WindOptionData, With<ScatterRoot>>,
) where
    T: ScatterMaterial,
{
    let current_wind = global_wind.current;
    let previous_wind = global_wind.previous;
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

        let wind = current_wind.with(root_wind_data).with(wind_data);
        let prev_wind = previous_wind.with(root_wind_data).with(wind_data);

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

            T::update_material(material, wind, prev_wind, asset.properties.options);
        }
    }
}

pub(super) fn setup_wind_texture(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let texture_size = 512;
    let mut image_buffer = Vec::with_capacity((texture_size * texture_size * 4 * 4) as usize);

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

            let macro_val = (macro_noise_value * 0.5 + 0.5) as f32;
            let micro_val = (micro_noise_value * 0.5 + 0.5) as f32;

            image_buffer.extend_from_slice(&macro_val.to_le_bytes()); // R - Macro
            image_buffer.extend_from_slice(&micro_val.to_le_bytes()); // G - Micro
            image_buffer.extend_from_slice(&0.0f32.to_le_bytes()); // B - Unused
            image_buffer.extend_from_slice(&1.0f32.to_le_bytes()); // A - Unused
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
        TextureFormat::Rgba32Float,
        default(),
    );

    let sampler_descriptor = ImageSampler::Descriptor(ImageSamplerDescriptor {
        label: Some("Wind Noise Sampler".into()),
        address_mode_u: ImageAddressMode::MirrorRepeat,
        address_mode_v: ImageAddressMode::MirrorRepeat,
        address_mode_w: ImageAddressMode::MirrorRepeat,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        ..default()
    });

    wind_image.sampler = sampler_descriptor;

    let handle = images.add(wind_image);

    commands.insert_resource(WindTexture(handle));
}

pub fn sync_wind_preset(mut wind: ResMut<GlobalWind>, mut last_preset: Local<WindPreset>) {
    if wind.preset != *last_preset {
        wind.current = wind.preset.into();
        *last_preset = wind.preset;
    }
}

pub fn cycle_wind_history(mut wind: ResMut<GlobalWind>) {
    wind.previous = wind.current;
}
