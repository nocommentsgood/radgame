use std::collections::HashMap;

use godot::{builtin::Vector2, classes::Input, obj::Gd};

use crate::entities::{
    entity_stats::{StatVal, Stats},
    player::character_state_machine::State,
};

const GRAVITY: f32 = 200.0;
const TERMINAL_VELOCITY: f32 = -200.0;
type Event = crate::entities::player::character_state_machine::Event;

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
    pub fn get_movement(input: &Gd<Input>, mut velocity: Vector2) -> Vector2 {
        if input.is_action_pressed("east") {
            velocity += Vector2::RIGHT;
        }
        if input.is_action_pressed("west") {
            velocity += Vector2::LEFT;
        }
        velocity
    }
    pub fn get_vel_and_something(
        state: &State,
        stats: &HashMap<Stats, StatVal>,
        mut velocity: Vector2,
    ) -> (Event, Vector2) {
        match state {
            State::Falling {} => {
                if velocity.y >= TERMINAL_VELOCITY {
                    velocity.y += GRAVITY
                }
            }
            State::Jumping {} => {
                velocity.y = Vector2::UP.y * stats.get(&Stats::JumpingSpeed).unwrap().0 as f32
            }
            State::Dodging {} => {
                velocity.x *= stats.get(&Stats::DodgingSpeed).unwrap().0 as f32;
            }
            State::Moving {} => {
                velocity.x *= stats.get(&Stats::RunningSpeed).unwrap().0 as f32;
            }
            _ => (),
        }

        if velocity.x != 0.0 {
            (Event::Wasd, velocity)
        } else {
            (Event::None, velocity)
        }
    }

    pub fn get_vel(input: &Gd<Input>) -> Vector2 {
        let mut velocity = Vector2::ZERO;
        if input.is_action_pressed("east") {
            velocity += Vector2::RIGHT;
        }
        if input.is_action_pressed("west") {
            velocity += Vector2::LEFT;
        }
        velocity
    }
}
