use godot::{
    builtin::Vector2,
    classes::{Engine, ProjectSettings},
    prelude::GodotClass,
};

use crate::entities::enemies::enemy_state_machine::State;

pub struct Speeds {
    patrol: f32,
    aggro: f32,
}

impl Speeds {
    pub fn new(patrol: f32, aggro: f32) -> Self {
        Self { patrol, aggro }
    }
}

/// Used for setting the maximum distance an enemy can move in its patrol state.
#[derive(GodotClass, Default)]
#[class(no_init)]
pub struct PatrolComp {
    #[export]
    c: Vector2,
    /// The furthest distance the entity should move to the left in its patrol state.
    /// Note that only the x-axis is considered.
    pub left_target: Vector2,

    /// The furthest distance the entity should move to the right in its patrol state.
    /// Note that only the x-axis is considered.
    pub right_target: Vector2,
}

impl PatrolComp {
    pub fn get_furthest_distance_x_axis(&self, current_pos: Vector2) -> Vector2 {
        let left_dist = (self.left_target.x - current_pos.x).abs();
        let right_dist = (self.right_target.x - current_pos.x).abs();

        if left_dist > right_dist {
            Vector2::LEFT
        } else {
            Vector2::RIGHT
        }
    }
}

pub struct Movement {
    speeds: Speeds,
    pub velocity: Vector2,
}

impl Movement {
    pub fn new(speeds: Speeds, velocity: Vector2) -> Self {
        Self { speeds, velocity }
    }

    pub fn apply_gravity(&mut self, delta: f32) {
        const GRAVITY: f32 = 1500.0;
        self.velocity.y = Vector2::DOWN.y * GRAVITY * delta;
    }

    pub fn stop(&mut self) {
        self.velocity = Vector2::ZERO;
    }

    pub fn move_to(&mut self, cur_position: Vector2, target: Vector2) {
        self.velocity.x = cur_position.direction_to(target).x;
    }

    pub fn update_patrol_target(&mut self, frame: &FrameData) {
        self.velocity = frame.patrol.get_furthest_distance_x_axis(frame.cur) * self.speeds.patrol;
    }

    pub fn update(&mut self, frame: &FrameData) {
        match frame.state {
            State::ChasePlayer {} => {
                if let Some(pos) = frame.player {
                    self.velocity.x = frame.cur.direction_to(pos).x * self.speeds.aggro;
                }
            }
            State::Falling {} => self.apply_gravity(frame.delta),
            State::Idle {} => self.velocity = Vector2::ZERO,
            State::Attack {} | State::Attack2 {} => self.velocity = Vector2::ZERO,
            _ => (),
        }
    }
}

pub struct FrameData<'a> {
    state: &'a State,
    on_floor: bool,
    cur: Vector2,
    player: Option<Vector2>,
    patrol: &'a PatrolComp,
    delta: f32,
}

impl<'a> FrameData<'a> {
    pub fn new(
        state: &'a State,
        on_floor: bool,
        cur: Vector2,
        player: Option<Vector2>,
        patrol: &'a PatrolComp,
        delta: f32,
    ) -> Self {
        Self {
            state,
            on_floor,
            cur,
            player,
            patrol,
            delta,
        }
    }
}
