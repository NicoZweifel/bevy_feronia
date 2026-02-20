use crate::prelude::*;
use bevy_eidolon::prelude::*;

use bevy_asset::{Asset, AssetPath, Handle, embedded_path};
use bevy_camera::primitives::Aabb;
use bevy_color::LinearRgba;
use bevy_ecs::prelude::*;
use bevy_image::Image;
use bevy_math::Vec2;
use bevy_mesh::MeshVertexBufferLayoutRef;
use bevy_reflect::TypePath;
use bevy_render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
};
use bevy_shader::ShaderRef;

use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
#[uniform(50, InstancedWindAffectedMaterialUniform)]
#[bind_group_data(InstancedWindAffectedMaterialKey)]
pub struct InstancedWindAffectedMaterial {
    pub current: Wind,
    pub previous: Wind,
    pub aabb: Aabb,
    pub options: ScatterMaterialOptions,
    #[texture(51)]
    #[sampler(52)]
    pub noise_texture: Handle<Image>,
    pub disable_prepass: bool,
}

impl InstancedWindAffectedMaterial {
    pub fn new(properties: &ScatterAssetProperties, noise_texture: Handle<Image>) -> Self {
        Self {
            previous: properties.wind,
            current: properties.wind,
            aabb: properties.aabb,
            options: properties.options.clone(),
            noise_texture,
            disable_prepass: false,
        }
    }
}

#[derive(Clone, ShaderType, Debug)]
struct InstancedWindAffectedMaterialUniform {
    pub current: WindUniform,
    pub previous: WindUniform,
    pub top_color: LinearRgba,
    pub bottom_color: LinearRgba,
    pub tint_factor: f32,
    pub gradient_start: f32,
    pub gradient_end: f32,
    pub curve_factor: f32,
    pub translucency: f32,
    pub specular_strength: f32,
    pub specular_power: f32,
    pub diffuse_scaling: f32,
    pub light_intensity: f32,
    pub ambient_light_intensity: f32,
    pub edge_correction_factor: f32,
    pub metallic: f32,
    pub roughness: f32,
    pub reflectance: f32,
    pub static_bend_strength: f32,
    pub static_bend_direction: Vec2,
    pub static_bend_control_point: Vec2,
    pub static_bend_min_max: Vec2,
    pub subsurface_scattering_scale: f32,
    pub subsurface_scattering_intensity: f32,
}

impl From<&InstancedWindAffectedMaterial> for InstancedWindAffectedMaterialUniform {
    fn from(material: &InstancedWindAffectedMaterial) -> Self {
        let ColorOptions {
            top_color,
            bottom_color,
            gradient_start,
            gradient_end,
            tint_factor,
            ..
        } = material.options.color;

        let CommonLightingOptions {
            subsurface_scattering_scale,
            subsurface_scattering_intensity,
            ..
        } = material.options.lighting.common;
        let NormalOptions { curve_factor, .. } = material.options.lighting.normals;
        let BlinnPhongOptions {
            translucency,
            light_intensity,
            ambient_light_intensity,
            specular_strength,
            specular_power,
            diffuse_scaling,
            ..
        } = material.options.lighting.blinn_phong;
        let PbrOptions {
            roughness,
            metallic,
            reflectance,
            ..
        } = material.options.lighting.pbr;

        let GeometryOptions {
            edge_correction_factor,
            ..
        } = material.options.geometry;
        let StaticBendOptions {
            strength: static_bend_strength,
            direction: static_bend_direction,
            control_point: static_bend_control_point,
            min_max: static_bend_min_max,
        } = material.options.bend;

        Self {
            current: WindUniform::from(&material.current).with_aabb(&material.aabb),
            previous: WindUniform::from(&material.previous).with_aabb(&material.aabb),
            top_color: top_color.unwrap_or_default().to_linear(),
            bottom_color: bottom_color.unwrap_or_default().to_linear(),
            tint_factor,
            gradient_start,
            gradient_end,
            curve_factor,
            translucency,
            specular_strength,
            specular_power,
            diffuse_scaling,
            light_intensity,
            ambient_light_intensity,
            edge_correction_factor,
            metallic,
            roughness,
            reflectance,
            static_bend_strength,
            static_bend_direction,
            static_bend_control_point,
            static_bend_min_max,
            subsurface_scattering_scale,
            subsurface_scattering_intensity,
        }
    }
}

impl InstancedMaterial for InstancedWindAffectedMaterial {
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

    fn prepass_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("prepass.wgsl")).with_source("embedded"),
        )
    }

    fn disable_prepass(&self) -> bool {
        self.disable_prepass
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: Self::Data,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;

        if descriptor.multisample.count > 1 {
            descriptor.multisample.alpha_to_coverage_enabled = true;
        }

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

        if key
            .material_key
            .contains(InstancedMaterialKey::SUBSURFACE_SCATTERING)
        {
            shader_defs.push("SUBSURFACE_SCATTERING".into());
        }

        if key
            .material_key
            .contains(InstancedMaterialKey::AMBIENT_OCCLUSION)
        {
            shader_defs.push("AMBIENT_OCCLUSION".into());
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

            if key
                .material_key
                .contains(InstancedMaterialKey::CURVE_NORMALS)
            {
                fragment.shader_defs.push("CURVE_NORMALS".into());
            }

            if key
                .material_key
                .contains(InstancedMaterialKey::POINT_LIGHTS)
            {
                fragment.shader_defs.push("POINT_LIGHTS".into());
            }

            if key
                .material_key
                .contains(InstancedMaterialKey::DIRECTIONAL_LIGHTS)
            {
                fragment.shader_defs.push("DIRECTIONAL_LIGHTS".into());
            }

            if key
                .material_key
                .contains(InstancedMaterialKey::STANDARD_PBR)
            {
                fragment.shader_defs.push("STANDARD_PBR".into());
            }

            if key
                .material_key
                .contains(InstancedMaterialKey::SUBSURFACE_SCATTERING)
            {
                fragment.shader_defs.push("SUBSURFACE_SCATTERING".into());
            }

            if key
                .material_key
                .contains(InstancedMaterialKey::AMBIENT_OCCLUSION)
            {
                fragment.shader_defs.push("AMBIENT_OCCLUSION".into());
            }
        }
        Ok(())
    }
}

#[repr(C)]
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub struct InstancedWindAffectedMaterialKey {
    wind_key: WindAffectedKey,
    material_key: InstancedMaterialKey,
}

impl From<&InstancedWindAffectedMaterial> for InstancedWindAffectedMaterialKey {
    fn from(material: &InstancedWindAffectedMaterial) -> Self {
        let wind_key: WindAffectedKey = (&material.options).into();
        let material_key: InstancedMaterialKey = (&material.options).into();

        Self {
            wind_key,
            material_key,
        }
    }
}

bitflags! {
    #[repr(C)]
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Pod, Zeroable)]
    pub struct InstancedMaterialKey: u64 {
        const POINT_LIGHTS = 1 << 1;
        const DIRECTIONAL_LIGHTS = 1 << 2;
        const GPU_CULL = 1 << 3;
        const AMBIENT_OCCLUSION= 1 << 4;
        const CURVE_NORMALS = 1 << 5;
        const STANDARD_PBR = 1<< 6;
        const SUBSURFACE_SCATTERING = 1 << 7;
    }
}

impl From<&ScatterMaterialOptions> for InstancedMaterialKey {
    fn from(options: &ScatterMaterialOptions) -> InstancedMaterialKey {
        let mut key = InstancedMaterialKey::empty();

        let GeneralOptions { gpu_cull, .. } = options.general;
        let NormalOptions { curve_factor, .. } = options.lighting.normals;
        let CommonLightingOptions {
            ambient_occlusion,
            directional_lights,
            point_lights,
            subsurface_scattering,
            ..
        } = options.lighting.common;
        let PbrOptions { enabled, .. } = options.lighting.pbr;

        key.set(InstancedMaterialKey::POINT_LIGHTS, point_lights);
        key.set(InstancedMaterialKey::DIRECTIONAL_LIGHTS, directional_lights);
        key.set(InstancedMaterialKey::GPU_CULL, gpu_cull);
        key.set(InstancedMaterialKey::AMBIENT_OCCLUSION, ambient_occlusion);
        key.set(InstancedMaterialKey::CURVE_NORMALS, curve_factor > 0.);
        key.set(InstancedMaterialKey::STANDARD_PBR, enabled);
        key.set(
            InstancedMaterialKey::SUBSURFACE_SCATTERING,
            subsurface_scattering,
        );

        key
    }
}
