use bevy_app::{App, Plugin, Update};
use bevy_color::{Color, palettes::basic::RED};
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use bevy_gizmos::gizmos::Gizmos;
use bevy_math::{IVec2, Quat, Vec3};
use bevy_platform::collections::HashMap;
use bevy_reflect::Reflect;
use bevy_transform::components::Transform;
use std::fmt;
use std::fmt::Debug;

use crate::prelude::ScatterRoot;

pub struct ScatterOccupancyDebugPlugin;

impl Plugin for ScatterOccupancyDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AvoidanceDataDebugConfig>()
            .register_type::<AvoidanceDataDebugConfig>()
            .add_systems(
                Update,
                draw_scatter_debug_gizmos.run_if(resource_exists::<AvoidanceDataDebugConfig>),
            );
    }
}

/// A component on the [`ScatterRoot`] that accumulates obstacle data from processed layers.
///
/// This allows later layers to avoid spawning on top of instances from previous layers, e.g., no foliage on rocks.
///
/// Defines a 2.5D avoidance zone used by the scatter systems.
///
/// It stores the height of the obstacle at the occupied location,
/// which is used to avoid spawning on top of it while still spawning above rocks in the ground.
///
/// TODO
/// https://github.com/NicoZweifel/bevy_feronia/issues/56
/// https://github.com/NicoZweifel/bevy_feronia/issues/43
#[derive(Component, Reflect, Clone)]
#[reflect(Component, Debug, Clone)]
pub struct ScatterOccupancyMap {
    pub cell_size: f32,
    pub cells: HashMap<IVec2, f32>,
}

impl Default for ScatterOccupancyMap {
    fn default() -> Self {
        Self {
            cell_size: 1.,
            cells: HashMap::default(),
        }
    }
}

impl Debug for ScatterOccupancyMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScatterOccupancyMap")
            .field("cell_size", &self.cell_size)
            .field("cells", &self.cells.len())
            .finish()
    }
}

impl ScatterOccupancyMap {
    /// Convert a world position to a grid space coordinate.
    #[inline]
    fn to_grid(&self, pos: Vec3) -> IVec2 {
        IVec2::new(
            (pos.x / self.cell_size).floor() as i32,
            (pos.z / self.cell_size).floor() as i32,
        )
    }

    /// Check if a position is occupied and if it is, check if it is below the stored height in this cell, e.g.,
    /// it's not above a rock in the ground, but inside/on it.
    pub fn is_occupied(&self, pos: Vec3) -> bool {
        let grid_pos = self.to_grid(pos);

        self.cells
            .get(&grid_pos)
            .map(|height| pos.y <= *height)
            .unwrap_or_default()
    }

    /// Adds a circular obstacle to the map.
    ///
    /// # Arguments
    /// * `center` - World position of the object.
    /// * `radius` - Scaled radius of the circle in world units.
    pub fn add_circle(&mut self, center: Vec3, radius: f32) {
        if radius <= 0.0 {
            return;
        }

        let min_world = center - Vec3::new(radius, 0.0, radius);
        let max_world = center + Vec3::new(radius, 0.0, radius);

        let min_grid = self.to_grid(min_world);
        let max_grid = self.to_grid(max_world);

        let radius_sq = radius.powi(2);
        let half_cell = self.cell_size / 2.0;

        for x in min_grid.x..=max_grid.x {
            for z in min_grid.y..=max_grid.y {
                let grid_pos = IVec2::new(x, z);

                let world_cell_x = (x as f32 * self.cell_size) + half_cell;
                let world_cell_z = (z as f32 * self.cell_size) + half_cell;

                let dist_x = world_cell_x - center.x;
                let dist_z = world_cell_z - center.z;
                let dist_sq = dist_x.powi(2) + dist_z.powi(2);

                if dist_sq <= radius_sq {
                    self.cells
                        .entry(grid_pos)
                        .and_modify(|h| *h = h.max(center.y))
                        .or_insert(center.y);
                }
            }
        }
    }
}

#[derive(Resource, Reflect, Deref, DerefMut)]
#[reflect(Resource)]
pub struct AvoidanceDataDebugConfig(Color);

impl AvoidanceDataDebugConfig {
    pub fn new(color: impl Into<Color>) -> Self {
        Self(color.into())
    }
}

impl Default for AvoidanceDataDebugConfig {
    fn default() -> Self {
        Self::new(RED)
    }
}

pub fn draw_scatter_debug_gizmos(
    mut gizmos: Gizmos,
    q_roots: Query<&ScatterOccupancyMap, With<ScatterRoot>>,
) {
    for map in q_roots.iter() {
        let cell_size = map.cell_size;
        let half_cell = cell_size / 2.0;

        for (grid_pos, height) in &map.cells {
            let world_x = (grid_pos.x as f32 * cell_size) + half_cell;
            let world_z = (grid_pos.y as f32 * cell_size) + half_cell;

            let cell_center = Vec3::new(world_x, *height, world_z);

            gizmos.cuboid(
                Transform {
                    translation: cell_center,
                    rotation: Quat::IDENTITY,
                    scale: Vec3::new(cell_size * 0.95, 0.1, cell_size * 0.95),
                },
                RED,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::{IVec2, Vec3};

    #[test]
    fn test_to_grid_should_return_correct_coordinates() {
        // Arrange
        let map = ScatterOccupancyMap {
            cell_size: 2.0,
            ..Default::default()
        };
        let input_position = Vec3::new(2.5, 0.0, -1.5);
        let expected = IVec2::new(1, -1);

        // Act
        let result = map.to_grid(input_position);

        // Assert
        assert_eq!(result, expected);
    }

    #[test]
    fn test_position_should_be_occupied() {
        // Arrange
        let mut map = ScatterOccupancyMap::default();
        let height = 5.0;
        let cell_coord = IVec2::new(0, 0);

        map.cells.insert(cell_coord, height);

        //  Position is inside the rock (y < height)
        let pos = Vec3::new(0.5, 4.0, 0.5);

        // Act
        let result = map.is_occupied(pos);

        // Assert
        assert!(result);
    }

    #[test]
    fn test_positions_should_be_free() {
        // Arrange
        let mut map = ScatterOccupancyMap::default();
        let height = 5.0;
        let cell_coord = IVec2::new(0, 0);

        map.cells.insert(cell_coord, height);

        // Position is above the rock (y > height)
        let pos = Vec3::new(0.5, 6.0, 0.5);

        // Act
        let result = !map.is_occupied(pos);

        // Assert
        assert!(result);
    }

    #[test]
    fn test_circle_should_occupy_area() {
        // Arrange
        let mut map = ScatterOccupancyMap {
            cell_size: 1.0,
            ..Default::default()
        };
        let center = Vec3::new(0.5, 0.0, 0.5);

        // Radius of 1.1 covers neighbor at 1,0 with distance 1.0,
        // but not diagonal at 1,1 with distance 1.41 (sqrt(2)).
        let radius = 1.1;

        // Act
        map.add_circle(center, radius);

        // Assert
        let center_occupied = map.cells.contains_key(&IVec2::new(0, 0));
        let neighbor_occupied = map.cells.contains_key(&IVec2::new(1, 0));
        let diagonal_occupied = map.cells.contains_key(&IVec2::new(1, 1));

        assert!(center_occupied, "Center should be occupied");
        assert!(neighbor_occupied, "Neighbor should be occupied");
        assert!(!diagonal_occupied, "Diagonal should not be occupied");
    }

    #[test]
    fn test_circle_should_store_height() {
        // Arrange
        let mut map = ScatterOccupancyMap::default();
        let center = Vec3::new(0.5, 12.5, 0.5);
        let radius = 0.5;

        // Act
        map.add_circle(center, radius);

        // Assert
        let stored_height = *map.cells.get(&IVec2::new(0, 0)).unwrap();
        assert_eq!(stored_height, 12.5);
    }
}
