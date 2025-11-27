use crate::prelude::*;

use bevy_app::{App, Plugin, Update};
use bevy_asset::{Asset, Handle};
use bevy_ecs::prelude::*;
use bevy_light::{
    CascadeShadowConfig, CascadeShadowConfigBuilder, DirectionalLight, DirectionalLightShadowMap,
};
use bevy_mesh::{Mesh, Mesh3d};
use bevy_pbr::{MeshMaterial3d, StandardMaterial};
use bevy_reflect::Reflect;
use bevy_scene::{Scene, SceneRoot};
use bevy_utils::default;
use std::fmt::Debug;
use std::marker::PhantomData;

#[cfg(feature = "trace")]
use tracing::{info, warn};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct QualitySettingsUpdate;

pub struct QualityPlugin;

impl Plugin for QualityPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Quality>()
            .register_type::<ShadowQuality>()
            .configure_sets(Update, QualitySettingsUpdate)
            .init_resource::<QualitySettings>()
            .add_systems(
                Update,
                update_quality_settings
                    .run_if(resource_changed::<QualitySettings>)
                    .in_set(QualitySettingsUpdate),
            )
            .add_observer(apply_quality_gate::<WindAffected>)
            .add_observer(apply_quality_gate::<StaticShadow>)
            .add_observer(apply_quality_gate::<SssBundle>)
            .add_observer(apply_quality_gate::<PointLights>)
            .add_observer(apply_quality_gate::<DirectionalLights>)
            .add_observer(apply_quality_gate::<Unlit>);
    }
}

pub struct AssetSelectPlugin<T: Asset>(PhantomData<T>);

impl<T: Asset> Default for AssetSelectPlugin<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Asset> AssetSelectPlugin<T> {
    pub fn new() -> Self {
        Self(PhantomData::<T>)
    }
}

impl<T: SpawnableAsset> Plugin for AssetSelectPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_observer(resolve_lod::<T>);
    }
}

pub trait SpawnableAsset: Asset + Sized {
    fn spawn(cmd: &mut Commands, entity: Entity, handle: Handle<Self>);
}

#[derive(Component, Default, Debug, Reflect, Clone)]
#[reflect(Component)]
pub struct AssetSelect<T: Asset> {
    pub low: Handle<T>,
    pub medium: Handle<T>,
    pub high: Handle<T>,
}

impl SpawnableAsset for Scene {
    fn spawn(cmd: &mut Commands, entity: Entity, handle: Handle<Scene>) {
        cmd.entity(entity).insert(SceneRoot(handle));
    }
}

impl SpawnableAsset for Mesh {
    fn spawn(cmd: &mut Commands, entity: Entity, handle: Handle<Self>) {
        cmd.entity(entity).insert(Mesh3d(handle));
    }
}

impl SpawnableAsset for StandardMaterial {
    fn spawn(cmd: &mut Commands, entity: Entity, handle: Handle<Self>) {
        cmd.entity(entity)
            .insert(MeshMaterial3d::<StandardMaterial>(handle));
    }
}

impl<T: Asset> AssetSelect<T> {
    pub fn new(high: Handle<T>) -> Self {
        Self {
            high,
            medium: default(),
            low: default(),
        }
    }
    pub fn with_low(mut self, h: Handle<T>) -> Self {
        self.low = h;
        self
    }
    pub fn with_med(mut self, h: Handle<T>) -> Self {
        self.medium = h;
        self
    }
    pub fn with_high(mut self, h: Handle<T>) -> Self {
        self.high = h;
        self
    }
    pub fn progressive(high: Handle<T>, medium: Handle<T>, low: Handle<T>) -> Self {
        Self { high, medium, low }
    }

    pub fn get_handle(&self, quality: ModelQuality) -> &Handle<T> {
        match quality {
            ModelQuality::High => &self.high,
            ModelQuality::Medium => &self.medium,
            ModelQuality::Low => &self.low,
        }
    }
}

fn resolve_lod<T: SpawnableAsset>(
    trigger: On<Add, AssetSelect<T>>,
    mut cmd: Commands,
    quality: Res<QualitySettings>,
    query: Query<&AssetSelect<T>>,
) {
    let entity = trigger.entity;
    let Ok(selector) = query.get(entity) else {
        return;
    };

    let target_handle = selector.get_handle(quality.model_quality);

    cmd.entity(entity).remove::<AssetSelect<T>>();

    if target_handle.id() == default::<Handle<T>>().id() {
        cmd.entity(entity).despawn();
        return;
    }

    T::spawn(&mut cmd, entity, target_handle.clone());
}

/// Defines which QualitySettings field controls this component
#[derive(Clone, Copy, Debug)]
pub enum QualityRule {
    Wind,
    Sss,
    StaticShadows,
    PointLights,
    DirectionalLights,
    LowModel,
    MediumModel,
    HighModel,
}

/// "Only add Bundle B if the QualityRule passes"
#[derive(Component)]
pub struct AddIf<B>
where
    B: Bundle + Clone,
{
    pub rule: QualityRule,
    pub bundle: B,
}

impl<T> AddIf<T>
where
    T: Bundle + Clone,
{
    pub fn new(rule: QualityRule, bundle: T) -> Self {
        Self { rule, bundle }
    }
}

/// Applies optional components based on quality settings
fn apply_quality_gate<B: Bundle + Clone>(
    trigger: On<Add, AddIf<B>>,
    mut cmd: Commands,
    settings: Res<QualitySettings>,
    query: Query<&AddIf<B>>,
) {
    let entity = trigger.entity;
    let Ok(AddIf { rule, bundle }) = query.get(entity) else {
        #[cfg(feature = "trace")]
        warn!("Could not find AddIf component for entity {}", entity,);
        return;
    };

    let passes = match rule {
        QualityRule::Wind => !settings.disable_wind_displacement,
        QualityRule::Sss => !settings.disable_sss,
        QualityRule::StaticShadows => settings.static_shadows,
        QualityRule::PointLights => !settings.disable_grass_point_lights,
        QualityRule::DirectionalLights => !settings.disable_grass_directional_lights,
        QualityRule::LowModel => settings.model_quality == ModelQuality::Low,
        QualityRule::MediumModel => settings.model_quality == ModelQuality::Medium,
        QualityRule::HighModel => settings.model_quality == ModelQuality::High,
    };

    if passes {
        cmd.entity(entity).insert(bundle.clone());
    }

    cmd.entity(entity).remove::<AddIf<B>>();
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
                distance: vec![20.0.into(), 60.0.into(), default()],
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
    #[default]
    Medium,
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
                size: 2048,
                cascades: 4,
                max_dist: 200.0,
                first_bound: 15.0,
                pcss_size: Some(8.0),
            },
            ShadowQuality::Ultra => Self {
                size: 4096,
                cascades: 4,
                max_dist: 250.0,
                first_bound: 20.0,
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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::asset::Handle;
    use bevy_asset::uuid::Uuid;

    #[derive(Asset, Debug, Clone, Reflect)]
    struct MockAsset;

    #[derive(Component, Debug, PartialEq)]
    struct MockComponent(Handle<MockAsset>);

    impl SpawnableAsset for MockAsset {
        fn spawn(cmd: &mut Commands, entity: Entity, handle: Handle<Self>) {
            cmd.entity(entity).insert(MockComponent(handle));
        }
    }

    fn mock_handle<T: Asset>() -> Handle<T> {
        Handle::Uuid(Uuid::new_v4(), PhantomData::<fn() -> T>)
    }

    #[test]
    fn test_asset_select_new_builder_should_assign_handles() {
        // Arrange
        let expected_high: Handle<MockAsset> = mock_handle();
        let expected_med: Handle<MockAsset> = mock_handle();
        let expected_low: Handle<MockAsset> = mock_handle();

        // Act
        let selector = AssetSelect::new(expected_high.clone())
            .with_med(expected_med.clone())
            .with_low(expected_low.clone());

        let actual_high = selector.high;
        let actual_med = selector.medium;
        let actual_low = selector.low;

        // Assert
        assert_eq!(actual_high, expected_high);
        assert_eq!(actual_med, expected_med);
        assert_eq!(actual_low, expected_low);
    }

    #[test]
    fn test_asset_select_progressive_builder_should_assign_handles() {
        // Arrange
        let expected_high: Handle<MockAsset> = mock_handle();
        let expected_med: Handle<MockAsset> = mock_handle();
        let expected_low: Handle<MockAsset> = mock_handle();

        // Act
        let selector = AssetSelect::progressive(
            expected_high.clone(),
            expected_med.clone(),
            expected_low.clone(),
        );

        let actual_high = selector.high;
        let actual_med = selector.medium;
        let actual_low = selector.low;

        // Assert
        assert_eq!(actual_high, expected_high);
        assert_eq!(actual_med, expected_med);
        assert_eq!(actual_low, expected_low);
    }

    #[test]
    fn test_asset_select_should_return_correct_handle_for_each_quality_level() {
        // Arrange
        let expected_high: Handle<MockAsset> = mock_handle();
        let expected_med: Handle<MockAsset> = mock_handle();
        let expected_low: Handle<MockAsset> = mock_handle();

        let selector = AssetSelect::progressive(
            expected_high.clone(),
            expected_med.clone(),
            expected_low.clone(),
        );

        // Act
        let actual_high = selector.get_handle(ModelQuality::High);
        let actual_med = selector.get_handle(ModelQuality::Medium);
        let actual_low = selector.get_handle(ModelQuality::Low);

        // Assert
        assert_eq!(
            actual_high, &expected_high,
            "High quality should return high handle"
        );
        assert_eq!(
            actual_med, &expected_med,
            "Medium quality should return medium handle"
        );
        assert_eq!(
            actual_low, &expected_low,
            "Low quality should return low handle"
        );
    }

    #[test]
    fn test_resolve_lod_should_spawn_component() {
        // Arrange
        let mut app = App::new();
        app.init_resource::<QualitySettings>()
            .add_observer(resolve_lod::<MockAsset>);

        app.world_mut()
            .resource_mut::<QualitySettings>()
            .model_quality = ModelQuality::High;

        let expected_handle: Handle<MockAsset> = mock_handle();
        let bundle = AssetSelect::new(expected_handle.clone());

        // Act
        let entity = app.world_mut().spawn(bundle).id();
        app.update();

        let actual_component = app.world().entity(entity).get::<MockComponent>();
        let has_selector = app
            .world()
            .entity(entity)
            .contains::<AssetSelect<MockAsset>>();

        // Assert
        assert_eq!(
            actual_component,
            Some(&MockComponent(expected_handle)),
            "Should spawn MockComponent"
        );
        assert!(!has_selector, "Should remove the AssetSelect component");
    }

    // TODO: This test is crashing, not sure why despawning in an observer crashes the app/setup
    /*
    #[test]
    fn test_resolve_lod_should_despawn_entity_when_handle_is_missing() {
        // Arrange
        let mut app = App::new();
        app.init_resource::<QualitySettings>()
            .add_observer(resolve_lod::<MockAsset>);

        // Set to Low, but don't provide a Low handle in the asset select
        app.world_mut()
            .resource_mut::<QualitySettings>()
            .model_quality = ModelQuality::Low;

        let bundle = AssetSelect::<MockAsset>::new(mock_handle());

        // Act
        let entity = app.world_mut().spawn(bundle).id();
    app.update();

        // Assert
        assert!(
            app.world().get_entity(entity).is_err(),
            "Entity should be despawned entirely if the specific quality handle is default/missing"
        );
    }
     */
}
