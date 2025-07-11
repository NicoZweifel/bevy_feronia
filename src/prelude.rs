use std::f32::consts::TAU;
use std::marker::PhantomData;

use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;

pub use crate::extension::*;

#[derive(Resource)]
pub struct WindAffectedTypes<M: Material> {
    pub values: Vec<WindAffectedType<M>>,
    pub _marker: PhantomData<M>,
}

impl<M: Material> Default for WindAffectedTypes<M> {
    fn default() -> Self {
        Self {
            values: Default::default(),
            _marker: Default::default(),
        }
    }
}

pub struct WindAffectedType<M: Material> {
    pub mesh: Handle<Mesh>,
    pub material: Handle<M>,
    pub wind: Wind,
}

impl<M: Material> WindAffectedTypes<M> {
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
    pub lod_threshold: f32
}

#[derive(ShaderType, Clone)]
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
    pub enable_billboarding: u32,
    pub lod_threshold: f32
}

impl Default for Wind {
    fn default() -> Self {
        let direction = Vec2::new(1.0, 0.5).normalize();
        Self {
            direction,
            strength: 0.5,
            noise_scale: 0.02,
            scroll_speed: 0.2,
            micro_strength: 0.1,
            micro_noise_scale: 1.0,
            micro_scroll_speed: 0.1,
            bend_exponent: 2.0,
            round_exponent: 2.0,
            s_curve_speed: 8.0,
            s_curve_strength: 0.1,
            s_curve_frequency: TAU * 8.0,
            bop_speed: 100.0,
            bop_strength: 0.01,
            twist_strength: 0.1,
            enable_billboarding: false,
            lod_threshold: 75.0
        }
    }
}
