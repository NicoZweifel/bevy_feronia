#[path = "utils/example.rs"]
mod example;

use bevy::color::palettes::tailwind::{GREEN_500, ORANGE_500, RED_500, YELLOW_500};
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::batching::NoAutomaticBatching;
use bevy::render::primitives::{Aabb, MeshAabb};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::render::view::VisibilityRange;
use bevy_feronia::chunking::plugin::ChunkPlugin;
use bevy_feronia::prelude::*;
use example::*;
use noise::{NoiseFn, Perlin};
use rand::Rng;

fn main() -> AppExit {
    App::new()
        .insert_resource(Wind {
            enable_billboarding: true,
            enable_edge_correction: true,
            strength: 0.8,
            micro_strength: 0.2,
            round_exponent: 15.,
            edge_correction_factor: 0.001,
            high_quality: false,
            lod_threshold: 10.0,
            ..default()
        })
        .insert_resource(ChunkDebugConfig {
            lod_colors: vec![RED_500.into(), ORANGE_500.into(), YELLOW_500.into()],
            aabb_color: GREEN_500.into(),
        })
        .init_resource::<FoliageConfig>()
        .insert_resource(ExamplePluginOptions {
            no_indirect_drawing: true,
        })
        .add_plugins((
            ExamplePlugin,
            WindPlugin,
            InstancedWindAffectedPlugin,
            ChunkPlugin,
        ))
        .insert_resource(ChunkConfig {
            lods: vec![
                // Level 0: High
                LodConfig {
                    distance: 40.0,
                    chunk_size_scalar: 1,
                },
                // Level 1: Medium
                LodConfig {
                    distance: 80.0,
                    chunk_size_scalar: 2,
                },
                // Level 2: Low
                LodConfig {
                    distance: f32::MAX,
                    chunk_size_scalar: 4,
                },
            ],
            base_chunk_size: 4.0,
            world_size_in_chunks: 4,
        })
        .add_systems(Startup, (setup, setup_density_map))
        .add_systems(Update, populate_chunks)
        .run()
}

fn setup(mut cmd: Commands, assets: Res<AssetServer>) {
    cmd.spawn((
        SceneRoot(assets.load("grass_low_lod.glb#Scene0")),
        WindAffected,
    ));
    cmd.spawn((SceneRoot(assets.load("grass.glb#Scene0")), WindAffected));
}

fn setup_density_map(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    config: Res<FoliageConfig>,
) {
    let size = config.density_map_size;
    let mut data_buffer = vec![0; (size * size) as usize];

    let perlin = Perlin::new(1);
    let sample_scale = 5.0;

    for y in 0..size {
        for x in 0..size {
            let point = [x as f64 / size as f64, y as f64 / size as f64];

            let noise_value = perlin.get([point[0] * sample_scale, point[1] * sample_scale]);

            let byte_value = ((noise_value * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0) as u8;

            let pixel_index = (y * size + x) as usize;
            data_buffer[pixel_index] = byte_value;
        }
    }

    let mut density_image = Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data_buffer,
        TextureFormat::R8Unorm,
        default(),
    );

    density_image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        label: Some("Foliage Density Sampler".into()),
        address_mode_u: ImageAddressMode::MirrorRepeat,
        address_mode_v: ImageAddressMode::MirrorRepeat,
        ..default()
    });

    let handle = images.add(density_image);
    commands.insert_resource(FoliageDensityMap(handle));

    info!("Generated foliage density map.");
}

// TODO see below
#[derive(Resource)]
pub struct FoliageConfig {
    /// How many instances fit along one dimension of a base (1x1) chunk.
    pub instances_per_base_chunk_dim: u32,
    /// The size of a single instance cell.
    pub cell_size: f32,
    /// The random offset applied to each instance.
    pub jitter_amount: f32,
    /// Resolution of the density map.
    pub density_map_size: u32,
}

impl Default for FoliageConfig {
    fn default() -> Self {
        Self {
            instances_per_base_chunk_dim: 32,
            cell_size: 0.12,
            jitter_amount: 0.06,
            density_map_size: 1024,
        }
    }
}

#[derive(Resource)]
struct FoliageDensityMap(Handle<Image>);

fn populate_chunks(
    mut commands: Commands,
    chunk_config: Res<ChunkConfig>,
    density_map: Res<FoliageDensityMap>,
    images: Res<Assets<Image>>,
    foliage_config: Res<FoliageConfig>,
    new_chunks_query: Query<
        (Entity, &Chunk, &GlobalTransform),
        (With<Chunk>, Without<WindAffected>),
    >,
    prototypes: Res<WindAffectedTypes<InstancedWindAffectedMaterial>>,
    mut materials: ResMut<Assets<InstancedWindAffectedMaterial>>,
    meshes: Res<Assets<Mesh>>,
    mut high_q_material_handle: Local<Option<Handle<InstancedWindAffectedMaterial>>>,
) {
    let Some(density_image) = images.get(&density_map.0) else {
        return;
    };

    if prototypes.get().is_empty() {
        return;
    }

    if high_q_material_handle.is_none() {
        let high_q_prototype = prototypes.get().last().unwrap();
        let mut material = materials.get(&high_q_prototype.material).unwrap().clone();
        material.wind.high_quality = true;
        material.controlled = true;
        *high_q_material_handle = Some(materials.add(material));
    }

    let hq_material_handle = high_q_material_handle.as_ref().unwrap();
    let top_lod_config = chunk_config.lods.last().unwrap();
    let total_world_size = chunk_config.world_size_in_chunks as f32
        * top_lod_config.chunk_size_scalar as f32
        * chunk_config.base_chunk_size;

    // TODO async cpu sampling & gpu sampling directly to buffer
    let sampler = DensityMapSampler::new(density_image, total_world_size);

    let mut rng = rand::rng();

    for (entity, chunk, chunk_transform) in &new_chunks_query {
        let lod_level = chunk.level as usize;
        let current_lod_config = chunk_config.lods.get(lod_level).unwrap();
        let current_lod_dist = current_lod_config.distance;

        let start_margin = if lod_level == 0 {
            0.0..0.0
        } else {
            let prev_lod_dist = chunk_config.lods[lod_level - 1].distance;
            prev_lod_dist - chunk_config.get_chunk_world_size(chunk.level)..prev_lod_dist
        };

        let end_margin = if lod_level as u32 == chunk_config.get_max_lod_level() {
            f32::MAX..f32::MAX
        } else {
            current_lod_dist - chunk_config.get_chunk_world_size(chunk.level)..current_lod_dist
        };

        let (prototype, material_handle) = if chunk.level == 0 {
            let proto = prototypes.get().last().unwrap();
            (proto, hq_material_handle.clone())
        } else {
            let proto = prototypes.get().first().unwrap();
            (proto, proto.material.clone())
        };

        let base_chunk_world_size =
            foliage_config.instances_per_base_chunk_dim as f32 * foliage_config.cell_size;
        let chunk_world_size = chunk.size as f32 * base_chunk_world_size;

        let chunk_center = chunk_transform.translation();
        let chunk_corner =
            chunk_center - Vec3::new(chunk_world_size / 2.0, 0.0, chunk_world_size / 2.0);

        let instances_per_dim = foliage_config.instances_per_base_chunk_dim * chunk.size;

        let instances = (0..instances_per_dim.pow(2))
            .filter_map(|i| {
                let local_x = i % instances_per_dim;
                let local_z = i / instances_per_dim;

                let instance_offset_x = local_x as f32 * foliage_config.cell_size;
                let instance_offset_z = local_z as f32 * foliage_config.cell_size;

                let x_jitter =
                    rng.random_range(-foliage_config.jitter_amount..foliage_config.jitter_amount);
                let z_jitter =
                    rng.random_range(-foliage_config.jitter_amount..foliage_config.jitter_amount);

                let instance_world_pos = Vec3::new(
                    chunk_corner.x + instance_offset_x + x_jitter,
                    0.0,
                    chunk_corner.z + instance_offset_z + z_jitter,
                );

                let density = match chunk.level {
                    0 => 1.0,
                    1 => 0.5,
                    _ => 0.1,
                };

                let density = sampler.sample(instance_world_pos) * density;

                if rng.random::<f32>() > density {
                    return None;
                }

                Some(InstanceData {
                    position: instance_world_pos,
                    scale: rng.random_range(1.0..3.0),
                    color: LinearRgba::from(Color::hsla(78., 0.98, 0.5, 1.0)).to_f32_array(),
                    index: i,
                })
            })
            .collect::<Vec<_>>();

        if instances.is_empty() {
            continue;
        }

        let mesh_handle = prototype.mesh.clone();
        let mesh_aabb = meshes.get(&mesh_handle).unwrap().compute_aabb().unwrap();
        let (mut min_point, mut max_point) = (Vec3::MAX, Vec3::MIN);

        for instance in &instances {
            let instance_min =
                instance.position + <Vec3A as Into<Vec3>>::into(mesh_aabb.min() * instance.scale);
            let instance_max =
                instance.position + <Vec3A as Into<Vec3>>::into(mesh_aabb.max() * instance.scale);
            min_point = min_point.min(instance_min);
            max_point = max_point.max(instance_max);
        }

        let chunk_center = chunk_transform.translation();

        let local_min = min_point - chunk_center;
        let local_max = max_point - chunk_center;

        let local_aabb = Aabb::from_min_max(local_min, local_max);

        commands.entity(entity).insert((
            InstancedWindAffectedMaterial::component(material_handle),
            Mesh3d(mesh_handle),
            InstanceMaterialData(instances),
            NoAutomaticBatching,
            WindAffected,
            WindAffectedReady,
            Aabb::from(local_aabb),
            VisibilityRange {
                end_margin,
                start_margin,
                use_aabb: false,
            },
        ));
    }
}

// TODO see above
struct DensityMapSampler {
    image_data: Vec<u8>,
    image_size: u32,
    total_world_size: f32,
    center_offset: f32,
}

// TODO see above
impl DensityMapSampler {
    /// Creates a new sampler from the density map image and world configuration.
    fn new(image: &Image, total_world_size: f32) -> Self {
        Self {
            image_data: image.data.clone().unwrap(),
            image_size: image.texture_descriptor.size.width,
            total_world_size,
            center_offset: total_world_size / 2.0,
        }
    }

    /// Takes a world position and returns the density value (0.0 to 1.0) at that point.
    fn sample(&self, world_pos: Vec3) -> f32 {
        // Convert world position to a normalized [0, 1] UV coordinate.
        let uv_x = ((world_pos.x + self.center_offset) / self.total_world_size).clamp(0.0, 1.0);
        let uv_y = ((world_pos.z + self.center_offset) / self.total_world_size).clamp(0.0, 1.0);

        // Convert UV coordinate to a pixel coordinate.
        let pixel_x = (uv_x * (self.image_size - 1) as f32).round() as u32;
        let pixel_y = (uv_y * (self.image_size - 1) as f32).round() as u32;

        // Index the raw data buffer to get the density byte.
        let pixel_index = (pixel_y * self.image_size + pixel_x) as usize;
        let sampled_byte = self.image_data.get(pixel_index).copied().unwrap_or(0);

        // Convert the byte back to a 0.0-1.0 float.
        sampled_byte as f32 / 255.0
    }
}
