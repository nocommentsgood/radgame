use godot::{
    classes::{Area2D, IArea2D},
    meta::ToGodot,
    obj::{Base, Gd, OnReady, WithBaseField},
    prelude::{GodotClass, godot_api, godot_dyn},
};

use crate::entities::damage::{Damaging, HasHealth};

#[derive(GodotClass)]
#[class(init, base = Area2D)]
pub struct EntityHitbox {
    pub parent: Option<Box<dyn HasHealth>>,
    base: Base<Area2D>,
}

#[godot_api]
impl EntityHitbox {
    #[signal]
    fn dummy();
}

impl super::damage::Damageable for Gd<EntityHitbox> {
    fn destroy(&mut self) {
        todo!()
    }
}

#[derive(GodotClass)]
#[class(init, base=Area2D)]
pub struct Hurtbox {
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
