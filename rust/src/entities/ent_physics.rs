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
    pub fn handle_acceleration(&mut self, state: &State, prev_frame: PhysicsFrameData) {
        match state {
            State::WallGrabLeft {} | State::WallGrabRight {} => {
                self.velocity.y = 50.0;
            }

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
                if prev_frame.on_wall_only {
                    self.apply_gravity(false, &prev_frame.delta);
                    self.velocity.x = self.speeds.running * Vector2::RIGHT.x;
                } else {
                    self.velocity.y = self.speeds.jumping * Vector2::UP.y;
                    self.velocity.x = 0.0;
                }
            }
            State::JumpingLeft {} => {
                if prev_frame.on_wall_only {
                    self.apply_gravity(false, &prev_frame.delta);
                    self.velocity.x = self.speeds.running * Vector2::LEFT.x;
                } else {
                    self.velocity.y = self.speeds.jumping * Vector2::UP.y;
                    self.velocity.x = 0.0;
                }
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
        if ent.upcast_ref().is_on_floor_only() && is_airborne(state, previous_state) {
            self.velocity.y = 0.0;
            self.early_gravity = 0.0;
            true
        } else {
            false
        }
    }
}

pub struct PhysicsFrameData {
    state: State,
    velocity: Vector2,
    on_floor: bool,
    on_floor_only: bool,
    on_wall: bool,
    on_wall_only: bool,
    on_ceiling: bool,
    on_ceiling_only: bool,
    delta: f32,
}

#[allow(clippy::too_many_arguments)]
impl PhysicsFrameData {
    pub fn new(
        state: State,
        velocity: Vector2,
        on_floor: bool,
        on_floor_only: bool,
        on_wall: bool,
        on_wall_only: bool,
        on_ceiling: bool,
        on_ceiling_only: bool,
        delta: f32,
    ) -> Self {
        Self {
            state,
            velocity,
            on_floor,
            on_floor_only,
            on_wall,
            on_wall_only,
            on_ceiling,
            on_ceiling_only,
            delta,
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn velocity(&self) -> &Vector2 {
        &self.velocity
    }

    pub fn on_floor(&self) -> bool {
        self.on_floor
    }

    pub fn on_floor_only(&self) -> bool {
        self.on_floor_only
    }

    pub fn on_wall(&self) -> bool {
        self.on_wall
    }

    pub fn on_wall_only(&self) -> bool {
        self.on_wall_only
    }

    pub fn on_ceiling(&self) -> bool {
        self.on_ceiling
    }

    pub fn on_ceiling_only(&self) -> bool {
        self.on_ceiling_only
    }

    pub fn delta(&self) -> f32 {
        self.delta
    }
}
