use godot::{
    classes::{CharacterBody2D, ICharacterBody2D},
    prelude::*,
};

use crate::{
    classes::characters::main_character::MainCharacter,
    components::state_machines::{main_character_state::CharacterState, movements::Directions},
    traits::{
        character_resources::CharacterResources, damageable::Damageable, damaging::Damaging,
        player_moveable::PlayerMoveable,
    },
};

#[derive(GodotClass)]
#[class(init, base=CharacterBody2D)]
pub struct TestEnemy {
    #[var]
    speed: real,
    #[var]
    health: i32,
    #[var]
    energy: i32,
    #[var]
    mana: i32,

    velocity: Vector2,
    running_speed: real,
    direction: Directions,
    state: CharacterState,
    base: Base<CharacterBody2D>,
}

#[godot_api]
impl ICharacterBody2D for TestEnemy {
    fn physics_process(&mut self, delta: f64) {
        let idle_vel = Vector2::ZERO;
        let left_vel = Vector2::LEFT;
        let right_vel = Vector2::RIGHT;
        let mut tree = self.base().get_tree().unwrap();

        self.run(left_vel);
        tree.create_timer_ex(3.0).process_in_physics(true).done();
        self.run(right_vel);
        tree.create_timer_ex(3.0).process_in_physics(true).done();
    }
}

#[godot_api]
impl TestEnemy {
    fn get_movement_animation(&mut self) -> String {
        let dir = self.direction.to_string();
        let mut state = self.state.to_string();

        state.push('_');
        format!("{}{}", state, dir)
    }

    fn set_direction(&mut self) {
        self.direction = MainCharacter::get_direction(self.base().get_velocity());
    }

    fn run(&mut self, vel: Vector2) {
        if vel.length() == 0.0 {
            self.velocity = vel;
            self.state = CharacterState::Idle;
            return;
        }

        if vel.x != 0.0 {
            self.velocity = vel;
            self.base_mut().set_velocity(vel);
            self.set_direction();
            self.base_mut().move_and_slide();
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
