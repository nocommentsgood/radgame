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
    (matches!(frame.state, State::Falling {} | State::AirDash {})
        || matches!(
            frame.previous_state,
            State::Jumping {} // | State::AirAttack {} | State::MovingAirAttack {}
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
    velocity: Vector2,
    early_gravity: f32,
    direction: Direction,
    pub speeds: Speeds,
}

impl Movement {
    pub fn run_right(&mut self) {
        self.velocity.x = self.speeds.running * Vector2::RIGHT.x;
    }
    pub fn run_left(&mut self) {
        self.velocity.x = self.speeds.running * Vector2::LEFT.x;
    }
    pub fn dodge_right(&mut self) {
        self.velocity.x = self.speeds.dodging * Vector2::RIGHT.x;
    }
    pub fn dodge_left(&mut self) {
        self.velocity.x = self.speeds.dodging * Vector2::LEFT.x;
    }
    pub fn stop_x(&mut self) {
        self.velocity.x = 0.0;
    }

    pub fn wall_grab_velocity(&mut self) {
        self.stop_x();
        self.velocity.y = 0.0;
    }

    pub fn jump(&mut self) {
        self.velocity.x = 0.0;
        self.velocity.y = Vector2::UP.y * self.speeds.jumping;
    }

    pub fn jump_left(&mut self) {
        self.velocity.x = self.speeds.running * Vector2::LEFT.x;
        self.velocity.y = self.speeds.jumping * Vector2::UP.y;
    }

    pub fn jump_right(&mut self) {
        self.velocity.x = self.speeds.running * Vector2::RIGHT.x;
        self.velocity.y = self.speeds.jumping * Vector2::UP.y;
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

    pub fn velocity(&self) -> Vector2 {
        self.velocity
    }

    pub fn bounce_off_ceiling(&mut self, collision: Gd<KinematicCollision2D>) {
        self.velocity = self
            .velocity
            .bounce(collision.get_normal().normalized_or_zero())
    }

    pub fn apply_gravity(&mut self, frame: PhysicsFrame) {
        const GRAVITY: f32 = 1500.0;
        const TERMINAL_VELOCITY: f32 = 500.0;

        if !frame.on_floor_only
            && (frame.state != State::WallGrab {} || frame.state != State::AirDash {})
        {
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
