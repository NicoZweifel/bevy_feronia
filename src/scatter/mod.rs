use crate::height_map::cpu_sampler::HeightMapCpuSampler;
use crate::prelude::*;
use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use bevy::render::view::Layer;
use rand::Rng;
use rand::rngs::ThreadRng;
use std::borrow::Cow;
use std::slice::Iter;

#[derive(BufferedEvent, Event, Component, Reflect, Deref)]
pub struct Scatter(pub Entity);

#[derive(Component, Reflect)]
#[require(Transform, Visibility, GlobalTransform)]
#[reflect(Component)]
pub struct ScatterLayer;

#[derive(Component, Debug, Clone, Reflect, Deref)]
#[reflect(Component)]
#[relationship(relationship_target = ScatterRoot)]
pub struct LayerOf(pub Entity);

#[derive(Component, Debug, Clone, Reflect, Deref, Default)]
#[reflect(Component)]
#[require(Transform, Visibility, GlobalTransform)]
#[relationship_target(relationship = LayerOf)]
pub struct ScatterRoot(Vec<Entity>);

#[derive(Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct ScatterLayerEnabled(bool);

#[derive(Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct DistributionDensity(pub f32);

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct DistributionPattern {
    pub density_map: Handle<Image>,
    pub scale: f32,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct InstanceRotationYaw {
    pub min: f32,
    pub max: f32,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct InstanceScale {
    pub min: f32,
    pub max: f32,
}

#[derive(Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct InstanceJitter(pub f32);

#[derive(Clone, Debug)]
pub struct ScatterResult {
    pub global_transform: Transform,
    pub layer: Entity,
}

#[derive(EntityEvent, BufferedEvent, Clone, Debug)]
pub struct ScatterResults {
    pub results: Vec<ScatterResult>,
    pub chunk: Option<Entity>,
}

impl ScatterResults {
    pub fn get(&self) -> &Vec<ScatterResult> {
        &self.results
    }

    pub fn iter(&self) -> Iter<'_, ScatterResult> {
        self.results.iter()
    }
}

pub fn scatter_layer(name: impl Into<Cow<'static, str>>) -> impl Bundle {
    (Name::new(name), ScatterLayer)
}

pub struct ScatterPlugin;

impl Plugin for ScatterPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Scatter>()
            .add_event::<ScatterResults>()
            .add_systems(
                Update,
                (compute_root_aabb, set_layer_root, generate_scatter_points),
            );
    }
}

fn combine_aabbs(aabb1: &Aabb, aabb2: &Aabb) -> Aabb {
    let min_x = aabb1.min().x.min(aabb2.min().x);
    let min_y = aabb1.min().y.min(aabb2.min().y);
    let min_z = aabb1.min().z.min(aabb2.min().z);
    let max_x = aabb1.max().x.max(aabb2.max().x);
    let max_y = aabb1.max().y.max(aabb2.max().y);
    let max_z = aabb1.max().z.max(aabb2.max().z);

    Aabb::from_min_max(
        Vec3::new(min_x, min_y, min_z),
        Vec3::new(max_x, max_y, max_z),
    )
}

fn compute_root_aabb(
    mut commands: Commands,
    root_query: Query<Entity, (With<ScatterRoot>, Without<Aabb>)>,
    children_query: Query<&Children>,
    aabb_query: Query<&Aabb>,
) {
    for root_entity in &root_query {
        let mut root_aabb: Option<Aabb> = None;

        for descendant_entity in children_query.iter_descendants(root_entity) {
            if let Ok(descendant_aabb) = aabb_query.get(descendant_entity) {
                match root_aabb.as_mut() {
                    Some(existing_aabb) => {
                        combine_aabbs(existing_aabb, descendant_aabb);
                    }
                    None => {
                        root_aabb = Some(descendant_aabb.clone());
                    }
                }
            }
        }

        if let Some(aabb) = root_aabb {
            commands.entity(root_entity).insert(aabb);
        }
    }
}

pub fn set_layer_root(
    mut cmd: Commands,
    layer_query: Query<(Entity, &ChildOf), (With<ScatterLayer>, Without<LayerOf>)>,
) {
    for (layer, parent) in &layer_query {
        cmd.entity(layer).insert(LayerOf(parent.get()));
    }
}

pub fn generate_scatter_points(
    mut cmd: Commands,
    q_root: Query<(
        Entity,
        Option<&ChunkConfig>,
        Option<&ChunkRoot>,
        &ScatterRoot,
        Option<&MapHeight>,
        &Aabb,
    )>,
    layer_query: Query<
        (
            Entity,
            Option<&Name>,
            Option<&DistributionDensity>,
            Option<&DistributionPattern>,
            Option<&InstanceRotationYaw>,
            Option<&InstanceScale>,
            Option<&ScatterLayerEnabled>,
            Option<&InstanceJitter>,
            &GlobalTransform,
        ),
        With<ScatterLayer>,
    >,
    height_map_cfg: Option<Res<HeightMapConfig>>,
    chunk_query: Query<(&ChunkLevel, &ChunkSize, &GlobalTransform), With<Chunk>>,
    height_map: Option<Res<HeightMap>>,
    images: Res<Assets<Image>>,
    mut er_scatter: EventReader<Scatter>,
    mut ew_results: EventWriter<ScatterResults>,
) {
    let height_map_image = match height_map {
        None => None,
        Some(x) => images.get(&x.0),
    };

    let total_world_size = height_map_cfg.map_or_else(|| 0.0, |x| x.world_size);

    let height_sampler = height_map_image.map_or_else(
        || HeightMapSampler::Default(DefaultSampler),
        |x| HeightMapSampler::CpuHeightMap(HeightMapCpuSampler::new(x, total_world_size)),
    );

    let mut rng = rand::rng();

    for e in er_scatter.read() {
        let Ok((root, chunk_config, child_chunks, layers, map_height, aabb)) = q_root.get(**e)
        else {
            warn!("ScatterRoot not found!");
            continue;
        };

        for layer_entity in layers.iter() {
            let Ok((
                layer_entity,
                layer_name,
                density_dist,
                pattern_dist,
                rotation,
                scale,
                enabled,
                jitter,
                layer_gtf,
            )) = layer_query.get(layer_entity)
            else {
                warn!("ScatterLayer not found!");
                continue;
            };

            let density_sampler = pattern_dist
                .and_then(|p| images.get(&p.density_map))
                .map(|density_image| DensityMapSampler::new(density_image, total_world_size));

            let scatter_layer_enabled = ScatterLayerEnabled(true);

            if !**enabled.unwrap_or(&scatter_layer_enabled) {
                let name = layer_name
                    .unwrap_or(&Name::new(layer_entity.to_string()))
                    .to_string();

                warn!("ScatterLayer {name} is disabled!");
                continue;
            }

            cmd.entity(layer_entity).insert(scatter_layer_enabled);

            let instances_dim = density_dist.map_or(10., |d| **d);

            if let (Some(chunk_config), Some(child_chunks)) = (chunk_config, child_chunks) {
                for chunk_entity in child_chunks.iter() {
                    let Ok((chunk_level, chunk_size, chunk_gtf)) = chunk_query.get(chunk_entity)
                    else {
                        warn!("Chunk not found!");
                        continue;
                    };

                    let chunk_corner = chunk_gtf.translation()
                        - Vec3::new(
                            chunk_config.get_chunk_world_size(**chunk_level) / 2.0,
                            0.0,
                            chunk_config.get_chunk_world_size(**chunk_level) / 2.0,
                        );

                    let results = (0..(instances_dim as u32).pow(2))
                        .filter_map(|i| {
                            let local_x = i as f32 % instances_dim;
                            let local_z = i as f32 / instances_dim;

                            let mut instance_world_pos = chunk_corner
                                + Vec3::new(local_x, 0.0, local_z)
                                + get_jitter(jitter, &mut rng);

                            instance_world_pos.y = match map_height {
                                None => 0.0,
                                Some(_) => height_sampler.sample(instance_world_pos),
                            };

                            if let Some(sampler) = &density_sampler {
                                if rng.random::<f32>() > sampler.sample(instance_world_pos) {
                                    return None;
                                }
                            }

                            let final_scale = scale.map_or(1.0, |s| rng.random_range(s.min..s.max));
                            let final_rotation = rotation.map_or(Quat::IDENTITY, |r| {
                                Quat::from_rotation_y(rng.random_range(r.min..r.max))
                            });

                            Some(ScatterResult {
                                layer: layer_entity,
                                global_transform: Transform {
                                    translation: instance_world_pos,
                                    rotation: final_rotation,
                                    scale: Vec3::splat(final_scale),
                                },
                            })
                        })
                        .collect::<Vec<_>>();

                    let results = ScatterResults {
                        results: results.clone(),
                        chunk: Some(chunk_entity),
                    };

                    cmd.trigger_targets(results.clone(), [root, layer_entity, chunk_entity]);
                    ew_results.write(results);
                }
            } else {
                let size = aabb.half_extents * 2.0;

                info!(
                    "Scattering {} instances for {}",
                    (instances_dim as u32).pow(2),
                    size
                );

                let jitter_value = jitter.map_or(0., |x| **x);

                let corner = layer_gtf.translation()
                    - <Vec3A as Into<Vec3>>::into(aabb.half_extents)
                    + Vec3::splat(jitter_value);

                let results = (0..(instances_dim as u32).pow(2))
                    .filter_map(|i| {
                        let world_x = i as f32 % instances_dim;
                        let world_z = i as f32 / instances_dim;


                        let mut instance_world_pos = corner
                            + Vec3::new(
                                world_x * (size.x - jitter_value* 2.) / instances_dim,
                                0.0,
                                world_z * (size.z - jitter_value * 2.) / instances_dim,
                            )
                            + get_jitter(jitter, &mut rng);

                        instance_world_pos.y = map_height.map_or_else(
                            || layer_gtf.translation().y,
                            |_| height_sampler.sample(instance_world_pos),
                        );

                        if let Some(sampler) = &density_sampler {
                            if rng.random::<f32>() > sampler.sample(instance_world_pos) {
                                return None;
                            }
                        }

                        let final_scale = scale.map_or(1.0, |s| rng.random_range(s.min..s.max));

                        let final_rotation = rotation.map_or(Quat::IDENTITY, |r| {
                            Quat::from_rotation_y(rng.random_range(r.min..r.max))
                        });

                        Some(ScatterResult {
                            layer: layer_entity,
                            global_transform: Transform {
                                translation: instance_world_pos,
                                rotation: final_rotation,
                                scale: Vec3::splat(final_scale),
                            },
                        })
                    })
                    .collect::<Vec<_>>();

                info!("Scattered {} instances", results.len());

                let results = ScatterResults {
                    results: results.clone(),
                    chunk: None,
                };

                cmd.trigger_targets(results.clone(), [root, layer_entity]);
                ew_results.write(results);
            }
        }
    }
}

fn get_jitter(jitter: Option<&InstanceJitter>, rng: &mut ThreadRng) -> Vec3 {
    jitter.map_or_else(
        || Vec3::ZERO,
        |x| Vec3::new(rng.random_range(-**x..**x), 0., rng.random_range(-**x..**x)),
    )
}
