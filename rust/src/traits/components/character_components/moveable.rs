use godot::{
    builtin::{real, Vector2},
    classes::{CharacterBody2D, Node2D},
    obj::{Inherits, WithBaseField},
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

pub trait MoveableEntity
where
    Self: Inherits<Node2D> + WithBaseField<Base: Inherits<Node2D>>,
{
    fn move_to(&mut self, target: Vector2) {
        let delta = self.base().upcast_ref().get_process_delta_time();
        let pos = self.base().upcast_ref().get_global_position();

        self.base_mut()
            .upcast_mut()
            .set_global_position(pos + target * delta as f32);
    }
}
