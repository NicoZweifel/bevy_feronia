use crate::core::Sampler;
use crate::density_map::DensityMapSampler;
use crate::height_map::cpu_sampler::HeightMapCpuSampler;
use crate::prelude::*;
use crate::scatter::utils::*;
use bevy_derive::Deref;
use bevy_ecs::prelude::*;
use bevy_math::{Quat, Vec3};
use bevy_pbr::StandardMaterial;
use bevy_reflect::Reflect;
use bevy_transform::prelude::Transform;
use rand::Rng;
use rand_pcg::Pcg64;
use rand_pcg::rand_core::SeedableRng;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::slice::Iter;

#[derive(EntityEvent, Message, Component, Reflect)]
pub struct Scatter<T = StandardMaterial>
where
    T: ScatterMaterial,
{
    pub entity: Entity,
    _phantom: PhantomData<T>,
}

impl<T> Scatter<T>
where
    T: ScatterMaterial,
{
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            _phantom: PhantomData,
        }
    }
}

impl<T> From<Entity> for Scatter<T>
where
    T: ScatterMaterial,
{
    fn from(value: Entity) -> Self {
        Self::new(value)
    }
}

#[derive(EntityEvent, Message, Component, Reflect)]
pub struct ScatterChunk<T = StandardMaterial>
where
    T: ScatterMaterial,
{
    pub entity: Entity,
    pub scatter_layer: Entity,
    _phantom: PhantomData<T>,
}

impl<T> ScatterChunk<T>
where
    T: ScatterMaterial,
{
    pub fn new(entity: Entity, scatter_layer: Entity) -> Self {
        Self {
            entity,
            scatter_layer,
            _phantom: PhantomData,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ScatterResult {
    pub transform: Transform,
    pub seed: u64,
}

impl ScatterResult {
    // TODO add GPU pipeline
    pub fn try_from_container_and_modifiers<R: Rng + ?Sized>(
        container: &Container,
        modifiers: &InstanceModifiers,
        rng: &mut R,
        external_avoidance_data: &[AvoidanceData],
    ) -> Option<ScatterResult> {
        let instances_dim_f = container.instances_dim;
        let cell_width = container.size.x / instances_dim_f;
        let cell_depth = container.size.z / instances_dim_f;

        let world_corner_pos = container.transform.translation + container.corner;

        let local_cell_x_idx = rng.random_range(0.0..instances_dim_f).floor();
        let local_cell_z_idx = rng.random_range(0.0..instances_dim_f).floor();

        let snapped_world_cell_corner = world_corner_pos
            + Vec3::new(
                local_cell_x_idx * cell_width,
                0.0,
                local_cell_z_idx * cell_depth,
            );

        let mut final_world_pos =
            snapped_world_cell_corner + Vec3::new(cell_width / 2.0, 0.0, cell_depth / 2.0);

        let jitter_strength = modifiers.jitter.map_or(0., |j| **j).clamp(0.0, 1.0);

        if jitter_strength > 0. {
            let max_offset_x = (cell_width * jitter_strength) / 2.0;
            let max_offset_z = (cell_depth * jitter_strength) / 2.0;

            let random_offset = Vec3::new(
                rng.random_range(-max_offset_x..max_offset_x),
                0.0,
                rng.random_range(-max_offset_z..max_offset_z),
            );

            final_world_pos += random_offset;
        };

        if let Some(sampler) = &modifiers.density_sampler
            && rng.random::<f32>() > sampler.sample(final_world_pos)
        {
            return None;
        }

        if external_avoidance_data.iter().any(|obstacle| {
            final_world_pos
                .with_y(0.)
                .distance_squared(obstacle.world_pos.with_y(0.))
                < (obstacle.radius_sq * obstacle.scale)
        }) {
            return None;
        }

        let mut instance_pos = final_world_pos - container.transform.translation;
        instance_pos.y = match modifiers.map_height {
            None => container.height,
            Some(_) => {
                modifiers.height_sampler.sample(final_world_pos) - container.transform.translation.y
            }
        };

        let final_scale = modifiers.scale.map_or(1.0, |s| {
            if s.min == s.max {
                s.min
            } else {
                rng.random_range(s.min..s.max)
            }
        });

        let final_rotation = modifiers.rotation.map_or(Quat::IDENTITY, |r| {
            Quat::from_rotation_y(if r.min == r.max {
                r.min
            } else {
                rng.random_range(r.min..r.max)
            })
        });

        let instance_seed = generate_instance_seed(container.seed, final_world_pos);

        Some(ScatterResult {
            seed: instance_seed,
            transform: Transform {
                translation: instance_pos,
                rotation: final_rotation,
                scale: Vec3::splat(final_scale),
            },
        })
    }
}

impl PartialEq for ScatterResult {
    fn eq(&self, other: &Self) -> bool {
        self.transform.translation.x.to_bits() == other.transform.translation.x.to_bits()
            && self.transform.translation.y.to_bits() == other.transform.translation.y.to_bits()
            && self.transform.translation.z.to_bits() == other.transform.translation.z.to_bits()
    }
}

impl Eq for ScatterResult {}

impl Hash for ScatterResult {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.transform.translation.x.to_bits().hash(state);
        self.transform.translation.y.to_bits().hash(state);
        self.transform.translation.z.to_bits().hash(state);
    }
}

#[derive(EntityEvent, Message, Clone, Debug)]
pub struct ScatterResults<T = StandardMaterial>
where
    T: ScatterMaterial,
{
    pub entity: Entity,
    pub data: Vec<ScatterResult>,
    pub chunk: Option<Entity>,
    pub layer: Entity,
    pub root: Entity,
    pub seed: u64,
    pub container_transform: Transform,
    _phantom: PhantomData<T>,
}

impl<T> From<ScatterTaskData> for ScatterResults<T>
where
    T: ScatterMaterial,
{
    fn from(task_data: ScatterTaskData) -> Self {
        let density_sampler = task_data
            .density_map_image
            .as_ref()
            .map(|img| DensityMapSampler::new(img, task_data.container.root_size));

        let height_sampler = task_data
            .height_map_config
            .as_ref()
            .and_then(|cfg| {
                task_data
                    .height_map_image
                    .as_ref()
                    .map(|img| HeightMapSampler::Cpu(HeightMapCpuSampler::new(img, cfg)))
            })
            .unwrap_or(HeightMapSampler::Default(DefaultSampler));

        let instance_modifiers = InstanceModifiers::from(&task_data)
            .with_density_sampler(&density_sampler)
            .with_height_sampler(&height_sampler);

        ScatterResults::<T>::from_container_with_data(
            &task_data.container,
            instance_modifiers,
            &task_data.external_avoidance_data,
        )
    }
}

impl<T> ScatterResults<T>
where
    T: ScatterMaterial,
{
    pub fn get(&self) -> &Vec<ScatterResult> {
        &self.data
    }

    pub fn iter(&self) -> Iter<'_, ScatterResult> {
        self.data.iter()
    }

    pub fn new(
        entity: Entity,
        root: Entity,
        layer: Entity,
        chunk: Option<Entity>,
        data: Vec<ScatterResult>,
        seed: u64,
        container_transform: Transform,
    ) -> Self {
        Self {
            entity,
            root,
            layer,
            chunk,
            data,
            seed,
            container_transform,
            _phantom: PhantomData,
        }
    }

    pub fn with_data(mut self, data: Vec<ScatterResult>) -> Self {
        self.data = data;
        self
    }

    pub fn from_container_with_data(
        container: &Container,
        modifiers: InstanceModifiers,
        external_avoidance_data: &[AvoidanceData],
    ) -> ScatterResults<T>
    where
        T: ScatterMaterial,
    {
        let mut rng = Pcg64::seed_from_u64(container.seed);
        let mut results = Vec::new();

        let density = modifiers.density.map_or(1.0, |d| **d).clamp(0.0, 1.0);

        for _ in 0..(container.instances_dim as u32).pow(2) {
            if rng.random::<f32>() > density {
                continue;
            }

            let Some(candidate) = ScatterResult::try_from_container_and_modifiers(
                container,
                &modifiers,
                &mut rng,
                external_avoidance_data,
            ) else {
                continue;
            };

            results.push(candidate);
        }

        ScatterResults::<T>::from(container).with_data(results)
    }
}

impl<T> From<&Container> for ScatterResults<T>
where
    T: ScatterMaterial,
{
    fn from(value: &Container) -> Self {
        Self::new(
            value.entity,
            value.root_entity,
            value.layer_entity,
            value.chunk_entity,
            vec![],
            value.seed,
            value.transform,
        )
    }
}

#[derive(EntityEvent, Message, Clone)]
pub struct ScatterFinished<T = StandardMaterial>
where
    T: ScatterMaterial,
{
    pub entity: Entity,
    _phantom: PhantomData<T>,
}

impl<T> From<Entity> for ScatterFinished<T>
where
    T: ScatterMaterial,
{
    fn from(value: Entity) -> Self {
        Self::new(value)
    }
}

impl<T> ScatterFinished<T>
where
    T: ScatterMaterial,
{
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            _phantom: PhantomData,
        }
    }
}

#[derive(EntityEvent, Message, Clone, Deref, Debug)]
pub struct ClearScatterLayer(pub Entity);

impl From<Entity> for ClearScatterLayer {
    fn from(value: Entity) -> Self {
        Self(value)
    }
}

#[derive(EntityEvent, Message, Clone, Deref, Debug)]
pub struct ClearScatterRoot(pub Entity);

impl From<Entity> for ClearScatterRoot {
    fn from(value: Entity) -> Self {
        Self(value)
    }
}
