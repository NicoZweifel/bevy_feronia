use crate::height_map::cpu_sampler::HeightMapCpuSampler;
use crate::prelude::Sampler;
use bevy::math::Vec3;

pub struct DefaultSampler;

impl Sampler for DefaultSampler {
    fn sample(&self, _world_pos: Vec3) -> f32 {
        0.
    }
}

impl<'a> Sampler for HeightMapSampler<'a> {
    fn sample(&self, world_pos: Vec3) -> f32 {
        match self {
            HeightMapSampler::Default(sampler) => sampler.sample(world_pos),
            HeightMapSampler::Cpu(sampler) => sampler.sample(world_pos),
        }
    }
}

pub enum HeightMapSampler<'a> {
    Default(DefaultSampler),
    Cpu(HeightMapCpuSampler<'a>),
}
