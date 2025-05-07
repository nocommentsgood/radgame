use godot::{classes::AnimationPlayer, obj::Gd};

use crate::components::state_machines::movements::PlatformerDirection;

use super::has_state::HasState;

pub trait Animatable: HasState {
    fn get_anim_player(&self) -> Gd<AnimationPlayer>;

    // TODO: Although this fn is relevant when dealing with animations, maybe it would be better
    // implemented in a different trait, which could be used as a supertrait.
    fn get_direction(&self) -> PlatformerDirection;
    fn update_direction(&mut self);

    fn current_animation(&self) -> String {
        let direction = self.get_direction();
        let mut state = self.sm().state().to_string();
        state.push('_');
        format!("{}{}", state, direction)
    }

    fn update_animation(&mut self) {
        let mut anim_player = self.get_anim_player();
        let anim = self.current_animation();
        anim_player.play_ex().name(&anim).done();
    }
}
