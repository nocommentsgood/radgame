use godot::{
    classes::{CharacterBody2D, ICharacterBody2D},
    prelude::*,
};

use crate::traits::{
    character_resources::CharacterResources, damageable::Damageable, damaging::Damaging,
    player_moveable::PlayerMoveable,
};

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct MainCharacter {
    #[export]
    speed: real,
    #[var]
    health: i32,
    #[var]
    energy: i32,
    #[var]
    mana: i32,
    base: Base<CharacterBody2D>,
}

#[godot_api]
impl MainCharacter {
    // #[func]
    // fn move_main_character(&mut self) {
    //     self.move_character();
    // }
}

#[godot_api]
impl ICharacterBody2D for MainCharacter {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            speed: 200.0,
            health: 50,
            energy: 50,
            mana: 50,
            base,
        }
    }

    fn physics_process(&mut self, delta: f64) {}
}

impl PlayerMoveable for MainCharacter {
    fn move_character(&mut self, velocity: Vector2) {
        self.base_mut().set_velocity(velocity);
        self.base_mut().move_and_slide();
    }
}

#[godot_dyn]
impl CharacterResources for MainCharacter {
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
impl Damageable for MainCharacter {}

#[godot_dyn]
impl Damaging for MainCharacter {}
