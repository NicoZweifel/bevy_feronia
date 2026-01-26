use bevy_camera::visibility::VisibilityRange;
use bevy_ecs::prelude::*;
use bevy_math::Vec4;
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
