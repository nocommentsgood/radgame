use std::{collections::HashMap, mem::discriminant};

use godot::{
    classes::{
        AnimationPlayer, Area2D, CharacterBody2D, ICharacterBody2D, Input, RayCast2D, Timer,
    },
    obj::WithBaseField,
    prelude::*,
};
use statig::prelude::StateMachine;

use crate::{
    entities::{
        damage::{AttackData, Damageable, Damaging, TestDamaging},
        enemies::projectile::Projectile,
        entity_hitbox::EntityHitbox,
        entity_stats::{EntityResources, StatModifier, StatVal, Stats},
        hurtbox::Hurtbox,
        movements::Direction,
        player::{
            abilities::AbilityComp,
            character_state_machine as csm,
            item_component::ItemComponent,
            shaky_player_camera::{ShakyPlayerCamera, TraumaLevel},
        },
        time::PlayerTimer,
    },
    utils::{
        global_data_singleton::GlobalData,
        input_hanlder::{DevInputHandler, InputHandler, Inputs},
    },
};

type State = csm::State;
type PT = PlayerTimer;
type Event = csm::Event;
const GRAVITY: f32 = 1500.0;
const TERMINAL_VELOCITY: f32 = 500.0;

#[derive(GodotClass)]
#[class(init, base=CharacterBody2D)]
pub struct MainCharacter {
    inputs: Inputs,
    pub velocity: Vector2,
    previous_state: State,
    pub timers: HashMap<PlayerTimer, Gd<Timer>>,
    pub state: StateMachine<csm::CharacterStateMachine>,
    pub stats: HashMap<Stats, StatVal>,
    #[init(val = AbilityComp::new_test())]
    pub ability_comp: AbilityComp,
    base: Base<CharacterBody2D>,

    #[export]
    #[init(val = 500.0)]
    terminal_y_speed: f32,

    #[export]
    #[init(val = 280.0)]
    temp_jump_speed: f32,

    #[export]
    #[init(val = 980.0)]
    temp_gravity: f32,

    #[init(node = "ItemComponent")]
    pub item_comp: OnReady<Gd<ItemComponent>>,

    #[var]
    #[init(node = "AnimationPlayer")]
    animation_player: OnReady<Gd<AnimationPlayer>>,

    #[var]
    #[init(node = "LedgeSensor")]
    ledge_sensor: OnReady<Gd<RayCast2D>>,

    #[init(node = "ShakyPlayerCamera")]
    pub camera: OnReady<Gd<ShakyPlayerCamera>>,
}

#[godot_api]
impl ICharacterBody2D for MainCharacter {
    fn ready(&mut self) {
        let this = self.to_gd();

        GlobalData::singleton()
            .bind_mut()
            .sig_handler()
            .new_modifier()
            .connect_other(&this, Self::on_new_modifier);

        GlobalData::singleton()
            .bind_mut()
            .sig_handler()
            .modifier_removed()
            .connect_other(&this, Self::on_modifier_removed);

        self.stats.insert(Stats::Health, StatVal::new(50));
        self.stats.insert(Stats::MaxHealth, StatVal::new(50));
        self.stats.insert(Stats::HealAmount, StatVal::new(10));
        self.stats.insert(Stats::AttackDamage, StatVal::new(30));
        self.stats.insert(Stats::RunningSpeed, StatVal::new(180));
        self.stats.insert(Stats::JumpingSpeed, StatVal::new(500));
        self.stats.insert(Stats::DodgingSpeed, StatVal::new(250));
        self.stats.insert(Stats::AttackingSpeed, StatVal::new(10));
        self.stats.insert(Stats::Level, StatVal::new(1));

        self.init_timers();

        let hitbox = self.base().get_node_as::<Area2D>("Hitbox");
        hitbox
            .signals()
            .area_entered()
            .connect_other(&this, Self::on_area_entered_hitbox);
        let mut hurtbox = self.base().get_node_as::<Hurtbox>("Hurtbox");
        hurtbox.bind_mut().attack_damage = self.stats.get(&Stats::AttackDamage).unwrap().0;
        hurtbox
            .signals()
            .area_entered()
            .connect_other(&this, Self::on_area_entered_hurtbox);

        self.previous_state = State::IdleRight {};
    }

    fn physics_process(&mut self, delta: f32) {
        let input = DevInputHandler::handle_unhandled(&Input::singleton(), self);
        if self.inputs != input {
            self.inputs = input;
            self.transition_sm(&Event::InputChanged(input));
        }

        self.not_on_floor();
        self.player_landed_check();
        self.update_state();
        self.apply_gravity(&delta);
        dbg!(&self.state.state());
        self.accelerate();
    }
}

impl TestDamaging for Gd<MainCharacter> {}

#[godot_api]
impl MainCharacter {
    #[signal]
    pub fn player_health_changed(previous_health: u32, new_health: u32, damage_amount: u32);

    #[signal]
    fn parried_attack();

    #[signal]
    fn player_died();

    #[signal]
    pub fn animation_state_changed();

    /// Applies accelerated movement depending on current state.
    /// Moves the camera if the player's velocity has changed.
    fn accelerate(&mut self) {
        let stat = |map: &HashMap<Stats, StatVal>, stat: &Stats| map.get(stat).unwrap().0 as f32;

        let velocity = self.velocity;
        match self.state.state() {
            State::MoveFallingLeft {} | State::MoveLeftAirAttack {} => {
                self.velocity.x = self.velocity.x.lerp(-120.0, 0.7);
            }
            State::MoveFallingRight {} | State::MoveRightAirAttack {} => {
                self.velocity.x = self.velocity.x.lerp(120.0, 0.7);
            }
            State::DodgingLeft {} => {
                self.velocity.x = stat(&self.stats, &Stats::DodgingSpeed) * Vector2::LEFT.x;
            }
            State::DodgingRight {} => {
                self.velocity.x = stat(&self.stats, &Stats::DodgingSpeed) * Vector2::RIGHT.x;
            }
            State::MoveLeft {} => {
                self.velocity.x = self.velocity.x.lerp(
                    stat(&self.stats, &Stats::RunningSpeed) * Vector2::LEFT.x,
                    0.7,
                );
            }
            State::MoveRight {} => {
                self.velocity.x = self.velocity.x.lerp(
                    stat(&self.stats, &Stats::RunningSpeed) * Vector2::RIGHT.x,
                    0.7,
                );
            }
            State::JumpingRight {} => {
                self.velocity.y = self
                    .velocity
                    .y
                    .lerp(stat(&self.stats, &Stats::JumpingSpeed) * Vector2::UP.y, 0.4);
                self.velocity.x = 0.0;
            }
            State::JumpingLeft {} => {
                self.velocity.y = self
                    .velocity
                    .y
                    .lerp(stat(&self.stats, &Stats::JumpingSpeed) * Vector2::UP.y, 0.5);
                self.velocity.x = 0.0;
            }
            State::MoveJumpingRight {} => {
                self.velocity.y = self
                    .velocity
                    .y
                    .lerp(stat(&self.stats, &Stats::JumpingSpeed) * Vector2::UP.y, 0.5);
                self.velocity.x = self.velocity.x.lerp(120.0, 0.7);
            }
            State::MoveJumpingLeft {} => {
                self.velocity.y = self
                    .velocity
                    .y
                    .lerp(stat(&self.stats, &Stats::JumpingSpeed) * Vector2::UP.y, 0.5);
                self.velocity.x = self.velocity.x.lerp(-120.0, 0.7);
            }
            _ => self.velocity.x = 0.0,
        }

        // Player velocity changed.
        if self.velocity != velocity && !self.velocity.is_zero_approx() {
            self.update_camera();
        }

        // Apply movement.
        let velocity = self.velocity;
        // dbg!(&self.velocity);
        self.base_mut().set_velocity(velocity);
        self.base_mut().move_and_slide();

        // Ceiling collision handling.
        let ceiling = self.base().is_on_ceiling_only();
        let collisions = self.base_mut().get_last_slide_collision();
        if let Some(c) = collisions
            && ceiling
        {
            self.velocity = self.velocity.bounce(c.get_normal().normalized_or_zero());
            let velocity = self.velocity;
            self.base_mut().set_velocity(velocity);
        }
    }

    fn apply_gravity(&mut self, delta: &f32) {
        if !self.base().is_on_floor() && self.velocity.y < TERMINAL_VELOCITY {
            self.velocity.y += GRAVITY * delta;
        }
    }

    /// Transition state to `falling` when Y axis velocity is positive.
    fn not_on_floor(&mut self) {
        if !self.base().is_on_floor() {
            let is_falling = matches!(
                self.state.state(),
                State::MoveFallingLeft {}
                    | State::MoveFallingRight {}
                    | State::FallingLeft {}
                    | State::FallingRight {}
            );

            if self.velocity.y.is_sign_positive() && !is_falling {
                let input = InputHandler::handle(&Input::singleton(), self);
                self.transition_sm(&Event::FailedFloorCheck(input));
            }
        }
    }

    /// Checks if the player is on the floor.
    /// If so, sends the `Landed` event to the state machine, sets Y axis velocity to 0, and resets
    /// jumping timer.
    fn player_landed_check(&mut self) {
        let was_airborne = (matches!(
            self.state.state(),
            State::FallingRight {}
                | State::MoveFallingLeft {}
                | State::MoveFallingRight {}
                | State::FallingLeft {}
        ) || matches!(
            self.previous_state,
            State::JumpingLeft {}
                | State::JumpingRight {}
                | State::MoveJumpingRight {}
                | State::MoveJumpingLeft {}
                | State::AirAttackRight {}
                | State::AirAttackLeft {}
                | State::MoveLeftAirAttack {}
                | State::MoveRightAirAttack {}
        ));

        if self.base().is_on_floor() && was_airborne {
            self.velocity.y = 0.0;
            self.timers.get_mut(&PT::JumpTimeLimit).unwrap().reset();
            self.transition_sm(&Event::Landed(Inputs(
                InputHandler::get_movement(&Input::singleton()).0,
                None,
            )));
        }
    }

    // TODO: The comment below isn't currently true. Started making an attacking system then became
    // sidetracked refactoring so much... so... much...
    // Had to resort to enabling and disabling the collision shape manually, otherwise the
    // `area_entered()` signal of the `Hurtbox` would emit twice.
    fn on_area_entered_hurtbox(&mut self, area: Gd<Area2D>) {
        dbg!();
        if let Ok(mut hitbox) = area.try_cast::<EntityHitbox>() {
            dbg!();
            dbg!(crate::entities::damage::test_damage(&mut AttackData::new(
                self.stats.get(&Stats::AttackDamage).unwrap().0,
                &mut hitbox,
                &mut self.to_gd(),
            )));
        }
        //     self.hit_enemy = true;
        //     self.base()
        //         .get_node_as::<godot::classes::CollisionShape2D>("Hurtbox/HurtboxShape")
        //         .set_deferred("disabled", &true.to_variant());
        // }
    }

    fn on_area_entered_hitbox(&mut self, area: Gd<Area2D>) {
        if let Ok(h_box) = &area.try_cast::<Hurtbox>()
            && !self.parried_attack(h_box)
        {
            self.timers.get_mut(&PT::HurtAnimation).unwrap().start();
            let damaging =
                DynGd::<Area2D, dyn Damaging>::from_godot(h_box.clone().upcast::<Area2D>());
            let target = self.to_gd().upcast::<Node2D>();
            let guard = self.base_mut();
            let damageable = DynGd::<Node2D, dyn Damageable>::from_godot(target);
            damaging.dyn_bind().do_damage(damageable);
            drop(guard);
            let mut camera = self
                .base()
                .get_node_as::<ShakyPlayerCamera>("ShakyPlayerCamera");
            camera
                .bind_mut()
                .add_trauma(TraumaLevel::from(damaging.dyn_bind().damage_amount()));
            self.transition_sm(&Event::Hurt);
        }
    }

    fn on_parry_timeout(&mut self) {
        self.transition_sm(&Event::TimerElapsed(Inputs(
            InputHandler::get_movement(&Input::singleton()).0,
            None,
        )));
    }

    // TODO: Chain attacking.
    fn on_attack_timeout(&mut self) {
        self.transition_sm(&Event::TimerElapsed(Inputs(
            InputHandler::get_movement(&Input::singleton()).0,
            None,
        )));
    }

    fn on_attack_2_timeout(&mut self) {
        self.transition_sm(&Event::TimerElapsed(Inputs(
            InputHandler::get_movement(&Input::singleton()).0,
            None,
        )));
    }

    fn on_healing_timeout(&mut self) {
        self.timers.get_mut(&PT::HealingCooldown).unwrap().start();
        self.transition_sm(&Event::TimerElapsed(Inputs(
            InputHandler::get_movement(&Input::singleton()).0,
            None,
        )));
    }

    fn on_dodge_animation_timeout(&mut self) {
        self.timers.get_mut(&PT::DodgeCooldown).unwrap().start();
        self.transition_sm(&Event::TimerElapsed(Inputs(
            InputHandler::get_movement(&Input::singleton()).0,
            None,
        )));
    }

    fn on_hurt_animation_timeout(&mut self) {
        self.transition_sm(&Event::TimerElapsed(Inputs(
            InputHandler::get_movement(&Input::singleton()).0,
            None,
        )));
    }

    fn on_jump_limit_timeout(&mut self) {
        println!("Jump timer timeout");
        let input = InputHandler::handle(&Input::singleton(), self);
        self.transition_sm(&Event::TimerElapsed(input));
    }

    fn parried_attack(&mut self, area: &Gd<Hurtbox>) -> bool {
        match self.state.state() {
            State::ParryLeft {} | State::ParryRight {} => {
                if self.timers.get(&PT::PerfectParry).unwrap().get_time_left() > 0.0 {
                    println!("\nPERFECT PARRY\n");
                    if area.is_in_group("enemy_projectile")
                        && let Some(parent) = area.get_parent()
                        && let Ok(mut projectile) = parent.try_cast::<Projectile>()
                    {
                        projectile.bind_mut().on_parried();
                    }
                    self.signals().parried_attack().emit();
                    true
                } else if self.timers.get(&PT::Parry).unwrap().get_time_left() > 0.0 {
                    println!("\nNORMAL PARRY\n");
                    if area.is_in_group("enemy_projectile")
                        && let Some(parent) = area.get_parent()
                        && let Ok(mut projectile) = parent.try_cast::<Projectile>()
                    {
                        projectile.bind_mut().on_parried();
                    }
                    self.signals().parried_attack().emit();
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Transitions the state machine, checking and returning if the previous state is equal to the
    /// current state.
    pub fn transition_sm(&mut self, event: &Event) {
        let prev = *self.state.state();
        self.state.handle(event);
        let new = *self.state.state();
        if prev != new {
            println!("Updating animation");
            self.update_animation();
        }
    }

    /// Checks if the previous state is equal to the current state.
    fn state_matches(&self) -> bool {
        *self.state.state() == self.previous_state
    }

    /// Updates the state, setting `previous_state` to the current state.
    fn update_state(&mut self) {
        self.previous_state = *self.state.state();
    }

    fn update_animation(&mut self) {
        self.animation_player
            .play_ex()
            .name(&format!("{}", self.state.state()))
            .done();
        self.animation_player.advance(0.0);
    }

    /// Sets timer lengths, timer callbacks, and adds timers as children of the player.
    fn init_timers(&mut self) {
        // Animations, independent of cardinal direction, are all of the same length.
        // Therefore, it is acceptable to use the length of any dodging animation.
        // East was arbitrarily chosen.
        let dodge_animation_length = (self
            .get_animation_player()
            .get_animation("dodge_right")
            .unwrap()
            .get_length()
            / 1.5) as f64;

        let attack_animation_length = self
            .get_animation_player()
            .get_animation("attack_right")
            .unwrap()
            .get_length() as f64;

        let attack_2_animation_length = self
            .get_animation_player()
            .get_animation("chainattack_right")
            .unwrap()
            .get_length() as f64;

        let healing_animation_length = self
            .get_animation_player()
            .get_animation("heal_right")
            .unwrap()
            .get_length() as f64;

        let parry_animation_length = self
            .get_animation_player()
            .get_animation("parry_right")
            .unwrap()
            .get_length() as f64;

        let hurt_animation_length = self
            .get_animation_player()
            .get_animation("hurt_right")
            .unwrap()
            .get_length() as f64;
        let this = &self.to_gd();

        // Dodge animation
        let mut timer = Timer::new_alloc();
        timer.set_wait_time(dodge_animation_length);
        timer.set_one_shot(true);
        timer
            .signals()
            .timeout()
            .connect_other(this, Self::on_dodge_animation_timeout);
        self.timers.insert(PlayerTimer::DodgeAnimation, timer);

        // Dodge cooldown
        let mut timer = Timer::new_alloc();
        timer.set_wait_time(2.5);
        timer.set_one_shot(true);
        self.timers.insert(PT::DodgeCooldown, timer);

        // Attack anim
        let mut timer = Timer::new_alloc();
        timer.set_wait_time(attack_animation_length);
        timer.set_one_shot(true);
        timer
            .signals()
            .timeout()
            .connect_other(this, Self::on_attack_timeout);
        self.timers.insert(PlayerTimer::AttackAnimation, timer);

        // Attack 2 animation
        let mut timer = Timer::new_alloc();
        timer.set_wait_time(attack_2_animation_length);
        timer.set_one_shot(true);
        timer
            .signals()
            .timeout()
            .connect_other(this, Self::on_attack_2_timeout);
        self.timers.insert(PlayerTimer::Attack2Animation, timer);

        // Healing animation
        let mut timer = Timer::new_alloc();
        timer.set_wait_time(healing_animation_length);
        timer.set_one_shot(true);
        timer
            .signals()
            .timeout()
            .connect_other(&self.to_gd(), Self::on_healing_timeout);
        self.timers.insert(PlayerTimer::HealingAnimation, timer);

        // Healing cooldown
        let mut timer = Timer::new_alloc();
        timer.set_wait_time(1.5);
        timer.set_one_shot(true);
        self.timers.insert(PlayerTimer::HealingCooldown, timer);

        // Parry animation
        let mut timer = Timer::new_alloc();
        timer.set_wait_time(parry_animation_length);
        timer.set_one_shot(true);
        timer
            .signals()
            .timeout()
            .connect_other(this, Self::on_parry_timeout);
        self.timers.insert(PlayerTimer::ParryAnimation, timer);

        // Successful parry
        let mut timer = Timer::new_alloc();
        timer.set_wait_time(0.3);
        timer.set_one_shot(true);
        self.timers.insert(PlayerTimer::Parry, Timer::new_alloc());

        // Perfect parry
        let mut timer = Timer::new_alloc();
        timer.set_wait_time(0.15);
        timer.set_one_shot(true);
        self.timers.insert(PlayerTimer::PerfectParry, timer);

        self.timers.insert(PlayerTimer::Coyote, Timer::new_alloc());

        // Hurt animation
        let mut timer = Timer::new_alloc();
        timer.set_wait_time(hurt_animation_length);
        timer.set_one_shot(true);
        timer
            .signals()
            .timeout()
            .connect_other(this, Self::on_hurt_animation_timeout);
        self.timers.insert(PlayerTimer::HurtAnimation, timer);

        // Jump time limit
        let mut timer = Timer::new_alloc();
        timer.set_wait_time(0.2);
        timer.set_one_shot(true);
        timer
            .signals()
            .timeout()
            .connect_other(this, Self::on_jump_limit_timeout);
        self.timers.insert(PlayerTimer::JumpTimeLimit, timer);

        let mut pt = self.timers.clone();
        pt.values_mut().for_each(|timer| {
            timer.set_one_shot(true);
            self.base_mut().add_child(&timer.clone());
        });
    }

    fn update_camera(&mut self) {
        if self.velocity.x.is_sign_positive() {
            self.camera.bind_mut().set_right(Some(true));
        }
        if self.velocity.x.is_sign_negative() {
            self.camera.bind_mut().set_right(Some(false));
        }
    }

    fn on_new_modifier(&mut self, modifier: Gd<StatModifier>) {
        if let Some(val) = self.stats.get_mut(&modifier.bind().stat) {
            val.apply_modifier(modifier.bind().clone());
        }
    }

    fn on_modifier_removed(&mut self, modifier: Gd<StatModifier>) {
        if let Some(val) = self.stats.get_mut(&modifier.bind().stat) {
            val.remove_modifier(modifier.bind().clone());
        }
    }

    pub fn get_direction(&self) -> Direction {
        let state = self.state.state().to_string();
        if state.contains("right") {
            Direction::East
        } else {
            Direction::West
        }
    }

    /// Transitions state machine from it's current state to `disabled`.
    /// Effectively disables input handling.
    pub fn force_disabled(&mut self) {
        self.transition_sm(&csm::Event::ForceDisabled);
    }

    /// Transitions state machine from `disabled` to `idle`.
    pub fn force_enabled(&mut self) {
        self.transition_sm(&csm::Event::ForceEnabled);
    }
}

#[godot_dyn]
impl EntityResources for MainCharacter {
    fn get_health(&self) -> u32 {
        self.stats.get(&Stats::Health).unwrap().0
    }

    fn set_health(&mut self, amount: u32) {
        self.stats.get_mut(&Stats::Health).unwrap().0 = amount;
    }

    fn get_energy(&self) -> u32 {
        self.stats.get(&Stats::Energy).unwrap().0
    }

    fn set_energy(&mut self, amount: u32) {
        self.stats.get_mut(&Stats::Energy).unwrap().0 = amount;
    }

    fn get_mana(&self) -> u32 {
        self.stats.get(&Stats::Mana).unwrap().0
    }

    fn set_mana(&mut self, amount: u32) {
        self.stats.get_mut(&Stats::Mana).unwrap().0 = amount;
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
        self.stats.get(&Stats::AttackDamage).unwrap().0
    }
}

trait Reset {
    fn reset(&mut self);
}

impl Reset for Gd<Timer> {
    fn reset(&mut self) {
        if self.get_time_left() != 0.0 {
            self.stop();
            self.start();
            self.stop();
        }
    }
}
