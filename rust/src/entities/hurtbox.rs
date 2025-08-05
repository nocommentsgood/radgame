use godot::prelude::*;
use godot::{
    classes::{Area2D, IArea2D},
    obj::Base,
    prelude::{GodotClass, godot_api, godot_dyn},
};

use super::damage::Damaging;

#[derive(GodotClass)]
#[class(init, base=Area2D)]
pub struct Hurtbox {
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

#[godot_api]
impl Hurtbox {
    #[signal]
    fn dummy_sig();
}

#[godot_dyn]
impl Damaging for Hurtbox {
    fn damage_amount(&self) -> u32 {
        self.attack_damage
    }
}
