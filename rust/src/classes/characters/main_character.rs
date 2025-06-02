use std::collections::HashMap;

use godot::{
    classes::{
        AnimationPlayer, Area2D, CharacterBody2D, CollisionObject2D, ICharacterBody2D, Input,
        RayCast2D, Timer,
    },
    obj::WithBaseField,
    prelude::*,
};

use crate::{
    classes::enemies::projectile::Projectile,
    components::{
        managers::{
            input_hanlder::InputHandler, item::StatModifier, item_component::ItemComponent,
        },
        state_machines::{
            character_state_machine::{self, *},
            movements::PlatformerDirection,
        },
    },
    traits::components::character_components::{
        character_resources::CharacterResources, damageable::Damageable, damaging::Damaging,
        player::Player,
    },
};

use super::character_stats::{CharacterStats, StatVal, Stats, Stats::*};
use crate::classes::components::timer_component::PlayerTimers;

type Event = crate::components::state_machines::character_state_machine::Event;
const GRAVITY: f32 = 1100.0;

#[derive(GodotClass)]
#[class(init, base=CharacterBody2D)]
pub struct MainCharacter {
    direction: PlatformerDirection,
    velocity: Vector2,
    active_velocity: Vector2,
    can_attack_chain: bool,
    stats: CharacterStats,
    timers: PlayerTimers,
    state: statig::blocking::StateMachine<CharacterStateMachine>,
    test_stats: HashMap<Stats, StatVal>,
    base: Base<CharacterBody2D>,

    #[init(node = "ItemComponent")]
    pub item_comp: OnReady<Gd<ItemComponent>>,

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

        let this = self.to_gd();
        self.item_comp
            .signals()
            .new_modifier()
            .connect_other(&this, Self::on_new_modifier);

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

        let jumping_animation_length = 0.1;

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

        self.timers = PlayerTimers::new(
            0.6,
            dodge_animation_length,
            jumping_animation_length,
            attack_animation_length,
            attack_animation_length,
            healing_animation_length,
            parry_animation_length,
            self.stats.parry_length,
            self.stats.perfect_parry_length,
        );

        self.test_stats.insert(Stats::Health, StatVal(50.0));
        self.test_stats.insert(Stats::MaxHealth, StatVal(50.0));
        self.test_stats.insert(Stats::HealAmount, StatVal(10.0));
        self.test_stats.insert(Stats::AttackDamage, StatVal(30.0));
        self.test_stats.insert(Stats::RunningSpeed, StatVal(150.0));
        self.test_stats.insert(Stats::JumpingSpeed, StatVal(300.0));
        self.test_stats.insert(Stats::DodgingSpeed, StatVal(250.0));
        self.test_stats.insert(Stats::AttackingSpeed, StatVal(10.0));
        self.test_stats.insert(Stats::ParryLength, StatVal(0.3));
        self.test_stats
            .insert(Stats::PerfectParryLength, StatVal(0.15));
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
        self.velocity.x = InputHandler::get_velocity(&input).x;

        if !self.base().is_on_floor() {
            self.state.handle(&Event::FailedFloorCheck);
        }

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
    pub fn player_health_changed(previous_health: u32, new_health: u32, damage_amount: u32);

    #[signal]
    fn parried_attack();

    #[signal]
    fn player_died();

    fn connect_hitbox(&self) {
        let mut this = self.to_gd();
        let hitbox = self.base().get_node_as::<Area2D>("Hitbox");
        hitbox
            .signals()
            .area_entered()
            .connect(move |area| this.bind_mut().on_area_entered_hitbox(area));
    }

    fn on_area_entered_hitbox(&mut self, area: Gd<Area2D>) {
        if !self.parried_attack(area.clone()) {
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
                let collision = obj.cast::<CollisionObject2D>();
                let shape_id = self.get_ledge_sensor().get_collider_shape();
                let owner = collision.shape_find_owner(shape_id);
                let shape = collision.shape_owner_get_owner(owner);
                let s = shape.unwrap().cast::<godot::classes::CollisionShape2D>();
                dbg!(&s.get_shape().unwrap().get_rect());
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
        let delta = self.base().get_physics_process_delta_time() as f32;
        let mut cooldown_timer = self.get_dodging_cooldown_timer();
        let time = self.timers.dodging_animation_timer.value;

        if cooldown_timer.get_time_left() > 0.0 {
            self.state.handle(&Event::TimerInProgress);
        } else if time < self.timers.dodging_animation_timer.initial_value() && time > 0.0 {
            self.base_mut().move_and_slide();
            self.timers.dodging_animation_timer.value -= delta;
        } else {
            let speed = self.test_stats.get(&DodgingSpeed).unwrap().0;
            let velocity = self.velocity;

            self.base_mut().set_velocity(velocity * speed);
            self.base_mut().move_and_slide();
            self.update_animation();
            self.timers.dodging_animation_timer.value -= delta;

            if time <= 0.0 {
                self.timers.dodging_animation_timer.reset();
                self.state.handle(&Event::TimerElapsed);
                cooldown_timer.start();
            }
        }
    }

    fn attack(&mut self) {
        self.base_mut().set_process_unhandled_input(false);
        let speed = self.stats.attacking_speed;
        let time = self.timers.attack_animation_timer.value;
        let velocity = self.velocity;
        let delta = self.base().get_physics_process_delta_time() as f32;

        if time < self.timers.attack_animation_timer.initial_value() && time > 0.0 {
            if Input::singleton().is_action_just_pressed("attack") {
                self.can_attack_chain = true;
            }
            self.update_direction();
            self.base_mut().move_and_slide();
            self.timers.attack_animation_timer.value -= delta;
        } else {
            self.update_direction();
            self.update_animation();
            self.base_mut().set_velocity(velocity * speed);
            self.base_mut().move_and_slide();
            self.timers.attack_animation_timer.value -= delta;
        }

        if time <= 0.0 {
            self.timers.attack_animation_timer.reset();
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
        let time = self.timers.attack_animation_timer_2.value;
        let delta = self.base().get_physics_process_delta_time() as f32;

        if time < self.timers.attack_animation_timer_2.initial_value() && time > 0.0 {
            self.timers.attack_animation_timer_2.value -= delta;
        } else {
            self.update_animation();
            self.timers.attack_animation_timer_2.value -= delta;

            if time <= 0.0 {
                self.timers.attack_animation_timer_2.reset();
                self.state.handle(&Event::TimerElapsed);
            }
        }
    }

    fn idle(&mut self) {
        self.active_velocity = Vector2::ZERO;
        self.update_animation();
    }

    fn move_character(&mut self) {
        let target_velocity = self.velocity * self.test_stats.get(&RunningSpeed).unwrap().0;
        self.active_velocity = self.active_velocity.lerp(target_velocity, 0.2);
        let velocity = self.active_velocity;

        self.update_direction();
        self.update_animation();
        self.base_mut().set_velocity(velocity);
        self.base_mut().move_and_slide();
    }

    fn jump(&mut self) {
        let time = self.timers.jumping_animation_timer.value;
        let delta = self.base().get_physics_process_delta_time() as f32;

        if self.base().is_on_floor() {
            self.velocity.y = Vector2::UP.y * self.test_stats.get(&JumpingSpeed).unwrap().0;
            self.velocity.x *= self.test_stats.get(&RunningSpeed).unwrap().0;
            let velocity = self.velocity;
            self.base_mut().set_velocity(velocity);
            self.base_mut().move_and_slide();
        } else {
            self.velocity.y += GRAVITY * delta;
            let target_x = self.velocity.x * self.test_stats.get(&RunningSpeed).unwrap().0;
            self.active_velocity.x = self.active_velocity.x.lerp(target_x, 0.2);
            let velocity = Vector2::new(self.active_velocity.x, self.velocity.y);
            self.update_direction();
            self.update_animation();
            self.detect_ledges();
            self.base_mut().set_velocity(velocity);
            self.base_mut().move_and_slide();
            self.timers.jumping_animation_timer.value -= delta;
        }

        if time <= 0.0 {
            self.timers.jumping_animation_timer.reset();
            self.state.handle(&Event::TimerElapsed);
        }
    }

    fn heal(&mut self) {
        let time = self.timers.healing_animation_timer.value;
        let current_health = self.stats.health;
        let delta = self.base().get_physics_process_delta_time() as f32;
        self.velocity = Vector2::ZERO;
        let velocity = self.velocity;

        self.update_animation();
        self.base_mut().set_velocity(velocity);
        self.timers.healing_animation_timer.value -= delta;

        if time <= 0.0 {
            self.stats.heal();
            let new_health = self.stats.health;
            let amount = self.stats.healing_amount;
            self.signals()
                .player_health_changed()
                .emit(current_health, new_health, amount);
            self.timers.healing_animation_timer.reset();
            self.state.handle(&Event::TimerElapsed);
        }
    }

    fn fall(&mut self) {
        if !self.base().is_on_floor() {
            let delta = self.base().get_physics_process_delta_time() as f32;
            self.velocity.y += GRAVITY * delta;
            self.velocity.x *= self.test_stats.get(&RunningSpeed).unwrap().0;

            let velocity = self.velocity;
            self.update_direction();
            self.update_animation();
            self.detect_ledges();
            self.base_mut().set_velocity(velocity);
            self.base_mut().move_and_slide();
        } else if self.base().is_on_floor() {
            self.velocity.y = 0.0;
            self.state.handle(&Event::OnFloor);
            if self.timers.jumping_animation_timer.value
                < self.timers.jumping_animation_timer.initial_value()
            {
                self.timers.jumping_animation_timer.reset();
            }
        }
    }

    fn parry(&mut self) {
        let time = self.timers.parry_animation_timer.value;
        let delta = self.base().get_physics_process_delta_time() as f32;
        self.update_animation();
        self.timers.parry_animation_timer.value -= delta;
        self.timers.parry_timer.value -= delta;
        self.timers.perfect_parry_timer.value -= delta;

        if time <= 0.0 {
            self.timers.parry_animation_timer.reset();
            self.timers.parry_timer.reset();
            self.timers.perfect_parry_timer.reset();
            self.state.handle(&Event::TimerElapsed);
        }
    }

    fn parried_attack(&mut self, area: Gd<Area2D>) -> bool {
        match self.state.state() {
            State::Parry {} => {
                if self.timers.perfect_parry_timer.value > 0.0 {
                    println!("\nPERFECT PARRY\n");
                    if area.is_in_group("enemy_projectile") {
                        if let Some(parent) = area.get_parent() {
                            if let Ok(mut projectile) = parent.try_cast::<Projectile>() {
                                projectile.bind_mut().on_parried();
                            }
                        }
                    }
                    self.signals().parried_attack().emit();
                    self.timers.perfect_parry_timer.reset();
                    true
                } else if self.timers.parry_timer.value > 0.0 {
                    println!("\nNORMAL PARRY\n");
                    if area.is_in_group("enemy_projectile") {
                        if let Some(parent) = area.get_parent() {
                            if let Ok(mut projectile) = parent.try_cast::<Projectile>() {
                                projectile.bind_mut().on_parried();
                            }
                        }
                    }
                    self.signals().parried_attack().emit();
                    self.timers.parry_timer.reset();
                    true
                } else {
                    false
                }
            }
            _ => false,
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
            self.direction = PlatformerDirection::from_platformer_velocity(&self.velocity);
        }
    }

    fn on_new_modifier(&mut self, modifier: Gd<StatModifier>) {
        if let Some(val) = self.test_stats.get_mut(&modifier.bind().stat) {
            let x = modifier.bind().clone();
            val.apply_modifier(x);
        }
    }
}

#[godot_dyn]
impl CharacterResources for MainCharacter {
    fn get_health(&self) -> u32 {
        self.stats.health
    }

    fn set_health(&mut self, amount: u32) {
        self.stats.health = amount;
    }

    fn get_energy(&self) -> u32 {
        self.stats.energy
    }

    fn set_energy(&mut self, amount: u32) {
        self.stats.energy = amount;
    }

    fn get_mana(&self) -> u32 {
        self.stats.mana
    }

    fn set_mana(&mut self, amount: u32) {
        self.stats.mana = amount;
    }
}

#[godot_dyn]
impl Damageable for MainCharacter {
    fn take_damage(&mut self, amount: u32) {
        let previous_health = self.get_health();
        let current_health = previous_health.saturating_sub(amount);
        println!("previous health {previous_health} ... current health {current_health}");

        self.set_health(current_health);
        self.signals()
            .player_health_changed()
            .emit(previous_health, current_health, amount);

        if self.is_dead() {
            println!("You died");
            self.destroy();
        }
    }

    fn destroy(&mut self) {
        self.signals().player_died().emit();
        self.base_mut().queue_free();
    }
}

#[godot_dyn]
impl Damaging for MainCharacter {
    fn damage_amount(&self) -> u32 {
        self.test_stats.get(&AttackDamage).unwrap().0 as u32
    }
}

#[godot_dyn]
impl Player for MainCharacter {}
