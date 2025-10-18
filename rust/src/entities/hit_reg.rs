use godot::{
    classes::{Area2D, IArea2D},
    meta::ToGodot,
    obj::{Base, Gd, WithBaseField},
    prelude::{GodotClass, godot_api},
};

use crate::entities::{
    damage::{Attack, AttackData, Damageable},
    entity::ID,
};

// TODO: Add resistances here.
#[derive(GodotClass)]
#[class(init, base = Area2D)]
pub struct Hitbox {
    /// The Damageable entity which owns the Hitbox.
    pub damageable_parent: Option<Box<dyn Damageable>>,
    base: Base<Area2D>,
}

#[godot_api]
impl Hitbox {
    #[signal]
    fn dummy();
}

#[derive(GodotClass)]
#[class(init, base=Area2D)]
pub struct Hurtbox {
    pub attack: Option<super::damage::Attack>,
    pub data: Option<AttackData>,
    base: Base<Area2D>,
}

#[godot_api]
impl IArea2D for Hurtbox {
    fn ready(&mut self) {
        self.base_mut().set_deferred("disabled", &true.to_variant());
        self.base_mut().add_to_group("Hurtbox");
    }
}

#[godot_api]
impl Hurtbox {
    pub fn set_attack(&mut self, attack: Attack) {
        self.attack.replace(attack);
    }
}

#[derive(Clone, Debug)]
pub struct HitReg {
    pub hitbox: Gd<Hitbox>,
    pub hurtbox: Gd<Hurtbox>,
}

impl HitReg {
    pub fn new(hitbox: Gd<Hitbox>, hurtbox: Gd<Hurtbox>) -> Self {
        Self { hitbox, hurtbox }
    }

    /// Connects the given callbacks.
    /// Expected callbacks:
    /// - hitbox entered
    /// - hitbox exited
    /// - hurtbox entered
    /// - hurtbox exited
    pub fn connect_signals<A, B, C, D>(
        &mut self,
        on_hitbox_entered: A,
        on_hitbox_exited: B,
        on_hurtbox_entered: C,
        on_hurtbox_exited: D,
    ) where
        A: FnMut(Gd<Area2D>) + 'static,
        B: FnMut(Gd<Area2D>) + 'static,
        C: FnMut(Gd<Area2D>) + 'static,
        D: FnMut(Gd<Area2D>) + 'static,
    {
        self.hitbox
            .signals()
            .area_entered()
            .connect(on_hitbox_entered);

        self.hitbox
            .signals()
            .area_exited()
            .connect(on_hitbox_exited);
        self.hurtbox
            .signals()
            .area_entered()
            .connect(on_hurtbox_entered);
        self.hurtbox
            .signals()
            .area_exited()
            .connect(on_hurtbox_exited);
    }
}
