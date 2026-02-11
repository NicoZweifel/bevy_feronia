use bevy_camera::primitives::Aabb;
use bevy_camera::visibility::VisibilityRange;
use bevy_ecs::prelude::*;
use bevy_math::{Vec3, Vec4};
use bevy_transform::prelude::*;

pub fn despawn(cmd: &mut Commands, iter: impl IntoIterator<Item = Entity>) {
    for e in iter.into_iter() {
        cmd.entity(e).despawn();
    }
}

pub trait TransformUtils {
    /// Returns this GlobalTransform relative to a new parent GlobalTransform.
    fn relative_to(&self, new_parent: &GlobalTransform) -> Transform;
}

impl TransformUtils for GlobalTransform {
    #[inline]
    fn relative_to(&self, new_parent: &GlobalTransform) -> Transform {
        GlobalTransform::from(new_parent.affine().inverse() * self.affine()).compute_transform()
    }
}

pub trait VisibilityRangeUtils {
    fn into_vec4(self) -> Vec4;

    fn as_vec4(&self) -> Vec4;
}

impl VisibilityRangeUtils for VisibilityRange {
    #[inline]
    fn into_vec4(self) -> Vec4 {
        self.as_vec4()
    }

    #[inline]
    fn as_vec4(&self) -> Vec4 {
        Vec4::new(
            self.start_margin.start,
            self.start_margin.end,
            self.end_margin.start,
            self.end_margin.end,
        )
    }
}

pub trait AabbUtils {
    fn transform(&self, transform: &Transform) -> Self;
}

impl AabbUtils for Aabb {
    fn transform(&self, transform: &Transform) -> Self {
        let center = Vec3::from(self.center);
        let half_extents = Vec3::from(self.half_extents);
        let min = center - half_extents;
        let max = center + half_extents;

        let corners = [
            Vec3::new(min.x, min.y, min.z),
            Vec3::new(min.x, min.y, max.z),
            Vec3::new(min.x, max.y, min.z),
            Vec3::new(min.x, max.y, max.z),
            Vec3::new(max.x, min.y, min.z),
            Vec3::new(max.x, min.y, max.z),
            Vec3::new(max.x, max.y, min.z),
            Vec3::new(max.x, max.y, max.z),
        ];

        let mat = transform.to_matrix();
        let mut new_min = Vec3::splat(f32::MAX);
        let mut new_max = Vec3::splat(f32::MIN);

        for corner in corners {
            let transformed = mat.transform_point3(corner);
            new_min = new_min.min(transformed);
            new_max = new_max.max(transformed);
        }

        Aabb::from_min_max(new_min, new_max)
    }
}
