use godot::{builtin::Vector2, classes::Input, obj::Gd};

use crate::components::state_machines::character_state_machine::Event;

#[derive(Default, Clone)]
pub struct InputHandler;

impl InputHandler {
    pub fn get_velocity(input: &Gd<Input>) -> Vector2 {
        let mut vel = Vector2::ZERO;
        if input.is_action_pressed("east") {
            vel += Vector2::RIGHT;
        } else if input.is_action_pressed("west") {
            vel += Vector2::LEFT;
        } else {
            vel = Vector2::ZERO;
        }

        vel
    }

    pub fn to_platformer_event(input: &Gd<Input>) -> Event {
        let mut velocity = Vector2::ZERO;
        if input.is_action_pressed("east") {
            velocity += Vector2::RIGHT;
        }
        if input.is_action_pressed("west") {
            velocity += Vector2::LEFT;
        }
        if input.is_action_just_pressed("dodge") && velocity.length() > 0.0 {
            return Event::DodgeButton;
        }
        if input.is_action_just_pressed("attack") {
            return Event::AttackButton;
        }
        if input.is_action_pressed("jump") {
            return Event::JumpButton;
        }
        if velocity.length() > 0.0 {
            Event::Wasd
        } else {
            Event::None
        }
    }

    pub fn to_event(input: &Gd<Input>) -> Event {
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
        if input.is_action_just_pressed("dodge") && vel.length() > 0.0 {
            return Event::DodgeButton;
        }
        if input.is_action_just_pressed("attack") {
            return Event::AttackButton;
        }

        if vel.length() > 0.0 {
            Event::Wasd
        } else {
            Event::None
        }
    }
}
