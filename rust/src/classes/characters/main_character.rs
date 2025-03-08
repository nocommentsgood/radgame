use std::thread::current;

use godot::{
    classes::{
        AnimationPlayer, Area2D, CharacterBody2D, CollisionShape2D, ICharacterBody2D, Timer,
    },
    obj::WithBaseField,
    prelude::*,
};

use crate::{
    components::{
        managers::input_hanlder::InputHandler,
        state_machines::{
            character_state_machine::CharacterStateMachine,
            movements::{Directions, PlatformerDirection},
        },
    },
    traits::components::character_components::{
        character_resources::CharacterResources, damageable::Damageable, damaging::Damaging,
    },
    utils::constants::{self, PLAYER_HURTBOX},
};

type Event = crate::components::state_machines::character_state_machine::Event;
type State = crate::components::state_machines::character_state_machine::State;

#[derive(GodotClass)]
#[class(init, base=CharacterBody2D)]
pub struct MainCharacter {
    direction: Directions,
    platformer_direction: PlatformerDirection,
    #[export]
    #[init(val = 60.0)]
    running_speed: real,
    #[export]
    #[init(val = 30.0)]
    walking_speed: real,
    #[export]
    #[init(val = 10.0)]
    attacking_speed: real,
    #[export]
    #[init(val = 80.0)]
    dodging_speed: real,
    #[var]
    #[init(node = "DodgingCooldownTimer")]
    dodging_cooldown_timer: OnReady<Gd<Timer>>,
    #[var]
    #[init(val = OnReady::manual())]
    dodging_animation_timer: OnReady<f64>,
    #[var]
    #[init(val = OnReady::manual())]
    jumping_animation_timer: OnReady<f64>,
    #[var]
    velocity: Vector2,
    #[var]
    #[init(val = 40)]
    health: i32,
    #[var]
    energy: i32,
    #[var]
    mana: i32,
    #[var]
    #[init(val = 10)]
    attack_damage: i32,
    #[init(val = OnReady::manual())]
    #[var]
    attack_animation_timer: OnReady<f64>,
    #[var]
    #[init(node = "AnimationPlayer")]
    animation_player: OnReady<Gd<AnimationPlayer>>,
    state: statig::blocking::StateMachine<CharacterStateMachine>,
    base: Base<CharacterBody2D>,
}

#[godot_api]
impl ICharacterBody2D for MainCharacter {
    fn ready(&mut self) {
        self.connect_attack_signal();

        // TODO: Find how to get tracks for specific animations.
        // That way we can dynamically divide by scaling speed.

        // Dodging animations, independent of cardinal direction, are all of the same length.
        // Therefore, it is acceptable to use the length of any dodging animation.
        // East was arbitrarily chosen.
        let dodge_animation_length = self
            .get_animation_player()
            .get_animation("dodge_east")
            .unwrap()
            .get_length()
            / 1.5;

        let attack_animation_length = self
            .get_animation_player()
            .get_animation("attack_east_1")
            .unwrap()
            .get_length()
            / 1.5;

        let jumping_animation_length = self
            .get_animation_player()
            .get_animation("jumping_east")
            .unwrap()
            .get_length();

        self.jumping_animation_timer
            .init(jumping_animation_length as f64);

        self.attack_animation_timer
            .init(attack_animation_length as f64);
        self.dodging_animation_timer
            .init(dodge_animation_length as f64);
    }

    fn physics_process(&mut self, delta: f64) {
        let input = Input::singleton();
        // let event = InputHandler::to_event(&input, &delta);

        let event = InputHandler::platformer_to_event(&input, &delta);
        let mut temp_state = self.state.clone();
        temp_state.handle_with_context(&event, self);
        self.state = temp_state;
        self.update_animation();
    }
}

#[godot_api]
impl MainCharacter {
    #[signal]
    fn player_damaged(previous_health: i32, new_health: i32, damage_amount: i32);

    fn connect_attack_signal(&mut self) {
        let mut hurtbox = self.base().get_node_as::<Area2D>(PLAYER_HURTBOX);
        let callable = self
            .base()
            .callable(constants::CALLABLE_BODY_ENTERED_HURTBOX);

        hurtbox.connect(constants::SIGNAL_HURTBOX_BODY_ENTERED, &callable);
    }

    #[func]
    fn on_body_entered_hurtbox(&mut self, body: Gd<Node2D>) {
        if body.is_in_group("enemy") {
            // I was under the impression that DynGd kept a lookup table at compile time of all
            // classes and their implemented traits. However, when the player's Area2D detects the
            // players hitbox, gdext errors with:
            // ERROR: godot-rust function call failed: MainCharacter::on_body_entered_hurtbox()
            // Reason: [panic]  FromGodot::from_godot() failed: none of the classes derived from `StaticBody2D` have been linked to trait
            // `dyn Damageable` with #[godot_dyn]: Gd { id: 39392905054, class: StaticBody2D }
            // The error goes away when performing a group check like above.
            let mut damagable = DynGd::<Node2D, dyn Damageable>::from_godot(body);
            damagable.dyn_bind_mut().take_damage(10);
        }
    }

    pub fn dodge(&mut self, event: &Event, velocity: Vector2, delta: f64) -> State {
        self.direction = Directions::from_velocity(&velocity);
        self.platformer_direction = PlatformerDirection::from_platformer_velocity(&velocity);
        self.set_velocity(velocity);

        let mut cooldown_timer = self.get_dodging_cooldown_timer();
        if cooldown_timer.get_time_left() > 0.0 {
            State::Moving { velocity, delta }
        } else {
            let speed = self.get_dodging_speed();
            let time = self.get_dodging_animation_timer();
            let mut hitbox = self
                .base()
                .get_node_as::<CollisionShape2D>(constants::PLAYER_HITBOX);

            hitbox.set_disabled(true);
            self.base_mut().set_velocity(velocity.to_owned() * speed);
            self.base_mut().move_and_slide();
            self.set_dodging_animation_timer(time - delta);

            if time <= 0.0 {
                hitbox.set_disabled(false);
                self.reset_dodging_animation_timer();
                cooldown_timer.start();
                match event {
                    Event::None => State::Idle {},
                    Event::Wasd { velocity, delta } => State::Moving {
                        velocity: *velocity,
                        delta: *delta,
                    },
                    _ => State::Handle {},
                }
            } else {
                State::Handle {}
            }
        }
    }

    pub fn attack(&mut self, event: &Event, velocity: Vector2, delta: f64) -> State {
        self.direction = Directions::from_velocity(&velocity);
        let speed = self.get_attacking_speed();
        let time = self.get_attack_animation_timer();

        // When attacking with a velocity of zero, the direction defaults to East. This check
        // maintains the last direction the player was facing.
        if !velocity.length().is_zero_approx() {
            self.platformer_direction = PlatformerDirection::from_platformer_velocity(&velocity);
        }

        self.set_velocity(velocity);
        self.base_mut().set_velocity(velocity * speed);
        self.base_mut().move_and_slide();
        self.set_attack_animation_timer(time - delta);

        if time <= 0.0 {
            self.reset_attacking_animation_timer();
            match event {
                Event::None => State::Idle {},
                Event::Wasd { velocity, delta } => State::Moving {
                    velocity: *velocity,
                    delta: *delta,
                },
                Event::DodgeButton { velocity, delta } => State::Dodging {
                    velocity: *velocity,
                    delta: *delta,
                },
                _ => State::Handle {},
            }
        } else {
            State::Handle {}
        }
    }

    pub fn idle(&mut self, event: &Event) -> State {
        self.set_velocity(Vector2::ZERO);
        match event {
            Event::Wasd { velocity, delta } => State::Moving {
                velocity: *velocity,
                delta: *delta,
            },
            Event::AttackButton { velocity, delta } => State::Attacking {
                velocity: *velocity,
                delta: *delta,
            },
            Event::JumpButton { velocity, delta } => State::Jumping {
                velocity: *velocity,
                delta: *delta,
            },
            Event::DodgeButton { velocity, delta } => {
                if self.get_dodging_cooldown_timer().get_time_left() <= 0.0 {
                    State::Dodging {
                        velocity: *velocity,
                        delta: *delta,
                    }
                } else {
                    State::Handle {}
                }
            }
            _ => State::Handle {},
        }
    }

    pub fn move_character(&mut self, event: &Event, velocity: Vector2, _delta: f64) -> State {
        self.direction = Directions::from_velocity(&velocity);
        self.platformer_direction = PlatformerDirection::from_platformer_velocity(&velocity);
        let speed = self.running_speed;
        self.set_velocity(velocity);
        self.base_mut().set_velocity(velocity * speed);
        self.base_mut().move_and_slide();

        match event {
            Event::Wasd { velocity, delta } => State::Moving {
                velocity: *velocity,
                delta: *delta,
            },
            Event::AttackButton { velocity, delta } => State::Attacking {
                velocity: *velocity,
                delta: *delta,
            },
            Event::JumpButton { velocity, delta } => State::Jumping {
                velocity: *velocity,
                delta: *delta,
            },
            Event::DodgeButton { velocity, delta } => {
                if self.get_dodging_cooldown_timer().get_time_left() <= 0.0 {
                    State::Dodging {
                        velocity: *velocity,
                        delta: *delta,
                    }
                } else {
                    State::Handle {}
                }
            }
            Event::None => State::Idle {},
        }
    }

    pub fn jump(&mut self, event: &Event, velocity: Vector2, delta: f64) -> State {
        let speed = self.running_speed;
        let time = self.get_jumping_animation_timer();
        let mut vel = velocity;

        vel.y = Vector2::UP.y;
        self.platformer_direction = PlatformerDirection::from_platformer_velocity(&vel);
        self.set_velocity(vel);
        self.base_mut().set_velocity(vel * speed);
        self.base_mut().move_and_slide();
        self.set_jumping_animation_timer(time - delta);

        if time <= 0.0 {
            self.reset_jumping_animation_timer();
            if !self.base().is_on_floor() {
                State::Falling { velocity, delta }
            } else {
                match event {
                    Event::Wasd { velocity, delta } => State::Moving {
                        velocity: *velocity,
                        delta: *delta,
                    },
                    Event::DodgeButton { velocity, delta } => State::Dodging {
                        velocity: *velocity,
                        delta: *delta,
                    },
                    Event::AttackButton { velocity, delta } => State::Attacking {
                        velocity: *velocity,
                        delta: *delta,
                    },
                    Event::None => State::Idle {},
                    _ => State::Handle {},
                }
            }
        } else {
            match event {
                Event::Wasd { velocity, delta } => State::Jumping {
                    velocity: *velocity,
                    delta: *delta,
                },
                _ => State::Handle {},
            }
        }
    }

    pub fn fall(&mut self, event: &Event, velocity: Vector2, _delta: f64) -> State {
        if !self.base().is_on_floor() {
            let mut vel = velocity;
            let speed = self.get_running_speed();

            vel.y = Vector2::DOWN.y;
            self.set_velocity(vel);
            self.platformer_direction = PlatformerDirection::from_platformer_velocity(&vel);
            self.base_mut().set_velocity(vel * speed);
            self.base_mut().move_and_slide();

            match event {
                Event::Wasd { velocity, delta } => State::Falling {
                    velocity: *velocity,
                    delta: *delta,
                },
                _ => State::Handle {},
            }
        } else if self.base().is_on_floor() {
            match event {
                Event::Wasd { velocity, delta } => State::Moving {
                    velocity: *velocity,
                    delta: *delta,
                },
                Event::DodgeButton { velocity, delta } => State::Dodging {
                    velocity: *velocity,
                    delta: *delta,
                },
                Event::AttackButton { velocity, delta } => State::Attacking {
                    velocity: *velocity,
                    delta: *delta,
                },
                Event::None => State::Idle {},
                _ => State::Handle {},
            }
        } else {
            State::Handle {}
        }
    }

    fn reset_jumping_animation_timer(&mut self) {
        let jump_animation_time = self
            .get_animation_player()
            .get_animation("jumping_east")
            .unwrap()
            .get_length();
        self.set_jumping_animation_timer(jump_animation_time as f64);
    }

    fn reset_attacking_animation_timer(&mut self) {
        let attack_animation_time = self
            .get_animation_player()
            .get_animation("attack_east_1")
            .unwrap()
            .get_length()
            / 1.5;
        self.set_attack_animation_timer(attack_animation_time as f64);
    }

    fn reset_dodging_animation_timer(&mut self) {
        let dodge_animation_time = self
            .get_animation_player()
            .get_animation("dodge_east")
            .unwrap()
            .get_length()
            / 1.5;
        self.set_dodging_animation_timer(dodge_animation_time as f64);
    }

    fn get_current_animation(&self) -> String {
        let direction = &self.platformer_direction;
        let mut state = self.state.state().to_string();
        state.push('_');

        let s = format!("{}{}", state, direction);

        // TODO: This check is temporary.
        if s == "attack_east" || s == "attack_west" {
            format!("{}{}{}", state, direction, "_1")
        } else {
            s
        }
    }

    fn update_animation(&mut self) {
        let animation = self.get_current_animation();
        self.animation_player.play_ex().name(&animation).done();
        self.animation_player.advance(0.0);
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
impl Damageable for MainCharacter {
    fn take_damage(&mut self, amount: i32) {
        let previous_health = self.get_health();
        let current_health = previous_health.saturating_sub(amount);

        self.set_health(current_health);
        self.base_mut().emit_signal(
            constants::SIGNAL_PLAYER_DAMAGED,
            &[
                previous_health.to_variant(),
                current_health.to_variant(),
                amount.to_variant(),
            ],
        );

        if self.is_dead() {
            println!("You died");
            self.base_mut()
                .emit_signal(constants::SIGNAL_PLAYER_DIED, &[]);
            self.base_mut().queue_free();
        }
    }
}

#[godot_dyn]
impl Damaging for MainCharacter {}
