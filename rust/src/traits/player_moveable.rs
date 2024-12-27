use godot::{builtin::Vector2, classes::Input};

pub trait PlayerMoveable {
    fn move_character(&mut self, velocity: Vector2);
}
