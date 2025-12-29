use bevy_ecs::prelude::*;
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
