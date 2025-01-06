use godot::{
    classes::{
        AnimatedSprite2D, CharacterBody2D, CollisionShape2D, ICharacterBody2D, InputEvent, Timer,
    },
    prelude::*,
};

use crate::{
    components::state_machines::{main_character_state::CharacterState, movements::Directions},
    traits::{
        character_resources::CharacterResources, damageable::Damageable, damaging::Damaging,
        player_moveable::PlayerMoveable,
    },
};

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct MainCharacter {
    #[export]
    running_speed: real,
    #[export]
    walking_speed: real,
    #[export]
    attacking_speed: real,
    #[export]
    dodging_speed: real,
    dodging_timer: OnReady<Gd<Timer>>,
    is_dodging: bool,
    velocity: Vector2,
    #[var]
    health: i32,
    #[var]
    energy: i32,
    #[var]
    mana: i32,
    attack_timer: OnReady<Gd<Timer>>,
    is_attacking: bool,
    state: CharacterState,
    direction: Directions,
    base: Base<CharacterBody2D>,
}

#[godot_api]
impl ICharacterBody2D for MainCharacter {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            running_speed: 7000.0,
            walking_speed: 5000.0,
            attacking_speed: 3500.0,
            dodging_speed: 7000.0,
            dodging_timer: OnReady::manual(),
            is_dodging: false,
            velocity: Vector2::ZERO,
            health: 50,
            energy: 50,
            mana: 50,
            is_attacking: false,
            state: CharacterState::Idle,
            direction: Directions::East,
            attack_timer: OnReady::manual(),
            base,
        }
    }

    fn ready(&mut self) {
        let attack_timer = self.base().get_node_as::<Timer>("AttackAnimationTimer");
        self.attack_timer.init(attack_timer);

        let mut dodge_timer = self.base().get_node_as::<Timer>("DodgingTimer");
        let call = self.base().callable("on_dodge_timer_timeout");
        dodge_timer.connect("timeout", &call);
        self.dodging_timer.init(dodge_timer);
    }

    fn physics_process(&mut self, delta: f64) {
        self.move_character(delta as f32);
        let animation = self.get_movement_animation();
        let mut animate = self
            .base_mut()
            .get_node_as::<AnimatedSprite2D>("AnimatedSprite2D");

        animate.play_ex().name(&animation).done();
    }
}

#[godot_api]
impl MainCharacter {
    fn set_direction(&mut self) {
        self.direction = MainCharacter::get_direction(self.base().get_velocity());
    }

    fn run(&mut self, mut vel: Vector2) {
        if vel.length() == 0.0 {
            self.state = CharacterState::Idle;
            return;
        }
        vel *= self.running_speed;
        self.velocity = vel;
        self.base_mut().set_velocity(vel);
        self.set_direction();
        self.base_mut().move_and_slide();
    }

    #[func]
    fn dodge(&mut self, mut vel: Vector2) {
        let mut collision = self
            .base_mut()
            .get_node_as::<CollisionShape2D>("CollisionShape2D");

        self.dodging_timer.start();
        if !self.dodging_timer.is_stopped() && self.state == CharacterState::Dodging {
            vel *= self.dodging_speed;
            collision.set_deferred("disabled", &true.to_variant());

            self.velocity = vel;
            self.set_direction();
            self.base_mut().set_velocity(vel);
            self.base_mut().move_and_slide();
        } else if self.dodging_timer.is_stopped() {
            self.state = CharacterState::Idle;
            collision.set_deferred("disabled", &false.to_variant());
        }
    }

    fn attack(&mut self) {}

    fn walk(&mut self) {}

    fn idle(&mut self) {}

    // #[func]
    // fn on_dodge_timer_timeout(&mut self) {
    //     godot_print!("Dodge timer timeout");
    //     let mut collision = self
    //         .base_mut()
    //         .get_node_as::<CollisionShape2D>("CollisionShape2D");
    //
    //     collision.set_deferred("disabled", &false.to_variant());
    //     self.state = CharacterState::;
    // }
}

impl PlayerMoveable for MainCharacter {
    fn move_character(&mut self, delta: f32) {
        let input = Input::singleton();
        let mut vel = Vector2::new(0.0, 0.0);

        if input.is_action_pressed("east") {
            vel += Vector2::RIGHT;
            if input.is_action_pressed("dodge") {
                self.state = CharacterState::Dodging;
            } else {
                self.state = CharacterState::Running;
            }
        }
        if input.is_action_pressed("west") {
            vel += Vector2::LEFT;
            if input.is_action_pressed("dodge") {
                self.state = CharacterState::Dodging;
            } else {
                self.state = CharacterState::Running;
            }
        }
        if input.is_action_pressed("north") {
            vel += Vector2::UP;
            if input.is_action_pressed("dodge") {
                self.state = CharacterState::Dodging;
            } else {
                self.state = CharacterState::Running;
            }
        }
        if input.is_action_pressed("south") {
            vel += Vector2::DOWN;
            if input.is_action_pressed("dodge") {
                self.state = CharacterState::Dodging;
            } else {
                self.state = CharacterState::Running;
            }
        }
        if vel.length() != 0.0 {
            vel = vel.normalized() * delta;
        } else if vel.length() == 0.0 {
            self.state = CharacterState::Idle;
        }

        match self.state {
            CharacterState::Idle => {
                self.idle();
            }
            CharacterState::Running => {
                self.run(vel);
            }
            CharacterState::Dodging => {
                self.dodge(vel);
            }
            CharacterState::Walking => {
                self.walk();
            }
            CharacterState::Attacking => {
                self.attack();
            }
        }
    }

    fn get_movement_animation(&mut self) -> String {
        let dir = self.direction.to_string();
        let mut state = self.state.to_string();

        state.push('_');
        format!("{}{}", state, dir)
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
