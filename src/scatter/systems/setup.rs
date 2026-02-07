use crate::prelude::*;
use bevy_camera::primitives::Aabb;
use bevy_camera::visibility::NoAutoAabb;
use bevy_ecs::prelude::*;
use bevy_math::{Mat3A, Vec3A};
use bevy_state::state::NextState;
use bevy_transform::prelude::GlobalTransform;

#[cfg(feature = "trace")]
use tracing::debug;

pub fn transition_to_collecting(
    q_pending_roots: Query<Entity, (With<ScatterRoot>, Without<Aabb>)>,
    mut next_state: ResMut<NextState<ScatterState>>,
) {
    if !q_pending_roots.is_empty() {
        return;
    };

    #[cfg(feature = "trace")]
    debug!("Setting ScatterState::Collecting.");

    next_state.set(ScatterState::Collecting);
}

pub fn setup_root_aabb(
    mut cmd: Commands,
    q_root: Query<(Entity, &Children, &GlobalTransform), (With<ScatterRoot>, Without<Aabb>)>,
    q_children: Query<&Children, Without<ScatterLayer>>,
    q_mesh_info: Query<(&Aabb, &GlobalTransform), Without<ScatterLayer>>,
) {
    for (root_entity, children, root_transform) in &q_root {
        let mut local_min = Vec3A::splat(f32::MAX);
        let mut local_max = Vec3A::splat(f32::MIN);
        let mut found_any = false;

        let root_inv = root_transform.affine().inverse();

        for (child_aabb, child_transform) in children
            .iter()
            .flat_map(|child| std::iter::once(child).chain(q_children.iter_descendants(child)))
            .filter_map(|entity| q_mesh_info.get(entity).ok())
        {
            found_any = true;

            let relative_affine = root_inv * child_transform.affine();

            let relative_mat = Mat3A::from_cols(
                relative_affine.x_axis,
                relative_affine.y_axis,
                relative_affine.z_axis,
            );

            let local_center = relative_affine.transform_point3a(child_aabb.center);
            let local_half_extents = relative_mat.abs() * child_aabb.half_extents;

            local_min = local_min.min(local_center - local_half_extents);
            local_max = local_max.max(local_center + local_half_extents);
        }

        if !found_any {
            continue;
        }

        let center = (local_min + local_max) * 0.5;
        let half_extents = (local_max - local_min) * 0.5;

        let final_aabb = Aabb {
            center,
            half_extents,
        };

        #[cfg(feature = "trace")]
        debug!("Calculated Root AABB: {:?}", final_aabb);

        cmd.entity(root_entity).insert((
            final_aabb, // Required since bevy 0.18 if adding aabb manually.
            NoAutoAabb,
        ));
    }
}
