use godot::builtin::Vector2;

use crate::components::state_machines::movements::Directions;

pub trait PlayerMoveable {
    fn move_character(&mut self, delta: f32);
    fn get_movement_animation(&mut self) -> String;
    fn get_direction(vel: Vector2) -> Directions {
        if vel.x > 0.0 {
            Directions::East
        } else if vel.x < 0.0 {
            Directions::West
        } else if vel.x > 0.0 && vel.y < 0.0 {
            Directions::NorthEast
        } else if vel.x < 0.0 && vel.y < 0.0 {
            Directions::NorthWest
        } else if vel.y < 0.0 {
            Directions::North
        } else if vel.y > 0.0 {
            Directions::South
        } else if vel.x > 0.0 && vel.y > 0.0 {
            Directions::SouthEast
        } else {
            Directions::SouthWest
        }
    }
}
