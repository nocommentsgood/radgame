use godot::builtin::Vector2;

use crate::components::state_machines::movements::Directions;

pub trait PlayerMoveable {
    fn move_character(&mut self, delta: f32);
    fn get_movement_animation(&mut self) -> String;
    fn get_direction(vel: Vector2) -> Directions {
        if vel.x > 0.0 && vel.y < 0.0 {
            return Directions::NorthEast;
        }
        if vel.x < 0.0 && vel.y < 0.0 {
            return Directions::NorthWest;
        }
        if vel.x > 0.0 && vel.y > 0.0 {
            return Directions::SouthEast;
        }
        if vel.x < 0.0 && vel.y > 0.0 {
            return Directions::SouthWest;
        }
        if vel.x > 0.0 {
            return Directions::East;
        }
        if vel.x < 0.0 {
            return Directions::West;
        }
        if vel.y < 0.0 {
            Directions::North
        } else {
            Directions::South
        }
    }
}
