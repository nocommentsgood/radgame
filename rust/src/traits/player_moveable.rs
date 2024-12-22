use godot::classes::Input;

pub trait PlayerMoveable {
    fn move_character(direction: &str, input: Input);
}
