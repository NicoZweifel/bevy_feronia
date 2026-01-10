use bevy_feronia::prelude::*;

use bevy_app::{App, Plugin, Startup, Update};
use bevy_ecs::prelude::*;
use bevy_light::{
    CascadeShadowConfig, CascadeShadowConfigBuilder, DirectionalLight, DirectionalLightShadowMap,
};
use bevy_reflect::Reflect;
use bevy_utils::default;

use std::fmt::Debug;

#[cfg(feature = "trace")]
use tracing::info;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct QualitySettingsSetup;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct QualitySettingsUpdating;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct QualitySettingsUpdated;

pub struct QualityPlugin;

impl Plugin for QualityPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Quality>()
            .register_type::<ShadowQuality>()
            .configure_sets(
                Startup,
                (
                    QualitySettingsSetup,
                    QualitySettingsUpdating,
                    QualitySettingsUpdated,
                )
                    .chain(),
            )
            .configure_sets(
                Update,
                (QualitySettingsUpdating, QualitySettingsUpdated).chain(),
            )
            .init_resource::<QualitySettings>()
            .add_systems(
                Startup,
                update_quality_settings.in_set(QualitySettingsUpdating),
            )
            .add_systems(
                Update,
                update_quality_settings
                    .run_if(resource_changed::<QualitySettings>)
                    .in_set(QualitySettingsUpdating),
            );
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Reflect, Default, PartialOrd, Ord)]
pub enum Quality {
    Custom,
    Low,
    Medium,
    #[default]
    High,
    Ultra,
}

impl From<Quality> for ShadowQuality {
    fn from(value: Quality) -> Self {
        match value {
            Quality::Custom => default(),
            Quality::Low => ShadowQuality::Low,
            Quality::Medium => ShadowQuality::Medium,
            Quality::High => ShadowQuality::High,
            Quality::Ultra => ShadowQuality::Ultra,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Reflect, Default, PartialOrd, Ord)]
pub enum ModelQuality {
    Low,
    Medium,
    #[default]
    High,
}

impl From<Quality> for ModelQuality {
    fn from(value: Quality) -> Self {
        match value {
            Quality::Custom => default(),
            Quality::Low => ModelQuality::Low,
            Quality::Medium => ModelQuality::Medium,
            Quality::High | Quality::Ultra => ModelQuality::High,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Reflect, Default, PartialOrd, Ord)]
pub enum GrassDensityQuality {
    Low,
    Medium,
    #[default]
    High,
    Ultra,
}

impl From<Quality> for GrassDensityQuality {
    fn from(value: Quality) -> Self {
        match value {
            Quality::Custom => default(),
            Quality::Low => GrassDensityQuality::Low,
            Quality::Medium => GrassDensityQuality::Medium,
            Quality::High => GrassDensityQuality::High,
            Quality::Ultra => GrassDensityQuality::Ultra,
        }
    }
}

impl From<GrassDensityQuality> for DistributionDensity {
    fn from(value: GrassDensityQuality) -> Self {
       match value {
            GrassDensityQuality::Low => 150.into(),
            GrassDensityQuality::Medium => 160.into(),
            GrassDensityQuality::High => 180.into(),
            GrassDensityQuality::Ultra => 200.into(),
        }
    }
}

#[derive(Resource, Debug, Reflect, Default, Clone, Eq, PartialEq)]
#[reflect(Resource)]
pub struct QualitySettings {
    pub quality: Quality,
    pub range_quality: VisibilityRangeQuality,
    pub shadow_quality: ShadowQuality,
    pub grass_density: GrassDensityQuality,
    pub model_quality: ModelQuality,
    pub disable_sss: bool,
    pub disable_grass_point_lights: bool,
    pub disable_grass_directional_lights: bool,
    pub static_shadows: bool,
    pub disable_wind_displacement: bool,
}

impl QualitySettings {
    pub fn get_preset(&self) -> Self {
        Self {
            quality: self.quality,
            range_quality: self.quality.into(),
            shadow_quality: self.quality.into(),
            grass_density: self.quality.into(),
            model_quality: self.quality.into(),
            disable_sss: matches!(self.quality, Quality::Low),
            disable_grass_point_lights: matches!(self.quality, Quality::Low),
            disable_grass_directional_lights: matches!(self.quality, Quality::Low),
            static_shadows: matches!(self.quality, Quality::Low),
            disable_wind_displacement: default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Reflect, Default, PartialOrd, Ord)]
pub enum VisibilityRangeQuality {
    Low,
    Medium,
    #[default]
    High,
    Ultra,
}

impl From<Quality> for VisibilityRangeQuality {
    fn from(value: Quality) -> Self {
        match value {
            Quality::Custom => default(),
            Quality::Low => VisibilityRangeQuality::Low,
            Quality::Medium => VisibilityRangeQuality::Medium,
            Quality::High => VisibilityRangeQuality::High,
            Quality::Ultra => VisibilityRangeQuality::Ultra,
        }
    }
}

impl From<VisibilityRangeQuality> for LodConfig {
    fn from(quality: VisibilityRangeQuality) -> Self {
        match quality {
            VisibilityRangeQuality::Low => Self {
                distance: vec![20.0.into(), default()],
                density: vec![1.0.into(), 0.3.into()],
            },
            VisibilityRangeQuality::Medium => Self {
                distance: vec![20.0.into(), 50.0.into(), default()],
                density: vec![1.0.into(), 0.3.into(), 0.1.into()],
            },
            VisibilityRangeQuality::High => default(),
            VisibilityRangeQuality::Ultra => Self {
                distance: vec![40.0.into(), 120.0.into(), 360.0.into(), default()],
                density: vec![1.0.into(), 0.3.into(), 0.1.into(), default()],
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Reflect, Default, PartialOrd, Ord)]
pub enum ShadowQuality {
    Off,
    Low,
    Medium,
    #[default]
    High,
    Ultra,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ShadowSettings {
    pub size: usize,
    pub cascades: usize,
    pub max_dist: f32,
    pub first_bound: f32,
    pub pcss_size: Option<f32>,
}

impl From<ShadowQuality> for ShadowSettings {
    fn from(quality: ShadowQuality) -> Self {
        match quality {
            ShadowQuality::Off | ShadowQuality::Low => Self {
                size: 1024,
                cascades: 1,
                max_dist: 100.0,
                first_bound: 5.0,
                pcss_size: None,
            },
            ShadowQuality::Medium => Self {
                size: 2048,
                cascades: 4,
                max_dist: 150.0,
                first_bound: 10.0,
                pcss_size: None,
            },
            ShadowQuality::High => Self {
                size: 4096,
                cascades: 4,
                max_dist: 200.0,
                first_bound: 10.0,
                pcss_size: Some(8.0),
            },
            ShadowQuality::Ultra => Self {
                size: 4096,
                cascades: 4,
                max_dist: 250.0,
                first_bound: 10.0,
                pcss_size: Some(8.0),
            },
        }
    }
}

fn update_quality_settings(
    mut settings: ResMut<QualitySettings>,
    mut shadow_map_resource: ResMut<DirectionalLightShadowMap>,
    mut lights: Query<(&mut DirectionalLight, &mut CascadeShadowConfig)>,
    mut last_applied_settings: Local<Option<QualitySettings>>,
) {
    let mut new_settings = settings.clone();

    let last_settings = last_applied_settings.clone().unwrap_or_default();

    let preset_changed = settings.quality != last_settings.quality;

    if preset_changed && settings.quality != Quality::Custom {
        new_settings = settings.get_preset();
    } else if settings.quality != Quality::Custom {
        let preset = settings.get_preset();

        if *settings != preset {
            new_settings.quality = Quality::Custom;
        }
    }

    if new_settings.shadow_quality != last_settings.shadow_quality {
        #[cfg(feature = "trace")]
        info!(
            "Applying new shadow quality: {:?}",
            new_settings.shadow_quality
        );

        let ShadowSettings {
            size,
            #[cfg(feature = "pcss")]
            pcss_size,
            cascades,
            max_dist,
            first_bound,
            ..
        } = new_settings.shadow_quality.into();

        shadow_map_resource.size = size;

        for (mut light, mut cascade_config) in &mut lights {
            if new_settings.shadow_quality == ShadowQuality::Off {
                light.shadows_enabled = false;
                continue;
            }

            light.shadows_enabled = true;

            #[cfg(feature = "pcss")]
            {
                light.soft_shadow_size = pcss_size;
            }

            *cascade_config = CascadeShadowConfigBuilder {
                num_cascades: cascades,
                maximum_distance: max_dist,
                first_cascade_far_bound: first_bound,
                ..default()
            }
            .build();
        }
    }

    *last_applied_settings = Some(new_settings.clone());

    settings.set_if_neq(new_settings);
}
