use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use std::num::NonZeroU32;

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Chunk>()
            .init_resource::<ChunkConfig>()
            .add_event::<SplitChunk>()
            .add_event::<MergeChunks>()
            .add_systems(Startup, setup_chunks)
            .add_systems(
                Update,
                (
                    update_chunk_lods,
                    (apply_splits, apply_merges).after(update_chunk_lods),
                ),
            );
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

impl ChunkConfig {
    /// Calculates the world size (width/depth) of a chunk at a given LOD level.
    pub fn get_chunk_world_size(&self, level: u32) -> f32 {
        if let Some(lod) = self.lods.get(level as usize) {
            lod.chunk_size_scalar as f32 * self.base_chunk_size
        } else {
            self.lods[self.get_max_lod_level() as usize].chunk_size_scalar as f32 * self.base_chunk_size
        }
    }

    /// Calculates the world-space center of a chunk from its grid coordinate and level.
    pub fn get_center(&self, grid_coord: IVec3, level: u32) -> Vec3 {
        let world_size = self.get_chunk_world_size(level);
        let half_size = world_size / 2.0;

        grid_coord.as_vec3() * world_size + Vec3::new(half_size, 0.0, half_size)
    }

    pub fn get_size_scalar(&self, level: u32) -> u32 {
        if let Some(lod) = self.lods.get(level as usize) {
            lod.chunk_size_scalar
        } else {
            self.lods[self.get_max_lod_level() as usize].chunk_size_scalar
        }
    }

    pub fn get_max_lod_level(&self) -> u32 {
        (self.lods.len() - 1) as u32
    }

    pub fn get_lod_config(&self, level: u32) -> &LodConfig {
        &self.lods[level as usize]
    }

    /// Calculates the level, size, and world-space offsets for a chunk's children.
    pub fn calculate_child_data(&self, level: NonZeroU32, size: u32) -> ChildChunkData {
        let parent_world_size = size as f32 * self.base_chunk_size;
        let child_level = level.get() - 1;
        let child_size = self.get_size_scalar(child_level);
        let offset = parent_world_size / 4.0;
        let offsets = [
            Vec3::new(-offset, 0.0, -offset),
            Vec3::new(offset, 0.0, -offset),
            Vec3::new(-offset, 0.0, offset),
            Vec3::new(offset, 0.0, offset),
        ];

        ChildChunkData {
            level: child_level,
            size: child_size,
            offsets,
        }
    }
}

pub struct ChildChunkData {
    pub level: u32,
    pub size: u32,
    pub offsets: [Vec3; 4],
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
                Visibility::Visible,
                ViewVisibility::default(),
            ));
        }
    }
}

fn update_chunk_lods(
    config: Res<ChunkConfig>,
    center_query: Query<&GlobalTransform, With<ChunkCenter>>,
    chunk_query: Query<(Entity, &Chunk, &GlobalTransform)>,
    mut ew_split: EventWriter<SplitChunk>,
    mut ew_merge: EventWriter<MergeChunks>,
) {
    let Ok(center) = center_query.single() else {
        return;
    };

    let center_translation = center.translation();
    let max_lod_level = (config.lods.len() - 1) as u32;
    let mut potential_parents: HashMap<IVec3, Vec<Entity>> = HashMap::new();

    for (entity, chunk, chunk_transform) in &chunk_query {
        if chunk.level > 0 {
            let dist = center_translation.distance(chunk_transform.translation());

            let child_chunk_data =
                config.calculate_child_data(NonZeroU32::new(chunk.level).unwrap(), chunk.size);
            let child_lod_config = config.get_lod_config(child_chunk_data.level);

            if dist < child_lod_config.distance {
                ew_split.write(SplitChunk(entity));
            }

            continue;
        }

        if chunk.level < max_lod_level {
            let parent_level = chunk.level + 1;
            let parent_world_size = config.get_chunk_world_size(parent_level);
            let parent_grid_coord = (chunk_transform.translation() / parent_world_size)
                .floor()
                .as_ivec3();

            potential_parents
                .entry(parent_grid_coord)
                .or_default()
                .push(entity);
        }
    }

    for (parent_grid_coord, siblings) in potential_parents {
        if siblings.len() < 4 {
            continue;
        };

        let chunk_level = chunk_query.get(siblings[0]).unwrap().1.level;
        let merge_dist = config.lods[chunk_level as usize].distance;

        let parent_level = chunk_level + 1;
        let parent_center = config.get_center(parent_grid_coord, parent_level);

        if center_translation.distance(parent_center) <= merge_dist {
            continue;
        }

        ew_merge.write(MergeChunks {
            siblings,
            parent_center,
            parent_level,
        });
    }
}

#[derive(Event, BufferedEvent)]
struct SplitChunk(Entity);

#[derive(Event, BufferedEvent)]
struct MergeChunks {
    siblings: Vec<Entity>,
    parent_center: Vec3,
    parent_level: u32,
}

fn apply_splits(
    mut commands: Commands,
    config: Res<ChunkConfig>,
    mut split_events: EventReader<SplitChunk>,
    chunk_query: Query<(&Chunk, &GlobalTransform)>,
) {
    for event in split_events.read() {
        let parent_entity = event.0;

        let Ok((parent_chunk, parent_transform)) = chunk_query.get(parent_entity) else {
            continue;
        };

        let child_chunk_data = config.calculate_child_data(
            NonZeroU32::new(parent_chunk.level)
                .expect("Cannot split chunk at level 0!"),
            parent_chunk.size,
        );

        commands.entity(parent_entity).despawn();

       commands.spawn_batch(child_chunk_data.offsets.map(|offset| {
           (
               Chunk {
                   level: child_chunk_data.level,
                   size: child_chunk_data.size,
               },
               Transform::from_translation(parent_transform.translation() + offset),
               GlobalTransform::from_translation(parent_transform.translation() + offset),
               Visibility::Visible,
               ViewVisibility::default(),
           )
       }));
    }
}

fn apply_merges(
    mut commands: Commands,
    config: Res<ChunkConfig>,
    mut merge_events: EventReader<MergeChunks>,
) {
    for event in merge_events.read() {
        for sibling_entity in &event.siblings {
            commands.entity(*sibling_entity).despawn();
        }

        commands.spawn((
            Chunk {
                level: event.parent_level,
                size: config.get_size_scalar(event.parent_level),
            },
            Transform::from_translation(event.parent_center),
            GlobalTransform::from_translation(event.parent_center),
            Visibility::Visible,
            ViewVisibility::default(),
        ));
    }
}
