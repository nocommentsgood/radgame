use godot::{
    builtin::Vector2,
    classes::{CharacterBody2D, KinematicCollision2D},
    obj::{Gd, Inherits},
};

use crate::{
    entities::{movements::Direction, player::character_state_machine::State},
    utils::input_hanlder::Inputs,
};

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
fn is_airborne(frame: &PhysicsFrame) -> bool {
    (matches!(frame.state, State::Falling {} | State::MoveFalling {})
        || matches!(
            frame.previous_state,
            State::Jumping {}
                | State::MoveJumping {}
                | State::AirAttack {}
                | State::MovingAirAttack {}
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
    direction: Direction,
    pub speeds: Speeds,
}

impl Movement {
    pub fn new_handle(&mut self, state: &State, dir: Direction) {
        match (state, &dir) {
            (State::Idle {}, _) => {
                self.velocity = Vector2::ZERO;
            }

            (State::Run {}, Direction::Right) => {
                self.velocity.x = self.speeds.running * Vector2::RIGHT.x;
                dbg!(&self.velocity);
            }
            (State::Run {}, Direction::Left) => {
                self.velocity.x = self.speeds.running * Vector2::LEFT.x;
                dbg!(&self.velocity);
            }

            _ => (),
        }
    }

    /// Applies accelerated movement depending on current state.
    pub fn handle_acceleration(&mut self, state: &State) {
        match state {
            State::WallGrab {} => {
                self.velocity.y = 50.0;
            }

            State::MoveFalling {} => {
                let dir = self.get_direction();
                match dir {
                    Direction::Right => self.velocity.x = self.speeds.running * Vector2::RIGHT.x,
                    Direction::Left => self.velocity.x = self.speeds.running * Vector2::LEFT.x,
                }
            }

            State::MovingAirAttack {} => {
                let dir = self.get_direction();
                match dir {
                    Direction::Right => {
                        self.velocity.x = self.speeds.running * Vector2::RIGHT.x;
                    }
                    Direction::Left => {
                        self.velocity.x = self.speeds.running * Vector2::LEFT.x;
                    }
                }
            }
            State::Dodging {} => {
                let dir = self.get_direction();
                match dir {
                    Direction::Right => {
                        self.velocity.x = self.speeds.dodging * Vector2::RIGHT.x;
                    }

                    Direction::Left => {
                        self.velocity.x = self.speeds.dodging * Vector2::LEFT.x;
                    }
                }
            }
            State::Run {} => (),
            State::Jumping {} => {
                let dir = self.get_direction();
                match dir {
                    Direction::Right => {
                        self.velocity.y = self.speeds.jumping * Vector2::UP.y;
                        self.velocity.x = 0.0;
                    }
                    Direction::Left => {
                        self.velocity.y = self.speeds.jumping * Vector2::UP.y;
                        self.velocity.x = 0.0;
                    }
                }
            }
            State::MoveJumping {} => {
                let dir = self.get_direction();
                match dir {
                    Direction::Right => {
                        self.velocity.x = self.speeds.running * Vector2::RIGHT.x;
                        self.velocity.y = self.speeds.jumping * Vector2::UP.y;
                    }
                    Direction::Left => {
                        self.velocity.x = self.speeds.running * Vector2::LEFT.x;
                        self.velocity.y = self.speeds.jumping * Vector2::UP.y;
                    }
                }
            }
            State::AirDash {} => {
                let dir = self.get_direction();
                match dir {
                    Direction::Right => {
                        self.velocity.x = self.speeds.dodging * 3.0 * Vector2::RIGHT.x;
                    }
                    Direction::Left => {
                        self.velocity.x = self.speeds.dodging * 3.0 * Vector2::LEFT.x;
                    }
                }
            }
            _ => self.velocity.x = 0.0,
        }
    }

    pub fn get_direction(&mut self) -> Direction {
        let cur = self.direction;
        if self.velocity.x != 0.0 {
            let new = Direction::from_vel(&self.velocity);
            self.direction = new;
            new
        } else {
            cur
        }
    }

    pub fn bounce_off_ceiling(&mut self, collision: Gd<KinematicCollision2D>) {
        self.velocity = self
            .velocity
            .bounce(collision.get_normal().normalized_or_zero())
    }

    pub fn apply_gravity(&mut self, frame: PhysicsFrame) {
        const GRAVITY: f32 = 1500.0;
        const TERMINAL_VELOCITY: f32 = 500.0;

        if !frame.on_floor_only && (frame.state != State::AirDash {}) {
            self.early_gravity += frame.delta;

            if self.velocity.y < TERMINAL_VELOCITY {
                if self.early_gravity >= 0.8 {
                    self.velocity.y += GRAVITY * frame.delta;
                } else if self.early_gravity < 0.8 && self.early_gravity >= 0.4 {
                    self.velocity.y += 1700.0 * frame.delta;
                } else {
                    self.velocity.y += 2000.0 * frame.delta;
                }
            }
        }
    }

    /// Checks if the entity was airborne in the previous physics frame and if the entity has since
    /// landed on the floor.
    pub fn landed(&mut self, frame: &PhysicsFrame) -> bool {
        if frame.on_floor_only && is_airborne(frame) {
            self.velocity.y = 0.0;
            self.early_gravity = 0.0;
            true
        } else {
            false
        }
    }

    pub fn wall_grab(frame: &PhysicsFrame, input: &Inputs) -> bool {
        if frame.on_wall_only && !matches!(frame.state, State::WallGrab {}) {
            match input.0 {
                Some(crate::utils::input_hanlder::MoveButton::Left) => frame.left_wall_colliding,
                Some(crate::utils::input_hanlder::MoveButton::Right) => frame.right_wall_colliding,
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn not_on_floor(&self, frame: &PhysicsFrame) -> bool {
        !frame.on_floor && self.velocity.y.is_sign_positive()
    }
}

pub struct PhysicsFrame {
    state: State,
    previous_state: State,
    on_floor: bool,
    on_floor_only: bool,
    on_wall_only: bool,
    left_wall_colliding: bool,
    right_wall_colliding: bool,
    delta: f32,
}

impl PhysicsFrame {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        state: State,
        previous_state: State,
        on_floor: bool,
        on_floor_only: bool,
        on_wall_only: bool,
        left_wall_colliding: bool,
        right_wall_colliding: bool,
        delta: f32,
    ) -> Self {
        Self {
            state,
            previous_state,
            on_floor,
            on_floor_only,
            on_wall_only,
            left_wall_colliding,
            right_wall_colliding,
            delta,
        }
    }
}
