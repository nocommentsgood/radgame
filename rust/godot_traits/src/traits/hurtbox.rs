use godot::{
    classes::Area2D,
    obj::Base,
    prelude::{GodotClass, godot_dyn},
};

#[derive(GodotClass)]
#[class(init, base=Area2D)]
pub struct Hurtbox {
    #[export]
    attack_damage: u32,
    base: Base<Area2D>,
}

// TODO: Remove 'godot_dyn' and experiement with Rust's dyn traits. For example using Box<dyn
// Damaging>.
#[godot_dyn]
impl super::damaging::Damaging for Hurtbox {
    fn damage_amount(&self) -> u32 {
        self.attack_damage
    }
}
