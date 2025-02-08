use godot::{builtin::Vector2, classes::Input, obj::Gd};

use crate::components::state_machines::character_state_machine::Event;

#[derive(Default, Clone)]
pub struct InputHandler;

impl InputHandler {
    pub fn get_velocity(input: &Gd<Input>) -> Vector2 {
        let mut vel = Vector2::ZERO;
        if input.is_action_pressed("east") {
            vel += Vector2::RIGHT;
        }
        if input.is_action_pressed("west") {
            vel += Vector2::LEFT;
        }
        if input.is_action_pressed("north") {
            vel += Vector2::UP;
        }
        if input.is_action_pressed("south") {
            vel += Vector2::DOWN;
        }

        vel
    }

    pub fn to_event(input: &Gd<Input>, delta: &f64) -> Event {
        let mut vel = Vector2::ZERO;
        let delta = delta.to_owned();
        if input.is_action_pressed("east") {
            vel += Vector2::RIGHT;
        }
        if input.is_action_pressed("west") {
            vel += Vector2::LEFT;
        }
        if input.is_action_pressed("north") {
            vel += Vector2::UP;
        }
        if input.is_action_pressed("south") {
            vel += Vector2::DOWN;
        }
        if input.is_action_pressed("dodge") {
            return Event::DodgeButton {
                velocity: vel,
                delta,
            };
        }

        if vel.length() != 0.0 {
            Event::Wasd {
                velocity: vel,
                delta,
            }
        } else {
            Event::None
        }
    }
}
