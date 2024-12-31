use godot::{builtin::Vector2, classes::Input};

use crate::classes::characters::direction::Direction;

pub trait PlayerMoveable {
    fn move_character(&mut self);
    fn get_movement_animation(&mut self) -> String;
}
