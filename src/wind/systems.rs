use crate::prelude::*;

use noise::{NoiseFn, Perlin};
use std::f64::consts::PI;

use bevy_asset::Assets;
use bevy_ecs::prelude::*;
use bevy_image::*;
use bevy_render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_utils::default;

pub fn update_materials<T>(
    mut materials: ResMut<Assets<T>>,
    global_wind: Res<GlobalWind>,
    mut scatter_assets: ResMut<Assets<ScatterAsset<T>>>,
    q_layer: Query<
        (&ScatterLayerOf, WindOptionData, MaterialOptionData),
        (With<ScatterLayer>, With<ScatterLayerType<T>>),
    >,
    q_root: Query<(WindOptionData, MaterialOptionData), With<ScatterRoot>>,
    scatter_asset_manager: Res<ScatterAssetManager<T>>,
) where
    T: ScatterMaterial,
{
    for (asset, (wind_data, material_options), (root_wind_data, root_material_options)) in
        scatter_assets
            .iter_mut()
            .filter(|(_, asset)| !asset.properties.options.general.controlled)
            .filter_map(|(id, asset)| {
                let layer = scatter_asset_manager.asset_to_layer.get(&id)?;

                let (root, wind_data, material_options) = q_layer
                    .get(*layer)
                    .map_err(|e| {
                        #[cfg(feature = "trace")]
                        tracing::error!("ScatterLayer not found! {e}")
                    })
                    .ok()?;

                let root_data = q_root
                    .get(**root)
                    .map_err(|e| {
                        #[cfg(feature = "trace")]
                        tracing::error!("ScatterRoot not found! {e}");
                    })
                    .ok()?;

                Some((asset, (wind_data, material_options), root_data))
            })
    {
        let wind = global_wind
            .current
            .multiply(root_wind_data)
            .multiply(wind_data);

        let prev_wind = global_wind
            .previous
            .multiply(root_wind_data)
            .multiply(wind_data);

        let options = ScatterMaterialOptions::from(root_material_options).with(material_options);

        asset.properties.wind = wind;
        asset.properties.options = options;

        for part in &asset.parts {
            let Some(mut material) = materials.get_mut(&part.h_material) else {
                #[cfg(feature = "trace")]
                tracing::warn!("Material not found!");
                continue;
            };

            T::update_material(&mut *material, wind, prev_wind, asset.properties.options.clone());
        }
    }
}

/// Sets up a Wind texture with Toroidal mapping for seamless 2D noise.
pub(super) fn setup_wind_texture(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let texture_size = 512;
    let mut image_buffer = Vec::with_capacity((texture_size * texture_size * 4 * 4) as usize);

    let macro_perlin = Perlin::new(1);
    let micro_perlin = Perlin::new(2);

    let macro_scale = 5.0 / (2.0 * PI);
    let micro_scale = 20.0 / (2.0 * PI);

    for y in 0..texture_size {
        for x in 0..texture_size {
            let u = x as f64 / texture_size as f64;
            let v = y as f64 / texture_size as f64;

            // map x/y to angles, then to 4D coordinates (nx, ny, nz, nw)
            // macro
            let nx = (u * 2.0 * PI).cos() * macro_scale;
            let ny = (u * 2.0 * PI).sin() * macro_scale;
            let nz = (v * 2.0 * PI).cos() * macro_scale;
            let nw = (v * 2.0 * PI).sin() * macro_scale;

            let macro_noise_value = macro_perlin.get([nx, ny, nz, nw]);

            //  micro
            let mx = (u * 2.0 * PI).cos() * micro_scale;
            let my = (u * 2.0 * PI).sin() * micro_scale;
            let mz = (v * 2.0 * PI).cos() * micro_scale;
            let mw = (v * 2.0 * PI).sin() * micro_scale;

            let micro_noise_value = micro_perlin.get([mx, my, mz, mw]);

            // normalize
            let macro_val = (macro_noise_value * 0.5 + 0.5) as f32;
            let micro_val = (micro_noise_value * 0.5 + 0.5) as f32;

            let sine_val = (u * 4.0 * PI).sin() * 0.5 + 0.5;
            let cos_val = (u * 8.0 * PI).cos() * 0.5 + 0.5;

            image_buffer.extend_from_slice(&macro_val.to_le_bytes());
            image_buffer.extend_from_slice(&micro_val.to_le_bytes());
            image_buffer.extend_from_slice(&(sine_val as f32).to_le_bytes());
            image_buffer.extend_from_slice(&(cos_val as f32).to_le_bytes());
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
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        address_mode_w: ImageAddressMode::Repeat,
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
    if wind.previous != wind.current {
        wind.previous = wind.current;
    }
}
