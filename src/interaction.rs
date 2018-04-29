use cgmath::{self, Vector2, Zero};

pub struct InteractionState {
    pub cursor_prev: Vector2<f32>,
    pub cursor_curr: Vector2<f32>,
    pub cursor_pressed: Vector2<f32>,
    pub lmouse_pressed: bool,
    pub rmouse_pressed: bool,
    pub shift_pressed: bool,
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
}
