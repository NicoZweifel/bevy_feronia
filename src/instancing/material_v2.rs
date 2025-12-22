use bevy_app::{App, Plugin};
use bevy_asset::{Asset, AssetPath, Handle, embedded_asset, embedded_path};
use bevy_camera::primitives::Aabb;
use bevy_color::{Color, LinearRgba};
use bevy_ecs::prelude::*;
use bevy_eidolon::prelude::*;
use bevy_image::Image;
use bevy_math::{Vec3, Vec4};
use bevy_mesh::{Mesh3d, MeshVertexBufferLayoutRef};
use bevy_pbr::StandardMaterial;
use bevy_platform::collections::hash_map::Entry;
use bevy_platform::collections::{HashMap, HashSet};
use bevy_reflect::TypePath;
use bevy_render::batching::NoAutomaticBatching;
use bevy_render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
};
use std::sync::Arc;

use crate::prelude::{
    GpuCull, LodConfiguration, ProtoType, ScatterAssetPlugin, ScatterAssetProperties,
    ScatterAssetsPlugin, ScatterMaterial, ScatterMaterialOptions, ScatteredAsset,
    ScatteredInstance, SpawnRequest, Wind, WindAffected, WindAffectedKey, WindUniform,
};
use bevy_shader::ShaderRef;
use bevy_transform::prelude::Transform;
use bevy_utils::default;
use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};
use rand::SeedableRng;
use rand::prelude::IndexedRandom;
use rand_pcg::Pcg64;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
#[uniform(50, InstancedWindAffectedMaterialUniform)]
#[bind_group_data(InstancedWindAffectedMaterialKey)]
pub struct InstancedWindAffectedMaterialV2 {
    pub wind: Wind,
    pub aabb: Aabb,
    pub options: ScatterMaterialOptions,
    #[texture(51)]
    #[sampler(52)]
    pub noise_texture: Handle<Image>,
}

impl InstancedWindAffectedMaterialV2 {
    pub fn new(properties: &ScatterAssetProperties, noise_texture: Handle<Image>) -> Self {
        Self {
            wind: properties.wind,
            aabb: properties.aabb,
            options: properties.options,
            noise_texture,
        }
    }
}

#[derive(Clone, ShaderType, Debug)]
struct InstancedWindAffectedMaterialUniform {
    pub wind: WindUniform,
    pub top_color: LinearRgba,
    pub bottom_color: LinearRgba,
    pub static_bend_strength: f32,
    pub curve_factor: f32,
    pub translucency: f32,
    pub specular_strength: f32,
    pub specular_power: f32,
}

impl From<&InstancedWindAffectedMaterialV2> for InstancedWindAffectedMaterialUniform {
    fn from(material: &InstancedWindAffectedMaterialV2) -> Self {
        Self {
            wind: WindUniform::from(&material.wind).with_aabb(&material.aabb),
            top_color: material.options.top_color.unwrap_or_default().to_linear(),
            bottom_color: material
                .options
                .bottom_color
                .unwrap_or_default()
                .to_linear(),
            static_bend_strength: material.options.static_bend_strength,
            curve_factor: material.options.curve_factor,
            translucency: material.options.translucency,
            specular_strength: material.options.specular_strength,
            specular_power: material.options.specular_power,
        }
    }
}

impl InstancedMaterial for InstancedWindAffectedMaterialV2 {
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("instanced.wgsl")).with_source("embedded"),
        )
    }

    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("instanced.wgsl")).with_source("embedded"),
        )
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: Self::Data,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;

        let shader_defs = &mut descriptor.vertex.shader_defs;
        if key.wind_key.contains(WindAffectedKey::BILLBOARDING) {
            shader_defs.push("BILLBOARDING".into());
        }

        if key.wind_key.contains(WindAffectedKey::EDGE_CORRECTION) {
            shader_defs.push("EDGE_CORRECTION".into());
        }

        if key.wind_key.contains(WindAffectedKey::WIND_LOW_QUALITY) {
            shader_defs.push("WIND_LOW_QUALITY".into());
        }

        if key.wind_key.contains(WindAffectedKey::FAST_NORMALS) {
            shader_defs.push("FAST_NORMALS".into());
        }

        if key.wind_key.contains(WindAffectedKey::WIND_AFFECTED) {
            shader_defs.push("WIND_AFFECTED".into());
        }

        if key.wind_key.contains(WindAffectedKey::STATIC_BEND) {
            shader_defs.push("STATIC_BEND".into());
        }

        if key.wind_key.contains(WindAffectedKey::ANALYTICAL_NORMALS) {
            shader_defs.push("ANALYTICAL_NORMALS".into());
        }

        if key.wind_key.contains(WindAffectedKey::ANALYTICAL_NORMALS) {
            shader_defs.push("CURVE_NORMALS".into());
        }

        // TODO cull in compute shader
        // https://github.com/NicoZweifel/bevy_feronia/issues/51
        /*
        let gpu_cull = key.wind_key.contains(WindAffectedKey::GPU_CULL);
        if gpu_cull {
            key.mesh_key
                .remove(MeshPipelineKey::VISIBILITY_RANGE_DITHER);
        }
        */

        if let Some(fragment) = descriptor.fragment.as_mut() {
            if let Some(target) = fragment.targets.get_mut(0)
                && let Some(target) = target
            {
                target.blend = None;
            }

            // TODO cull in compute shader
            // https://github.com/NicoZweifel/bevy_feronia/issues/51
            /*
            if !gpu_cull {
                fragment.shader_defs.push("VISIBILITY_RANGE_DITHER".into());
            }
             */
            fragment.shader_defs.push("VISIBILITY_RANGE_DITHER".into());

            if key.wind_key.contains(WindAffectedKey::CURVE_NORMALS) {
                fragment.shader_defs.push("CURVE_NORMALS".into());
            }

            if key.wind_key.contains(WindAffectedKey::POINT_LIGHTS) {
                fragment.shader_defs.push("POINT_LIGHTS".into());
            }

            if key.wind_key.contains(WindAffectedKey::DIRECTIONAL_LIGHTS) {
                fragment.shader_defs.push("DIRECTIONAL_LIGHTS".into());
            }

            if key.wind_key.contains(WindAffectedKey::DEBUG) {
                fragment.shader_defs.push("MATERIAL_DEBUG".into());
            }
        }
        Ok(())
    }
}

struct GroupData {
    instances: Vec<InstanceData>,
    min_pos: Vec3,
    max_pos: Vec3,
    max_scale: f32,
}

impl ScatterMaterial for InstancedWindAffectedMaterialV2 {
    fn create_material(
        _base: Option<StandardMaterial>,
        noise_texture: Handle<Image>,
        properties: &ScatterAssetProperties,
    ) -> InstancedWindAffectedMaterialV2 {
        InstancedWindAffectedMaterialV2::new(properties, noise_texture)
    }

    fn update_material(
        material: &mut InstancedWindAffectedMaterialV2,
        wind: Wind,
        options: ScatterMaterialOptions,
    ) {
        material.wind = wind;
        material.options = options;
    }

    fn component(material: Handle<InstancedWindAffectedMaterialV2>) -> impl Component {
        InstancedMeshMaterial(material)
    }

    fn spawn(cmd: &mut Commands, request: SpawnRequest<InstancedWindAffectedMaterialV2>) {
        let names: HashSet<Name> = request
            .names
            .iter()
            .filter_map(|name| Some((name, request.name_map.get(name)?)))
            .fold(HashSet::new(), |mut acc, (name, group)| {
                let min_lod = group
                    .iter()
                    .map(|h| *h.asset.properties.lod)
                    .min()
                    .unwrap_or_default();

                if group.iter().any(|h| h.is_lod(request.is_chunked, min_lod)) {
                    acc.insert(name.clone());
                }

                acc
            });

        let groups: HashMap<Name, GroupData> = request
            .event
            .trigger
            .data
            .iter()
            .filter_map(|res| {
                (names.len() == 1)
                    .then(|| names.iter().next().map(|x| (x, res)))
                    .flatten();

                let mut rng = Pcg64::seed_from_u64(res.seed);

                let name = request.names.choose(&mut rng)?;

                names.contains(name).then_some((name, res))
            })
            .enumerate()
            .fold(HashMap::new(), |mut acc, (i, (name, res))| {
                let position = res.transform.translation;

                let scale = res.transform.scale.element_sum() / 3.0;

                let instance = InstanceData {
                    position,
                    scale,
                    index: i as u32,
                    ..default()
                };

                match acc.entry(name.clone()) {
                    Entry::Occupied(mut e) => {
                        let g = e.get_mut();
                        g.instances.push(instance);
                        g.min_pos = g.min_pos.min(position);
                        g.max_pos = g.max_pos.max(position);
                        g.max_scale = g.max_scale.max(scale);
                    }
                    Entry::Vacant(e) => {
                        e.insert(GroupData {
                            instances: vec![instance],
                            min_pos: position,
                            max_pos: position,
                            max_scale: scale,
                        });
                    }
                };

                acc
            });

        for (name, group_data) in groups {
            let base_instances = Arc::new(group_data.instances);

            for handle_asset in request.prototypes_from_name_iter(&name) {
                let asset = &handle_asset.asset;

                let half_extents =
                    Vec3::from(asset.properties.aabb.half_extents * group_data.max_scale);
                let center_offset = Vec3::from(asset.properties.aabb.center * group_data.max_scale);

                let local_min = group_data.min_pos + center_offset - half_extents;
                let local_max = group_data.max_pos + center_offset + half_extents;

                let local_aabb = Aabb::from_min_max(local_min, local_max);

                let visibility_range = request
                    .lod_config
                    .get_visibility_range(asset.properties.lod);

                for part in asset.parts.iter() {
                    let instances = if part.transform == Transform::default() {
                        base_instances.clone()
                    } else {
                        Arc::new(
                            base_instances
                                .iter()
                                .map(|original| {
                                    let mut inst = *original;
                                    inst.position += part.transform.translation * inst.scale;
                                    inst.scale *= part.transform.scale.element_sum() / 3.0;
                                    inst
                                })
                                .collect(),
                        )
                    };

                    let entity = cmd
                        .spawn((
                            InstancedMeshMaterial(part.material().clone()),
                            Mesh3d(part.mesh().clone()),
                            InstanceMaterialData {
                                visibility_range: Vec4::new(
                                    visibility_range.start_margin.start,
                                    visibility_range.start_margin.end,
                                    visibility_range.end_margin.start,
                                    visibility_range.end_margin.end,
                                ),
                                instances,
                                color: default(),
                            },
                            NoAutomaticBatching,
                            ScatteredInstance(request.event.trigger.layer),
                            ScatteredAsset(handle_asset.handle.clone()),
                        ))
                        .id();

                    cmd.entity(entity).insert((
                        Transform::default(),
                        local_aabb,
                        ChildOf(request.parent),
                    ));

                    if asset.properties.wind_affected {
                        cmd.entity(entity).insert(WindAffected);
                    }

                    if asset.properties.options.gpu_cull {
                        cmd.entity(entity).insert(GpuCull);
                    }
                }
            }
        }
    }
}

pub struct InstancedWindAffectedPluginV2;

impl Plugin for InstancedWindAffectedPluginV2 {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "instanced.wgsl");
        app.add_plugins((
            InstancedMaterialCorePlugin,
            InstancedMaterialPlugin::<InstancedWindAffectedMaterialV2>::default(),
        ));
    }
}

pub struct InstancedWindAffectedScatterPluginV2;

impl Plugin for InstancedWindAffectedScatterPluginV2 {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            InstancedWindAffectedPluginV2,
            ScatterAssetsPlugin::<InstancedWindAffectedMaterialV2>::new(),
            ScatterAssetPlugin::<InstancedWindAffectedMaterialV2>::new(),
        ));
    }
}

#[repr(C)]
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub struct InstancedWindAffectedMaterialKey {
    wind_key: WindAffectedKey,
    material_key: MaterialKey,
}

impl From<&InstancedWindAffectedMaterialV2> for InstancedWindAffectedMaterialKey {
    fn from(material: &InstancedWindAffectedMaterialV2) -> Self {
        let wind_key: WindAffectedKey = material.options.into();
        let material_key: MaterialKey = material.options.into();

        Self {
            wind_key,
            material_key,
        }
    }
}

bitflags! {
    #[repr(C)]
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Pod, Zeroable)]
    pub struct MaterialKey: u64 {
        const POINT_LIGHTS = 1 << 11;
        const DIRECTIONAL_LIGHTS = 1 << 12;
        const GPU_CULL = 1 << 13;
    }
}

impl From<ScatterMaterialOptions> for MaterialKey {
    fn from(options: ScatterMaterialOptions) -> MaterialKey {
        let mut key = MaterialKey::empty();

        key.set(MaterialKey::POINT_LIGHTS, options.point_lights);
        key.set(MaterialKey::DIRECTIONAL_LIGHTS, options.directional_lights);
        key.set(MaterialKey::GPU_CULL, options.gpu_cull);

        key
    }
}
