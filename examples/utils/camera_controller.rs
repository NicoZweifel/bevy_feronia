use std::f32::consts::FRAC_PI_2;

use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*};

pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_camera);
    }
}

fn move_camera(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut transform: Single<&mut Transform, With<Camera3d>>,
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
) {
    let mut direction = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::KeyW) {
        direction += transform.forward().as_vec3();
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        direction += transform.back().as_vec3();
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        direction += transform.left().as_vec3();
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        direction += transform.right().as_vec3();
    }

    if direction.length_squared() > 0.0 {
        direction = direction.normalize();

        transform.translation += direction * 10.0 * time.delta_secs();
    }

    let delta = accumulated_mouse_motion.delta;

    if delta != Vec2::ZERO {
        let delta_yaw = -delta.x * 0.001;
        let delta_pitch = -delta.y * 0.001;

        let (yaw, pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);
        let yaw = yaw + delta_yaw;

        const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;
        let pitch = (pitch + delta_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);

        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
    }
}
