use std::collections::HashSet;

/// Keyboard key codes (subset of winit VirtualKeyCode).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    // Letters
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    // Numbers
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    // Function keys
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    // Special
    Escape,
    Space,
    Enter,
    Tab,
    Backspace,
    Delete,
    Up,
    Down,
    Left,
    Right,
    LeftShift,
    RightShift,
    LeftControl,
    RightControl,
    LeftAlt,
    RightAlt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Tracks pressed/released state for keyboard and mouse.
/// Inserted as a Resource by WindowPlugin.
#[derive(Debug, Clone, Default)]
pub struct InputState {
    keys_pressed: HashSet<KeyCode>,
    keys_just_pressed: HashSet<KeyCode>,
    keys_just_released: HashSet<KeyCode>,
    mouse_pressed: HashSet<MouseButton>,
    mouse_just_pressed: HashSet<MouseButton>,
    mouse_just_released: HashSet<MouseButton>,
    pub mouse_position: [f32; 2],
    pub mouse_delta: [f32; 2],
    pub scroll_delta: f32,
}

impl InputState {
    /// Call at the start of each frame to clear "just" states.
    pub fn begin_frame(&mut self) {
        self.keys_just_pressed.clear();
        self.keys_just_released.clear();
        self.mouse_just_pressed.clear();
        self.mouse_just_released.clear();
        self.mouse_delta = [0.0, 0.0];
        self.scroll_delta = 0.0;
    }

    pub fn press_key(&mut self, key: KeyCode) {
        if self.keys_pressed.insert(key) {
            self.keys_just_pressed.insert(key);
        }
    }

    pub fn release_key(&mut self, key: KeyCode) {
        if self.keys_pressed.remove(&key) {
            self.keys_just_released.insert(key);
        }
    }

    pub fn press_mouse(&mut self, button: MouseButton) {
        if self.mouse_pressed.insert(button) {
            self.mouse_just_pressed.insert(button);
        }
    }

    pub fn release_mouse(&mut self, button: MouseButton) {
        if self.mouse_pressed.remove(&button) {
            self.mouse_just_released.insert(button);
        }
    }

    // Queries
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.keys_just_pressed.contains(&key)
    }

    pub fn is_key_just_released(&self, key: KeyCode) -> bool {
        self.keys_just_released.contains(&key)
    }

    pub fn is_mouse_pressed(&self, button: MouseButton) -> bool {
        self.mouse_pressed.contains(&button)
    }

    pub fn is_mouse_just_pressed(&self, button: MouseButton) -> bool {
        self.mouse_just_pressed.contains(&button)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn press_and_release_key_tracking() {
        let mut input = InputState::default();

        // Initially nothing is pressed.
        assert!(!input.is_key_pressed(KeyCode::W));
        assert!(!input.is_key_just_pressed(KeyCode::W));

        // Press W.
        input.press_key(KeyCode::W);
        assert!(input.is_key_pressed(KeyCode::W));
        assert!(input.is_key_just_pressed(KeyCode::W));

        // Pressing again should NOT re-trigger just_pressed.
        input.press_key(KeyCode::W);
        assert!(input.is_key_pressed(KeyCode::W));
        // just_pressed was already set from the first press, still true this frame.
        assert!(input.is_key_just_pressed(KeyCode::W));

        // Release W.
        input.release_key(KeyCode::W);
        assert!(!input.is_key_pressed(KeyCode::W));
        assert!(input.is_key_just_released(KeyCode::W));

        // Releasing again when not pressed should NOT set just_released.
        input.begin_frame();
        input.release_key(KeyCode::W);
        assert!(!input.is_key_just_released(KeyCode::W));
    }

    #[test]
    fn just_pressed_clears_after_begin_frame() {
        let mut input = InputState::default();

        input.press_key(KeyCode::Space);
        assert!(input.is_key_just_pressed(KeyCode::Space));
        assert!(input.is_key_pressed(KeyCode::Space));

        // After begin_frame, just_pressed should clear but pressed persists.
        input.begin_frame();
        assert!(!input.is_key_just_pressed(KeyCode::Space));
        assert!(input.is_key_pressed(KeyCode::Space));
    }

    #[test]
    fn just_released_clears_after_begin_frame() {
        let mut input = InputState::default();

        input.press_key(KeyCode::Escape);
        input.release_key(KeyCode::Escape);
        assert!(input.is_key_just_released(KeyCode::Escape));

        input.begin_frame();
        assert!(!input.is_key_just_released(KeyCode::Escape));
    }

    #[test]
    fn mouse_button_tracking() {
        let mut input = InputState::default();

        // Press left mouse.
        input.press_mouse(MouseButton::Left);
        assert!(input.is_mouse_pressed(MouseButton::Left));
        assert!(input.is_mouse_just_pressed(MouseButton::Left));
        assert!(!input.is_mouse_pressed(MouseButton::Right));

        // After begin_frame, just_pressed clears.
        input.begin_frame();
        assert!(input.is_mouse_pressed(MouseButton::Left));
        assert!(!input.is_mouse_just_pressed(MouseButton::Left));

        // Release left mouse.
        input.release_mouse(MouseButton::Left);
        assert!(!input.is_mouse_pressed(MouseButton::Left));
    }

    #[test]
    fn mouse_delta_resets_each_frame() {
        let mut input = InputState::default();

        input.mouse_delta = [10.0, -5.0];
        input.scroll_delta = 3.0;
        assert_eq!(input.mouse_delta, [10.0, -5.0]);
        assert_eq!(input.scroll_delta, 3.0);

        input.begin_frame();
        assert_eq!(input.mouse_delta, [0.0, 0.0]);
        assert_eq!(input.scroll_delta, 0.0);
    }

    #[test]
    fn multiple_keys_tracked_independently() {
        let mut input = InputState::default();

        input.press_key(KeyCode::A);
        input.press_key(KeyCode::D);
        assert!(input.is_key_pressed(KeyCode::A));
        assert!(input.is_key_pressed(KeyCode::D));
        assert!(!input.is_key_pressed(KeyCode::W));

        input.release_key(KeyCode::A);
        assert!(!input.is_key_pressed(KeyCode::A));
        assert!(input.is_key_pressed(KeyCode::D));
    }
}
