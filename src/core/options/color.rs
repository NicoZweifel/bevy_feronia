use crate::prelude::*;

use bevy_color::Color;
use bevy_reflect::Reflect;

#[derive(Clone, Debug, Reflect, Copy, Default, PartialEq)]
pub struct ColorOptions {
    pub base_color: Option<Color>,
    pub top_color: Option<Color>,
    pub bottom_color: Option<Color>,
    pub tint_factor: f32,
    pub gradient_start: f32,
    pub gradient_end: f32,
}

impl ColorOptions {
    pub fn from_data(data: &MaterialOptionDataItem) -> Self {
        Self {
            base_color: data.base_color.map(|c| **c),
            top_color: data.color_gradient.map(|g| g.top),
            bottom_color: data.color_gradient.map(|g| g.bottom),
            tint_factor: data.color_gradient.map(|g| g.tint).unwrap_or(0.),
            gradient_end: data.color_gradient.map(|g| g.end).unwrap_or(0.),
            gradient_start: data.color_gradient.map(|g| g.start).unwrap_or(0.),
        }
    }

    pub fn with_data(mut self, data: &MaterialOptionDataItem) -> Self {
        if let Some(val) = data.base_color {
            self.base_color = Some(**val);
        }
        if let Some(val) = data.color_gradient {
            self.top_color = Some(val.top);
            self.bottom_color = Some(val.bottom);
            self.tint_factor = val.tint;
            self.gradient_start = val.start;
            self.gradient_end = val.end;
        }
        self
    }

    pub fn with(mut self, other: Self) -> Self {
        self.base_color = other.base_color.or(self.base_color);
        self.top_color = other.top_color.or(self.top_color);
        self.bottom_color = other.bottom_color.or(self.bottom_color);
        if other.tint_factor > 0. {
            self.tint_factor = other.tint_factor;
        }
        if other.gradient_start > 0. {
            self.gradient_start = other.gradient_start;
        }
        if other.gradient_end > 0. {
            self.gradient_end = other.gradient_end;
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_color::palettes::css::*;
    use bevy_eidolon::prelude::InstanceColor;
    use bevy_utils::default;

    fn empty_data() -> MaterialOptionDataItem<'static, 'static> {
        unsafe { std::mem::zeroed() }
    }

    #[test]
    fn test_color_with_data() {
        // Arrange
        let options = ColorOptions::default();
        let base = InstanceColor(RED.into());
        let gradient = InstanceColorGradient {
            top: BLUE.into(),
            bottom: GREEN.into(),
            tint: 0.5,
            start: 0.2,
            end: 0.8,
        };
        let data = MaterialOptionDataItem {
            base_color: Some(&base),
            color_gradient: Some(&gradient),
            ..empty_data()
        };

        // Act
        let result = options.with_data(&data);

        // Assert
        assert_eq!(result.base_color, Some(RED.into()));
        assert_eq!(result.top_color, Some(BLUE.into()));
        assert_eq!(result.bottom_color, Some(GREEN.into()));
        assert_eq!(result.tint_factor, 0.5);
        assert_eq!(result.gradient_start, 0.2);
        assert_eq!(result.gradient_end, 0.8);
    }

    #[test]
    fn test_color_with() {
        let base = ColorOptions {
            base_color: Some(BLACK.into()),
            tint_factor: 0.1,
            ..default()
        };
        let other = ColorOptions {
            base_color: Some(WHITE.into()), // Override
            top_color: Some(RED.into()),
            tint_factor: 0.9, // Override
            ..default()
        };

        let result = base.with(other);

        assert_eq!(result.base_color, Some(WHITE.into()));
        assert_eq!(result.top_color, Some(RED.into()));
        assert_eq!(result.tint_factor, 0.9);
    }

    #[test]
    fn test_color_with_preservation() {
        // Arrange
        let base = ColorOptions {
            base_color: Some(BLACK.into()),
            tint_factor: 0.5,
            ..default()
        };
        let other = ColorOptions {
            base_color: None, // Preserve
            tint_factor: 0.0, // Preserve
            ..default()
        };

        // Act
        let result = base.with(other);

        // Assert
        assert_eq!(result.base_color, Some(BLACK.into()));
        assert_eq!(result.tint_factor, 0.5);
    }
}
