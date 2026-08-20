//! Mostly a testing ground at the moment, but will eventually turn into an example for foliage assembly assets (instanced parts).

#[path = "utils/example.rs"]
mod example;

use bevy::prelude::*;
use bevy_feronia::asset::backend::scene_backend::SceneAssetBackendPlugin;
use bevy_feronia::extension;
use bevy_feronia::prelude::*;
use example::*;
use rand::{RngCore, rng};

fn main() -> AppExit {
    App::new()
        .insert_resource(GlobalWind::from(WindPreset::Mild))
        .insert_resource(ExamplePluginOptions {
            show_wind_settings: true,
            ..default()
        })
        .add_plugins((
            ExamplePlugin,
            SceneAssetBackendPlugin,
            ExtendedWindAffectedScatterPlugin,
        ))
        .init_state::<AppState>()
        .insert_state(HeightMapState::Setup)
        .add_systems(Startup, load_assets)
        .add_systems(
            Update,
            check_assets_loaded.run_if(in_state(AppState::Loading)),
        )
        .add_systems(OnEnter(AppState::InGame), spawn_scene)
        .add_systems(Update, scatter_on_keypress.in_set(ScatterSet::Ready))
        .run()
}

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
enum AppState {
    #[default]
    Loading,
    InGame,
}

#[derive(Resource)]
struct Scenes {
    landscape: Handle<WorldAsset>,
    lod_low: Handle<WorldAsset>,
}

fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Scenes {
        landscape: asset_server.load("landscape_flat_large.glb#Scene0"),
        lod_low: asset_server.load("tree.glb#Scene0"),
    });
}

fn check_assets_loaded(
    mut next_state: ResMut<NextState<AppState>>,
    asset_server: Res<AssetServer>,
    handles: Res<Scenes>,
) {
    let all_loaded = [handles.landscape.id(), handles.lod_low.id()]
        .iter()
        .all(|id| {
            asset_server
                .get_load_state(*id)
                .is_some_and(|s| s.is_loaded())
        });

    if all_loaded {
        next_state.set(AppState::InGame);
    }
}

fn spawn_scene(
    mut cmd: Commands,
    handles: Res<Scenes>,
    mut ns_scatter: ResMut<NextState<ScatterState>>,
) {
    cmd.spawn((
        WorldAssetRoot(handles.landscape.clone()),
        ScatterRoot::default(),
        LodConfig::none(),
        children![(
            extension::scatter_layer("Tree Layer"),
            Avoid,
            DistributionDensity(10.),
            InstanceRotationYaw,
            InstanceScaleRange { min: 1., max: 4. },
            WindAffected,
            InstanceJitter,
            children![WorldAssetRoot(handles.lod_low.clone()),]
        )],
    ));

    ns_scatter.set(ScatterState::Setup);
}

fn scatter_on_keypress(
    mut cmd: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut world_seed: ResMut<WorldSeed>,
    root: Single<Entity, With<ScatterRoot>>,
) {
    if !keyboard_input.just_pressed(KeyCode::Space) {
        return;
    };

    **world_seed = rng().next_u64();

    cmd.trigger(Scatter::<ExtendedWindAffectedMaterial>::new(*root))
}
