use crate::asset::systems::{MaterialOptionData, WindData};
use crate::prelude::*;
use bevy::asset::{Asset, Assets};
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::pbr::Material;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use noise::{NoiseFn, Perlin};

pub fn update_materials<TIn, TOut>(
    mut materials: ResMut<Assets<TOut>>,
    wind: Res<Wind>,
    mut scatter_assets: ResMut<Assets<ScatterAsset<TOut>>>,
    q_layer: Query<
        (WindData, MaterialOptionData, &ScatterLayerOf),
        (With<ScatterLayer>, With<ScatterLayerType<TIn, TOut>>),
    >,
    q_root: Query<(WindData, MaterialOptionData), With<ScatterRoot>>,
) where
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
{
    for (_, asset) in scatter_assets.iter_mut() {
        let Some(material) = materials.get_mut(&asset.material) else {
            dbg!("Material not found!");
            continue;
        };

        if asset.material_options.controlled {
            continue;
        };

        let Ok((wind_data, material_options, root)) = q_layer.get(asset.layer) else {
            dbg!("ScatterLayer not found!");
            continue;
        };

        let Ok((root_wind_data, root_material_options)) = q_root.get(**root) else {
            dbg!("ScatterRoot not found!");
            continue;
        };

        let wind = wind.with(root_wind_data).with(wind_data);
        let options = MaterialOptions::from(root_material_options).with(material_options);

        asset.wind = wind.clone();
        asset.material_options = options.clone();

        TOut::update_material(material, wind, options);
    }
}

pub fn replace_materials<TIn, TOut>(
    mut cmd: Commands,
    q: Query<(Entity, &WindAffectedRegistered<ScatterAsset<TOut>>), Without<WindAffectedReady>>,
    scatter_assets: Res<Assets<ScatterAsset<TOut>>>,
) where
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
{
    for (entity, wind_affected) in &q {
        let Some(scatter_asset) = scatter_assets.get(&**wind_affected) else {
            continue;
        };

        debug!("Replacing Material with WindAffected material...");

        cmd.entity(entity).insert((
            TOut::component(scatter_asset.material.clone()),
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
