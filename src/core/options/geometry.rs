use crate::prelude::*;

use bevy_reflect::Reflect;

#[derive(Clone, Debug, Reflect, Copy, Default, PartialEq)]
pub struct GeometryOptions {
    pub enable_billboarding: bool,
    pub edge_correction_factor: f32,
}

impl GeometryOptions {
    pub fn from_data(data: &MaterialOptionDataItem) -> Self {
        Self {
            enable_billboarding: data.enable_billboarding.is_some(),
            edge_correction_factor: data.edge_correction_factor.map(|e| **e).unwrap_or(0.),
        }
    }

    pub fn with_data(mut self, data: &MaterialOptionDataItem) -> Self {
        self.enable_billboarding |= data.enable_billboarding.is_some();
        if let Some(val) = data.edge_correction_factor {
            self.edge_correction_factor = **val;
        }
        self
    }

    pub fn with(mut self, other: Self) -> Self {
        self.enable_billboarding |= other.enable_billboarding;
        if other.edge_correction_factor > 0. {
            self.edge_correction_factor = other.edge_correction_factor;
        }
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
    fn test_geometry_with_data() {
        // Arrange
        let options = GeometryOptions::default();
        let billboard = EnableBillboarding;
        let edge = EdgeCorrectionFactor(0.5);
        let data = MaterialOptionDataItem {
            enable_billboarding: Some(&billboard),
            edge_correction_factor: Some(&edge),
            ..empty_data()
        };

        // Act
        let result = options.with_data(&data);

        // Assert
        assert!(result.enable_billboarding);
        assert_eq!(result.edge_correction_factor, 0.5);
    }

    #[test]
    fn test_geometry_with() {
        // Arrange
        let base = GeometryOptions {
            edge_correction_factor: 0.1,
            ..default()
        };

        let other = GeometryOptions {
            enable_billboarding: true,
            edge_correction_factor: 0.9,
        };

        // Act
        let result = base.with(other);

        // Assert
        assert!(result.enable_billboarding);
        assert_eq!(result.edge_correction_factor, 0.9);
    }
}
