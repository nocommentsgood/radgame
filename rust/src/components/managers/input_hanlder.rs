use godot::{builtin::Vector2, classes::Input, obj::Gd};

use crate::components::state_machines::character_state_machine::Event;

#[derive(Default, Clone)]
pub struct InputHandler;

impl InputHandler {
    pub fn get_vel_and_event(input: &Gd<Input>) -> (Event, f32) {
        let mut velocity = Vector2::ZERO;
        if input.is_action_pressed("east") {
            velocity += Vector2::RIGHT;
        }
        if input.is_action_pressed("west") {
            velocity += Vector2::LEFT;
        }

        if velocity.length() > 0.0 {
            (Event::Wasd, velocity.normalized().x)
        } else {
            (Event::None, velocity.normalized_or_zero().x)
        }
    }
}
