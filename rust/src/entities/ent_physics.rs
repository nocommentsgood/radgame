use godot::{
    builtin::Vector2,
    classes::{CharacterBody2D, KinematicCollision2D},
    obj::{Gd, Inherits},
};

use crate::entities::player::character_state_machine::State;

/// Ceiling collision handling and response.
pub fn hit_ceiling(ent: &mut Gd<impl Inherits<CharacterBody2D>>, movement: &mut Movement) -> bool {
    let ceiling = ent.upcast_ref().is_on_ceiling_only();
    let collisions = ent.upcast_mut().get_last_slide_collision();
    if let Some(c) = collisions
        && ceiling
    {
        movement.bounce_off_ceiling(c);
        true
    } else {
        false
    }
}

/// Whether the entity is or was previously in an airborne state.
fn is_airborne(state: &State, previous_state: &State) -> bool {
    (matches!(
        state,
        State::FallingRight {}
            | State::MoveFallingLeft {}
            | State::MoveFallingRight {}
            | State::FallingLeft {}
    ) || matches!(
        previous_state,
        State::JumpingLeft {}
            | State::JumpingRight {}
            | State::MoveJumpingRight {}
            | State::MoveJumpingLeft {}
            | State::AirAttackRight {}
            | State::AirAttackLeft {}
            | State::MoveLeftAirAttack {}
            | State::MoveRightAirAttack {}
    ))
}

pub fn not_on_floor(ent: &Gd<impl Inherits<CharacterBody2D>>, state: &State) -> bool {
    if !ent.upcast_ref().is_on_floor() {
        matches!(
            state,
            State::MoveFallingLeft {}
                | State::MoveFallingRight {}
                | State::FallingLeft {}
                | State::FallingRight {}
        )
    } else {
        false
    }
}

#[derive(Default, Clone, Copy)]
pub struct Speeds {
    pub running: f32,
    pub jumping: f32,
    pub dodging: f32,
}

#[derive(Default, Clone, Copy)]
pub struct Movement {
    pub velocity: Vector2,
    early_gravity: f32,
    pub speeds: Speeds,
}

impl Movement {
    /// Applies accelerated movement depending on current state.
    pub fn handle_acceleration(&mut self, state: &State) {
        match state {
            State::MoveFallingLeft {} | State::MoveLeftAirAttack {} => {
                self.velocity.x = self.speeds.running * Vector2::LEFT.x;
            }
            State::MoveFallingRight {} | State::MoveRightAirAttack {} => {
                self.velocity.x = self.speeds.running * Vector2::RIGHT.x;
            }
            State::DodgingLeft {} => {
                self.velocity.x = self.speeds.dodging * Vector2::LEFT.x;
            }
            State::DodgingRight {} => {
                self.velocity.x = self.speeds.dodging * Vector2::RIGHT.x;
            }
            State::MoveLeft {} => {
                self.velocity.x = self.speeds.running * Vector2::LEFT.x;
            }
            State::MoveRight {} => {
                self.velocity.x = self.speeds.running * Vector2::RIGHT.x;
            }
            State::JumpingRight {} => {
                self.velocity.y = self.speeds.jumping * Vector2::UP.y;
                self.velocity.x = 0.0;
            }
            State::JumpingLeft {} => {
                self.velocity.y = self.speeds.jumping * Vector2::UP.y;
                self.velocity.x = 0.0;
            }
            State::MoveJumpingRight {} => {
                self.velocity.x = self.speeds.running * Vector2::RIGHT.x;
                self.velocity.y = self.speeds.jumping * Vector2::UP.y;
            }
            State::MoveJumpingLeft {} => {
                self.velocity.x = self.speeds.running * Vector2::LEFT.x;
                self.velocity.y = self.speeds.jumping * Vector2::UP.y;
            }
            _ => self.velocity.x = 0.0,
        }
    }

    pub fn bounce_off_ceiling(&mut self, collision: Gd<KinematicCollision2D>) {
        self.velocity = self
            .velocity
            .bounce(collision.get_normal().normalized_or_zero())
    }

    pub fn apply_gravity(&mut self, on_floor: bool, delta: &f32) {
        const GRAVITY: f32 = 1500.0;
        const TERMINAL_VELOCITY: f32 = 500.0;

        if !on_floor {
            self.early_gravity += delta;

            if self.velocity.y < TERMINAL_VELOCITY {
                if self.early_gravity >= 0.8 {
                    self.velocity.y += GRAVITY * delta;
                } else if self.early_gravity < 0.8 && self.early_gravity >= 0.4 {
                    self.velocity.y += 1700.0 * delta;
                } else {
                    self.velocity.y += 2000.0 * delta;
                }
            }
        }
    }

    /// Checks if the entity was airborne in the previous physics frame and if the entity has since
    /// landed on the floor.
    pub fn landed(
        &mut self,
        ent: Gd<impl Inherits<CharacterBody2D>>,
        state: &State,
        previous_state: &State,
    ) -> bool {
        if ent.upcast_ref().is_on_floor() && is_airborne(state, previous_state) {
            self.velocity.y = 0.0;
            self.early_gravity = 0.0;
            true
        } else {
            false
        }
    }
}
