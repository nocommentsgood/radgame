use godot::{
    classes::{AnimationPlayer, Area2D, CharacterBody2D, ICharacterBody2D, RayCast2D, Timer},
    prelude::*,
};

use crate::{
    components::{
        managers::input_hanlder::InputHandler,
        state_machines::{
            character_state_machine::{self, *},
            movements::PlatformerDirection,
        },
    },
    traits::components::character_components::{
        character_resources::CharacterResources, damageable::Damageable, damaging::Damaging,
    },
    utils::constants::{self, PLAYER_HURTBOX},
};

use super::character_stats::CharacterStats;

type Event = crate::components::state_machines::character_state_machine::Event;

#[derive(GodotClass)]
#[class(init, base=CharacterBody2D)]
pub struct MainCharacter {
    delta: f64,
    direction: PlatformerDirection,
    velocity: Vector2,
    active_velocity: Vector2,
    can_attack_chain: bool,
    stats: CharacterStats,
    state: statig::blocking::StateMachine<CharacterStateMachine>,
    base: Base<CharacterBody2D>,

    #[var]
    #[init(val = OnReady::manual())]
    attack_chain_timer: OnReady<f64>,

    #[var]
    #[init(node = "DodgingCooldownTimer")]
    dodging_cooldown_timer: OnReady<Gd<Timer>>,

    #[var]
    #[init(val = OnReady::manual())]
    dodging_animation_timer: OnReady<f64>,

    #[var]
    #[init(val = OnReady::manual())]
    dodging_animation_length: OnReady<f64>,

    #[var]
    #[init(val = OnReady::manual())]
    jumping_animation_timer: OnReady<f64>,

    #[var]
    #[init(val = OnReady::manual())]
    jumping_animation_length: OnReady<f64>,

    #[var]
    #[init(val = OnReady::manual())]
    attack_animation_timer: OnReady<f64>,

    #[var]
    #[init(val = OnReady::manual())]
    attack_animation_timer_2: OnReady<f64>,

    #[var]
    #[init(val = OnReady::manual())]
    attack_animation_length: OnReady<f64>,

    #[var]
    #[init(node = "AnimationPlayer")]
    animation_player: OnReady<Gd<AnimationPlayer>>,

    #[var]
    #[init(node = "LedgeSensor")]
    ledge_sensor: OnReady<Gd<RayCast2D>>,
}

#[godot_api]
impl ICharacterBody2D for MainCharacter {
    fn ready(&mut self) {
        self.attack_chain_timer.init(0.6);

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
            .get_animation("attack_1_east")
            .unwrap()
            .get_length();

        let jumping_animation_length = self
            .get_animation_player()
            .get_animation("jumping_east")
            .unwrap()
            .get_length();

        self.jumping_animation_timer
            .init(jumping_animation_length as f64);

        self.jumping_animation_length
            .init(jumping_animation_length as f64);

        self.attack_animation_length
            .init(attack_animation_length as f64);

        self.attack_animation_timer_2
            .init(attack_animation_length as f64);

        self.attack_animation_timer
            .init(attack_animation_length as f64);

        self.dodging_animation_length
            .init(dodge_animation_length as f64);

        self.dodging_animation_timer
            .init(dodge_animation_length as f64);
    }

    fn unhandled_input(&mut self, input: Gd<godot::classes::InputEvent>) {
        if input.is_action_pressed("attack") {
            self.state.handle(&Event::AttackButton);
        }
        if input.is_action_pressed("jump") {
            self.state.handle(&Event::JumpButton);
        }
        if input.is_action_released("jump") {
            self.state.handle(&Event::ActionReleasedEarly);
        }
        if input.is_action_pressed("dodge") {
            self.state.handle(&Event::DodgeButton);
        }
    }

    fn physics_process(&mut self, delta: f64) {
        let input = Input::singleton();
        let event = InputHandler::to_platformer_event(&Input::singleton());
        self.velocity = InputHandler::get_velocity(&input);
        self.delta = delta;

        match self.state.state() {
            character_state_machine::State::Idle {} => self.idle(),
            character_state_machine::State::Dodging {} => self.dodge(),
            character_state_machine::State::Jumping {} => self.jump(),
            character_state_machine::State::Falling {} => self.fall(),
            character_state_machine::State::Moving {} => self.move_character(),
            character_state_machine::State::Attacking {} => self.attack(),
            &character_state_machine::State::Attack2 {} => self.attack_2(),
            character_state_machine::State::Grappling {} => self.grapple(),
        }

        self.state.handle(&event);
    }
}

#[godot_api]
impl MainCharacter {
    #[signal]
    fn player_damaged(previous_health: i32, new_health: i32, damage_amount: i32);

    #[func]
    fn on_body_entered_hurtbox(&mut self, body: Gd<Node2D>) {
        let mut damagable = DynGd::<Node2D, dyn Damageable>::from_godot(body);
        damagable
            .dyn_bind_mut()
            .take_damage(self.stats.attack_damage);
    }

    fn detect_ledges(&mut self) {
        if let Some(_collider) = self.get_ledge_sensor().get_collider() {
            self.state.handle(&Event::GrabbedLedge);
        }
    }

    fn grapple(&mut self) {
        let input = Input::singleton();
        self.base_mut().set_velocity(Vector2::ZERO);
        self.update_animation();

        if input.is_action_just_pressed("west")
            || input.is_action_just_pressed("east")
            || !self.get_ledge_sensor().is_colliding()
        {
            self.state.handle(&Event::WasdJustPressed);
        }
    }

    fn dodge(&mut self) {
        let mut cooldown_timer = self.get_dodging_cooldown_timer();
        let time = self.get_dodging_animation_timer();

        if cooldown_timer.get_time_left() > 0.0 {
            self.state.handle(&Event::TimerInProgress);
        } else if time < self.get_dodging_animation_length() && time > 0.0 {
            self.base_mut().move_and_slide();
            self.set_dodging_animation_timer(time - self.delta);
        } else {
            let speed = self.stats.dodging_speed;
            let velocity = self.velocity;

            self.base_mut().set_velocity(velocity * speed);
            self.base_mut().move_and_slide();
            self.update_animation();
            self.set_dodging_animation_timer(time - self.delta);

            if !self.base().is_on_floor() {
                self.state.handle(&Event::FailedFloorCheck);
            }
            if time <= 0.0 {
                self.reset_dodging_animation_timer();
                self.state.handle(&Event::TimerElapsed);
                cooldown_timer.start();
            }
        }
    }

    fn attack(&mut self) {
        self.base_mut().set_process_unhandled_input(false);
        let speed = self.stats.attacking_speed;
        let time = self.get_attack_animation_timer();
        let velocity = self.velocity;

        if time < self.get_attack_animation_length() && time > 0.0 {
            if Input::singleton().is_action_just_pressed("attack") {
                self.can_attack_chain = true;
            }
            self.base_mut().move_and_slide();
            self.set_attack_animation_timer(time - self.delta);
        } else {
            self.base_mut().set_velocity(velocity * speed);
            self.base_mut().move_and_slide();
            self.update_animation();
            self.set_attack_animation_timer(time - self.delta);
            if !self.base().is_on_floor() {
                self.state.handle(&Event::FailedFloorCheck);
            }
        }

        if time <= 0.0 {
            self.reset_attacking_animation_timer();
            self.base_mut().set_process_unhandled_input(true);
            if self.can_attack_chain {
                self.can_attack_chain = false;
                self.state.handle(&Event::AttackButton);
            } else {
                self.state.handle(&Event::TimerElapsed);
            }
        }
    }

    fn attack_2(&mut self) {
        let time = self.get_attack_animation_timer_2();

        if time < self.get_attack_animation_length() && time > 0.0 {
            self.set_attack_animation_timer_2(time - self.delta);
        } else {
            self.update_animation();
            self.set_attack_animation_timer_2(time - self.delta);

            if !self.base().is_on_floor() {
                self.state.handle(&Event::FailedFloorCheck);
            }

            if time <= 0.0 {
                self.set_attack_animation_timer_2(self.get_attack_animation_length());
                self.state.handle(&Event::TimerElapsed);
            }
        }
    }

    fn idle(&mut self) {
        self.active_velocity = Vector2::ZERO;
        self.update_animation();
        if !self.base().is_on_floor() {
            self.state.handle(&Event::FailedFloorCheck);
        }
    }

    fn move_character(&mut self) {
        let target_velocity = self.velocity * self.stats.running_speed;
        self.active_velocity = self.active_velocity.lerp(target_velocity, 0.2);
        let velocity = self.active_velocity;

        self.update_direction();
        self.update_animation();
        self.base_mut().set_velocity(velocity);
        self.base_mut().move_and_slide();

        if !self.base().is_on_floor() {
            self.state.handle(&Event::FailedFloorCheck);
        }
    }

    fn jump(&mut self) {
        let time = self.get_jumping_animation_timer();
        self.velocity.y = Vector2::UP.y;
        let target_velocity = Vector2::new(
            self.velocity.x * self.stats.running_speed,
            self.velocity.y * self.stats.jumping_speed,
        );
        self.active_velocity = self.active_velocity.lerp(target_velocity, 0.2);
        let velocity = self.active_velocity;

        self.update_direction();
        self.update_animation();
        self.detect_ledges();
        self.base_mut().set_velocity(velocity);
        self.base_mut().move_and_slide();
        self.set_jumping_animation_timer(time - self.delta);

        if time <= 0.0 {
            self.reset_jumping_animation_timer();
            self.state.handle(&Event::TimerElapsed);
        }
    }

    fn fall(&mut self) {
        if !self.base().is_on_floor() {
            self.velocity.y = Vector2::DOWN.y;
            let target_velocity = Vector2::new(
                self.velocity.x * self.stats.running_speed,
                self.velocity.y * self.stats.falling_speed,
            );

            self.active_velocity = self.active_velocity.lerp(target_velocity, 0.1);
            let velocity = self.active_velocity;

            self.update_direction();
            self.update_animation();
            self.detect_ledges();
            self.base_mut().set_velocity(velocity);
            self.base_mut().move_and_slide();
        } else if self.base().is_on_floor() {
            self.state.handle(&Event::OnFloor);
            if self.get_jumping_animation_timer() < self.get_jumping_animation_length() {
                self.reset_jumping_animation_timer();
            }
        }
    }

    fn connect_attack_signal(&mut self) {
        let mut hurtbox = self.base().get_node_as::<Area2D>(PLAYER_HURTBOX);
        let callable = self
            .base()
            .callable(constants::CALLABLE_ON_PLAYER_HURTBOX_ENTERED);

        hurtbox.connect(constants::SIGNAL_PLAYER_HURTBOX_ENTERED, &callable);
    }

    fn reset_jumping_animation_timer(&mut self) {
        self.set_jumping_animation_timer(self.get_jumping_animation_length());
    }

    fn reset_attacking_animation_timer(&mut self) {
        self.set_attack_animation_timer(self.get_attack_animation_length());
    }

    fn reset_dodging_animation_timer(&mut self) {
        self.set_dodging_animation_timer(self.get_dodging_animation_length());
    }

    fn get_current_animation(&self) -> String {
        let mut animation = self.state.state().to_string();
        animation.push('_');
        animation.push_str(self.direction.to_string().as_str());

        animation
    }

    fn update_animation(&mut self) {
        let animation = self.get_current_animation();
        self.animation_player.play_ex().name(&animation).done();
        self.animation_player.advance(0.0);
    }

    fn update_direction(&mut self) {
        if !self.velocity.is_zero_approx() {
            self.direction = PlatformerDirection::from_platformer_velocity(&self.velocity)
        }
    }
}

#[godot_dyn]
impl CharacterResources for MainCharacter {
    fn get_health(&self) -> i32 {
        self.stats.health
    }

    fn set_health(&mut self, amount: i32) {
        self.stats.health = amount;
    }

    fn get_energy(&self) -> i32 {
        self.stats.energy
    }

    fn set_energy(&mut self, amount: i32) {
        self.stats.energy = amount;
    }

    fn get_mana(&self) -> i32 {
        self.stats.mana
    }

    fn set_mana(&mut self, amount: i32) {
        self.stats.mana = amount;
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
