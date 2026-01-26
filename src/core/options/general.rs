use crate::prelude::*;

use bevy_color::Color;
use bevy_reflect::Reflect;
use bevy_utils::default;

#[derive(Clone, Debug, Reflect, Copy, Default, PartialEq)]
pub struct GeneralOptions {
    pub controlled: bool,
    pub debug: bool,
    pub debug_color: Color,
    pub gpu_cull: bool,
}

impl GeneralOptions {
    pub fn from_data(data: &MaterialOptionDataItem) -> Self {
        Self {
            debug: data.enable_debug.is_some(),
            gpu_cull: data.gpu_cull.is_some(),
            ..default()
        }
    }

    pub fn with_data(mut self, data: &MaterialOptionDataItem) -> Self {
        self.debug |= data.enable_debug.is_some();
        self.gpu_cull |= data.gpu_cull.is_some();
        self
    }

    pub fn with(mut self, other: Self) -> Self {
        self.debug |= other.debug;
        self.gpu_cull |= other.gpu_cull;
        self.controlled = other.controlled;
        if other.debug_color != Color::default() {
            self.debug_color = other.debug_color;
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_color::palettes::css::*;
    use bevy_eidolon::prelude::GpuCullCompute;

    fn empty_data() -> MaterialOptionDataItem<'static, 'static> {
        unsafe { std::mem::zeroed() }
    }

    #[test]
    fn test_general_with_data() {
        // Arrange
        let options = GeneralOptions::default();
        let debug = EnableDebug;
        let cull = GpuCullCompute;
        let data = MaterialOptionDataItem {
            enable_debug: Some(&debug),
            gpu_cull: Some(&cull),
            ..empty_data()
        };

        // Act
        let result = options.with_data(&data);

        // Assert
        assert!(result.debug);
        assert!(result.gpu_cull);
    }

    #[test]
    fn test_general_with() {
        // Arrange
        let base = GeneralOptions {
            debug: true,
            debug_color: RED.into(),
            ..default()
        };
        let other = GeneralOptions {
            debug: false, // Preserves
            gpu_cull: true,
            controlled: true,         // Overwrite
            debug_color: BLUE.into(), // Overwrite
        };

        // Act
        let result = base.with(other);

        // Assert
        assert!(result.debug);
        assert!(result.gpu_cull);
        assert!(result.controlled);
        assert_eq!(result.debug_color, BLUE.into());
    }
}
