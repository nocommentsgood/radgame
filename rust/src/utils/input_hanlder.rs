use std::collections::HashMap;

use godot::{
    builtin::Vector2,
    classes::{Input, Timer},
    obj::Gd,
};

use crate::entities::{
    entity_stats::{StatVal, Stats},
    player::character_state_machine::State,
};

const GRAVITY: f32 = 980.0;
const TERMINAL_VELOCITY: f32 = 200.0;
type Event = crate::entities::player::character_state_machine::Event;

#[derive(Default, Clone)]
pub struct InputHandler;

impl InputHandler {
    pub fn set_vel_get_event(
        input: &Gd<Input>,
        state: &State,
        stats: &HashMap<Stats, StatVal>,
        velocity: &mut Vector2,
        delta: &f32,
    ) -> Event {
        velocity.x = 0.0;
        if input.is_action_pressed("east") {
            velocity.x = Vector2::RIGHT.x;
        } else if input.is_action_pressed("west") {
            velocity.x = Vector2::LEFT.x;
        } else {
            velocity.x = 0.0;
        }
        match state {
            // State::Falling {} => {
            //     if velocity.y <= TERMINAL_VELOCITY {
            //         velocity.y += GRAVITY * delta;
            //     }
            //     velocity.x *= stats.get(&Stats::RunningSpeed).unwrap().0 as f32;
            // }
            State::Jumping {} => {
                velocity.y = Vector2::UP.y * stats.get(&Stats::JumpingSpeed).unwrap().0 as f32;
                velocity.x *= stats.get(&Stats::RunningSpeed).unwrap().0 as f32;
            }
            State::Moving {} => {
                velocity.x *= stats.get(&Stats::RunningSpeed).unwrap().0 as f32;
            }
            _ => (),
        }

        if velocity.x != 0.0 {
            Event::Wasd
        } else {
            Event::None
        }
    }

    pub fn get_velocity(input: &Gd<Input>) -> Vector2 {
        let mut velocity = Vector2::ZERO;
        if input.is_action_pressed("east") {
            velocity.x = Vector2::RIGHT.x;
        } else if input.is_action_pressed("west") {
            velocity.x = Vector2::LEFT.x;
        }

        velocity
    }

    fn get_horiz_movement(input: Gd<Input>) -> Option<HorizMovement> {
        if input.is_action_pressed("right") {
            Some(HorizMovement::Right)
        } else if input.is_action_pressed("left") {
            Some(HorizMovement::Left)
        } else {
            None
        }
    }
}

pub enum HorizMovement {
    Left,
    Right,
}

pub enum VertMovement {
    Up,
    Down,
}

/// Represents a normalized movement, as well as no movement.
pub struct Movement(pub Option<HorizMovement>, pub Option<VertMovement>);

impl Movement {
    pub fn from_vel(vel: Vector2) -> (Option<HorizMovement>, Option<VertMovement>) {
        let horz = if vel.x > 0.0 {
            Some(HorizMovement::Right)
        } else if vel.x < 0.0 {
            Some(HorizMovement::Left)
        } else {
            None
        };

        let vert = if vel.y > 0.0 {
            Some(VertMovement::Down)
        } else if vel.y < 0.0 {
            Some(VertMovement::Up)
        } else {
            None
        };
        (horz, vert)
    }

    fn test_machine(movement: Movement) {
        // These would be events. The todo!() would be the state.
        match (movement.0, movement.1) {
            (Some(HorizMovement::Left), None) => todo!("run left"),
            (Some(HorizMovement::Right), None) => todo!("run right"),
            (Some(HorizMovement::Left), Some(VertMovement::Up)) => todo!("jumping left"),
            (Some(HorizMovement::Left), Some(VertMovement::Down)) => todo!("falling left"),
            (Some(HorizMovement::Right), Some(VertMovement::Up)) => todo!("jumping right"),
            (Some(HorizMovement::Right), Some(VertMovement::Down)) => todo!("falling right"),
            (None, Some(VertMovement::Up)) => todo!("jumping"),
            (None, Some(VertMovement::Down)) => todo!("falling"),
            (None, None) => todo!("idle"),
        }
    }
}
