use godot::{
    builtin::{math::FloatExt, real, Vector2},
    classes::CharacterBody2D,
    obj::WithBaseField,
};

use crate::components::state_machines::movements::PlatformerDirection;

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

    // fn update_direction(velocity: &Vector2) -> Option<PlatformerDirection> {
    //     if !velocity.x.is_zero_approx() {
    //         Some(PlatformerDirection::from_platformer_velocity(velocity))
    //     } else {
    //         None
    //     }
    // }
}
