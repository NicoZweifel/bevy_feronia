use crate::prelude::*;
use bevy::camera::primitives::Aabb;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;
use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};
use rand::SeedableRng;
use rand::prelude::IndexedRandom;
use rand_pcg::Pcg64;

pub trait ScatterMaterial<TIn = StandardMaterial>: Asset + Clone
where
    TIn: Material,
{
    // TODO refactor
    fn create_material(
        base: Option<TIn>,
        noise_texture: Handle<Image>,
        properties: &ScatterAssetProperties,
    ) -> Self;
    fn update_material(material: &mut Self, wind: Wind, options: MaterialOptions);

    fn component(material: Handle<Self>) -> impl Component;

    fn spawn(
        cmd: Commands,
        mr_spawn: MessageReader<SpawnProtoTypes<Self>>,
        prototype_assets: Res<Assets<ScatterAsset<Self>>>,
        q_chunks: Query<(&GlobalTransform, &ChunkLevel), (With<Chunk>, Without<Merging>)>,
        q_root: Query<&LodConfig, With<ScatterRoot>>,
        q_layers: Query<(), With<ScatterChunked>>,
    );
}

impl ScatterMaterial for StandardMaterial {
    fn create_material(
        base: Option<StandardMaterial>,
        _noise_texture: Handle<Image>,
        _properties: &ScatterAssetProperties,
    ) -> StandardMaterial {
        base.unwrap_or_default()
    }

    fn update_material(_material: &mut StandardMaterial, _wind: Wind, _options: MaterialOptions) {}

    fn component(material: Handle<StandardMaterial>) -> impl Component {
        MeshMaterial3d(material)
    }

    fn spawn(
        mut cmd: Commands,
        mut mr_spawn: MessageReader<SpawnProtoTypes<StandardMaterial>>,
        prototype_assets: Res<Assets<ScatterAsset<StandardMaterial>>>,
        q_chunks: Query<(&GlobalTransform, &ChunkLevel), (With<Chunk>, Without<Merging>)>,
        q_root: Query<&LodConfig, With<ScatterRoot>>,
        q_layers: Query<(), With<ScatterChunked>>,
    ) {
        for event in mr_spawn.read() {
            debug!("Spawning extended wind affected!");

            let chunk_level = event
                .trigger
                .chunk
                .map(|x| q_chunks.get(x).map(|(_, lvl)| lvl).ok())
                .flatten()
                .cloned()
                .unwrap_or_default();

            let is_chunked =
                event.trigger.chunk.is_some() && q_layers.get(event.trigger.layer).is_ok();

            let prototypes: Vec<_> = event
                .items
                .iter()
                .filter_map(|h| prototype_assets.get(&**h))
                .collect();

            let mut name_map: HashMap<Name, Vec<&ScatterAsset<_>>> = HashMap::new();

            prototypes.iter().for_each(|p| {
                let name = p.properties.name.clone().unwrap_or_else(|| Name::new(""));
                name_map.entry(name).or_default().push(*p);
            });

            if name_map.is_empty() {
                continue;
            }

            let mut sorted_names: Vec<&Name> = name_map.keys().collect();
            sorted_names.sort();

            let parent = event.trigger.chunk.unwrap_or(event.trigger.layer);

            let Ok(lod_config) = q_root.get(event.trigger.root) else {
                warn!("Couldn't get ScatterRoot!");
                continue;
            };

            cmd.spawn_batch(
                event
                    .trigger
                    .data
                    .iter()
                    .flat_map(|res| {
                        let mut rng = Pcg64::seed_from_u64(res.seed);

                        let Some(chosen_name) = sorted_names.choose(&mut rng) else {
                            return vec![];
                        };

                        let Some(prototypes_to_spawn) = name_map.get(*chosen_name) else {
                            return vec![];
                        };

                        prototypes_to_spawn
                            .iter()
                            .filter(|p| {
                                if is_chunked {
                                    *p.properties.lod == *chunk_level
                                } else {
                                    *p.properties.lod >= *chunk_level
                                }
                            })
                            .map(move |p| {
                                let visibility_range =
                                    lod_config.get_visibility_range(p.properties.lod);
                                (
                                    res.transform,
                                    Mesh3d(p.mesh().clone()),
                                    MeshMaterial3d(p.material().clone()),
                                    ChildOf(parent),
                                    visibility_range,
                                    ScatteredInstance(event.trigger.layer),
                                )
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>(),
            );
        }
    }
}

#[repr(C)]
#[derive(ShaderType, Clone, Zeroable, Copy)]
pub struct WindUniform {
    pub direction: Vec2,
    pub strength: f32,
    pub noise_scale: f32,
    pub scroll_speed: f32,
    pub bend_exponent: f32,
    pub curve_factor: f32,
    pub micro_strength: f32,
    pub s_curve_speed: f32,
    pub s_curve_strength: f32,
    pub s_curve_frequency: f32,
    pub bop_speed: f32,
    pub bop_strength: f32,
    pub twist_strength: f32,

    // TODO move to uniform for Options/InstanceData
    pub edge_correction_factor: f32,
    pub aabb_min: Vec3,
    pub aabb_max: Vec3,
    pub debug_color: Vec4,
}

impl From<&Wind> for WindUniform {
    fn from(wind: &Wind) -> Self {
        WindUniform {
            direction: wind.direction,
            strength: wind.strength,
            noise_scale: wind.noise_scale,
            scroll_speed: wind.scroll_speed,
            bend_exponent: wind.bend_exponent,
            micro_strength: wind.micro_strength,
            s_curve_speed: wind.s_curve_speed,
            s_curve_strength: wind.s_curve_strength,
            s_curve_frequency: wind.s_curve_frequency,
            bop_speed: wind.bop_speed,
            bop_strength: wind.bop_strength,
            twist_strength: wind.twist_strength,
            edge_correction_factor: 0.,
            curve_factor: 0.,
            aabb_max: Vec3::splat(1.),
            aabb_min: Vec3::splat(0.),
            debug_color: Vec4::splat(1.),
        }
    }
}

impl WindUniform {
    pub fn with_curve_factor(mut self, curve_factor: f32) -> Self {
        self.curve_factor = curve_factor;
        self
    }

    pub fn with_edge_correction_factor(mut self, edge_correction_factor: f32) -> Self {
        self.edge_correction_factor = edge_correction_factor;
        self
    }

    pub fn with_aabb(mut self, aabb: &Aabb) -> Self {
        self.aabb_min = aabb.min().into();
        self.aabb_max = aabb.max().into();
        self
    }

    pub fn with_debug_color(mut self, color: Vec4) -> Self {
        self.debug_color = color;
        self
    }
}

bitflags! {
    #[repr(C)]
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Pod, Zeroable)]
    pub struct WindAffectedKey: u64 {
        const ENABLE_BILLBOARDING    = 1 << 0;
        const ENABLE_EDGE_CORRECTION = 1 << 1;
        const WIND_LOW_QUALITY = 1 << 2;
        const FAST_NORMALS = 1 << 3;
        const DEBUG = 1 << 4;
        const WIND_AFFECTED= 1 << 5;
        const SUBSURFACE_SCATTERING = 1 << 6;
        const STATIC_BEND = 1 << 7;
        const ANALYTICAL_NORMALS = 1 << 8;
    }
}
