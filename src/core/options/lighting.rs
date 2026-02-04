use crate::prelude::*;
use bevy_reflect::Reflect;

#[derive(Clone, Debug, Reflect, Copy, Default, PartialEq)]
pub struct NormalOptions {
    pub fast_normals: bool,
    pub analytical_normals: bool,
    pub curve_factor: f32,
}

impl NormalOptions {
    pub fn from_data(data: &MaterialOptionDataItem) -> Self {
        Self {
            fast_normals: data.fast_normals.is_some(),
            analytical_normals: data.analytical_normals.is_some(),
            curve_factor: data.curve_factor.map(|c| **c).unwrap_or(0.),
        }
    }

    pub fn with_data(mut self, data: &MaterialOptionDataItem) -> Self {
        self.fast_normals |= data.fast_normals.is_some();
        self.analytical_normals |= data.analytical_normals.is_some();

        if let Some(val) = data.curve_factor {
            self.curve_factor = **val;
        }

        self
    }

    pub fn with(mut self, other: Self) -> Self {
        self.fast_normals |= other.fast_normals;
        self.analytical_normals |= other.analytical_normals;

        if other.curve_factor > 0. {
            self.curve_factor = other.curve_factor
        }

        self
    }
}

#[derive(Clone, Debug, Reflect, Copy, Default, PartialEq)]
pub struct CommonLightingOptions {
    pub unlit: bool,
    pub directional_lights: bool,
    pub point_lights: bool,
    pub static_shadows: bool,
    pub ambient_occlusion: bool,
    pub subsurface_scattering: bool,
    pub subsurface_scattering_scale: f32,
    pub subsurface_scattering_intensity: f32,
}

impl CommonLightingOptions {
    pub fn from_data(data: &MaterialOptionDataItem) -> Self {
        Self {
            unlit: data.unlit.is_some(),
            directional_lights: data.directional_lights.is_some(),
            point_lights: data.point_lights.is_some(),
            static_shadows: data.static_shadow.is_some(),
            ambient_occlusion: data.ambient_occlusion.is_some(),
            subsurface_scattering: data.sss.is_some(),
            subsurface_scattering_intensity: data.sss_intensity.map(|s| **s).unwrap_or(0.),
            subsurface_scattering_scale: data.sss_scale.map(|s| **s).unwrap_or(0.),
        }
    }

    pub fn with_data(mut self, data: &MaterialOptionDataItem) -> Self {
        self.unlit |= data.unlit.is_some();
        self.directional_lights |= data.directional_lights.is_some();
        self.point_lights |= data.point_lights.is_some();
        self.static_shadows |= data.static_shadow.is_some();
        self.ambient_occlusion |= data.ambient_occlusion.is_some();
        self.subsurface_scattering |= data.sss.is_some();

        if let Some(val) = data.sss_scale {
            self.subsurface_scattering_scale = **val;
        }
        if let Some(val) = data.sss_intensity {
            self.subsurface_scattering_intensity = **val;
        }

        self
    }

    pub fn with(mut self, other: Self) -> Self {
        self.unlit |= other.unlit;
        self.directional_lights |= other.directional_lights;
        self.point_lights |= other.point_lights;
        self.static_shadows |= other.static_shadows;
        self.ambient_occlusion |= other.ambient_occlusion;
        self.subsurface_scattering |= other.subsurface_scattering;

        if other.subsurface_scattering_scale > 0. {
            self.subsurface_scattering_scale = other.subsurface_scattering_scale;
        }
        if other.subsurface_scattering_intensity > 0. {
            self.subsurface_scattering_intensity = other.subsurface_scattering_intensity;
        }

        self
    }
}

#[derive(Clone, Debug, Reflect, Copy, Default, PartialEq)]
pub struct PbrOptions {
    pub enabled: bool,
    pub roughness: f32,
    pub metallic: f32,
    pub reflectance: f32,
}

impl PbrOptions {
    pub fn from_data(data: &MaterialOptionDataItem) -> Self {
        Self {
            enabled: data.standard_pbr.is_some(),
            roughness: data.roughness.map(|r| **r).unwrap_or(0.),
            metallic: data.metallic.map(|m| **m).unwrap_or(0.),
            reflectance: data.reflectance.map(|r| **r).unwrap_or(0.),
        }
    }

    pub fn with_data(mut self, data: &MaterialOptionDataItem) -> Self {
        self.enabled |= data.standard_pbr.is_some();
        if let Some(val) = data.roughness {
            self.roughness = **val;
        }
        if let Some(val) = data.metallic {
            self.metallic = **val;
        }
        if let Some(val) = data.reflectance {
            self.reflectance = **val;
        }
        self
    }

    pub fn with(mut self, other: Self) -> Self {
        self.enabled |= other.enabled;

        if other.roughness > 0. {
            self.roughness = other.roughness;
        }
        if other.metallic > 0. {
            self.metallic = other.metallic;
        }
        if other.reflectance > 0. {
            self.reflectance = other.reflectance;
        }

        self
    }
}

#[derive(Clone, Debug, Reflect, Copy, Default, PartialEq)]
pub struct BlinnPhongOptions {
    pub translucency: f32,
    pub specular_strength: f32,
    pub specular_power: f32,
    pub diffuse_scaling: f32,
    pub light_intensity: f32,
    pub ambient_light_intensity: f32,
}

impl BlinnPhongOptions {
    pub fn from_data(data: &MaterialOptionDataItem) -> Self {
        Self {
            translucency: data.translucency.map(|s| **s).unwrap_or(0.),
            specular_strength: data.specular_strength.map(|s| **s).unwrap_or(0.),
            specular_power: data.specular_power.map(|s| **s).unwrap_or(0.),
            diffuse_scaling: data.diffuse_scaling.map(|s| **s).unwrap_or(0.),
            light_intensity: data.light_intensity.map(|s| **s).unwrap_or(0.),
            ambient_light_intensity: data.ambient_light_intensity.map(|s| **s).unwrap_or(0.),
        }
    }

    pub fn with_data(mut self, data: &MaterialOptionDataItem) -> Self {
        if let Some(val) = data.translucency {
            self.translucency = **val;
        }
        if let Some(val) = data.specular_strength {
            self.specular_strength = **val;
        }
        if let Some(val) = data.specular_power {
            self.specular_power = **val;
        }
        if let Some(val) = data.diffuse_scaling {
            self.diffuse_scaling = **val;
        }
        if let Some(val) = data.light_intensity {
            self.light_intensity = **val;
        }
        if let Some(val) = data.ambient_light_intensity {
            self.ambient_light_intensity = **val;
        }
        self
    }

    pub fn with(mut self, other: Self) -> Self {
        if other.translucency > 0. {
            self.translucency = other.translucency;
        }
        if other.specular_strength > 0. {
            self.specular_strength = other.specular_strength;
        }
        if other.specular_power > 0. {
            self.specular_power = other.specular_power;
        }
        if other.diffuse_scaling > 0. {
            self.diffuse_scaling = other.diffuse_scaling;
        }
        if other.light_intensity > 0. {
            self.light_intensity = other.light_intensity;
        }
        if other.ambient_light_intensity > 0. {
            self.ambient_light_intensity = other.ambient_light_intensity;
        }
        self
    }
}

#[derive(Clone, Debug, Reflect, Copy, Default, PartialEq)]
pub struct LightingOptions {
    pub common: CommonLightingOptions,
    pub normals: NormalOptions,
    pub pbr: PbrOptions,
    pub blinn_phong: BlinnPhongOptions,
}

impl LightingOptions {
    pub fn from_data(data: &MaterialOptionDataItem) -> Self {
        Self {
            common: CommonLightingOptions::from_data(data),
            normals: NormalOptions::from_data(data),
            pbr: PbrOptions::from_data(data),
            blinn_phong: BlinnPhongOptions::from_data(data),
        }
    }

    pub fn with_data(mut self, data: &MaterialOptionDataItem) -> Self {
        self.common = self.common.with_data(data);
        self.normals = self.normals.with_data(data);
        self.pbr = self.pbr.with_data(data);
        self.blinn_phong = self.blinn_phong.with_data(data);
        self
    }

    pub fn with(mut self, other: Self) -> Self {
        self.common = self.common.with(other.common);
        self.normals = self.normals.with(other.normals);
        self.pbr = self.pbr.with(other.pbr);
        self.blinn_phong = self.blinn_phong.with(other.blinn_phong);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_utils::default;

    fn empty_data() -> MaterialOptionDataItem<'static, 'static> {
        unsafe { std::mem::zeroed() }
    }

    #[test]
    fn test_normals_with_data() {
        // Arrange
        let options = NormalOptions::default();
        let fast = FastNormals;
        let curve = CurveFactor(0.5);
        let data = MaterialOptionDataItem {
            fast_normals: Some(&fast),
            curve_factor: Some(&curve),
            ..empty_data()
        };

        // Act
        let result = options.with_data(&data);

        // Assert
        assert!(result.fast_normals);
        assert_eq!(result.curve_factor, 0.5);
    }

    #[test]
    fn test_common_with_data() {
        // Arrange
        let options = CommonLightingOptions::default();
        let unlit = Unlit;
        let sss_scale = SubsurfaceScatteringScale(2.0);
        let data = MaterialOptionDataItem {
            unlit: Some(&unlit),
            sss_scale: Some(&sss_scale),
            ..empty_data()
        };

        // Act
        let result = options.with_data(&data);

        // Assert
        assert!(result.unlit);
        assert_eq!(result.subsurface_scattering_scale, 2.0);
    }

    #[test]
    fn test_pbr_with_data() {
        // Arrange
        let options = PbrOptions::default();
        let std_pbr = StandardPbr;
        let rough = Roughness(0.8);
        let reflectance = Reflectance(0.5);
        let data = MaterialOptionDataItem {
            standard_pbr: Some(&std_pbr),
            roughness: Some(&rough),
            reflectance: Some(&reflectance),
            ..empty_data()
        };

        // Act
        let result = options.with_data(&data);

        // Assert
        assert!(result.enabled);
        assert_eq!(result.roughness, 0.8);
        assert_eq!(result.reflectance, 0.5);
    }

    #[test]
    fn test_blinn_phong_with_data() {
        // Arrange
        let options = BlinnPhongOptions::default();
        let trans = Translucency(0.3);
        let spec = SpecularPower(32.0);
        let data = MaterialOptionDataItem {
            translucency: Some(&trans),
            specular_power: Some(&spec),
            ..empty_data()
        };

        // Act
        let result = options.with_data(&data);

        // Assert
        assert_eq!(result.translucency, 0.3);
        assert_eq!(result.specular_power, 32.0);
    }

    #[test]
    fn test_blinn_phong_with() {
        // Arrange
        let base = BlinnPhongOptions {
            translucency: 0.5,
            specular_power: 10.0,
            ..default()
        };
        let other = BlinnPhongOptions {
            translucency: 0.8,
            specular_power: 0.0, // Preserve
            specular_strength: 1.0,
            ..default()
        };

        // Act
        let result = base.with(other);

        // Assert
        assert_eq!(result.translucency, 0.8);
        assert_eq!(result.specular_power, 10.0);
        assert_eq!(result.specular_strength, 1.0);
    }
}
