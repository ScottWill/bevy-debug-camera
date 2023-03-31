use bevy::prelude::*;

#[derive(Resource, Default, Debug)]
pub struct ActiveGamepad(pub Option<Gamepad>);

/// This system signals whether the debug camera should be active. You can selectively pick which
/// input types are active at a given time. You can
#[derive(Resource, Debug, Clone)]
pub struct DebugCameraActive {
    /// If set to true, our keyboard + mouse bindings will be active for any cameras marked as
    /// [`crate::DebugCamera`].
    pub keymouse: bool,
    /// If set to true, our gamepad bindings will be active for any cameras marked as
    /// [`crate::DebugCamera`].
    pub gamepad: bool,

    pub invert_x: bool,
    pub invert_y: bool,
}

impl Default for DebugCameraActive {
    fn default() -> DebugCameraActive {
        DebugCameraActive {
            keymouse: true,
            gamepad: true,
            invert_x: false,
            invert_y: false,
        }
    }
}

/// Configurable bindings for keyboard input. Field defaults can be found in the crate root
/// documentation.
#[derive(Resource, Debug, Clone)]
pub struct KeyboardBindings {
    pub fwd: KeyCode,
    pub bwd: KeyCode,
    pub up: KeyCode,
    pub down: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    pub roll_left: KeyCode,
    pub roll_right: KeyCode,
}

impl Default for KeyboardBindings {
    fn default() -> KeyboardBindings {
        KeyboardBindings {
            fwd: KeyCode::W,
            bwd: KeyCode::S,
            up: KeyCode::Space,
            down: KeyCode::LShift,
            left: KeyCode::A,
            right: KeyCode::D,
            roll_left: KeyCode::Q,
            roll_right: KeyCode::E,
        }
    }
}

#[derive(Debug, Clone)]
pub enum GamepadInput {
    Axis(GamepadAxisType),
    Button(GamepadButtonType),
    Trigger(GamepadButtonType),
}

/// Configurable bindings for gamepad input. Field defaults can be found in the crate root
/// documentation.
#[derive(Resource, Debug, Clone)]
pub struct GamepadBindings {
    pub fwd: GamepadInput,
    pub bwd: GamepadInput,
    pub up: GamepadInput,
    pub down: GamepadInput,
    pub left: GamepadInput,
    pub right: GamepadInput,
    pub roll_left: GamepadInput,
    pub roll_right: GamepadInput,
    pub yaw_left: GamepadInput,
    pub yaw_right: GamepadInput,
    pub pitch_up: GamepadInput,
    pub pitch_down: GamepadInput,
}

impl Default for GamepadBindings {
    fn default() -> GamepadBindings {
        GamepadBindings {
            fwd: GamepadInput::Axis(GamepadAxisType::LeftStickY),
            bwd: GamepadInput::Axis(GamepadAxisType::LeftStickY),
            up: GamepadInput::Trigger(GamepadButtonType::RightTrigger2),
            down: GamepadInput::Trigger(GamepadButtonType::LeftTrigger2),
            left: GamepadInput::Axis(GamepadAxisType::LeftStickX),
            right: GamepadInput::Axis(GamepadAxisType::LeftStickX),
            roll_left: GamepadInput::Button(GamepadButtonType::LeftTrigger),
            roll_right: GamepadInput::Button(GamepadButtonType::RightTrigger),
            yaw_left: GamepadInput::Axis(GamepadAxisType::RightStickX),
            yaw_right: GamepadInput::Axis(GamepadAxisType::RightStickX),
            pitch_up: GamepadInput::Axis(GamepadAxisType::RightStickY),
            pitch_down: GamepadInput::Axis(GamepadAxisType::RightStickY),
        }
    }
}
