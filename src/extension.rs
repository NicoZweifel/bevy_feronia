use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};

use crate::{WindPlugin, prelude::*};

pub struct ExtendedMaterialWindPlugin;

impl Plugin for ExtendedMaterialWindPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<WindAffectedExtendedMaterial>::default())
            .add_plugins(WindPlugin::<StandardMaterial, WindAffectedExtendedMaterial>::default());
    }
}

pub type WindAffectedExtendedMaterial = ExtendedMaterial<StandardMaterial, WindAffectedExtension>;

pub trait WindAffectable<M: Material, R: Material> {
    fn create_material(base: M, wind: Wind, noise_texture: Handle<Image>) -> R;
    fn update_material(materials: ResMut<Assets<R>>, wind: Wind);
}

impl WindAffectable<StandardMaterial, WindAffectedExtendedMaterial>
    for WindAffectedExtendedMaterial
{
    fn create_material(
        base: StandardMaterial,
        wind: Wind,
        noise_texture: Handle<Image>,
    ) -> WindAffectedExtendedMaterial {
        ExtendedMaterial {
            base,
            extension: WindAffectedExtension {
                noise_texture,
                wind,
            },
        }
    }

    fn update_material(mut materials: ResMut<Assets<WindAffectedExtendedMaterial>>, wind: Wind) {
        for (_, material) in materials.iter_mut() {
            let ext = &mut material.extension;
            ext.wind = wind.clone();
        }
    }
}

#[derive(Asset, Reflect, AsBindGroup, Debug, Clone)]
#[data(50, WindUniform, binding_array(101))]
#[bindless(index_table(range(50..53), binding(100)))]
pub struct WindAffectedExtension {
    pub wind: Wind,

    #[texture(51)]
    #[sampler(52)]
    pub noise_texture: Handle<Image>,
}

impl From<&Wind> for WindUniform {
    fn from(wind: &Wind) -> Self {
        WindUniform {
            direction: wind.direction,
            strength: wind.strength,
            noise_scale: wind.noise_scale,
            scroll_speed: wind.scroll_speed,
            bend_exponent: wind.bend_exponent,
            round_exponent: wind.round_exponent,
            micro_strength: wind.micro_strength,
            micro_noise_scale: wind.micro_noise_scale,
            micro_scroll_speed: wind.micro_scroll_speed,
            s_curve_speed: wind.s_curve_speed,
            s_curve_strength: wind.s_curve_strength,
            s_curve_frequency: wind.s_curve_frequency,
            bop_speed: wind.bop_speed,
            bop_strength: wind.bop_strength,
            twist_strength: wind.twist_strength,
            enable_billboarding: match wind.enable_billboarding {
                true => 1,
                _ => 0,
            },
            enable_edge_correction: match wind.enable_edge_correction {
                true => 1,
                _ => 0,
            },
            lod_threshold: wind.lod_threshold,
        }
    }
}

impl<'a> From<&'a WindAffectedExtension> for WindUniform {
    fn from(material_extension: &'a WindAffectedExtension) -> Self {
        WindUniform::from(&material_extension.wind)
    }
}

const SHADER_MAIN_ASSET_PATH: &str = "shaders/wind_main.wgsl";
const SHADER_PREPASS_ASSET_PATH: &str = "shaders/wind_prepass.wgsl";

impl MaterialExtension for WindAffectedExtension {
    fn vertex_shader() -> ShaderRef {
        SHADER_MAIN_ASSET_PATH.into()
    }

    fn fragment_shader() -> ShaderRef {
        SHADER_MAIN_ASSET_PATH.into()
    }

    fn prepass_vertex_shader() -> ShaderRef {
        SHADER_PREPASS_ASSET_PATH.into()
    }
}
