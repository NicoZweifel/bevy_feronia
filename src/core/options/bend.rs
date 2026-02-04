use crate::prelude::*;

use bevy_math::Vec2;
use bevy_reflect::Reflect;

#[derive(Clone, Debug, Reflect, Copy, Default, PartialEq)]
pub struct StaticBendOptions {
    pub strength: f32,
    pub direction: Vec2,
    pub control_point: Vec2,
    pub min_max: Vec2,
}

impl StaticBendOptions {
    pub fn from_data(data: &MaterialOptionDataItem) -> Self {
        Self {
            strength: data.static_bend_strength.map(|s| **s).unwrap_or(0.),
            direction: data.static_bend_direction.map(|b| **b).unwrap_or_default(),
            control_point: data
                .static_bend_control_point
                .cloned()
                .map(|b| b.into())
                .unwrap_or_default(),
            min_max: data
                .static_bend_min_max
                .cloned()
                .map(|b| b.into())
                .unwrap_or_default(),
        }
    }

    pub fn with_data(mut self, data: &MaterialOptionDataItem) -> Self {
        if let Some(val) = data.static_bend_strength {
            self.strength = **val;
        }
        if let Some(val) = data.static_bend_direction {
            self.direction = **val;
        }
        if let Some(val) = data.static_bend_control_point {
            self.control_point = (*val).into();
        }
        if let Some(val) = data.static_bend_min_max {
            self.min_max = (*val).into();
        }
        self
    }

    pub fn with(mut self, other: Self) -> Self {
        if other.strength > 0. {
            self.strength = other.strength
        };
        if other.direction != Vec2::ZERO {
            self.direction = other.direction
        };
        if other.control_point != Vec2::ZERO {
            self.control_point = other.control_point
        };
        if other.min_max != Vec2::ZERO {
            self.min_max = other.min_max
        };
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
    fn test_bend_with_data() {
        // Arrange
        let options = StaticBendOptions::default();
        let strength = StaticBendStrength(0.8);
        let direction = StaticBendDirection(Vec2::new(1.0, 0.0));
        let control_point = StaticBendControlPoint::new(0.5, 0.5);
        let min_max = StaticBendMinMax::new(0.2, 0.9);
        let data = MaterialOptionDataItem {
            static_bend_strength: Some(&strength),
            static_bend_direction: Some(&direction),
            static_bend_control_point: Some(&control_point),
            static_bend_min_max: Some(&min_max),
            ..empty_data()
        };

        // Act
        let result = options.with_data(&data);

        // Assert
        assert_eq!(result.strength, 0.8);
        assert_eq!(result.direction, Vec2::new(1.0, 0.0));
        assert_eq!(result.control_point, Vec2::new(0.5, 0.5));
        assert_eq!(result.min_max, Vec2::new(0.2, 0.9));
    }

    #[test]
    fn test_bend_with() {
        // Arrange
        let base = StaticBendOptions {
            strength: 0.1,
            direction: Vec2::Y,
            ..default()
        };
        let other = StaticBendOptions {
            strength: 0.9,
            direction: Vec2::X,
            control_point: Vec2::ONE,
            min_max: Vec2::ONE,
        };

        // Act
        let result = base.with(other);

        // Assert
        assert_eq!(result.strength, 0.9);
        assert_eq!(result.direction, Vec2::X);
        assert_eq!(result.control_point, Vec2::ONE);
        assert_eq!(result.min_max, Vec2::ONE);
    }

    #[test]
    fn test_bend_with_partial() {
        // Arrange
        let base = StaticBendOptions {
            strength: 0.5,
            direction: Vec2::Y,
            ..default()
        };
        let other = StaticBendOptions {
            strength: 0.0,         // Preserve
            direction: Vec2::ZERO, // Preserve
            ..default()
        };

        // Act
        let result = base.with(other);

        // Assert
        assert_eq!(result.strength, 0.5);
        assert_eq!(result.direction, Vec2::Y);
    }
}
