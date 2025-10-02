use godot::builtin::Vector2;

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
    pub fn new(speeds: Speeds, left_target: Vector2, right_target: Vector2) -> Self {
        Self {
            current_position: Vector2::ZERO,
            speeds,
            velocity: Vector2::ZERO,
            left_target,
            right_target,
        }
    }

    pub fn set_patrol_targets(&mut self, left: Vector2, right: Vector2) {
        self.left_target = left;
        self.right_target = right;
    }

    fn apply_gravity(&mut self, delta: f32) {
        const GRAVITY: f32 = 1500.0;
        self.velocity.y = Vector2::DOWN.y * GRAVITY * delta;
    }

    pub fn velocity(&self) -> Vector2 {
        self.velocity
    }

    pub fn patrol(&mut self) {
        let left_dist = (self.left_target.x - self.current_position.x).abs();
        let right_dist = (self.right_target.x - self.current_position.x).abs();

        if left_dist > right_dist {
            self.velocity = Vector2::LEFT * self.speeds.patrol;
        } else {
            self.velocity = Vector2::RIGHT * self.speeds.patrol;
        }
    }

    /// Applies movement dependent on the current state.
    pub fn update(&mut self, frame: &PhysicsFrameData) {
        self.current_position = frame.cur_pos;
        match frame.state {
            State::ChasePlayer {} => {
                if let Some(pos) = frame.player {
                    self.velocity.x = self.current_position.direction_to(pos).x * self.speeds.aggro;
                }
            }
            State::Falling {} => self.apply_gravity(frame.delta),
            State::Idle {} => self.velocity = Vector2::ZERO,
            State::Attack {} | State::Attack2 {} => self.velocity = Vector2::ZERO,
            _ => (),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PhysicsFrameData {
    state: State,
    pub on_floor: bool,
    cur_pos: Vector2,
    player: Option<Vector2>,
    delta: f32,
}

impl PhysicsFrameData {
    pub fn new(
        state: State,
        on_floor: bool,
        cur: Vector2,
        player: Option<Vector2>,
        delta: f32,
    ) -> Self {
        Self {
            state,
            on_floor,
            cur_pos: cur,
            player,
            delta,
        }
    }
}
