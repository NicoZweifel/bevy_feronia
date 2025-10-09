use crate::prelude::*;
use bevy::camera::primitives::Aabb;
use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;
use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};

pub trait WindAffectable<TType, TIn, TOut>
where
    TType: ProtoType<TOut> + Asset + Clone,
    TIn: Material,
    TOut: Asset + Clone,
{
    // TODO refactor
    fn create_material(
        base: Option<TIn>,
        wind: Wind,
        noise_texture: Handle<Image>,
        aabb: Aabb,
        options: MaterialOptions,
    ) -> TOut;
    fn update_material(materials: ResMut<Assets<TOut>>, wind: Wind);

    fn component(material: Handle<TOut>) -> impl Component;
}

#[repr(C)]
#[derive(ShaderType, Clone, Pod, Zeroable, Copy)]
pub struct WindUniform {
    pub direction: Vec2,
    pub strength: f32,
    pub noise_scale: f32,
    pub scroll_speed: f32,
    pub bend_exponent: f32,
    pub round_exponent: f32,
    pub micro_strength: f32,
    pub micro_noise_scale: f32,
    pub micro_scroll_speed: f32,
    pub s_curve_speed: f32,
    pub s_curve_strength: f32,
    pub s_curve_frequency: f32,
    pub bop_speed: f32,
    pub bop_strength: f32,
    pub twist_strength: f32,

    // TODO create another uniform for Options
    pub edge_correction_factor: f32,
    pub lod_threshold: f32,
    pub aabb_min: Vec3,
    pub aabb_max: Vec3,
    pub debug_color: Vec4,
}

impl From<&Wind> for WindUniform {
    fn from(wind: &Wind) -> Self {
        WindUniform {
            direction: wind.direction,
            strength: wind.strength,
            noise_scale: wind.noise_scale,
            scroll_speed: wind.scroll_speed,
            bend_exponent: wind.bend_exponent,
            micro_strength: wind.micro_strength,
            micro_noise_scale: wind.micro_noise_scale,
            micro_scroll_speed: wind.micro_scroll_speed,
            s_curve_speed: wind.s_curve_speed,
            s_curve_strength: wind.s_curve_strength,
            s_curve_frequency: wind.s_curve_frequency,
            bop_speed: wind.bop_speed,
            bop_strength: wind.bop_strength,
            twist_strength: wind.twist_strength,
            // TODO sync/cleanup with LOD systems / chunks systems
            lod_threshold: 50.,
            // TODO create another uniform for Options
            edge_correction_factor: 0.,
            round_exponent: 0.,
            aabb_max: Vec3::splat(1.),
            aabb_min: Vec3::splat(0.),
            debug_color: Vec4::splat(1.),
        }
    }
}

// TODO create another uniform for Options
impl WindUniform {


    pub fn with_lod_threshold(mut self, lod_threshold: f32) -> Self {
        self.lod_threshold = lod_threshold;
        self
    }

    pub fn with_round_exponent(mut self, round_exponent: f32) -> Self {
        self.round_exponent = round_exponent;
        self
    }

    pub fn with_edge_correction_factor(mut self, edge_correction_factor: f32) -> Self {
        self.edge_correction_factor = edge_correction_factor;
        self
    }

    pub fn with_aabb(mut self, aabb: &Aabb) -> Self {
        self.aabb_min = aabb.min().into();
        self.aabb_max = aabb.max().into();
        self
    }

    pub fn with_debug_color(mut self, color: Vec4) -> Self {
        self.debug_color = color;
        self
    }
}

bitflags! {
    #[repr(C)]
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Pod, Zeroable)]
    pub struct WindAffectedKey: u64 {
        const ENABLE_BILLBOARDING    = 1 << 0;
        const ENABLE_EDGE_CORRECTION = 1 << 1;
        const WIND_LOW_QUALITY = 1 << 2;
        const FAST_NORMALS = 1 << 3;
        const DEBUG = 1 << 4;
    }
}
