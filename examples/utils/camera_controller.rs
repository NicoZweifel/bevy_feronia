use std::f32::consts::FRAC_PI_2;

use bevy::window::CursorOptions;
use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*, window::CursorGrabMode};
use bevy_inspector_egui::bevy_egui::{EguiContexts, EguiPreUpdateSet};

pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (block_mouse_input, block_keyboard_input)
                .after(EguiPreUpdateSet::ProcessInput)
                .before(EguiPreUpdateSet::BeginPass),
        );
        app.add_systems(Update, (enable_cursor, disable_cursor, move_camera));
    }
}

#[derive(Component, Default)]
pub struct Controller {
    pub enabled: bool,
}

fn move_camera(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    single: Single<(&mut Transform, &Controller)>,
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
) {
    let (mut transform, controller) = single.into_inner();

    if !controller.enabled {
        return;
    };

    let amplify = keyboard_input.pressed(KeyCode::ShiftLeft);

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

        let factor = if amplify { 40.0 } else { 10.0 };

        transform.translation += direction * factor * time.delta_secs();
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

fn disable_cursor(
    btn: Res<ButtonInput<MouseButton>>,
    mut q: Query<&mut CursorOptions, With<Window>>,
    controller: Single<&mut Controller>,
) {
    if !btn.just_pressed(MouseButton::Left) {
        return;
    };

    for mut options in &mut q {
        options.grab_mode = CursorGrabMode::Locked;
        options.visible = false;
    }

    let mut controller = controller.into_inner();
    controller.enabled = true;
}

fn enable_cursor(
    key: Res<ButtonInput<KeyCode>>,
    mut q: Query<&mut CursorOptions, With<Window>>,
    controller: Single<&mut Controller>,
) {
    if !key.just_pressed(KeyCode::Escape) {
        return;
    };

    for mut options in &mut q {
        options.grab_mode = CursorGrabMode::None;
        options.visible = true;
    }

    let mut controller = controller.into_inner();
    controller.enabled = false;
}

pub fn block_mouse_input(mut mouse: ResMut<ButtonInput<MouseButton>>, mut contexts: EguiContexts) {
    let Ok(context) = contexts.ctx_mut() else {
        return;
    };

    if context.is_pointer_over_area() || context.wants_pointer_input() {
        mouse.reset_all();
    }
}

pub fn block_keyboard_input(
    mut keyboard_keycode: ResMut<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
) {
    let Ok(context) = contexts.ctx_mut() else {
        return;
    };

    if context.wants_keyboard_input() {
        keyboard_keycode.reset_all();
    }
}
