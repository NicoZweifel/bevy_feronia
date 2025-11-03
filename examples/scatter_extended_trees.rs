#[path = "utils/example.rs"]
mod example;

use bevy::prelude::*;
use bevy_feronia::extension::scatter_layer;
use bevy_feronia::prelude::*;
use example::*;
use rand::{RngCore, rng};

fn main() -> AppExit {
    App::new()
        .insert_resource(Wind {
            strength: 0.2,
            micro_strength: 0.1,
            s_curve_strength: 0.01,
            bop_strength: 0.01,
            ..default()
        })
        .add_plugins((ExamplePlugin, ExtendedWindAffectedScatterPlugin))
        .init_state::<AppState>()
        .add_systems(Startup, load_assets)
        .add_systems(
            Update,
            check_assets_loaded.run_if(in_state(AppState::Loading)),
        )
        .add_systems(OnEnter(AppState::InGame), spawn_scene)
        .add_systems(Update, scatter_on_keypress)
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
    landscape: Handle<Scene>,
    lod_high: Handle<Scene>,
    lod_medium: Handle<Scene>,
    lod_low: Handle<Scene>,
}

fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Scenes {
        landscape: asset_server.load("landscape_flat_large.glb#Scene0"),
        lod_high: asset_server.load("trees_high_lod.glb#Scene0"),
        lod_medium: asset_server.load("trees_medium_lod.glb#Scene0"),
        lod_low: asset_server.load("trees_low_lod.glb#Scene0"),
    });
}

fn check_assets_loaded(
    mut next_state: ResMut<NextState<AppState>>,
    asset_server: Res<AssetServer>,
    handles: Res<Scenes>,
) {
    let all_loaded = [
        handles.landscape.id(),
        handles.lod_high.id(),
        handles.lod_medium.id(),
        handles.lod_low.id(),
    ]
    .iter()
    .all(|id| {
        asset_server
            .get_load_state(*id)
            .is_some_and(|x| x.is_loaded())
    });

    if all_loaded {
        next_state.set(AppState::InGame);
    }
}

fn spawn_scene(
    mut cmd: Commands,
    handles: Res<Scenes>,
    mut ns_scatter: ResMut<NextState<ScatterState>>,
    mut ns_height_map: ResMut<NextState<HeightMapState>>,
) {
    cmd.spawn((
        SceneRoot(handles.landscape.clone()),
        ScatterRoot::default(),
        ChunkRoot::default(),
        LodConfig::from(vec![10.0.into(), 35.0.into(), 85.0.into()]),
        children![(
            scatter_layer("Tree Layer"),
            DistributionDensity(10.),
            InstanceRotationYaw {
                min: 0.,
                max: std::f32::consts::PI * 2.
            },
            InstanceScale { min: 1., max: 4. },
            WindAffected,
            Avoidance(3.),
            children![
                (SceneRoot(handles.lod_high.clone()),),
                (LevelOfDetail(1), SceneRoot(handles.lod_medium.clone()),),
                (LevelOfDetail(2), SceneRoot(handles.lod_low.clone()),)
            ]
        )],
    ));

    ns_height_map.set(HeightMapState::Setup);
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
