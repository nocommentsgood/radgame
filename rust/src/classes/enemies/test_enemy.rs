use godot::{
    classes::{AnimationPlayer, CharacterBody2D, ICharacterBody2D, Timer},
    prelude::*,
};

use crate::traits::components::character_components::{
    character_resources::CharacterResources, damageable::Damageable, damaging::Damaging,
};

#[derive(GodotClass)]
#[class(init, base=CharacterBody2D)]
pub struct TestEnemy {
    #[var]
    speed: real,
    #[init(val = 30)]
    #[var]
    health: i32,
    #[var]
    energy: i32,
    #[var]
    mana: i32,
    velocity: Vector2,

    #[init(node = "AnimationPlayer2")]
    animation_player: OnReady<Gd<AnimationPlayer>>,

    #[init(node = "MovementTimer")]
    movement_timer: OnReady<Gd<Timer>>,
    running_speed: real,
    base: Base<CharacterBody2D>,
}

#[godot_api]
impl ICharacterBody2D for TestEnemy {
    fn process(&mut self, delta: f64) {
        if self.get_health() <= 0 {
            self.base_mut().queue_free();
        }
    }
}

#[godot_api]
impl TestEnemy {
    #[func]
    fn destroy(&mut self) {
        let h = self.get_health();
        self.set_health(h - 10);
        if self.get_health() <= 0 {
            self.base_mut().queue_free();
        }
    }
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
