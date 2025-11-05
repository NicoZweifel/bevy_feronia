use crate::prelude::*;
use bevy::asset::{AssetPath, embedded_path};
use bevy::camera::primitives::Aabb;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

#[derive(Asset, Reflect, AsBindGroup, Debug, Clone)]
#[bind_group_data(WindAffectedKey)]
#[data(50, WindUniform, binding_array(101))]
#[bindless(index_table(range(50..53), binding(100)))]
pub struct WindAffectedExtension {
    pub wind: Wind,

    pub aabb: Aabb,

    pub options: MaterialOptions,

    #[texture(51)]
    #[sampler(52)]
    pub noise_texture: Handle<Image>,
}

impl WindAffectedExtension {
    pub fn new(properties: &ScatterAssetProperties, noise_texture: Handle<Image>) -> Self {
        Self {
            wind: properties.wind,
            aabb: properties.aabb,
            options: properties.options,
            noise_texture,
        }
    }
}

impl<'a> From<&'a WindAffectedExtension> for WindUniform {
    fn from(material_extension: &'a WindAffectedExtension) -> Self {
        WindUniform::from(&material_extension.wind)
            .with_edge_correction_factor(material_extension.options.edge_correction_factor)
            .with_aabb(&material_extension.aabb)
            .with_debug_color(material_extension.options.debug_color.to_linear().to_vec4())
    }
}

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
enum ShaderStage {
    Vertex,
    Fragment,
    Both,
}

struct ShaderDefMap {
    flag: WindAffectedKey,
    def: &'static str,
    stage: ShaderStage,
}

const SHADER_DEFS: &[ShaderDefMap] = &[
    ShaderDefMap {
        flag: WindAffectedKey::ENABLE_BILLBOARDING,
        def: "WIND_BILLBOARDING",
        stage: ShaderStage::Vertex,
    },
    ShaderDefMap {
        flag: WindAffectedKey::ENABLE_EDGE_CORRECTION,
        def: "WIND_EDGE_CORRECTION",
        stage: ShaderStage::Vertex,
    },
    ShaderDefMap {
        flag: WindAffectedKey::WIND_LOW_QUALITY,
        def: "WIND_LOW_QUALITY",
        stage: ShaderStage::Vertex,
    },
    ShaderDefMap {
        flag: WindAffectedKey::FAST_NORMALS,
        def: "FAST_NORMALS",
        stage: ShaderStage::Vertex,
    },
    ShaderDefMap {
        flag: WindAffectedKey::DEBUG,
        def: "MATERIAL_DEBUG",
        stage: ShaderStage::Both,
    },
    ShaderDefMap {
        flag: WindAffectedKey::SUBSURFACE_SCATTERING,
        def: "SUBSURFACE_SCATTERING",
        stage: ShaderStage::Both,
    },
    ShaderDefMap {
        flag: WindAffectedKey::WIND_AFFECTED,
        def: "WIND_AFFECTED",
        stage: ShaderStage::Vertex,
    },
    ShaderDefMap {
        flag: WindAffectedKey::STATIC_BEND,
        def: "STATIC BEND",
        stage: ShaderStage::Vertex,
    },
    ShaderDefMap {
        flag: WindAffectedKey::ANALYTICAL_NORMALS,
        def: "ANALYTICAL_NORMALS",
        stage: ShaderStage::Vertex,
    },
];

impl MaterialExtension for WindAffectedExtension {
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("vertex.wgsl")).with_source("embedded"),
        )
    }

    fn prepass_vertex_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("prepass.wgsl")).with_source("embedded"),
        )
    }

    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("fragment.wgsl")).with_source("embedded"),
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

        for mapping in SHADER_DEFS {
            if !key.bind_group_data.contains(mapping.flag) {
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

impl From<&WindAffectedExtension> for WindAffectedKey {
    fn from(material: &WindAffectedExtension) -> Self {
        let mut key = WindAffectedKey::empty();
        key.set(WindAffectedKey::DEBUG, material.options.debug);
        key.set(
            WindAffectedKey::WIND_AFFECTED,
            material.options.wind_affected,
        );
        key.set(
            WindAffectedKey::ENABLE_BILLBOARDING,
            material.options.enable_billboarding,
        );
        key.set(
            WindAffectedKey::ENABLE_EDGE_CORRECTION,
            material.options.edge_correction_factor > 0.,
        );
        key.set(WindAffectedKey::FAST_NORMALS, material.options.fast_normals);
        key.set(
            WindAffectedKey::SUBSURFACE_SCATTERING,
            material.options.subsurface_scattering,
        );
        key.set(
            WindAffectedKey::WIND_LOW_QUALITY,
            material.options.low_quality,
        );
        key.set(
            WindAffectedKey::STATIC_BEND,
            material.options.static_bend_strength > 0.,
        );
        key.set(
            WindAffectedKey::ANALYTICAL_NORMALS,
            material.options.analytical_normals,
        );

        key
    }
}
