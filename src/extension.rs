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
    fn create_material(base: M, wind: Wind, wind_noise_texture: &Res<WindTexture>) -> R;
}

impl WindAffectable<StandardMaterial, WindAffectedExtendedMaterial>
    for WindAffectedExtendedMaterial
{
    fn create_material(
        base: StandardMaterial,
        wind: Wind,
        wind_noise_texture: &Res<WindTexture>,
    ) -> WindAffectedExtendedMaterial {
        ExtendedMaterial {
            base,
            extension: WindAffectedExtension {
                noise_texture: wind_noise_texture.0.clone(),
                wind,
            },
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
    fn from(val: &Wind) -> Self {
        WindUniform {
            direction: val.direction,
            strength: val.strength,
            noise_scale: val.noise_scale,
            scroll_speed: val.scroll_speed,
            bend_exponent: val.bend_exponent,
            round_exponent: val.round_exponent,
            micro_strength: val.micro_strength,
            micro_noise_scale: val.micro_noise_scale,
            micro_scroll_speed: val.micro_scroll_speed,
            s_curve_speed: val.s_curve_speed,
            s_curve_strength: val.s_curve_strength,
            s_curve_frequency: val.s_curve_frequency,
            bop_speed: val.bop_speed,
            bop_strength: val.bop_strength,
            twist_strength: val.twist_strength,
            enable_billboarding: val.enable_billboarding as u32,
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
