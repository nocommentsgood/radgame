use godot::{
    builtin::{real, Vector2},
    classes::CharacterBody2D,
    obj::WithBaseField,
};

use super::animatable::Animatable;

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
