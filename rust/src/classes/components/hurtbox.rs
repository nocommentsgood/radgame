use godot::prelude::*;
use godot::{
    classes::{Area2D, IArea2D},
    obj::Base,
    prelude::{godot_api, godot_dyn, GodotClass},
};

use crate::traits::components::character_components::damaging::Damaging;

#[derive(GodotClass)]
#[class(init, base=Area2D)]
pub struct Hurtbox {
    // TODO: Remove export.
    #[export]
    pub attack_damage: u32,
    base: Base<Area2D>,
}

#[godot_api]
impl IArea2D for Hurtbox {
    fn ready(&mut self) {
        self.base_mut().set_deferred("disabled", &true.to_variant());
    }
}

// TODO: I thought requiring a #[signal] to access `WithUserSignals` was removed... Check this.
#[godot_api]
impl Hurtbox {
    #[signal]
    fn test();
}

#[godot_dyn]
impl Damaging for Hurtbox {
    fn damage_amount(&self) -> u32 {
        self.attack_damage
    }
}
