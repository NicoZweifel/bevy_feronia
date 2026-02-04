use crate::prelude::*;
use bevy_reflect::Reflect;

#[derive(Clone, Debug, Reflect, Copy, Default, PartialEq)]
pub struct WindOptions {
    pub affected: bool,
    pub low_quality: bool,
}

impl WindOptions {
    pub fn from_data(data: &MaterialOptionDataItem) -> Self {
        Self {
            affected: data.wind_affected.is_some(),
            low_quality: data.low_q.is_some(),
        }
    }

    pub fn with_data(mut self, data: &MaterialOptionDataItem) -> Self {
        self.affected |= data.wind_affected.is_some();
        self.low_quality |= data.low_q.is_some();
        self
    }

    pub fn with(mut self, other: Self) -> Self {
        self.affected |= other.affected;
        self.low_quality |= other.low_quality;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_data() -> MaterialOptionDataItem<'static, 'static> {
        unsafe { std::mem::zeroed() }
    }

    #[test]
    fn test_wind_with_data() {
        // Arrange
        let options = WindOptions::default();
        let affected = WindAffected;
        let low_q = LowQuality;
        let data = MaterialOptionDataItem {
            wind_affected: Some(&affected),
            low_q: Some(&low_q),
            ..empty_data()
        };

        // Act
        let result = options.with_data(&data);

        // Assert
        assert!(result.affected);
        assert!(result.low_quality);
    }

    #[test]
    fn test_wind_with() {
        // Arrange
        let base = WindOptions {
            affected: false,
            low_quality: false,
        };
        let other = WindOptions {
            affected: true,
            low_quality: true,
        };

        // Act
        let result = base.with(other);

        // Assert
        assert!(result.affected);
        assert!(result.low_quality);
    }
}
