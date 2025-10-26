use crate::prelude::{
    BendExponent, BopSpeed, BopStrength, LowQuality, MicroStrengthMultiplier, SCurveFrequency,
    SCurveSpeed, SCurveStrength, StrengthMultiplier, TwistStrength,
};
use bevy::prelude::*;
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
    pub micro_noise_scale: f32,
    pub micro_scroll_speed: f32,
    pub s_curve_speed: f32,
    pub s_curve_strength: f32,
    pub s_curve_frequency: f32,
    pub bop_speed: f32,
    pub bop_strength: f32,
    pub twist_strength: f32,
    pub low_quality: bool,
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
            s_curve_speed: 2.0,
            s_curve_strength: 0.1,
            s_curve_frequency: PI,
            bop_speed: 1.0,
            bop_strength: 0.05,
            twist_strength: 0.05,
            low_quality: false,
        }
    }
}

pub type WindData<'w> = (
    Option<&'w StrengthMultiplier>,
    Option<&'w MicroStrengthMultiplier>,
    Option<&'w SCurveStrength>,
    Option<&'w SCurveSpeed>,
    Option<&'w SCurveFrequency>,
    Option<&'w BopStrength>,
    Option<&'w BopSpeed>,
    Option<&'w TwistStrength>,
    Option<&'w BendExponent>,
    Option<&'w LowQuality>,
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
            low_quality,
        ): WindData,
    ) -> Self {
        Wind {
            strength: strength
                .map(|x| **x * self.strength)
                .unwrap_or(self.strength),
            micro_strength: micro_strength
                .map(|x| **x * self.micro_strength)
                .unwrap_or(self.micro_strength),
            s_curve_strength: s_curve_strength
                .map(|x| **x * self.s_curve_strength)
                .unwrap_or(self.s_curve_strength),
            s_curve_speed: s_curve_speed
                .map(|x| **x * self.s_curve_speed)
                .unwrap_or(self.s_curve_speed),
            s_curve_frequency: s_curve_frequency
                .map(|x| **x)
                .unwrap_or(self.s_curve_frequency),
            bop_strength: bop_strength
                .map(|x| **x * self.bop_strength)
                .unwrap_or(self.bop_strength),
            bop_speed: bop_speed
                .map(|x| **x * self.bop_speed)
                .unwrap_or(self.bop_speed),
            twist_strength: twist_strength
                .map(|x| **x * self.twist_strength)
                .unwrap_or(self.twist_strength),
            bend_exponent: bend_exponent
                .map(|x| **x * self.bend_exponent)
                .unwrap_or(self.bend_exponent),
            low_quality: low_quality.map(|_| true).unwrap_or(self.low_quality),
            ..*self
        }
    }
}
