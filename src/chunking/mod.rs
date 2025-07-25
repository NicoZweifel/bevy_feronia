use bevy::prelude::*;
use bevy::prelude::Visibility::Visible;

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Chunk>()
            .init_resource::<ChunkConfig>()
            .add_systems(Startup, setup_chunks)
            .add_systems(Update, update_chunk_lods);
    }
}

#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct Chunk {
    /// The level of detail (0=Low, 1=Medium, 2=High).
    pub level: u32,
    pub size: u32,
}

#[derive(Resource)]
pub struct ChunkConfig {
    pub lods: Vec<LodConfig>,
    /// The size of the world in top-level (Low LOD) chunks.
    pub world_size_in_chunks: u32,
    pub base_chunk_size: f32,
}

pub struct LodConfig {
    /// The distance at which a chunk of this level subdivides into its children.
    pub distance: f32,
    /// The size of a chunk at this level, as a multiple of the highest-LOD chunk size.
    pub chunk_size_scalar: u32,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub enum LodLevel {
    High,
    Medium,
    Low,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            // LODs are ordered from High (0) to Low (n).
            lods: vec![
                // Level 0: High
                LodConfig {
                    distance: 50.0,
                    chunk_size_scalar: 1,
                },
                // Level 1: Medium
                LodConfig {
                    distance: 75.0,
                    chunk_size_scalar: 2,
                },
                // Level 2: Low
                LodConfig {
                    distance: f32::MAX,
                    chunk_size_scalar: 4,
                },
            ],
            base_chunk_size: 8.0,
            world_size_in_chunks: 8,
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ChunkCenter;

fn setup_chunks(mut commands: Commands, config: Res<ChunkConfig>) {
    let top_lod_level = (config.lods.len() - 1) as u32;
    let top_lod_config = &config.lods[top_lod_level as usize];

    let total_world_size = config.world_size_in_chunks as f32
        * top_lod_config.chunk_size_scalar as f32
        * config.base_chunk_size;
    let center_offset = total_world_size / 2.0;

    info!("Spawning initial world grid...");

    for z in 0..config.world_size_in_chunks {
        for x in 0..config.world_size_in_chunks {
            let chunk_size_in_base_units = top_lod_config.chunk_size_scalar;
            let chunk_world_size = chunk_size_in_base_units as f32 * config.base_chunk_size;

            let world_x = (x as f32 * chunk_world_size + chunk_world_size / 2.0) - center_offset;
            let world_z = (z as f32 * chunk_world_size + chunk_world_size / 2.0) - center_offset;

            commands.spawn((
                Chunk {
                    level: top_lod_level,
                    size: chunk_size_in_base_units,
                },
                Transform::from_xyz(world_x, 0.0, world_z),
                GlobalTransform::default(),
                Visible,
                ViewVisibility::default(),
            ));
        }
    }
}

fn update_chunk_lods(
    mut commands: Commands,
    config: Res<ChunkConfig>,
    center_query: Query<&GlobalTransform, With<ChunkCenter>>,
    chunk_query: Query<(Entity, &Chunk, &GlobalTransform)>,
) {
    let Ok(center) = center_query.single() else {
        return;
    };

    let translation = center.translation();

    for (entity, chunk, chunk_transform) in &chunk_query {
        if chunk.level == 0{
            continue;
        }

        let child_level = chunk.level - 1;
        let child_lod_config = &config.lods[child_level as usize];
        let dist = translation.distance(chunk_transform.translation());

        if dist < child_lod_config.distance {
            let parent_world_size = chunk.size as f32 * config.base_chunk_size;

            commands.entity(entity).despawn();

            let child_lod_config = &config.lods[child_level as usize];
            let child_size = child_lod_config.chunk_size_scalar;

            let offset = parent_world_size / 4.0;
            let child_offsets = [
                Vec3::new(-offset, 0.0, -offset),
                Vec3::new(offset, 0.0, -offset),
                Vec3::new(-offset, 0.0, offset),
                Vec3::new(offset, 0.0, offset),
            ];

            for i in 0..4 {
                commands.spawn((
                    Chunk {
                        level: child_level,
                        size: child_size,
                    },
                    Transform::from_translation(chunk_transform.translation() + child_offsets[i]),
                    GlobalTransform::from_translation(chunk_transform.translation() + child_offsets[i]),
                    Visible,
                    ViewVisibility::default(),
                ));
            }
        }

        // TODO
        // Add logic here to check if a set of four child chunks are all out of distance.
        // If they are, despawn the four children and respawn the parent.
    }
}
