use std::f32::consts::PI;
use std::marker::PhantomData;

use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;
use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};
pub use crate::extension::*;
pub use crate::instancing::*;

#[derive(Resource)]
pub struct WindAffectedTypes<M: Asset> {
    pub values: Vec<WindAffectedType<M>>,
    pub _marker: PhantomData<M>,
}

pub trait WindAffectable<M: Material, R: Asset> {
    fn create_material(base: M, wind: Wind, noise_texture: Handle<Image>) -> R;
    fn update_material(materials: ResMut<Assets<R>>, wind: Wind);
}

impl<M: Asset> Default for WindAffectedTypes<M> {
    fn default() -> Self {
        Self {
            values: Default::default(),
            _marker: Default::default(),
        }
    }
}

pub struct WindAffectedType<M: Asset> {
    pub mesh: Handle<Mesh>,
    pub material: Handle<M>,
    pub wind: Wind,
}

impl<M: Asset> WindAffectedTypes<M> {
    pub fn get(&self) -> &Vec<WindAffectedType<M>> {
        &self.values
    }
}

#[derive(Component)]
pub struct WindAffectedReady;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct WindAffected;

#[derive(Resource)]
pub struct WindTexture(pub Handle<Image>);

#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct Wind {
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
    pub enable_billboarding: bool,
    pub enable_edge_correction: bool,
    pub enable_lod:bool,
    pub edge_correction_factor: f32,
    pub lod_threshold: f32,
}

#[repr(C)]
#[derive(ShaderType, Clone,Pod,Zeroable,Copy)]
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
    pub edge_correction_factor: f32,
    pub lod_threshold: f32,
}

impl Default for Wind {
    fn default() -> Self {
        let direction = Vec2::new(1.0, 0.5).normalize();
        Self {
            direction,
            strength: 1.0,
            noise_scale: 0.02,
            scroll_speed: 0.2,
            micro_strength: 0.5,
            micro_noise_scale: 0.5,
            micro_scroll_speed: 0.2,
            bend_exponent: 2.0,
            round_exponent: 0.0,
            s_curve_speed: 8.0,
            s_curve_strength: 0.1,
            s_curve_frequency: PI,
            bop_speed: 8.0,
            bop_strength: 0.01,
            twist_strength: 0.1,
            enable_billboarding: false,
            enable_edge_correction: false,
            lod_threshold: 50.0,
            edge_correction_factor: 0.01,
            enable_lod:false
        }
    }
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
            edge_correction_factor: wind.edge_correction_factor,
            lod_threshold: wind.lod_threshold,
        }
    }
}

bitflags! {
    #[repr(C)]
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Pod, Zeroable)]
    pub struct WindAffectedKey: u32 {
        // avoid conflict with MeshPipelineKey's lower bits.
        const ENABLE_BILLBOARDING    = 1 << 24;
        const ENABLE_EDGE_CORRECTION = 1 << 25;
        const ENABLE_LOD = 1 << 26;
    }
}
