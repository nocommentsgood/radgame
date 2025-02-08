use godot::{
    classes::{AnimationPlayer, CharacterBody2D, ICharacterBody2D, Timer},
    obj::WithBaseField,
    prelude::*,
};
use statig::prelude::IntoStateMachineExt;

use crate::{
    components::{
        managers::input_hanlder::InputHandler,
        state_machines::{character_state_machine::CharacterStateMachine, movements::Directions},
    },
    traits::components::character_components::{
        character_resources::CharacterResources, damageable::Damageable, damaging::Damaging,
    },
};

#[derive(GodotClass)]
#[class(init, base=CharacterBody2D)]
pub struct MainCharacter {
    #[export]
    #[init(val = 7000.0)]
    running_speed: real,
    #[export]
    #[init(val = 5000.0)]
    walking_speed: real,
    #[export]
    #[init(val = 3500.0)]
    attacking_speed: real,
    #[export]
    #[init(val = 7000.0)]
    dodging_speed: real,
    #[init(node = "DodgingTimer")]
    #[var]
    dodging_cooldown_timer: OnReady<Gd<Timer>>,
    #[var]
    velocity: Vector2,
    #[var]
    health: i32,
    #[var]
    energy: i32,
    #[var]
    mana: i32,
    #[init(node = "AttackAnimationTimer")]
    attack_timer: OnReady<Gd<Timer>>,
    state: statig::blocking::StateMachine<CharacterStateMachine>,
    base: Base<CharacterBody2D>,
}

#[godot_api]
impl ICharacterBody2D for MainCharacter {
    fn physics_process(&mut self, delta: f64) {
        let input = Input::singleton();
        let event = InputHandler::to_event(&input, &delta);

        let mut state = self.state.clone();
        {
            let mut temp_state = self.state.clone();
            let mut context = self.to_gd();
            let _guard = self.base_mut();
            temp_state.handle_with_context(&event, &mut context);
            state = temp_state;
        }
        self.state = state;
        self.update_animation();
    }
}

#[godot_api]
impl MainCharacter {
    fn get_current_animation(&self) -> String {
        let direction = Directions::from_velocity(&self.get_velocity()).to_string();
        let mut state = self.state.state().to_string();
        state.push('_');

        format!("{}{}", state, direction)
    }

    fn update_animation(&mut self) {
        let mut animation_player = self
            .base()
            .get_node_as::<AnimationPlayer>("AnimationPlayer");

        let animation = self.get_current_animation();

        animation_player.play_ex().name(&animation).done();
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
