use std::{collections::HashMap, time::Duration};

// TODO: Refactor timers again...
use bevy_time::{Timer, TimerMode};
use godot::{
    classes::{
        AnimationPlayer, Area2D, CharacterBody2D, CollisionObject2D, ICharacterBody2D, Input,
        RayCast2D,
    },
    obj::WithBaseField,
    prelude::*,
};

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
    },
    utils::input_hanlder::InputHandler,
};

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum PlayerTimer {
    AttackChain,
    DodgeAnimation,
    JumpingAnimation,
    AttackAnimation,
    AttackAnimation2,
    HealingAnimation,
    HurtAnimation,
    ParryAnimation,
    Parry,
    PerfectParry,
    Coyote,
}

type State = csm::State;
type PT = PlayerTimer;
type Event = csm::Event;
const GRAVITY: f32 = 1100.0;
const TERMINAL_VELOCITY: f32 = 200.0;

#[derive(GodotClass)]
#[class(init, base=CharacterBody2D)]
pub struct MainCharacter {
    direction: Direction,
    velocity: Vector2,
    prev_velocity: Vector2,
    active_velocity: Vector2,
    hit_enemy: bool,
    can_attack_chain: bool,
    timers: HashMap<PlayerTimer, Timer>,
    state: statig::blocking::StateMachine<csm::CharacterStateMachine>,
    stats: HashMap<Stats, StatVal>,
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
    #[init(node = "DodgingCooldownTimer")]
    dodging_cooldown_timer: OnReady<Gd<godot::classes::Timer>>,
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

        self.signals()
            .animation_state_changed()
            .connect_self(Self::on_animation_state_changed);

        self.get_dodging_cooldown_timer()
            .signals()
            .timeout()
            .connect_other(&self.to_gd(), Self::on_dodge_timer_timeout);

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
        self.stats.insert(Stats::JumpingSpeed, StatVal::new(280));
        self.stats.insert(Stats::DodgingSpeed, StatVal::new(250));
        self.stats.insert(Stats::AttackingSpeed, StatVal::new(10));

        self.timers.insert(
            PlayerTimer::AttackChain,
            Timer::from_seconds(0.3, TimerMode::Once),
        );

        self.timers.insert(
            PlayerTimer::DodgeAnimation,
            Timer::from_seconds(dodge_animation_length, TimerMode::Once),
        );
        self.timers.insert(
            PlayerTimer::JumpingAnimation,
            Timer::from_seconds(jumping_animation_length, TimerMode::Once),
        );
        self.timers.insert(
            PlayerTimer::AttackAnimation,
            Timer::from_seconds(attack_animation_length, TimerMode::Once),
        );
        self.timers.insert(
            PlayerTimer::AttackAnimation2,
            Timer::from_seconds(attack_animation_length, TimerMode::Once),
        );
        self.timers.insert(
            PlayerTimer::HealingAnimation,
            Timer::from_seconds(healing_animation_length, TimerMode::Once),
        );
        self.timers.insert(
            PlayerTimer::HurtAnimation,
            Timer::from_seconds(hurt_animation_length, TimerMode::Once),
        );
        self.timers.insert(
            PlayerTimer::ParryAnimation,
            Timer::from_seconds(parry_animation_length, TimerMode::Once),
        );
        self.timers.insert(
            PlayerTimer::Parry,
            Timer::from_seconds(0.3, TimerMode::Once),
        );
        self.timers.insert(
            PlayerTimer::PerfectParry,
            Timer::from_seconds(0.15, TimerMode::Once),
        );
        self.timers.insert(
            PlayerTimer::Coyote,
            Timer::from_seconds(0.08, TimerMode::Once),
        );

        let mut hurtbox = self.base().get_node_as::<Hurtbox>("Hurtbox");
        hurtbox.bind_mut().attack_damage = self.stats.get(&Stats::AttackDamage).unwrap().0;

        self.animation_player.play_ex().name("idle_east").done();
    }

    // When a user provides input, execution of the relevant state function starts immediately.
    // This ensures that the `animation_state_changed` signal is emitted when an input changes the
    // state by eagerly checking the state machine's next state, just before it changes state.
    // Otherwise, `physics_process` handles emitting the signal.
    fn unhandled_input(&mut self, input: Gd<godot::classes::InputEvent>) {
        if input.is_action_pressed("attack") {
            self.state.handle(&Event::AttackButton);
            if self.state.state().as_descriminant() == csm::to_descriminant(&State::Attacking {}) {
                self.signals().animation_state_changed().emit();
            }
        }
        if input.is_action_pressed("jump") {
            self.state.handle(&Event::JumpButton);
            if self.state.state().as_descriminant() == csm::to_descriminant(&State::Jumping {}) {
                self.signals().animation_state_changed().emit();
            }
        }
        if input.is_action_released("jump") {
            self.timers.get_mut(&PT::JumpingAnimation).unwrap().reset();
            self.state.handle(&Event::ActionReleasedEarly);
        }
        if input.is_action_pressed("dodge")
            && self.get_dodging_cooldown_timer().get_time_left() <= 0.0
            && self.timers[&PT::DodgeAnimation].elapsed_secs() == 0.0
        {
            self.state.handle(&Event::DodgeButton);
            if self.state.state().as_descriminant() == csm::to_descriminant(&State::Dodging {}) {
                self.signals().animation_state_changed().emit();
            }
        }
        if input.is_action_pressed("heal") {
            self.state.handle(&Event::HealingButton);
            if self.state.state().as_descriminant() == csm::to_descriminant(&State::Healing {}) {
                self.signals().animation_state_changed().emit();
            }
        }
        if input.is_action_pressed("parry") {
            self.state.handle(&Event::ParryButton);
            if self.state.state().as_descriminant() == csm::to_descriminant(&State::Parry {}) {
                self.signals().animation_state_changed().emit();
            }
        }
    }

    fn physics_process(&mut self, delta: f32) {
        let delta = Duration::from_secs_f32(delta);
        let event;
        (event, self.velocity.x) = InputHandler::get_vel_and_event(&Input::singleton());

        // If we are in the moving state, update the animation each time the velocity changes.
        // Without this, the animation takes too long to update.
        if self.state.state().as_descriminant() == csm::to_descriminant(&State::Moving {})
            && self.prev_velocity.x != self.velocity.x
        {
            self.prev_velocity.x = self.velocity.x;
            self.signals().animation_state_changed().emit();
        }

        if !self.base().is_on_floor() {
            self.timers.get_mut(&PT::Coyote).unwrap().tick(delta);
            if self.timers.get(&PT::Coyote).unwrap().just_finished() {
                self.timers.get_mut(&PT::Coyote).unwrap().reset();
                self.state.handle(&Event::FailedFloorCheck);
                self.signals().animation_state_changed().emit();
            }
        }

        let prev_state = self.state.state().as_descriminant();
        match self.state.state() {
            csm::State::Attacking {} => self.attack(),
            csm::State::Attack2 {} => self.attack_2(),
            csm::State::Parry {} => self.parry(),
            csm::State::Idle {} => self.idle(),
            csm::State::Dodging {} => self.dodge(),
            csm::State::Jumping {} => self.jump(),
            csm::State::Falling {} => self.fall(),
            csm::State::Moving {} => self.move_character(),
            csm::State::Healing {} => self.heal(),
            csm::State::Grappling {} => self.grapple(),
            csm::State::Hurt {} => self.hurt(),
            csm::State::AirAttack {} => self.air_attack(),
        }
        self.state.handle(&event);
        // dbg!(self.state.state());
        let new_state = self.state.state().as_descriminant();

        // If the state machine changed states, the animation needs to change as well.
        if prev_state != new_state {
            self.signals().animation_state_changed().emit();
        }
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
            self.state.handle(&Event::Hurt);
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
        let delta = Duration::from_secs_f32(self.base().get_physics_process_delta_time() as f32);
        let mut cooldown_timer = self.get_dodging_cooldown_timer();
        let is_running =
            |timer: &Timer| !timer.paused() && !timer.finished() && timer.elapsed_secs() > 0.0;

        if is_running(&self.timers[&PT::DodgeAnimation]) {
            self.base_mut().move_and_slide();
            self.timers
                .get_mut(&PT::DodgeAnimation)
                .unwrap()
                .tick(delta);
        } else {
            let speed = self.stats.get(&Stats::DodgingSpeed).unwrap().0 as f32;
            let velocity = self.velocity;
            cooldown_timer.start();

            self.base_mut().set_velocity(velocity * speed);
            self.base_mut().move_and_slide();
            self.timers
                .get_mut(&PT::DodgeAnimation)
                .unwrap()
                .tick(delta);

            if self.timers[&PT::DodgeAnimation].finished() {
                self.timers.get_mut(&PT::DodgeAnimation).unwrap().reset();
                if self.velocity.x == 0.0 {
                    self.state.handle(&Event::MovingToIdle);
                } else {
                    self.state.handle(&Event::TimerElapsed);
                }
            }
        }
    }

    fn attack(&mut self) {
        // TODO: Maybe there is a better way of ignoring input. If the player is hit during an
        // attack, the state machine switches to 'hurt' state (as it should), but input handling is
        // never turned back on.
        self.base_mut().set_process_unhandled_input(false);
        let delta = Duration::from_secs_f32(self.base().get_physics_process_delta_time() as f32);

        let is_running =
            |timer: &Timer| !timer.paused() && !timer.finished() && timer.elapsed_secs() > 0.0;

        let mut h_shape = self
            .base()
            .get_node_as::<godot::classes::CollisionShape2D>("Hurtbox/HurtboxShape");

        if is_running(self.timers.get(&PlayerTimer::AttackAnimation).unwrap()) {
            h_shape.set_deferred("disabled", &true.to_variant());
            self.timers
                .get_mut(&PT::AttackAnimation)
                .unwrap()
                .tick(delta);

            if Input::singleton().is_action_pressed("parry") {
                self.timers.get_mut(&PT::AttackAnimation).unwrap().reset();
                self.base_mut().set_process_unhandled_input(true);
                self.state.handle(&Event::ParryButton);
            }

            if self.timers[&PT::AttackChain].remaining_secs() > 0.0 {
                if Input::singleton().is_action_just_pressed("attack") {
                    if self.hit_enemy {
                        self.can_attack_chain = true;
                        self.hit_enemy = false;
                    }
                } else {
                    self.timers.get_mut(&PT::AttackChain).unwrap().tick(delta);
                }
            }
        } else {
            h_shape.set_deferred("disabled", &false.to_variant());
            self.timers
                .get_mut(&PT::AttackAnimation)
                .unwrap()
                .tick(delta);
            self.timers.get_mut(&PT::AttackChain).unwrap().tick(delta);
        }
        if self.timers[&PT::AttackAnimation].just_finished() {
            self.base_mut().set_process_unhandled_input(true);
            h_shape.set_deferred("disabled", &true.to_variant());
            self.timers.get_mut(&PT::AttackAnimation).unwrap().reset();
            self.timers.get_mut(&PT::AttackChain).unwrap().reset();
            if self.can_attack_chain {
                self.can_attack_chain = false;
                self.hit_enemy = false;
                self.state.handle(&Event::AttackButton);
            } else {
                self.hit_enemy = false;
                if self.velocity.x == 0.0 {
                    self.state.handle(&Event::MovingToIdle);
                } else {
                    self.state.handle(&Event::TimerElapsed);
                }
            }
        }
    }

    fn attack_2(&mut self) {
        self.can_attack_chain = false;
        let delta = Duration::from_secs_f32(self.base().get_physics_process_delta_time() as f32);

        if self
            .timers
            .get(&PT::AttackAnimation2)
            .unwrap()
            .remaining_secs()
            > 0.0
        {
            self.timers
                .get_mut(&PT::AttackAnimation2)
                .unwrap()
                .tick(delta);
        } else {
            self.timers
                .get_mut(&PT::AttackAnimation2)
                .unwrap()
                .tick(delta);

            if self
                .timers
                .get(&PT::AttackAnimation2)
                .unwrap()
                .just_finished()
            {
                self.timers.get_mut(&PT::AttackAnimation2).unwrap().reset();
                self.state.handle(&Event::TimerElapsed);
            }
        }
    }

    fn air_attack(&mut self) {
        let time = self.base().get_physics_process_delta_time() as f32;
        let delta = Duration::from_secs_f32(self.base().get_physics_process_delta_time() as f32);

        if self.velocity.y <= self.terminal_y_speed {
            self.velocity.y += self.temp_gravity * time;
            self.velocity.x *= self.stats.get(&Stats::RunningSpeed).unwrap().0 as f32;

            let velocity = self.velocity;
            self.base_mut().set_velocity(velocity);
            self.base_mut().move_and_slide();
            self.timers
                .get_mut(&PT::AttackAnimation)
                .unwrap()
                .tick(delta);
        } else {
            self.base_mut().move_and_slide();
            self.timers
                .get_mut(&PT::AttackAnimation)
                .unwrap()
                .tick(delta);
        }

        if self.timers[&PT::AttackAnimation].just_finished() {
            self.timers.get_mut(&PT::AttackAnimation).unwrap().reset();
            self.state.handle(&Event::TimerElapsed);
        }
    }

    fn hurt(&mut self) {
        self.base_mut().set_process_unhandled_input(true);
        let delta = Duration::from_secs_f32(self.base().get_physics_process_delta_time() as f32);
        if self.timers[&PT::HurtAnimation].remaining_secs() > 0.0 {
            self.timers.get_mut(&PT::HurtAnimation).unwrap().tick(delta);
            self.base_mut().set_velocity(Vector2::ZERO);
        }

        if self.timers[&PT::HurtAnimation].just_finished() {
            self.timers.get_mut(&PT::HurtAnimation).unwrap().reset();
            self.state.handle(&Event::TimerElapsed);
        }
    }

    fn idle(&mut self) {
        self.active_velocity = Vector2::ZERO;
        if self.velocity.x != 0.0 {
            self.state.handle(&Event::Wasd);
        }
    }

    fn move_character(&mut self) {
        if self.velocity == Vector2::ZERO {
            self.state.handle(&Event::None);
        } else {
            let target_velocity =
                self.velocity * self.stats.get(&Stats::RunningSpeed).unwrap().0 as f32;
            self.active_velocity = self.active_velocity.lerp(target_velocity, 0.7);
            let velocity = self.active_velocity;

            self.base_mut().set_velocity(velocity);
            self.base_mut().move_and_slide();
        }
    }

    fn jump(&mut self) {
        // TODO: Use jumping speed player stat.
        self.velocity.y = Vector2::UP.y * self.temp_jump_speed;
        self.velocity.x *= self.stats.get(&Stats::RunningSpeed).unwrap().0 as f32;
        let velocity = self.velocity;
        self.base_mut().set_velocity(velocity);
        self.base_mut().move_and_slide();
    }

    fn heal(&mut self) {
        let current_health = self.stats.get(&Stats::Health).unwrap().0;
        let amount = self.stats.get(&Stats::HealAmount).unwrap().0;
        let max = self.stats.get(&Stats::MaxHealth).unwrap().0;
        let delta = Duration::from_secs_f32(self.base().get_physics_process_delta_time() as f32);
        self.velocity = Vector2::ZERO;
        let velocity = self.velocity;

        self.base_mut().set_velocity(velocity);
        self.timers
            .get_mut(&PT::HealingAnimation)
            .unwrap()
            .tick(delta);

        if self.timers[&PT::HealingAnimation].just_finished() {
            if current_health < max {
                self.stats.get_mut(&Stats::Health).unwrap().0 += amount;
                let new = self.stats.get(&Stats::Health).unwrap().0;
                self.signals()
                    .player_health_changed()
                    .emit(current_health, new, amount);
            }
            self.timers.get_mut(&PT::HealingAnimation).unwrap().reset();
            self.state.handle(&Event::TimerElapsed);
        }
    }

    fn fall(&mut self) {
        let delta = self.base().get_physics_process_delta_time() as f32;

        if self.velocity.y <= self.terminal_y_speed {
            self.velocity.y += self.temp_gravity * delta;
            self.velocity.x *= self.stats.get(&Stats::RunningSpeed).unwrap().0 as f32;
            let velocity = self.velocity;

            self.base_mut().set_velocity(velocity);
            self.base_mut().move_and_slide();
        } else {
            self.base_mut().move_and_slide();
            if Input::singleton().is_action_just_pressed("attack") {
                self.state.handle(&Event::AttackButton);
            }
        }

        if self.base().is_on_floor() {
            self.velocity.y = 0.0;
            if self.velocity.x == 0.0 {
                self.state.handle(&Event::MovingToIdle);
            } else {
                self.state.handle(&Event::OnFloor);
            }
            if self.timers[&PT::JumpingAnimation].finished() {
                self.timers.get_mut(&PT::JumpingAnimation).unwrap().reset();
            }
        }
    }

    fn parry(&mut self) {
        let delta = Duration::from_secs_f32(self.base().get_physics_process_delta_time() as f32);
        self.timers
            .get_mut(&PT::ParryAnimation)
            .unwrap()
            .tick(delta);
        self.timers.get_mut(&PT::Parry).unwrap().tick(delta);
        self.timers.get_mut(&PT::PerfectParry).unwrap().tick(delta);

        if self.timers[&PT::ParryAnimation].just_finished() {
            self.timers.get_mut(&PT::ParryAnimation).unwrap().reset();
            self.timers.get_mut(&PT::Parry).unwrap().reset();
            self.timers.get_mut(&PT::PerfectParry).unwrap().reset();
            self.state.handle(&Event::TimerElapsed);
        }
    }

    fn parried_attack(&mut self, area: &Gd<Hurtbox>) -> bool {
        match self.state.state() {
            State::Parry {} => {
                if self.timers.get(&PT::PerfectParry).unwrap().remaining_secs() > 0.0 {
                    println!("\nPERFECT PARRY\n");
                    if area.is_in_group("enemy_projectile")
                        && let Some(parent) = area.get_parent()
                        && let Ok(mut projectile) = parent.try_cast::<Projectile>()
                    {
                        projectile.bind_mut().on_parried();
                    }
                    self.signals().parried_attack().emit();
                    self.timers.get_mut(&PT::PerfectParry).unwrap().reset();
                    true
                } else if self.timers[&PT::Parry].remaining_secs() > 0.0 {
                    println!("\nNORMAL PARRY\n");
                    if area.is_in_group("enemy_projectile")
                        && let Some(parent) = area.get_parent()
                        && let Ok(mut projectile) = parent.try_cast::<Projectile>()
                    {
                        projectile.bind_mut().on_parried();
                    }
                    self.signals().parried_attack().emit();
                    self.timers.get_mut(&PT::Parry).unwrap().reset();
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn on_dodge_timer_timeout(&mut self) {
        self.timers.get_mut(&PT::DodgeAnimation).unwrap().reset();
    }

    fn on_animation_state_changed(&mut self) {
        self.update_animation();
    }

    fn get_animation_name(&self) -> String {
        let mut animation = self.state.state().to_string();
        animation.push('_');
        animation.push_str(self.direction.to_string().as_str());

        animation
    }

    fn update_animation(&mut self) {
        self.update_direction();
        let prev_anim = self.animation_player.get_current_animation().to_string();
        let next_anim = self.get_animation_name();
        let state = next_anim.split_once("_");
        if prev_anim != next_anim
            && let Some(s) = state
            && s.0 == self.state.state().to_string()
        {
            self.animation_player.play_ex().name(&next_anim).done();
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
