use std::{collections::HashMap, slice::SliceIndex, time::Duration};

use godot::{
    classes::{
        AnimationPlayer, Area2D, CharacterBody2D, CollisionObject2D, ICharacterBody2D, Input,
        RayCast2D, Timer,
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
            character_state_machine as csm,
            item_component::ItemComponent,
            shaky_player_camera::{ShakyPlayerCamera, TraumaLevel},
        },
        time::PlayerTimer,
    },
    utils::input_hanlder::{InputHandler, Movement},
};

type State = csm::State;
type PT = PlayerTimer;
type Event = csm::Event;
const GRAVITY: f32 = 500.0;
const TERMINAL_VELOCITY: f32 = 200.0;

#[derive(GodotClass)]
#[class(init, base=CharacterBody2D)]
pub struct MainCharacter {
    direction: Direction,
    pub velocity: Vector2,
    prev_velocity: Vector2,
    active_velocity: Vector2,
    hit_enemy: bool,
    can_attack_chain: bool,
    inputs: InputHandler,
    pub timers: HashMap<PlayerTimer, Gd<Timer>>,
    pub state: StateMachine<csm::CharacterStateMachine>,
    pub stats: HashMap<Stats, StatVal>,
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

        let hitbox = self.base().get_node_as::<Area2D>("Hitbox");
        hitbox
            .signals()
            .area_entered()
            .connect_other(&this, Self::on_area_entered_hitbox);
        self.item_comp
            .signals()
            .new_modifier()
            .connect_other(&this, Self::on_new_modifier);

        self.item_comp
            .signals()
            .modifier_removed()
            .connect_other(&this, Self::on_modifier_removed);

        let hurtbox = self.base().get_node_as::<Area2D>("Hurtbox");
        hurtbox
            .signals()
            .area_entered()
            .connect_other(&this, Self::on_area_entered_hurtbox);

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
            .get_animation("attack_east")
            .unwrap()
            .get_length();

        let healing_animation_length = self
            .get_animation_player()
            .get_animation("heal_east")
            .unwrap()
            .get_length() as f64;

        let parry_animation_length = self
            .get_animation_player()
            .get_animation("parry_east")
            .unwrap()
            .get_length();

        let hurt_animation_length = self
            .get_animation_player()
            .get_animation("hurt_east")
            .unwrap()
            .get_length();

        self.stats.insert(Stats::Health, StatVal::new(50));
        self.stats.insert(Stats::MaxHealth, StatVal::new(50));
        self.stats.insert(Stats::HealAmount, StatVal::new(10));
        self.stats.insert(Stats::AttackDamage, StatVal::new(30));
        self.stats.insert(Stats::RunningSpeed, StatVal::new(150));
        self.stats.insert(Stats::JumpingSpeed, StatVal::new(350));
        self.stats.insert(Stats::DodgingSpeed, StatVal::new(250));
        self.stats.insert(Stats::AttackingSpeed, StatVal::new(10));

        let this = &self.to_gd();

        // Attack chain
        // let mut timer = Timer::new_alloc();
        // timer.set_wait_time(attack_animation_length as f64);
        // timer
        //     .signals()
        //     .timeout()
        //     .connect_other(this, Self::on_attack_timeout);
        // self.timers
        //     .insert(PlayerTimer::AttackChain, Timer::new_alloc());

        // Dodge animation
        let mut timer = Timer::new_alloc();
        timer.set_wait_time(dodge_animation_length as f64);
        timer
            .signals()
            .timeout()
            .connect_other(this, Self::on_dodge_animation_timeout);
        self.timers.insert(PlayerTimer::DodgeAnimation, timer);

        // Dodge cooldown
        let mut timer = Timer::new_alloc();
        timer.set_wait_time(2.5);
        self.timers.insert(PT::DodgeCooldown, timer);

        // TODO: Used?
        self.timers
            .insert(PlayerTimer::JumpingAnimation, Timer::new_alloc());

        // Attack anim
        let mut timer = Timer::new_alloc();
        timer.set_wait_time(attack_animation_length as f64);
        timer
            .signals()
            .timeout()
            .connect_other(this, Self::on_attack_timeout);
        self.timers.insert(PlayerTimer::AttackAnimation, timer);

        // Healing Animation
        let mut timer = Timer::new_alloc();
        timer.set_wait_time(healing_animation_length);
        timer
            .signals()
            .timeout()
            .connect_other(&self.to_gd(), Self::on_healing_timeout);
        self.timers.insert(PlayerTimer::HealingAnimation, timer);

        let mut timer = Timer::new_alloc();
        timer.set_wait_time(1.5);
        self.timers.insert(PlayerTimer::HealingCooldown, timer);

        // Parry animation
        let mut timer = Timer::new_alloc();
        timer.set_wait_time(parry_animation_length as f64);
        timer
            .signals()
            .timeout()
            .connect_other(this, Self::on_parry_timeout);
        self.timers.insert(PlayerTimer::ParryAnimation, timer);

        // Parry
        let mut timer = Timer::new_alloc();
        timer.set_wait_time(0.3);
        self.timers.insert(PlayerTimer::Parry, Timer::new_alloc());

        // Perfect parry
        let mut timer = Timer::new_alloc();
        timer.set_wait_time(0.15);
        self.timers.insert(PlayerTimer::PerfectParry, timer);

        self.timers.insert(PlayerTimer::Coyote, Timer::new_alloc());

        let mut pt = self.timers.clone();
        pt.values_mut().for_each(|timer| {
            timer.set_one_shot(true);
            self.base_mut().add_child(&timer.clone());
        });

        let mut hurtbox = self.base().get_node_as::<Hurtbox>("Hurtbox");
        hurtbox.bind_mut().attack_damage = self.stats.get(&Stats::AttackDamage).unwrap().0;

        self.animation_player.play_ex().name("idle_east").done();
    }

    // When a user provides input, execution of the relevant state function starts immediately.
    // This ensures that the `animation_state_changed` signal is emitted when an input changes the
    // state by eagerly checking the state machine's next state, just before it changes state.
    // Otherwise, `physics_process` handles emitting the signal.
    fn unhandled_input(&mut self, input: Gd<godot::classes::InputEvent>) {
        let timer_ok = |timer: Option<&Gd<Timer>>| timer.is_some_and(|t| t.get_time_left() == 0.0);

        if input.is_action_pressed("attack") && timer_ok(self.timers.get(&PT::AttackAnimation)) {
            self.timers.get_mut(&PT::AttackAnimation).unwrap().start();
            self.transition_sm(&Event::AttackButton);
        }
        if input.is_action_pressed("jump") {
            self.transition_sm(&Event::JumpButton);
        }
        if input.is_action_released("jump") {
            self.transition_sm(&Event::ActionReleasedEarly);
        }

        if input.is_action_pressed("dodge")
            && timer_ok(self.timers.get(&PT::DodgeAnimation))
            && timer_ok(self.timers.get(&PT::DodgeCooldown))
        {
            self.transition_sm(&Event::DodgeButton);
        }

        if input.is_action_pressed("heal")
            && timer_ok(self.timers.get(&PT::HealingAnimation))
            && timer_ok(self.timers.get(&PT::HealingCooldown))
        {
            self.timers.get_mut(&PT::HealingAnimation).unwrap().start();
            self.transition_sm(&Event::HealingButton);
            let get_stat = |stat: Option<&StatVal>| stat.unwrap().0;
            let cur = get_stat(self.stats.get(&Stats::Health));
            let max = get_stat(self.stats.get(&Stats::MaxHealth));
            let amount = get_stat(self.stats.get(&Stats::HealAmount));
            if cur < max {
                self.stats.get_mut(&Stats::Health).unwrap().0 += amount;
                let new = get_stat(self.stats.get(&Stats::Health));
                self.signals()
                    .player_health_changed()
                    .emit(cur, new, amount);
            }
        }

        if input.is_action_pressed("parry") && timer_ok(self.timers.get(&PT::ParryAnimation)) {
            self.timers.get_mut(&PT::ParryAnimation).unwrap().start();
            self.timers.get_mut(&PT::PerfectParry).unwrap().start();
            self.transition_sm(&Event::ParryButton);
        }
    }

    fn physics_process(&mut self, delta: f32) {
        let prev_x_vel = self.velocity.x;
        let event = InputHandler::set_vel_get_event(
            &Input::singleton(),
            self.state.state(),
            &self.stats,
            &mut self.velocity,
            &delta,
        );
        let input = InputHandler::get_velocity(&Input::singleton());
        let movement = Movement::from_vel(input);
        let v = self.velocity;
        let new_x_vel = self.velocity.x;

        // Update animation direction on velocity change when not dodging.
        if prev_x_vel != new_x_vel
            && self.state.state().as_descriminant() != csm::to_descriminant(&State::Dodging {})
        {
            self.update_animation();
        }

        if !self.base().is_on_floor() {
            // self.transition_sm(&Event::FailedFloorCheck);
            if self.velocity.y <= TERMINAL_VELOCITY {
                self.velocity.y += GRAVITY * delta;
            }
            self.velocity.x *= self.stats.get(&Stats::RunningSpeed).unwrap().0 as f32;
        }

        if self.base().is_on_floor() {
            self.velocity.y = 0.0;
        }

        match self.state.state() {
            // csm::State::Falling {} => {
            //     if velocity.y <= TERMINAL_VELOCITY {
            //         velocity.y += GRAVITY * delta;
            //     }
            //     velocity.x *= stats.get(&Stats::RunningSpeed).unwrap().0 as f32;
            //     self.base_mut().set_velocity(v);
            //     self.base_mut().move_and_slide();
            //     if self.base().is_on_floor() {
            //         if self.velocity.x > 0.0 {
            //             self.transition_sm(&Event::OnFloor);
            //         } else {
            //             self.transition_sm(&Event::MovingToIdle);
            //         }
            //     }
            // }
            csm::State::Dodging {} => self.dodge(),
            _ => {
                self.base_mut().set_velocity(v);
                self.base_mut().move_and_slide();
            }
        }

        // TODO: See if I can get rid of these constant calls to transition.
        self.transition_sm(&event);
        // dbg!(self.state.state());
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

    // Had to resort to enabling and disabling the collision shape manually, otherwise the
    // `area_entered()` signal of the `Hurtbox` would emit twice.
    fn on_area_entered_hurtbox(&mut self, area: Gd<Area2D>) {
        if let Ok(mut hurtbox) = area.try_cast::<EntityHitbox>() {
            let this = &mut self.to_gd();
            let anim_player = &mut self.animation_player;
            let mut timer = godot::classes::Timer::new_alloc();
            let mut d = AttackData::new(
                "something",
                10,
                &mut hurtbox,
                this,
                "attack_east",
                anim_player,
                &mut timer,
            );

            crate::entities::damage::test_damage(&mut d);
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
        self.transition_sm(&Event::TimerElapsed);
    }

    fn on_attack_timeout(&mut self) {
        if self.velocity.x == 0.0 {
            self.transition_sm(&Event::MovingToIdle);
        } else {
            self.transition_sm(&Event::TimerElapsed);
        }
    }

    fn on_healing_timeout(&mut self) {
        self.timers.get_mut(&PT::HealingCooldown).unwrap().start();
        self.transition_sm(&Event::TimerElapsed);
    }

    fn on_dodge_animation_timeout(&mut self) {
        if self.velocity.x == 0.0 {
            self.transition_sm(&Event::MovingToIdle);
        } else {
            self.transition_sm(&Event::TimerElapsed);
        }
        self.timers.get_mut(&PT::DodgeCooldown).unwrap().start();
    }

    fn dodge(&mut self) {
        if self
            .timers
            .get(&PT::DodgeAnimation)
            .unwrap()
            .get_time_left()
            == 0.0
        {
            let dir = Direction::from_vel(&self.velocity);
            let mut velocity = dir.to_vel();
            velocity.x *= self.stats.get(&Stats::DodgingSpeed).unwrap().0 as f32;
            self.timers.get_mut(&PT::DodgeAnimation).unwrap().start();
            self.base_mut().set_velocity(velocity);
            self.base_mut().move_and_slide();
        } else {
            self.base_mut().move_and_slide();
        }
    }

    fn attack(&mut self) {
        // TODO: Maybe there is a better way of ignoring input. If the player is hit during an
        // attack, the state machine switches to 'hurt' state (as it should), but input handling is
        // never turned back on.
        // self.base_mut().set_process_unhandled_input(false);
        // let delta = Duration::from_secs_f32(self.base().get_physics_process_delta_time() as f32);
        //
        // let is_running =
        //     |timer: &Timer| !timer.paused() && !timer.finished() && timer.elapsed_secs() > 0.0;
        //
        // let mut h_shape = self
        //     .base()
        //     .get_node_as::<godot::classes::CollisionShape2D>("Hurtbox/HurtboxShape");
        //
        // if is_running(self.timers.get(&PlayerTimer::AttackAnimation).unwrap()) {
        //     h_shape.set_deferred("disabled", &true.to_variant());
        //     self.timers
        //         .get_mut(&PT::AttackAnimation)
        //         .unwrap()
        //         .tick(delta);
        //
        //     if Input::singleton().is_action_pressed("parry") {
        //         self.timers.get_mut(&PT::AttackAnimation).unwrap().reset();
        //         self.base_mut().set_process_unhandled_input(true);
        //         self.state.handle(&Event::ParryButton);
        //     }
        //
        //     if self.timers[&PT::AttackChain].remaining_secs() > 0.0 {
        //         if Input::singleton().is_action_just_pressed("attack") {
        //             if self.hit_enemy {
        //                 self.can_attack_chain = true;
        //                 self.hit_enemy = false;
        //             }
        //         } else {
        //             self.timers.get_mut(&PT::AttackChain).unwrap().tick(delta);
        //         }
        //     }
        // } else {
        //     h_shape.set_deferred("disabled", &false.to_variant());
        //     self.timers
        //         .get_mut(&PT::AttackAnimation)
        //         .unwrap()
        //         .tick(delta);
        //     self.timers.get_mut(&PT::AttackChain).unwrap().tick(delta);
        // }
        // if self.timers[&PT::AttackAnimation].just_finished() {
        //     self.base_mut().set_process_unhandled_input(true);
        //     h_shape.set_deferred("disabled", &true.to_variant());
        //     self.timers.get_mut(&PT::AttackAnimation).unwrap().reset();
        //     self.timers.get_mut(&PT::AttackChain).unwrap().reset();
        //     if self.can_attack_chain {
        //         self.can_attack_chain = false;
        //         self.hit_enemy = false;
        //         self.state.handle(&Event::AttackButton);
        //     } else {
        //         self.hit_enemy = false;
        //         if self.velocity.x == 0.0 {
        //             self.state.handle(&Event::MovingToIdle);
        //         } else {
        //             self.state.handle(&Event::TimerElapsed);
        //         }
        //     }
        // }
    }

    fn attack_2(&mut self) {
        // self.can_attack_chain = false;
        // let delta = Duration::from_secs_f32(self.base().get_physics_process_delta_time() as f32);
        //
        // if self
        //     .timers
        //     .get(&PT::AttackAnimation2)
        //     .unwrap()
        //     .remaining_secs()
        //     > 0.0
        // {
        //     self.timers
        //         .get_mut(&PT::AttackAnimation2)
        //         .unwrap()
        //         .tick(delta);
        // } else {
        //     self.timers
        //         .get_mut(&PT::AttackAnimation2)
        //         .unwrap()
        //         .tick(delta);
        //
        //     if self
        //         .timers
        //         .get(&PT::AttackAnimation2)
        //         .unwrap()
        //         .just_finished()
        //     {
        //         self.timers.get_mut(&PT::AttackAnimation2).unwrap().reset();
        //         self.state.handle(&Event::TimerElapsed);
        //     }
        // }
    }

    // fn air_attack(&mut self) {
    //     let time = self.base().get_physics_process_delta_time() as f32;
    //     let delta = Duration::from_secs_f32(self.base().get_physics_process_delta_time() as f32);
    //
    //     if self.velocity.y <= self.terminal_y_speed {
    //         self.velocity.y += self.temp_gravity * time;
    //         self.velocity.x *= self.stats.get(&Stats::RunningSpeed).unwrap().0 as f32;
    //
    //         let velocity = self.velocity;
    //         self.base_mut().set_velocity(velocity);
    //         self.base_mut().move_and_slide();
    //         self.timers
    //             .get_mut(&PT::AttackAnimation)
    //             .unwrap()
    //             .tick(delta);
    //     } else {
    //         self.base_mut().move_and_slide();
    //         self.timers
    //             .get_mut(&PT::AttackAnimation)
    //             .unwrap()
    //             .tick(delta);
    //     }
    //
    //     if self.timers[&PT::AttackAnimation].just_finished() {
    //         self.timers.get_mut(&PT::AttackAnimation).unwrap().reset();
    //         self.state.handle(&Event::TimerElapsed);
    //     }
    // }

    fn hurt(&mut self) {
        self.base_mut().set_process_unhandled_input(true);
    }

    fn parried_attack(&mut self, area: &Gd<Hurtbox>) -> bool {
        match self.state.state() {
            State::Parry {} => {
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

    fn transition_sm(&mut self, event: &Event) {
        let cur = self.state.state().as_descriminant();
        self.state.handle(event);
        let new = self.state.state().as_descriminant();

        if cur != new {
            self.update_animation();
        }
    }

    fn update_animation(&mut self) {
        self.update_direction();
        println!("Vel in update anim: {}", self.velocity);
        if self.velocity.y.is_sign_positive() && self.velocity.y != 0.0 {
            self.animation_player
                .play_ex()
                .name(&format!("{}_{}", "falling", self.direction))
                .done();
        }
        if self.velocity.y < 0.0 {
            self.animation_player
                .play_ex()
                .name(&format!("{}_{}", "jumping", self.direction))
                .done();
        } else {
            let anim = format!("{}_{}", self.state.state(), self.direction);
            self.animation_player.play_ex().name(&anim).done();
            self.animation_player.advance(0.0);
        }
    }

    fn update_direction(&mut self) {
        if !self.velocity.x.is_zero_approx() {
            self.direction = Direction::from_vel(&self.velocity);

            if self.velocity.x.is_sign_positive() {
                let mut camera = self
                    .base()
                    .get_node_as::<ShakyPlayerCamera>("ShakyPlayerCamera");
                camera.bind_mut().set_right = true;
            }

            if self.velocity.x.is_sign_negative() {
                let mut camera = self
                    .base()
                    .get_node_as::<ShakyPlayerCamera>("ShakyPlayerCamera");
                camera.bind_mut().set_right = false;
            }
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
