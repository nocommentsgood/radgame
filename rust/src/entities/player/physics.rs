use godot::{
    builtin::Vector2,
    classes::{CharacterBody2D, KinematicCollision2D},
    obj::{Gd, Inherits},
};

use crate::{
    entities::{movements::Direction, player::character_state_machine::State},
    utils::input_hanlder::{self, Inputs},
};

/// Ceiling collision handling and response.
pub fn hit_ceiling(ent: &mut Gd<impl Inherits<CharacterBody2D>>, movement: &mut Movement) -> bool {
    let ceiling = ent.upcast_ref().is_on_ceiling_only();
    let collisions = ent.upcast_mut().get_last_slide_collision();
    if let Some(c) = collisions
        && ceiling
    {
        movement.bounce_off_ceiling(&c);
        true
    } else {
        false
    }
}

/// Whether the entity is or was previously in an airborne state.
fn is_airborne(state: StateInfo) -> bool {
    (matches!(state.current, State::Falling {} | State::AirDash {})
        || matches!(state.previous, State::Jumping {}))
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

    pub fn stop_y(&mut self) {
        self.velocity.y = 0.0;
    }

    pub fn air_dash_right(&mut self) {
        self.velocity.y = 0.0;
        self.dodge_right();
    }
    pub fn air_dash_left(&mut self) {
        self.velocity.y = 0.0;
        self.dodge_left();
    }

    pub fn wall_grab_velocity(&mut self) {
        self.stop_x();
        self.velocity.y = 30.0;
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
        if self.velocity.x == 0.0 {
            cur
        } else {
            let new = Direction::from_vel(self.velocity);
            self.direction = new;
            new
        }
    }

    pub fn velocity(&self) -> Vector2 {
        self.velocity
    }

    pub fn bounce_off_ceiling(&mut self, collision: &Gd<KinematicCollision2D>) {
        self.velocity = self
            .velocity
            .bounce(collision.get_normal().normalized_or_zero());
    }

    pub fn apply_gravity(&mut self, state: StateInfo, delta: f32) {
        const GRAVITY: f32 = 900.0;
        const TERMINAL_VELOCITY: f32 = 1300.0;

        match state.current {
            State::Jumping {} | State::Falling {} if self.velocity.y < TERMINAL_VELOCITY => {
                self.velocity.y += GRAVITY * delta
            }
            State::Jumping {} => self.early_gravity += delta,
            _ => (),
        }

        // if frame.state == (State::Jumping {}) {
        //     self.early_gravity += frame.delta;
        // }
        // if matches!(frame.state, State::Jumping {} | State::Falling {})
        //     && self.velocity.y < TERMINAL_VELOCITY
        // {
        //     self.velocity.y += GRAVITY * frame.delta;
        // }
    }

    // Jump was released early, apply more gravity.
    pub fn apply_early_gravity(&mut self, time: f32) {
        if time > 0.5 {
            self.velocity.y = 300.0;
        } else if self.early_gravity < 0.5 && self.early_gravity >= 0.08 {
            self.velocity.y = 350.0;
        } else {
            self.velocity.y = 450.0;
        }
    }

    /// Checks if the entity was airborne in the previous physics frame and if the entity has since
    /// landed on the floor.
    pub fn landed(&mut self, floor: FloorState, state: StateInfo) -> bool {
        match floor {
            FloorState::OnlyOnFloor | FloorState::Both if is_airborne(state) => {
                self.velocity.y = 0.0;
                self.early_gravity = 0.0;
                true
            }
            _ => false,
        }
    }

    pub fn wall_grab(
        state: StateInfo,
        wall: WallState,
        input: &Inputs,
        wallcast: Option<WallCastCollision>,
    ) -> bool {
        match wall {
            WallState::OnWallOnly if !matches!(state.current, State::WallGrab {}) => {
                match input.0 {
                    Some(input_hanlder::MoveButton::Left) => {
                        wallcast.is_some_and(|v| WallCastCollision::Left == v)
                    }
                    Some(input_hanlder::MoveButton::Right) => {
                        wallcast.is_some_and(|v| WallCastCollision::Right == v)
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn not_on_floor(&self, floor: FloorState) -> bool {
        matches!(floor, FloorState::NotOnFloor if self.velocity.y.is_sign_positive())
        // !frame.on_floor && self.velocity.y.is_sign_positive()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WallState {
    OnWallOnly,
    OnWall,
    Both,
}

impl WallState {
    pub fn from_something(on_wall: bool, on_wall_only: bool) -> Option<Self> {
        match (on_wall, on_wall_only) {
            (true, true) => Some(Self::Both),
            (true, false) => Some(Self::OnWall),
            (false, true) => Some(Self::OnWallOnly),
            (false, false) => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FloorState {
    OnFloor,
    OnlyOnFloor,
    Both,
    NotOnFloor,
}

impl FloorState {
    pub fn from_something(on_floor: bool, on_floor_only: bool) -> Self {
        match (on_floor, on_floor_only) {
            (true, true) => Self::Both,
            (true, false) => Self::OnFloor,
            (false, true) => Self::OnlyOnFloor,
            (false, false) => Self::NotOnFloor,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WallCastCollision {
    Left,
    Right,
    Both,
}

impl WallCastCollision {
    pub fn from_something(left_cast: bool, right_cast: bool) -> Option<Self> {
        match (left_cast, right_cast) {
            (true, true) => Some(Self::Both),
            (true, false) => Some(Self::Left),
            (false, true) => Some(Self::Right),
            (false, false) => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StateInfo {
    previous: State,
    current: State,
}

impl StateInfo {
    pub fn new(previous: State, current: State) -> Self {
        Self { previous, current }
    }
}
