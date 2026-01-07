use crate::prelude::*;
use bevy_asset::Handle;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use bevy_image::Image;
use bevy_math::{FloatExt, Vec2};
use bevy_reflect::Reflect;
use bevy_utils::default;
use std::f32::consts::PI;

#[derive(Resource, Deref, DerefMut, Clone)]
pub struct WindTexture(pub Handle<Image>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Default)]
pub enum WindPreset {
    None,
    #[default]
    Normal,
    Mild,
    Strong,
    Storm,
}

#[derive(Debug, Clone, Copy, Reflect)]
pub struct Wind {
    pub direction: Vec2,
    pub strength: f32,
    pub noise_scale: f32,
    pub scroll_speed: f32,
    pub micro_strength: f32,
    pub s_curve_speed: f32,
    pub s_curve_strength: f32,
    pub s_curve_frequency: f32,
    pub bop_speed: f32,
    pub bop_strength: f32,
    pub twist_strength: f32,
}

#[derive(Resource, Debug, Reflect)]
#[reflect(Resource)]
pub struct GlobalWind {
    pub preset: WindPreset,
    pub current: Wind,
    pub previous: Wind,
}

impl Default for GlobalWind {
    fn default() -> Self {
        let preset = WindPreset::default();
        Self {
            preset,
            current: preset.into(),
            previous: preset.into(),
        }
    }
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
            twist_strength: 0.02,
            s_curve_speed: 6.,
            s_curve_strength: 0.02,
            s_curve_frequency: PI,
            bop_speed: 3.5,
            bop_strength: 0.02,
        }
    }
}

impl From<WindPreset> for GlobalWind {
    fn from(value: WindPreset) -> Self {
        Self {
            preset: value,
            current: value.into(),
            previous: value.into(),
        }
    }
}

impl From<WindPreset> for Wind {
    fn from(preset: WindPreset) -> Self {
        match preset {
            WindPreset::Normal => Wind::default(),
            WindPreset::None => Wind {
                strength: 0.,
                micro_strength: 0.,
                s_curve_strength: 0.0,
                bop_speed: 0.0,
                bop_strength: 0.0,
                twist_strength: 0.0,
                ..default()
            },
            WindPreset::Mild => Wind {
                strength: 0.1,
                scroll_speed: 0.05,
                micro_strength: 0.05,
                twist_strength: 0.01,
                s_curve_speed: 4.,
                s_curve_strength: 0.01,
                bop_speed: 2.5,
                bop_strength: 0.01,
                ..default()
            },
            WindPreset::Strong => Wind {
                strength: 0.4,
                scroll_speed: 0.15,
                micro_strength: 0.2,
                twist_strength: 0.04,
                s_curve_speed: 8.,
                s_curve_strength: 0.04,
                bop_speed: 5.,
                bop_strength: 0.04,
                ..default()
            },
            WindPreset::Storm => Wind {
                strength: 1.0,
                scroll_speed: 0.3,
                micro_strength: 0.5,
                twist_strength: 0.1,
                s_curve_speed: 12.,
                s_curve_strength: 0.1,
                bop_speed: 8.0,
                bop_strength: 0.1,
                ..default()
            },
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
            ..*self
        }
    }

    pub fn lerp(&self, target: &Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);

        Self {
            direction: self.direction.lerp(target.direction, t),
            strength: self.strength.lerp(target.strength, t),
            noise_scale: self.noise_scale.lerp(target.noise_scale, t),
            scroll_speed: self.scroll_speed.lerp(target.scroll_speed, t),
            micro_strength: self.micro_strength.lerp(target.micro_strength, t),
            s_curve_speed: self.s_curve_speed.lerp(target.s_curve_speed, t),
            s_curve_strength: self.s_curve_strength.lerp(target.s_curve_strength, t),
            // We usually don't want to lerp frequency if it causes phase issues,
            // but for simple wind it's fine.
            s_curve_frequency: self.s_curve_frequency.lerp(target.s_curve_frequency, t),
            bop_speed: self.bop_speed.lerp(target.bop_speed, t),
            bop_strength: self.bop_strength.lerp(target.bop_strength, t),
            twist_strength: self.twist_strength.lerp(target.twist_strength, t),
        }
    }
}
