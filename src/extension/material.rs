use crate::prelude::*;
use bevy_asset::*;
use bevy_camera::primitives::Aabb;
use bevy_ecs::prelude::*;
use bevy_image::Image;
use bevy_mesh::MeshVertexBufferLayoutRef;
use bevy_pbr::*;
use bevy_pbr::{ExtendedMaterial, StandardMaterial};
use bevy_reflect::Reflect;
use bevy_render::render_resource::*;
use bevy_shader::ShaderRef;
use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};

pub type ExtendedWindAffectedMaterial = ExtendedMaterial<StandardMaterial, WindAffectedExtension>;

#[derive(Asset, Reflect, AsBindGroup, Debug, Clone, Default)]
#[bind_group_data(ExtendedWindAffectedMaterialKey)]
#[data(50, ExtendedWindAffectedMaterialUniform, binding_array(101))]
#[bindless(index_table(range(50..53), binding(100)))]
pub struct WindAffectedExtension {
    pub current: Wind,
    pub previous: Wind,

    // TODO use extracted AAbb instead and add to uniforms.
    pub aabb: Aabb,

    pub options: ScatterMaterialOptions,

    #[texture(51)]
    #[sampler(52)]
    pub noise_texture: Handle<Image>,
}

#[derive(Clone, ShaderType, Debug)]
struct ExtendedWindAffectedMaterialUniform {
    previous: WindUniform,
    current: WindUniform,
    sss_scale: f32,
    sss_intensity: f32,
}

impl<'a> From<&'a WindAffectedExtension> for ExtendedWindAffectedMaterialUniform {
    fn from(extension: &'a WindAffectedExtension) -> Self {
        let CommonLightingOptions {
            subsurface_scattering_intensity,
            subsurface_scattering_scale,
            ..
        } = extension.options.lighting.common;
        Self {
            current: WindUniform::from(&extension.current),
            previous: WindUniform::from(&extension.previous),
            sss_intensity: subsurface_scattering_intensity,
            sss_scale: subsurface_scattering_scale,
        }
    }
}

impl WindAffectedExtension {
    pub fn new(properties: &ScatterAssetProperties, noise_texture: Handle<Image>) -> Self {
        Self {
            previous: properties.wind,
            current: properties.wind,
            aabb: properties.aabb,
            options: properties.options.clone(),
            noise_texture,
        }
    }
}

#[repr(C)]
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub struct ExtendedWindAffectedMaterialKey {
    wind_key: WindAffectedKey,
    material_key: ExtendedMaterialKey,
}

bitflags! {
    #[repr(C)]
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Pod, Zeroable)]
    pub struct ExtendedMaterialKey: u64 {
        const DEBUG= 1 << 1;
        const SUBSURFACE_SCATTERING= 1 << 2;
        const STATIC_SHADOW = 1<< 3;
    }
}

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
enum ShaderStage {
    Vertex,
    Fragment,
    Both,
}

struct WindShaderDefMap {
    flag: WindAffectedKey,
    def: &'static str,
    stage: ShaderStage,
}

const WIND_SHADER_DEFS: &[WindShaderDefMap] = &[
    WindShaderDefMap {
        flag: WindAffectedKey::BILLBOARDING,
        def: "BILLBOARDING",
        stage: ShaderStage::Vertex,
    },
    WindShaderDefMap {
        flag: WindAffectedKey::EDGE_CORRECTION,
        def: "EDGE_CORRECTION",
        stage: ShaderStage::Vertex,
    },
    WindShaderDefMap {
        flag: WindAffectedKey::WIND_LOW_QUALITY,
        def: "WIND_LOW_QUALITY",
        stage: ShaderStage::Vertex,
    },
    WindShaderDefMap {
        flag: WindAffectedKey::FAST_NORMALS,
        def: "FAST_NORMALS",
        stage: ShaderStage::Vertex,
    },
    WindShaderDefMap {
        flag: WindAffectedKey::WIND_AFFECTED,
        def: "WIND_AFFECTED",
        stage: ShaderStage::Vertex,
    },
    WindShaderDefMap {
        flag: WindAffectedKey::STATIC_BEND,
        def: "STATIC BEND",
        stage: ShaderStage::Vertex,
    },
    WindShaderDefMap {
        flag: WindAffectedKey::ANALYTICAL_NORMALS,
        def: "ANALYTICAL_NORMALS",
        stage: ShaderStage::Vertex,
    },
];

struct MaterialShaderDefMap {
    flag: ExtendedMaterialKey,
    def: &'static str,
    stage: ShaderStage,
}

const MATERIAL_SHADER_DEFS: &[MaterialShaderDefMap] = &[
    MaterialShaderDefMap {
        flag: ExtendedMaterialKey::DEBUG,
        def: "MATERIAL_DEBUG",
        stage: ShaderStage::Both,
    },
    MaterialShaderDefMap {
        flag: ExtendedMaterialKey::SUBSURFACE_SCATTERING,
        def: "SUBSURFACE_SCATTERING",
        stage: ShaderStage::Both,
    },
    MaterialShaderDefMap {
        flag: ExtendedMaterialKey::STATIC_SHADOW,
        def: "STATIC_SHADOW",
        stage: ShaderStage::Vertex,
    },
];

impl MaterialExtension for WindAffectedExtension {
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("vertex.wgsl")).with_source("embedded"),
        )
    }

    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("fragment.wgsl")).with_source("embedded"),
        )
    }

    fn prepass_vertex_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("prepass.wgsl")).with_source("embedded"),
        )
    }

    fn specialize(
        _pipeline: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_shader_defs = &mut descriptor.vertex.shader_defs;
        let mut fragment_shader_defs = descriptor.fragment.as_mut().map(|f| &mut f.shader_defs);

        for mapping in WIND_SHADER_DEFS {
            if !key.bind_group_data.wind_key.contains(mapping.flag) {
                continue;
            };

            if matches!(mapping.stage, ShaderStage::Vertex | ShaderStage::Both) {
                vertex_shader_defs.push(mapping.def.into());
            }

            let Some(fragment_shader_defs) = &mut fragment_shader_defs else {
                continue;
            };

            if matches!(mapping.stage, ShaderStage::Fragment | ShaderStage::Both) {
                fragment_shader_defs.push(mapping.def.into());
            }
        }

        for mapping in MATERIAL_SHADER_DEFS {
            if !key.bind_group_data.material_key.contains(mapping.flag) {
                continue;
            };

            if matches!(mapping.stage, ShaderStage::Vertex | ShaderStage::Both) {
                vertex_shader_defs.push(mapping.def.into());
            }

            let Some(fragment_shader_defs) = &mut fragment_shader_defs else {
                continue;
            };

            if matches!(mapping.stage, ShaderStage::Fragment | ShaderStage::Both) {
                fragment_shader_defs.push(mapping.def.into());
            }
        }

        Ok(())
    }
}

impl From<&WindAffectedExtension> for ExtendedWindAffectedMaterialKey {
    fn from(material: &WindAffectedExtension) -> Self {
        let wind_key: WindAffectedKey = (&material.options).into();
        let material_key: ExtendedMaterialKey = (&material.options).into();

        Self {
            wind_key,
            material_key,
        }
    }
}

impl From<&ScatterMaterialOptions> for ExtendedMaterialKey {
    fn from(options: &ScatterMaterialOptions) -> Self {
        let mut key = ExtendedMaterialKey::empty();

        let GeneralOptions { debug, .. } = options.general;
        let CommonLightingOptions {
            subsurface_scattering,
            static_shadows,
            ..
        } = options.lighting.common;

        key.set(ExtendedMaterialKey::DEBUG, debug);
        key.set(
            ExtendedMaterialKey::SUBSURFACE_SCATTERING,
            subsurface_scattering,
        );
        key.set(ExtendedMaterialKey::STATIC_SHADOW, static_shadows);

        key
    }
}
