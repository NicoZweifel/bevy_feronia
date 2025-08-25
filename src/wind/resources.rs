use bevy::prelude::*;
use std::f32::consts::PI;

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
    pub fast_normals: bool,
    pub high_quality: bool,
    pub edge_correction_factor: f32,
    pub lod_threshold: f32,
}

impl Default for Wind {
    fn default() -> Self {
        let direction = Vec2::new(1.0, 0.5).normalize();
        Self {
            direction,
            strength: 0.2,
            noise_scale: 0.01,
            scroll_speed: 0.1,
            micro_strength: 0.1,
            micro_noise_scale: 0.5,
            micro_scroll_speed: 0.1,
            bend_exponent: 2.0,
            round_exponent: 0.0,
            s_curve_speed: 2.0,
            s_curve_strength: 0.1,
            s_curve_frequency: PI,
            bop_speed: 1.0,
            bop_strength: 0.05,
            twist_strength: 0.05,
            enable_billboarding: false,
            enable_edge_correction: false,
            fast_normals: false,
            lod_threshold: 50.0,
            edge_correction_factor: 0.001,
            high_quality: true,
        }
    }
}
