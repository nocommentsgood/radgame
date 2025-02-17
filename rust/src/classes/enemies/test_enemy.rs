use godot::{
    classes::{AnimationPlayer, CharacterBody2D, ICharacterBody2D, Timer},
    prelude::*,
};

use crate::traits::components::character_components::{
    character_resources::CharacterResources, damageable::Damageable, damaging::Damaging,
};

use crate::utils::*;

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
    #[signal]
    fn test_enemy_died();

    #[func]
    fn destroy(&mut self) {
        if self.is_dead() {
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
impl Damageable for TestEnemy {
    fn take_damage(&mut self, amount: i32) {
        let mut current_health = self.get_health();
        current_health = current_health.saturating_sub(amount);
        self.set_health(current_health);

        if self.is_dead() {
            self.base_mut()
                .emit_signal(constants::SIGNAL_TESTENEMY_DIED, &[]);
        }
    }
}

#[godot_dyn]
impl Damaging for TestEnemy {}
