use godot::{
    classes::Area2D,
    obj::Base,
    prelude::{godot_dyn, GodotClass},
};

use crate::traits::components::character_components::damaging::Damaging;

#[derive(GodotClass)]
#[class(init, base=Area2D)]
pub struct Hurtbox {
    #[export]
    attack_damage: i32,
    base: Base<Area2D>,
}

#[godot_dyn]
impl Damaging for Hurtbox {
    fn damage_amount(&self) -> i32 {
        self.attack_damage
    }
}
