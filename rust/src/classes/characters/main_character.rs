use godot::{
    classes::{
        AnimationPlayer, Area2D, CharacterBody2D, CollisionObject2D, ICharacterBody2D, Input,
        RayCast2D, Timer,
    },
    obj::WithBaseField,
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
};

use super::{character_stats::CharacterStats, player_timers_component::TimerComponent};

type Event = crate::components::state_machines::character_state_machine::Event;

#[derive(GodotClass)]
#[class(init, base=CharacterBody2D)]
pub struct MainCharacter {
    direction: PlatformerDirection,
    velocity: Vector2,
    active_velocity: Vector2,
    can_attack_chain: bool,
    stats: CharacterStats,
    timer_component: TimerComponent,
    state: statig::blocking::StateMachine<CharacterStateMachine>,
    base: Base<CharacterBody2D>,

    #[var]
    #[init(node = "DodgingCooldownTimer")]
    dodging_cooldown_timer: OnReady<Gd<Timer>>,
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
        self.connect_hitbox();

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

        let healing_animation_length = self
            .get_animation_player()
            .get_animation("heal_east")
            .unwrap()
            .get_length();

        let parry_animation_length = self
            .get_animation_player()
            .get_animation("parry_east")
            .unwrap()
            .get_length();

        self.timer_component = TimerComponent::new(
            0.6,
            dodge_animation_length.into(),
            jumping_animation_length.into(),
            attack_animation_length.into(),
            attack_animation_length.into(),
            healing_animation_length.into(),
            parry_animation_length.into(),
            self.stats.parry_length,
            self.stats.perfect_parry_length,
        );
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
        if input.is_action_pressed("heal") {
            self.state.handle(&Event::HealingButton);
        }
        if input.is_action_pressed("parry") {
            self.state.handle(&Event::ParryButton);
        }
    }

    fn physics_process(&mut self, _delta: f64) {
        let input = Input::singleton();
        let event = InputHandler::to_platformer_event(&Input::singleton());
        self.velocity = InputHandler::get_velocity(&input);

        match self.state.state() {
            character_state_machine::State::Idle {} => self.idle(),
            character_state_machine::State::Dodging {} => self.dodge(),
            character_state_machine::State::Jumping {} => self.jump(),
            character_state_machine::State::Falling {} => self.fall(),
            character_state_machine::State::Moving {} => self.move_character(),
            character_state_machine::State::Attacking {} => self.attack(),
            character_state_machine::State::Attack2 {} => self.attack_2(),
            character_state_machine::State::Parry {} => self.parry(),
            character_state_machine::State::Healing {} => self.heal(),
            character_state_machine::State::Grappling {} => self.grapple(),
        }
        self.state.handle(&event);
    }
}

#[godot_api]
impl MainCharacter {
    #[signal]
    pub fn player_health_changed(previous_health: i32, new_health: i32, damage_amount: i32);

    #[signal]
    fn parried_attack();

    #[signal]
    fn player_died();

    fn connect_hitbox(&self) {
        let mut this = self.to_gd();
        let mut hitbox = self.base().get_node_as::<Area2D>("Hitbox");
        hitbox
            .signals()
            .area_entered()
            .connect(move |area| this.bind_mut().on_area_entered_hitbox(area));
    }

    fn on_area_entered_hitbox(&mut self, area: Gd<Area2D>) {
        if !self.parried_attack() {
            let damaging = DynGd::<Area2D, dyn Damaging>::from_godot(area);
            let target = self.to_gd().upcast::<Node2D>();
            let _guard = self.base_mut();
            let damageable = DynGd::<Node2D, dyn Damageable>::from_godot(target);
            damaging.dyn_bind().do_damage(damageable);
        }
    }

    fn detect_ledges(&mut self) {
        if self.get_ledge_sensor().is_colliding() {
            self.base_mut().set_velocity(Vector2::ZERO);
            self.state.handle(&Event::GrabbedLedge);
            if let Some(obj) = self.get_ledge_sensor().get_collider() {
                let collider = obj.cast::<CollisionObject2D>();
            }
        }
    }

    // TODO: Ledge grappling is buggy.
    fn grapple(&mut self) {
        let input = Input::singleton();
        self.base_mut().set_velocity(Vector2::ZERO);
        self.update_animation();

        if input.is_action_just_pressed("west") & self.get_ledge_sensor().is_colliding()
            || input.is_action_just_pressed("east") & self.get_ledge_sensor().is_colliding()
        {
            self.state.handle(&Event::WasdJustPressed);
        }
    }

    fn dodge(&mut self) {
        let delta = self.base().get_physics_process_delta_time();
        let mut cooldown_timer = self.get_dodging_cooldown_timer();
        let time = self.timer_component.dodging_animation_timer.value;

        if cooldown_timer.get_time_left() > 0.0 {
            self.state.handle(&Event::TimerInProgress);
        } else if time < self.timer_component.dodging_animation_timer.initial_value() && time > 0.0
        {
            self.base_mut().move_and_slide();
            self.timer_component.dodging_animation_timer.value -= delta;

            if !self.base().is_on_floor() {
                self.state.handle(&Event::FailedFloorCheck);
            }
        } else {
            let speed = self.stats.dodging_speed;
            let velocity = self.velocity;

            self.base_mut().set_velocity(velocity * speed);
            self.base_mut().move_and_slide();
            self.update_animation();
            self.timer_component.dodging_animation_timer.value -= delta;

            if !self.base().is_on_floor() {
                self.state.handle(&Event::FailedFloorCheck);
            }
            if time <= 0.0 {
                self.timer_component.dodging_animation_timer.reset();
                self.state.handle(&Event::TimerElapsed);
                cooldown_timer.start();
            }
        }
    }

    fn attack(&mut self) {
        self.base_mut().set_process_unhandled_input(false);
        let speed = self.stats.attacking_speed;
        let time = self.timer_component.attack_animation_timer.value;
        let velocity = self.velocity;
        let delta = self.base().get_physics_process_delta_time();

        if time < self.timer_component.attack_animation_timer.initial_value() && time > 0.0 {
            if Input::singleton().is_action_just_pressed("attack") {
                self.can_attack_chain = true;
            }
            self.base_mut().move_and_slide();
            self.timer_component.attack_animation_timer.value -= delta;
        } else {
            self.base_mut().set_velocity(velocity * speed);
            self.base_mut().move_and_slide();
            self.update_animation();
            self.timer_component.attack_animation_timer.value -= delta;
            if !self.base().is_on_floor() {
                self.state.handle(&Event::FailedFloorCheck);
            }
        }

        if time <= 0.0 {
            self.timer_component.attack_animation_timer.reset();
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
        let time = self.timer_component.attack_animation_timer_2.value;
        let delta = self.base().get_physics_process_delta_time();

        if time
            < self
                .timer_component
                .attack_animation_timer_2
                .initial_value()
            && time > 0.0
        {
            self.timer_component.attack_animation_timer_2.value -= delta;
        } else {
            self.update_animation();
            self.timer_component.attack_animation_timer_2.value -= delta;

            if !self.base().is_on_floor() {
                self.state.handle(&Event::FailedFloorCheck);
            }

            if time <= 0.0 {
                self.timer_component.attack_animation_timer_2.reset();
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
        let time = self.timer_component.jumping_animation_timer.value;
        let delta = self.base().get_physics_process_delta_time();
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
        self.timer_component.jumping_animation_timer.value -= delta;

        if time <= 0.0 {
            self.timer_component.jumping_animation_timer.reset();
            self.state.handle(&Event::TimerElapsed);
        }
    }

    fn heal(&mut self) {
        let time = self.timer_component.healing_animation_timer.value;
        let current_health = self.stats.health;
        let delta = self.base().get_physics_process_delta_time();
        self.velocity = Vector2::ZERO;
        let velocity = self.velocity;

        self.update_direction();
        self.update_animation();
        self.base_mut().set_velocity(velocity);
        self.timer_component.healing_animation_timer.value -= delta;

        if time <= 0.0 {
            self.stats.heal();
            let new_health = self.stats.health;
            let amount = self.stats.healing_amount;
            self.signals()
                .player_health_changed()
                .emit(current_health, new_health, amount);
            self.timer_component.healing_animation_timer.reset();
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
            if self.timer_component.jumping_animation_timer.value
                < self.timer_component.jumping_animation_timer.initial_value()
            {
                self.timer_component.jumping_animation_timer.reset();
            }
        }
    }

    fn parry(&mut self) {
        let time = self.timer_component.parry_animation_timer.value;
        let delta = self.base().get_physics_process_delta_time();
        self.update_animation();
        self.timer_component.parry_animation_timer.value -= delta;
        self.timer_component.parry_timer.value -= delta;
        self.timer_component.perfect_parry_timer.value -= delta;

        if time <= 0.0 {
            self.timer_component.parry_animation_timer.reset();
            self.timer_component.parry_timer.reset();
            self.timer_component.perfect_parry_timer.reset();
            self.state.handle(&Event::TimerElapsed);
        }
    }

    fn parried_attack(&mut self) -> bool {
        if let State::Parry {} = self.state.state() {
            if self.timer_component.perfect_parry_timer.value > 0.0 {
                println!("\nPERFECT PARRY\n");
                self.signals().parried_attack().emit();
                self.timer_component.perfect_parry_timer.reset();
                true
            } else if self.timer_component.parry_timer.value > 0.0 {
                println!("\nNORMAL PARRY\n");
                self.signals().parried_attack().emit();
                self.timer_component.parry_timer.reset();
                true
            } else {
                false
            }
        } else {
            false
        }
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
        if !self.velocity.x.is_zero_approx() {
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
        self.signals()
            .player_health_changed()
            .emit(previous_health, current_health, amount);

        if self.is_dead() {
            println!("You died");
            self.signals().player_died().emit();
            self.base_mut().queue_free();
        }
    }
}

#[godot_dyn]
impl Damaging for MainCharacter {
    fn damage_amount(&self) -> i32 {
        self.stats.attack_damage
    }
}
