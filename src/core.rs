use crate::prelude::*;
use bevy::camera::primitives::Aabb;
use bevy::prelude::*;

pub trait ProtoTypes<T, P>
where
    T: Asset + Clone + Send + Sync,
    P: ProtoType<T>,
{
    fn values(&self) -> &Vec<P>;
    fn choose(&self)-> Option<&P>;
}

pub trait ProtoType<T>
where
    T: Asset + Clone,
{
    fn mesh(&self) -> Handle<Mesh>;
    fn material(&self) -> Handle<T>;
    fn wind(&self) -> &Wind;
    fn aabb(&self) -> &Aabb;
}


pub trait Sampler {
    fn sample(&self, world_pos: Vec3) -> f32;
}
