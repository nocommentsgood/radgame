use std::default;

use godot::{
    classes::{
        AnimatedSprite2D, CharacterBody2D, ICharacterBody2D, IVisualShaderNodeVectorLen,
        InputEvent, Timer,
    },
    prelude::*,
};

use crate::{
    components::state_machines::main_character_state::CharacterState,
    traits::{
        character_resources::CharacterResources, damageable::Damageable, damaging::Damaging,
        player_moveable::PlayerMoveable,
    },
};

use super::direction::Direction;

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct MainCharacter {
    #[export]
    speed: real,
    #[export]
    attacking_speed: real,
    #[var]
    health: i32,
    #[var]
    energy: i32,
    #[var]
    mana: i32,
    attack_timer: OnReady<Gd<Timer>>,
    is_attacking: bool,
    state: CharacterState,
    direction: Direction,
    base: Base<CharacterBody2D>,
}

#[godot_api]
impl MainCharacter {
    fn get_animation(&self) -> String {
        match self.state {
            CharacterState::Default => "idle".into(),
            CharacterState::Jumping => "jump".into(),
            CharacterState::RunningLeft => "run_left".into(),
            CharacterState::RunningRight => "run_right".into(),
            CharacterState::RunningUp => "idle".into(),
            CharacterState::RunningDown => "idle".into(),
            CharacterState::LightAttackLeft => "attack_left_1".into(),
            CharacterState::LightAttackRight => "attack_right_1".into(),
            CharacterState::CastingSpell => "idle".into(),
        }
    }

    fn attacking(&self) -> bool {
        matches!(
            self.state,
            CharacterState::LightAttackRight | CharacterState::LightAttackLeft
        )
    }
}

#[godot_api]
impl ICharacterBody2D for MainCharacter {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            speed: 5000.0,
            attacking_speed: 3000.0,
            health: 50,
            energy: 50,
            mana: 50,
            is_attacking: false,
            state: CharacterState::Default,
            direction: Direction::None,
            attack_timer: OnReady::manual(),
            base,
        }
    }

    fn ready(&mut self) {
        let timer = self.base().get_node_as::<Timer>("AttackAnimationTimer");
        self.attack_timer.init(timer);
    }

    fn physics_process(&mut self, _delta: f64) {
        self.move_character();
        let animation = self.get_movement_animation();
        let mut animate = self
            .base_mut()
            .get_node_as::<AnimatedSprite2D>("AnimatedSprite2D");

        animate.play_ex().name(&animation).done();
    }
}

impl PlayerMoveable for MainCharacter {
    fn move_character(&mut self) {
        let input = Input::singleton();
        let move_direction = input.get_vector("left", "right", "up", "down");
        let mouse_position = self.base().get_global_mouse_position();
        let velocity = move_direction * self.speed;

        // self.base_mut().look_at(mouse_position);
        self.base_mut().set_velocity(velocity);
        self.base_mut().move_and_slide();
    }

    fn get_movement_animation(&mut self) -> String {
        let direction = self.base().get_velocity();

        if direction.x > 0.0 {
            return "run_right".to_string();
        }
        if direction.x < 0.0 {
            return "run_left".to_string();
        }
        if direction.x > 0.0 && direction.y < 0.0 {
            return "run_up_right".to_string();
        }
        if direction.x < 0.0 && direction.y < 0.0 {
            return "run_up_left".to_string();
        }
        if direction.y < 0.0 {
            return "run_up".to_string();
        }
        if direction.y > 0.0 {
            return "run_down".to_string();
        }
        if direction.x > 0.0 && direction.y > 0.0 {
            return "run_down_right".to_string();
        }
        if direction.x < 0.0 && direction.y > 0.0 {
            "run_down_left".to_string()
        } else {
            "idle".to_string()
        }
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
