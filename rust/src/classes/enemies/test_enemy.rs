use godot::prelude::*;

use crate::traits::{
    character_resources::CharacterResources, damageable::Damageable, damaging::Damaging,
};

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct TestEnemy {
    #[var]
    speed: real,
    #[var]
    health: i32,
    #[var]
    energy: i32,
    #[var]
    mana: i32,
    base: Base<Node2D>,
}

#[godot_dyn]
impl CharacterResources for TestEnemy {
    fn get_health(&self) -> i32 {
        self.health
    }

    fn set_health(&mut self, amount: i32) {
        self.health = amount;
    }

    fn get_energy(&self) -> i32 {
        self.energy
    }

    fn set_energy(&mut self, amount: i32) {
        self.energy = amount;
    }

    fn get_mana(&self) -> i32 {
        self.mana
    }

    fn set_mana(&mut self, amount: i32) {
        self.mana = amount;
    }
}

#[godot_dyn]
impl Damageable for TestEnemy {}

#[godot_dyn]
impl Damaging for TestEnemy {}
