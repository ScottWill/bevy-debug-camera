use std::ops::Neg;

use crate::{
    components::DebugCamera,
    resources::{ActiveGamepad, DebugCameraActive, GamepadBindings, KeyboardBindings, GamepadInput},
};
use bevy::{
    input::{
        gamepad::{GamepadButton, GamepadSettings, GamepadEvent, GamepadConnection},
        mouse::MouseMotion,
    },
    prelude::*,
    utils::tracing::{event, Level},
    window::{CursorGrabMode, PrimaryWindow},
};

/// This is the main system responsible for updating camera movement. It takes mouse, keyboard, and
/// gamepad input and updates the [`DebugCamera`] component acording to those changes. This
/// explicitly does *not* update the camera's tranform.
#[allow(clippy::too_many_arguments)]
pub fn camera_movement_system(
    mut q: Query<&mut DebugCamera>,
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    debug_camera_active: Res<DebugCameraActive>,
    keyboard_bindings: Res<KeyboardBindings>,
    gamepad_bindings: Res<GamepadBindings>,
    mut motion_evr: EventReader<MouseMotion>,
    axes: Res<Axis<GamepadAxis>>,
    buttons: Res<Input<GamepadButton>>,
    button_axes: Res<Axis<GamepadButton>>,
    active_gamepad: ResMut<ActiveGamepad>,
) {
    // Shortcut if neither control scheme is active. This is not strictly needed, but it avoids
    // some computation if controls are inactive.
    if !(debug_camera_active.gamepad || debug_camera_active.keymouse) {
        return;
    }

    // All calculations before going into each camera are done from the camera's frame
    // of reference. We assume x = fwd, y = right, z = up
    let mut rotate_vec = Vec3::default();
    let mut local_translate_vec = Vec3::default();

    // First, apply controller if present and active
    if debug_camera_active.gamepad {
        if let Some(gamepad) = active_gamepad.0 {
            // Apply translation
            let left = input_value(gamepad, &axes, &buttons, &button_axes, &gamepad_bindings.left, -1.0);
            let right = input_value(gamepad, &axes, &buttons, &button_axes, &gamepad_bindings.right, 1.0);
            let fwd = input_value(gamepad, &axes, &buttons, &button_axes, &gamepad_bindings.fwd, 1.0);
            let bwd = input_value(gamepad, &axes, &buttons, &button_axes, &gamepad_bindings.bwd, -1.0);
            let up = input_value(gamepad, &axes, &buttons, &button_axes, &gamepad_bindings.up, 1.0);
            let down = input_value(gamepad, &axes, &buttons, &button_axes, &gamepad_bindings.down, -1.0);
            local_translate_vec += time.delta_seconds() * Vec3::new(fwd + bwd, up + down, left + right);

            // Apply rotation
            let yaw_left = input_value(gamepad, &axes, &buttons, &button_axes, &gamepad_bindings.yaw_left, -1.0);
            let yaw_right = input_value(gamepad, &axes, &buttons, &button_axes, &gamepad_bindings.yaw_right, 1.0);
            let pitch_up = input_value(gamepad, &axes, &buttons, &button_axes, &gamepad_bindings.pitch_up, 1.0);
            let pitch_down = input_value(gamepad, &axes, &buttons, &button_axes, &gamepad_bindings.pitch_down, -1.0);
            let roll_left = input_value(gamepad, &axes, &buttons, &button_axes, &gamepad_bindings.roll_left, -1.0);
            let roll_right = input_value(gamepad, &axes, &buttons, &button_axes, &gamepad_bindings.roll_right, 1.0);
            rotate_vec += time.delta_seconds() * Vec3::new(-yaw_left - yaw_right, pitch_up + pitch_down, roll_left + roll_right);
        }
    }

    // Next, apply keyboard and mouse controls
    if debug_camera_active.keymouse {
        let key_fwd = keys.pressed(keyboard_bindings.fwd);
        let key_bwd = keys.pressed(keyboard_bindings.bwd);
        let key_up = keys.pressed(keyboard_bindings.up);
        let key_down = keys.pressed(keyboard_bindings.down);
        let key_left = keys.pressed(keyboard_bindings.left);
        let key_right = keys.pressed(keyboard_bindings.right);
        let key_roll_left = keys.pressed(keyboard_bindings.roll_left);
        let key_roll_right = keys.pressed(keyboard_bindings.roll_right);
        let mouse_delta = {
            let mut d = Vec2::default();
            for ev in motion_evr.iter() {
                d -= ev.delta;
            }
            d
        };

        // All keyboard and mouse input is multiplied by 0.5, as otherwise it will go too fast
        // compared with controller
        local_translate_vec += time.delta_seconds()
            * 0.5
            * Vec3::new(
                buttons_to_dir(key_fwd, key_bwd),
                buttons_to_dir(key_up, key_down),
                buttons_to_dir(key_right, key_left),
            );
        rotate_vec += time.delta_seconds()
            * 0.5
            * Vec3::new(
                mouse_delta.x,
                mouse_delta.y,
                buttons_to_dir(key_roll_right, key_roll_left),
            );
    }

    for mut controlled_camera in q.iter_mut() {
        // We start by computing and correcting all our basis vectors to be unit vectors that are
        // perpendicular to each other. This fixes any
        let mut right = controlled_camera.fwd.cross(controlled_camera.up);
        controlled_camera.up = right.cross(controlled_camera.fwd);
        controlled_camera.fwd = controlled_camera.up.cross(right);
        controlled_camera.up = controlled_camera.up.normalize();
        controlled_camera.fwd = controlled_camera.fwd.normalize();
        right = controlled_camera.fwd.cross(controlled_camera.up);

        // This matrix converts the local coordinate frame to world coordinates. General assumption
        // is that the up and fwd vectors are unit vectors and perpendicular to each other at this
        // point.
        let basis_matrix = Mat3::from_cols(controlled_camera.fwd, controlled_camera.up, right);
        let speed_translate = controlled_camera.speed_translate;

        // Translation first. It's just a simple basis matrix multiplication
        controlled_camera.position += speed_translate * (basis_matrix * local_translate_vec);

        // invert x?
        let x_rot = invert(rotate_vec.x, debug_camera_active.invert_x);
        // Rotation last. Rotation is applied on each direction individually for simplicity
        // x rotation is relative to the up vector. Should keep both vectors perpendicular
        let x_rot_quat = Quat::from_axis_angle(
            controlled_camera.up,
            x_rot * controlled_camera.speed_rotate,
        );
        controlled_camera.fwd = x_rot_quat * controlled_camera.fwd;
        right = x_rot_quat * right;

        // invert y?
        let y_rot = invert(rotate_vec.y, debug_camera_active.invert_y);
        // y rotation is done by the right axis, which we just updated to rotate both fwd and up.
        // Both are still perpendicular and unit vectors, so we don't need to normalise the result.
        let y_rot_quat =
            Quat::from_axis_angle(right, y_rot * controlled_camera.speed_rotate);
        controlled_camera.fwd = y_rot_quat * controlled_camera.fwd;
        controlled_camera.up = y_rot_quat * controlled_camera.up;
        // lastly, z rotation is done relative to the fwd vector.
        let z_rot_quat = Quat::from_axis_angle(
            controlled_camera.fwd,
            rotate_vec.z * controlled_camera.speed_rotate,
        );
        controlled_camera.up = z_rot_quat * controlled_camera.up;
    }
}

/// This system is responsible for updating the camera's transform according to the [`DebugCamera`]
/// component. When both control methods are off, this system stops updating, letting you control
/// the camera independently (though we recommend removing the component entirely if you want to
/// take over).
pub fn camera_update_system(
    mut q: Query<(&mut Transform, &DebugCamera), With<Camera>>,
    debug_camera_active: Res<DebugCameraActive>,
) {
    if debug_camera_active.gamepad || debug_camera_active.keymouse {
        for (mut transform, controlled_camera) in q.iter_mut() {
            *transform = Transform::from_translation(controlled_camera.position).looking_at(
                controlled_camera.position + controlled_camera.fwd,
                controlled_camera.up,
            );
        }
    }
}

/// This system ensures we're always locking the cursor in on the screen when running. We
/// stop running this logic if keymouse input is off, letting you change the cursor mode.
pub fn cursor_grab_system(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    debug_camera_active: Res<DebugCameraActive>,
) {
    if let Ok(mut window) = windows.get_single_mut() {
        if window.cursor.visible == debug_camera_active.keymouse {
            window.cursor.visible = !window.cursor.visible;
            window.cursor.grab_mode = match window.cursor.visible {
                true => CursorGrabMode::None,
                false => CursorGrabMode::Locked,
            };
        }
    }
}

/// This system manages gamepad connections and sets the current active gamepad. It will set the
/// [`ActiveGamepad`] resource to said gamepad ID, and will send a tracing event on set and unset.
pub fn gamepad_connections(
    mut active_gamepad: ResMut<ActiveGamepad>,
    mut gamepad_evr: EventReader<GamepadEvent>,
    mut settings: ResMut<GamepadSettings>,
) {
    for ev in gamepad_evr.iter() {
        // Only matching again
        match &ev {
            GamepadEvent::Connection(event_info) => {
                // the ID of the gamepad
                let id = event_info.gamepad.id;
                match &event_info.connection {
                    GamepadConnection::Connected(info) => {
                        if active_gamepad.0.is_none() {
                            event!(
                                Level::INFO,
                                event = "active_gamepad_set",
                                gamepad_name = info.name,
                                gamepad_id = id,
                            );
                            active_gamepad.0 = Some(event_info.gamepad);
        
                            // Configure controller for better use
                            settings.default_axis_settings.set_deadzone_lowerbound(-0.1);
                            settings.default_axis_settings.set_deadzone_upperbound(0.1);
                        }
                    }
                    GamepadConnection::Disconnected => {
                        let mut remove_gamepad = false;
                        if let Some(old_id) = active_gamepad.0 {
                            if old_id == event_info.gamepad {
                                event!(
                                    Level::INFO,
                                    event = "active_gamepad_removed",
                                    gamepad_id = id,
                                );
                                remove_gamepad = true;
                            }
                        }
                        if remove_gamepad {
                            active_gamepad.0 = None;
                        }
                    }
                }
            }
            _ => ()
        }
    }
}

fn input_value(
    gamepad: Gamepad,
    axes: &Axis<GamepadAxis>,
    buttons: &Input<GamepadButton>,
    button_axes: &Axis<GamepadButton>,
    input: &GamepadInput,
    dir: f32,
) -> f32 {
    match input {
        GamepadInput::Axis(axis) => {
            if let Some(v) = axes.get(GamepadAxis::new(gamepad, *axis)) {
                if v.signum() == dir.signum() {
                    return v;
                }
            }
        }
        GamepadInput::Button(button) => {
            if buttons.pressed(GamepadButton::new(gamepad, *button)) {
                return dir.signum();
            }
        }
        GamepadInput::Trigger(button) => {
            if let Some(v) = button_axes.get(GamepadButton::new(gamepad, *button)) {
                return v * dir.signum();
            }
        }
    }
    return 0.0;
}

fn buttons_to_dir<T>(positive: bool, negative: bool) -> T
where T: From<i8>
{
    (Into::<i8>::into(positive) - Into::<i8>::into(negative)).into()
}

#[inline(always)]
fn invert<T>(v: T, invert: bool) -> T
where T: Neg<Output = T>
{
    match invert {
        true => -v,
        false => v,
    }
}
