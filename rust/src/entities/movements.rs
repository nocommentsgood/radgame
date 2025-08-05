use godot::{
    builtin::{Vector2, real},
    classes::{CharacterBody2D, Node2D},
    obj::{Gd, Inherits, WithBaseField},
};

use super::{enemies::animatable::Animatable, player::main_character::MainCharacter};

#[derive(Default, Debug, Clone, PartialEq)]
pub enum Direction {
    #[default]
    East,
    West,
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::East => write!(f, "east"),
            Direction::West => write!(f, "west"),
        }
    }
}
impl Direction {
    pub fn from_vel(velocity: &Vector2) -> Direction {
        if velocity.x < 0.0 {
            Direction::West
        } else {
            Direction::East
        }
    }
}

pub trait PlayerMoveable {
    fn get_movement_animation(&mut self) -> String;

    fn move_main_character(mut main: Gd<MainCharacter>, velocity: Vector2) {
        main.set_velocity(velocity);
        // main.bind_mut().set_velocity(velocity);
        main.move_and_slide();
    }
}

pub trait MoveableCharacter: Animatable
where
    Self: WithBaseField<Base = CharacterBody2D>,
{
    fn slide(&mut self, velocity: &Vector2, speed: &real) {
        self.update_animation();
        self.base_mut().set_velocity(*velocity * *speed);
        self.base_mut().move_and_slide();
    }
}

pub trait MoveableEntity: Animatable
where
    Self: Inherits<Node2D> + WithBaseField<Base: Inherits<Node2D>>,
{
    fn move_to(&mut self, target: &Vector2, use_physics_delta: bool) {
        self.update_animation();
        let delta = if use_physics_delta {
            self.base().upcast_ref().get_physics_process_delta_time()
        } else {
            self.base().upcast_ref().get_process_delta_time()
        };
        let pos = self.base().upcast_ref().get_global_position();

        self.base_mut()
            .upcast_mut()
            .set_global_position(pos + *target * delta as f32);
    }
}

#[derive(Default, Debug, Clone)]
pub struct SpeedComponent {
    pub attack: real,
    pub patrol: real,
    pub aggro: real,
}

impl SpeedComponent {
    pub fn new(attack: real, patrol: real, aggro: real) -> Self {
        Self {
            attack,
            patrol,
            aggro,
        }
    }
}
