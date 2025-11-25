use crate::prelude::*;
use bevy_asset::Handle;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use bevy_image::Image;
use bevy_math::Vec2;
use bevy_reflect::Reflect;
use std::f32::consts::PI;

#[derive(Resource, Deref, DerefMut, Clone)]
pub struct WindTexture(pub Handle<Image>);

#[derive(Resource, Debug, Clone, Copy, Reflect)]
#[reflect(Resource)]
pub struct Wind {
    pub direction: Vec2,
    pub strength: f32,
    pub noise_scale: f32,
    pub scroll_speed: f32,
    pub bend_exponent: f32,
    pub micro_strength: f32,
    pub s_curve_speed: f32,
    pub s_curve_strength: f32,
    pub s_curve_frequency: f32,
    pub bop_speed: f32,
    pub bop_strength: f32,
    pub twist_strength: f32,
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
            bend_exponent: 2.25,
            twist_strength: 0.05,
            s_curve_speed: 3.,
            s_curve_strength: 0.02,
            s_curve_frequency: 2.0,
            bop_speed: 2.5,
            bop_strength: 0.01,
        }
    }
}

pub type WindOptionData<'w> = (
    Option<&'w Strength>,
    Option<&'w MicroStrength>,
    Option<&'w SCurveStrength>,
    Option<&'w SCurveSpeed>,
    Option<&'w SCurveFrequency>,
    Option<&'w BopStrength>,
    Option<&'w BopSpeed>,
    Option<&'w TwistStrength>,
    Option<&'w BendExponent>,
);

impl Wind {
    pub fn with(
        &self,
        (
            strength,
            micro_strength,
            s_curve_strength,
            s_curve_speed,
            s_curve_frequency,
            bop_strength,
            bop_speed,
            twist_strength,
            bend_exponent,
        ): WindOptionData,
    ) -> Self {
        Wind {
            strength: strength
                .map(|s| **s * self.strength)
                .unwrap_or(self.strength),
            micro_strength: micro_strength
                .map(|s| **s * self.micro_strength)
                .unwrap_or(self.micro_strength),
            s_curve_strength: s_curve_strength
                .map(|s| **s * self.s_curve_strength)
                .unwrap_or(self.s_curve_strength),
            s_curve_speed: s_curve_speed
                .map(|s| **s * self.s_curve_speed)
                .unwrap_or(self.s_curve_speed),
            s_curve_frequency: s_curve_frequency
                .map(|f| **f * self.s_curve_frequency)
                .unwrap_or(self.s_curve_frequency),
            bop_strength: bop_strength
                .map(|b| **b * self.bop_strength)
                .unwrap_or(self.bop_strength),
            bop_speed: bop_speed
                .map(|b| **b * self.bop_speed)
                .unwrap_or(self.bop_speed),
            twist_strength: twist_strength
                .map(|t| **t * self.twist_strength)
                .unwrap_or(self.twist_strength),
            bend_exponent: bend_exponent
                .map(|b| **b * self.bend_exponent)
                .unwrap_or(self.bend_exponent),
            ..*self
        }
    }
}
