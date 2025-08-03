use bevy::render::primitives::Aabb;
use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;
use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};
use crate::prelude::*;

pub trait WindAffectable<M, A, P, T>
where
    M: Material,
    A: Asset + Clone,
    P: ProtoTypes<A, T>,
    T: ProtoType<A>,
{
    fn create_material(
        base: Option<M>,
        wind: Wind,
        noise_texture: Handle<Image>,
        controlled: bool,
    ) -> A;
    fn update_material(materials: ResMut<Assets<A>>, wind: Wind);
    fn spawn(
        cmd: Commands,
        trigger: On<ScatterResults>,
        prototypes: &P,
        q_chunks: Query<(&GlobalTransform, &ChunkOf, &ChunkLevel), With<Chunk>>,
        q_chunk_config: Query<(&ChunkLodConfig, &Aabb), With<ChunkRoot>>,
    );

    fn component(material: Handle<A>) -> impl Component;
}

#[repr(C)]
#[derive(ShaderType, Clone, Pod, Zeroable, Copy)]
pub struct WindUniform {
    pub direction: Vec2,
    pub strength: f32,
    pub noise_scale: f32,
    pub scroll_speed: f32,
    pub bend_exponent: f32,
    pub round_exponent: f32,
    pub micro_strength: f32,
    pub micro_noise_scale: f32,
    pub micro_scroll_speed: f32,
    pub s_curve_speed: f32,
    pub s_curve_strength: f32,
    pub s_curve_frequency: f32,
    pub bop_speed: f32,
    pub bop_strength: f32,
    pub twist_strength: f32,
    pub edge_correction_factor: f32,
    pub lod_threshold: f32,
}

impl From<&Wind> for WindUniform {
    fn from(wind: &Wind) -> Self {
        WindUniform {
            direction: wind.direction,
            strength: wind.strength,
            noise_scale: wind.noise_scale,
            scroll_speed: wind.scroll_speed,
            bend_exponent: wind.bend_exponent,
            round_exponent: wind.round_exponent,
            micro_strength: wind.micro_strength,
            micro_noise_scale: wind.micro_noise_scale,
            micro_scroll_speed: wind.micro_scroll_speed,
            s_curve_speed: wind.s_curve_speed,
            s_curve_strength: wind.s_curve_strength,
            s_curve_frequency: wind.s_curve_frequency,
            bop_speed: wind.bop_speed,
            bop_strength: wind.bop_strength,
            twist_strength: wind.twist_strength,
            edge_correction_factor: wind.edge_correction_factor,
            lod_threshold: wind.lod_threshold,
        }
    }
}

bitflags! {
    #[repr(C)]
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Pod, Zeroable)]
    pub struct WindAffectedKey: u64 {
        const ENABLE_BILLBOARDING    = 1 << 0;
        const ENABLE_EDGE_CORRECTION = 1 << 1;
        const HIGH_QUALITY = 1 << 2;
        const FAST_NORMALS = 1 << 3;
    }
}
