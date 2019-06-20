use cgmath::{self, Vector2, Zero};

pub struct InteractionState {
    /// The previous position of the user's cursor (last frame)
    pub cursor_prev: Vector2<f32>,

    /// The current position of the user's cursor (this frame)
    pub cursor_curr: Vector2<f32>,

    /// The screen coordinates of where the mouse button was last pressed
    pub cursor_pressed: Vector2<f32>,

    /// Whether or not the left mouse button is pressed
    pub lmouse_pressed: bool,

    /// Whether or not the right mouse button is pressed
    pub rmouse_pressed: bool,

    /// Whether or not the shift key is pressed
    pub shift_pressed: bool,

    /// Whether or not the control key is pressed
    pub ctrl_pressed: bool,
}

impl InteractionState {
    pub fn new() -> InteractionState {
        InteractionState {
            cursor_prev: Vector2::zero(),
            cursor_curr: Vector2::zero(),
            cursor_pressed: Vector2::zero(),
            lmouse_pressed: false,
            rmouse_pressed: false,
            shift_pressed: false,
            ctrl_pressed: false,
        }
    }

    /// Returns the amount that the cursor has moved since it was last pressed (used
    /// during mouse-drag calculations).
    pub fn get_mouse_delta(&self) -> Vector2<f32> {
        self.cursor_curr - self.cursor_prev
    }
}
