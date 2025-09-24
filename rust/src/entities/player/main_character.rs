use std::collections::HashMap;

use godot::{
    classes::{AnimationPlayer, CharacterBody2D, ICharacterBody2D, Input, RayCast2D, Timer},
    obj::WithBaseField,
    prelude::*,
};
use statig::prelude::StateMachine;

use crate::{
    entities::{
        damage::{AttackData, Damage, DamageType, Damageable, HasHealth},
        enemies::projectile::Projectile,
        entity_stats::{EntityStats, Stat, StatModifier, StatVal},
        hit_reg::{Hitbox, Hurtbox},
        movements::Direction,
        player::{
            abilities::AbilityComp,
            character_state_machine as csm,
            item_component::ItemComponent,
            shaky_player_camera::{PlayerCamera, TraumaLevel},
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
    pub previous_state: State,
    pub timers: HashMap<PlayerTimer, Gd<Timer>>,
    early_gravity_time: f32,
    pub state: StateMachine<csm::CharacterStateMachine>,
    pub stats: EntityStats,
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
    pub camera: OnReady<Gd<PlayerCamera>>,

    #[init(node = "Hurtbox")]
    hurtbox: OnReady<Gd<Hurtbox>>,

    #[init(node = "Hitbox")]
    hitbox: OnReady<Gd<Hitbox>>,
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

        self.stats.add_slice(&[
            (Stat::Health, StatVal::new(50)),
            (Stat::MaxHealth, StatVal::new(50)),
            (Stat::HealAmount, StatVal::new(10)),
            (Stat::AttackDamage, StatVal::new(30)),
            (Stat::RunningSpeed, StatVal::new(180)),
            (Stat::JumpingSpeed, StatVal::new(300)),
            (Stat::DodgingSpeed, StatVal::new(250)),
            (Stat::AttackingSpeed, StatVal::new(10)),
            (Stat::Level, StatVal::new(1)),
        ]);

        // self.stats.insert(Stat::Health, StatVal::new(50));
        // self.stats.insert(Stat::MaxHealth, StatVal::new(50));
        // self.stats.insert(Stat::HealAmount, StatVal::new(10));
        // self.stats.insert(Stat::AttackDamage, StatVal::new(30));
        // self.stats.insert(Stat::RunningSpeed, StatVal::new(180));
        // self.stats.insert(Stat::JumpingSpeed, StatVal::new(300));
        // self.stats.insert(Stat::DodgingSpeed, StatVal::new(250));
        // self.stats.insert(Stat::AttackingSpeed, StatVal::new(10));
        // self.stats.insert(Stat::Level, StatVal::new(1));

        self.init_timers();

        self.previous_state = State::IdleRight {};

        self.hitbox.bind_mut().damageable_parent = Some(Box::new(self.to_gd()));
        self.hurtbox.bind_mut().data = Some(AttackData {
            hurtbox: self.hurtbox.clone(),
            parryable: false,
            damage: Damage {
                raw: self.stats.get_raw(Stat::AttackDamage),
                d_type: DamageType::Physical,
            },
        });
    }

    fn physics_process(&mut self, delta: f32) {
        let input = DevInputHandler::handle_unhandled(&Input::singleton(), self);
        if self.inputs != input {
            self.inputs = input;
            // dbg!(&input);
            self.transition_sm(&Event::InputChanged(input));
        }

        // TODO: Refactor early gravity time handling.
        if !self.base().is_on_floor() {
            self.early_gravity_time += delta;
        }

        self.not_on_floor();
        self.player_landed_check();
        self.update_state();
        self.apply_gravity(&delta);
        // dbg!(&self.state.state());
        self.accelerate();
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

    #[signal]
    pub fn animation_state_changed();

    /// Applies accelerated movement depending on current state.
    /// Moves the camera if the player's velocity has changed.
    fn accelerate(&mut self) {
        // let stat = |map: &HashMap<Stat, StatVal>, stat: &Stat| map.get(stat).unwrap().0 as f32;

        let velocity = self.velocity;
        match self.state.state() {
            State::MoveFallingLeft {} | State::MoveLeftAirAttack {} => {
                self.velocity.x = self.stats.get_raw(Stat::RunningSpeed) as f32;
            }
            State::MoveFallingRight {} | State::MoveRightAirAttack {} => {
                self.velocity.x = self.stats.get_raw(Stat::RunningSpeed) as f32 * Vector2::RIGHT.x;
            }
            State::DodgingLeft {} => {
                self.velocity.x = self.stats.get_raw(Stat::DodgingSpeed) as f32 * Vector2::LEFT.x;
            }
            State::DodgingRight {} => {
                self.velocity.x = self.stats.get_raw(Stat::DodgingSpeed) as f32 * Vector2::RIGHT.x;
            }
            State::MoveLeft {} => {
                self.velocity.x = self.stats.get_raw(Stat::RunningSpeed) as f32 * Vector2::LEFT.x;
            }
            State::MoveRight {} => {
                self.velocity.x = self.stats.get_raw(Stat::RunningSpeed) as f32 * Vector2::RIGHT.x;
            }
            State::JumpingRight {} => {
                self.velocity.y = self.stats.get_raw(Stat::JumpingSpeed) as f32 * Vector2::UP.y;
                self.velocity.x = 0.0;
            }
            State::JumpingLeft {} => {
                self.velocity.y = self.stats.get_raw(Stat::JumpingSpeed) as f32 * Vector2::UP.y;
                self.velocity.x = 0.0;
            }
            State::MoveJumpingRight {} => {
                self.velocity.x = self.stats.get_raw(Stat::RunningSpeed) as f32 * Vector2::RIGHT.x;
                self.velocity.y = self.stats.get_raw(Stat::JumpingSpeed) as f32 * Vector2::UP.y;
            }
            State::MoveJumpingLeft {} => {
                self.velocity.x = self.stats.get_raw(Stat::RunningSpeed) as f32 * Vector2::LEFT.x;
                self.velocity.y = self.stats.get_raw(Stat::JumpingSpeed) as f32 * Vector2::UP.y;
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
            if self.early_gravity_time >= 0.8 {
                self.velocity.y += GRAVITY * delta;
            } else if self.early_gravity_time < 0.8 && self.early_gravity_time >= 0.4 {
                self.velocity.y += 1700.0 * delta;
            } else {
                self.velocity.y += 2000.0 * delta;
            }
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

    // fn wall_grab(&mut self) {
    //     if !self.base().is_on_floor() && self.base().is_on_wall_only() {
    //         let input = InputHandler::
    //     }
    // }

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
            self.early_gravity_time = 0.0;
            self.transition_sm(&Event::Landed(Inputs(
                InputHandler::get_movement(&Input::singleton()).0,
                None,
            )));
            self.timers.get_mut(&PT::JumpTimeLimit).unwrap().reset();
        }
    }

    // fn on_area_entered_hitbox(&mut self, area: Gd<Area2D>) {
    //     if let Ok(h_box) = &area.try_cast::<Hurtbox>()
    //         && !self.parried_attack(h_box)
    //     {
    // self.timers.get_mut(&PT::HurtAnimation).unwrap().start();
    // let damaging =
    //     DynGd::<Area2D, dyn Damaging>::from_godot(h_box.clone().upcast::<Area2D>());
    // let target = self.to_gd().upcast::<Node2D>();
    // let guard = self.base_mut();
    // let damageable = DynGd::<Node2D, dyn Damageable>::from_godot(target);
    // damaging.dyn_bind().do_damage(damageable);
    // drop(guard);
    // let mut camera = self.base().get_node_as::<PlayerCamera>("ShakyPlayerCamera");
    // camera
    //     .bind_mut()
    //     .add_trauma(TraumaLevel::from(damaging.dyn_bind().damage_amount()));
    // self.transition_sm(&Event::Hurt);
    //     }
    // }

    fn on_parry_timeout(&mut self) {
        let input = InputHandler::handle(&Input::singleton(), self);
        self.transition_sm(&Event::TimerElapsed(input));
    }

    // TODO: Chain attacking.
    fn on_attack_timeout(&mut self) {
        let input = InputHandler::handle(&Input::singleton(), self);
        self.transition_sm(&Event::TimerElapsed(input));
    }

    fn on_attack_2_timeout(&mut self) {
        let input = InputHandler::handle(&Input::singleton(), self);
        self.transition_sm(&Event::TimerElapsed(input));
    }

    fn on_healing_timeout(&mut self) {
        self.timers.get_mut(&PT::HealingCooldown).unwrap().start();
        let input = InputHandler::handle(&Input::singleton(), self);
        self.transition_sm(&Event::TimerElapsed(input));
    }

    fn on_dodge_animation_timeout(&mut self) {
        self.timers.get_mut(&PT::DodgeCooldown).unwrap().start();
        let input = InputHandler::handle(&Input::singleton(), self);
        self.transition_sm(&Event::TimerElapsed(input));
    }

    fn on_hurt_animation_timeout(&mut self) {
        let input = InputHandler::handle(&Input::singleton(), self);
        self.transition_sm(&Event::TimerElapsed(input));
    }

    fn on_jump_limit_timeout(&mut self) {
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
        self.state.handle_with_context(event, &mut self.timers);
        let new = *self.state.state();
        if prev != new {
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
        timer.set_wait_time(0.4);
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
        if self.velocity.x > 5.0 {
            self.camera.bind_mut().set_right(Some(true));
        } else if self.velocity.x < -5.0 {
            self.camera.bind_mut().set_right(Some(false));
        }
    }

    fn on_new_modifier(&mut self, modifier: Gd<StatModifier>) {
        let modif = modifier.bind();
        self.stats.get_mut(modif.stat).apply_modifier(modif.clone());
    }

    fn on_modifier_removed(&mut self, modifier: Gd<StatModifier>) {
        let modif = modifier.bind();
        self.stats
            .get_mut(modif.stat)
            .remove_modifier(modif.clone());
    }

    pub fn get_direction(&self) -> Direction {
        let state = self.state.state().to_string();
        if state.contains("right") {
            Direction::Right
        } else {
            Direction::Left
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

impl HasHealth for Gd<MainCharacter> {
    fn get_health(&self) -> u32 {
        self.bind().stats.get_raw(Stat::Health)
    }

    fn set_health(&mut self, amount: u32) {
        self.bind_mut().stats.get_mut(Stat::Health).0 = amount;
    }

    fn on_death(&mut self) {
        println!("Player died");
        self.queue_free();
    }
}

impl Damageable for Gd<MainCharacter> {
    fn handle_attack(&mut self, attack: AttackData) {
        if attack.parryable {
            let parried = self.bind_mut().parried_attack(&attack.hurtbox);
            if !parried {
                let mut guard = self.bind_mut();
                guard.timers.get_mut(&PT::HurtAnimation).unwrap().start();
                guard
                    .camera
                    .bind_mut()
                    .add_trauma(TraumaLevel::from(attack.damage.raw));
                drop(guard);
                let cur = self.get_health();
                self.take_damage(attack.damage.raw);
                let new = self.get_health();
                self.signals()
                    .player_health_changed()
                    .emit(cur, new, attack.damage.raw);
                self.bind_mut().transition_sm(&Event::Hurt);
            }
        }
    }
}

// TODO: Move this.
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
