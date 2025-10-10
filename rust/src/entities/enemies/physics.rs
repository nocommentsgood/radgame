use godot::{
    builtin::Vector2,
    classes::{CharacterBody2D, Node2D},
    obj::Gd,
};

use crate::entities::enemies::enemy_state_machine::State;

#[derive(Clone, Copy)]
pub struct Speeds {
    patrol: f32,
    aggro: f32,
}

impl Speeds {
    pub fn new(patrol: f32, aggro: f32) -> Self {
        Self { patrol, aggro }
    }
}

#[derive(Clone, Copy)]
pub struct Movement {
    current_position: Vector2,
    speeds: Speeds,
    velocity: Vector2,
    left_target: Vector2,
    right_target: Vector2,
}

impl Movement {
    pub fn new(
        current_position: Vector2,
        speeds: Speeds,
        left_target: Vector2,
        right_target: Vector2,
    ) -> Self {
        Self {
            current_position,
            speeds,
            velocity: Vector2::ZERO,
            left_target,
            right_target,
        }
    }

    fn apply_gravity(&mut self, delta: f32) {
        const GRAVITY: f32 = 1500.0;
        self.velocity.y += GRAVITY * delta;
    }

    pub fn velocity(&self) -> Vector2 {
        self.velocity
    }

    pub fn patrol(&mut self) {
        let left_dist = (self.left_target.x - self.current_position.x).abs();
        let right_dist = (self.right_target.x - self.current_position.x).abs();

        if left_dist > right_dist {
            self.velocity = Vector2::LEFT;
        } else {
            self.velocity = Vector2::RIGHT;
        }
    }

    pub fn update(
        &mut self,
        strategy: &mut MovementStrategy,
        state: &State,
        player_pos: Option<Vector2>,
        delta: f32,
    ) {
        match state {
            State::RecoverLeft {} => self.velocity.x = Vector2::RIGHT.x,
            State::RecoverRight {} => self.velocity.x = Vector2::LEFT.x,
            State::Patrol {} | State::Falling {} => (),
            State::ChasePlayer {} => {
                if let Some(player_pos) = player_pos
                    && (self.current_position.x - player_pos.x).abs() >= 30.0
                {
                    self.velocity.x = self.current_position.direction_to(player_pos).x;
                } else {
                    self.velocity = Vector2::ZERO;
                }
            }
            _ => self.velocity = Vector2::ZERO,
        }
        self.accelerate(state, delta);
        self.apply_movement(strategy, delta);
    }

    fn accelerate(&mut self, state: &State, delta: f32) {
        self.velocity = self.velocity.normalized_or_zero();
        match state {
            State::Patrol {} | State::RecoverLeft {} | State::RecoverRight {} => {
                self.velocity.x *= self.speeds.patrol;
            }
            State::Falling {} => {
                self.apply_gravity(delta);
            }
            State::ChasePlayer {} => self.velocity *= self.speeds.aggro,
            _ => (),
        }
    }

    pub fn apply_movement(&mut self, strategy: &mut MovementStrategy, delta: f32) {
        match strategy {
            MovementStrategy::MoveAndSlide(gd) => {
                self.current_position = gd.get_global_position();
                gd.set_velocity(self.velocity);
                gd.move_and_slide();
            }
            MovementStrategy::ManualSetPosition(node) => {
                self.current_position = node.get_global_position();
                node.set_global_position(self.current_position + self.velocity * delta);
            }
        }
    }
}

/// `MoveAndSlide`: Internally calls `CharacterBody2D::move_and_slide()`
///
/// `ManualSetPosition`: Moves the node manually i.e. `current_position + velocity * delta`
pub enum MovementStrategy {
    MoveAndSlide(Gd<CharacterBody2D>),
    ManualSetPosition(Gd<Node2D>),
}
