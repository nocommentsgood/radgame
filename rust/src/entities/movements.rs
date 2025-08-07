use godot::{
    builtin::Vector2,
    classes::{CharacterBody2D, Node2D},
    obj::{Inherits, WithBaseField},
};

use super::enemies::animatable::Animatable;

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

pub trait MoveableCharacter: Animatable
where
    Self: WithBaseField<Base = CharacterBody2D>,
{
    fn slide(&mut self, velocity: &Vector2, speed: &f32) {
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
        // self.update_animation();
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
    pub attack: f32,
    pub patrol: f32,
    pub aggro: f32,
}

impl SpeedComponent {
    pub fn new(attack: u32, patrol: u32, aggro: u32) -> Self {
        Self {
            attack: attack as f32,
            patrol: patrol as f32,
            aggro: aggro as f32,
        }
    }
}
