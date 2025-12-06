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
    pub fn try_from_container_and_modifiers(
        container: &Container,
        modifiers: &InstanceModifiers,
        rng: &mut impl Rng,
        external_avoidance_data: &ScatterOccupancyMap,
    ) -> Option<ScatterResult> {
        let instances_dim_f = container.instances_dim;
        let cell_width = container.size.x / instances_dim_f;
        let cell_depth = container.size.z / instances_dim_f;

        let world_corner_pos = container.global_transform.translation + container.corner;

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

        if modifiers
            .density_sampler
            .as_ref()
            .is_some_and(|sampler| rng.random::<f32>() > sampler.sample(final_world_pos))
        {
            return None;
        }

        if external_avoidance_data.is_occupied(final_world_pos) {
            return None;
        }

        final_world_pos.y = match modifiers.map_height {
            None => container.global_transform.translation.y + container.height,
            Some(_) => modifiers.height_sampler.sample(final_world_pos),
        };

        let container_rot_inv = container.global_transform.rotation.inverse();
        let world_offset= final_world_pos - container.global_transform.translation;
        let translation = container_rot_inv * world_offset;

        let scale = modifiers
            .scale
            .cloned()
            .map_or(Vec3::splat(1.0), |s| s.into_vec3(rng));

        let intended_world_rotation = modifiers
            .rotation
            .cloned()
            .map_or(Quat::IDENTITY, |r| r.into_quad(rng));

        let rotation = container_rot_inv * intended_world_rotation;
        let instance_seed = generate_instance_seed(container.seed, final_world_pos);

        Some(ScatterResult {
            seed: instance_seed,
            transform: Transform {
                translation,
                rotation,
                scale,
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
    pub container_global_transform: Transform,
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
            .with_density_sampler(density_sampler.as_ref())
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
            container_global_transform: container_transform,
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
        external_avoidance_data: &ScatterOccupancyMap,
    ) -> ScatterResults<T>
    where
        T: ScatterMaterial,
    {
        let mut rng = Pcg64::seed_from_u64(container.seed);

        let density = modifiers.density.map_or(1.0, |d| **d).clamp(0.0, 1.0);
        let total_cells = (container.instances_dim as u32).pow(2);

        let capacity = (total_cells as f32 * density).ceil() as usize;
        let mut results = Vec::with_capacity(capacity);

        results.extend((0..total_cells).filter_map(|_| {
            if rng.random::<f32>() > density {
                return None;
            }

            ScatterResult::try_from_container_and_modifiers(
                container,
                &modifiers,
                &mut rng,
                external_avoidance_data,
            )
        }));

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
            value.global_transform,
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
