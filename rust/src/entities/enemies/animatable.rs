use godot::{classes::AnimationPlayer, obj::Gd};

use crate::entities::movements::Direction;

use super::has_state::HasState;

pub trait Animatable: HasState {
    fn anim_player_mut(&mut self) -> &mut Gd<AnimationPlayer>;

    // TODO: Although this fn is relevant when dealing with animations, maybe it would be better
    // implemented in a different trait, which could be used as a supertrait.
    fn get_direction(&self) -> &Direction;
    fn update_direction(&mut self);

    fn update_animation(&mut self) {
        self.update_direction();
        let anim = format!("{}_{}", self.sm().state(), self.get_direction());
        self.anim_player_mut().play_ex().name(&anim).done();
    }
}
