use std::collections::HashMap;

use godot::{
    classes::{
        AnimationPlayer, Area2D, CharacterBody2D, ICharacterBody2D, Input, RayCast2D, Timer,
        timer::TimerProcessCallback,
    },
    obj::WithBaseField,
    prelude::*,
};
use statig::prelude::StateMachine;

use crate::{
    entities::{
        damage::{AttackData, Damage, DamageType, Damageable, HasHealth},
        enemies::projectile::Projectile,
        ent_physics::{self, Movement, PhysicsFrameData, Speeds},
        entity_stats::{EntityStats, Stat, StatModifier, StatVal},
        hit_reg::{self, Hitbox, Hurtbox},
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

#[derive(GodotClass)]
#[class(init, base=CharacterBody2D)]
pub struct MainCharacter {
    inputs: Inputs,
    pub previous_state: State,
    pub timers: HashMap<PlayerTimer, Gd<Timer>>,
    pub state: StateMachine<csm::CharacterStateMachine>,
    pub stats: EntityStats,
    movements: Movement,
    base: Base<CharacterBody2D>,

    #[init(val = OnReady::from_base_fn(|this| {
        hit_reg::HitReg { hitbox: this.get_node_as::<Hitbox>("Hitbox"), 
            hurtbox: this.get_node_as::<Hurtbox>("Hurtbox"), 
            left_wall_cast: Some(this.get_node_as::<RayCast2D>("LeftWallSensor")), 
            right_wall_cast: Some(this.get_node_as::<RayCast2D>("RightWallSensor")) 
        }}))]
    hit_reg: OnReady<hit_reg::HitReg>,

    #[init(val = AbilityComp::new_test())]
    pub ability_comp: AbilityComp,

    #[init(node = "ItemComponent")]
    pub item_comp: OnReady<Gd<ItemComponent>>,

    #[init(node = "AnimationPlayer")]
    animation_player: OnReady<Gd<AnimationPlayer>>,

    #[init(node = "ShakyPlayerCamera")]
    pub camera: OnReady<Gd<PlayerCamera>>,
}

#[godot_api]
impl ICharacterBody2D for MainCharacter {
    fn ready(&mut self) {
        self.movements.speeds = Speeds {
            running: 180.0,
            jumping: 300.0,
            dodging: 250.0,
        };

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
            (Stat::Level, StatVal::new(1)),
        ]);

        self.init_timers();

        self.previous_state = State::IdleRight {};

        self.hit_reg.hitbox.bind_mut().damageable_parent = Some(Box::new(self.to_gd()));
        self.hit_reg.hurtbox.bind_mut().data = Some(AttackData {
            hurtbox: self.hit_reg.hurtbox.clone(),
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
            self.transition_sm(&Event::InputChanged(input));
        }
        let frame = self.new_frame();
        let prev_vel = self.movements.velocity;

        if !self.base().is_on_floor() && self.movements.velocity.y.is_sign_positive() {
            let input = InputHandler::handle(&Input::singleton(), self);
            self.transition_sm(&Event::FailedFloorCheck(input));
        }

        if self
            .movements
            .landed(self.to_gd(), self.state.state(), &self.previous_state)
        {
            self.transition_sm(&Event::Landed(Inputs(
                InputHandler::get_movement(&Input::singleton()).0,
                None,
            )));
        }

        self.wall_grab();
        dbg!(self.state.state());

        self.movements
            .apply_gravity(self.base().is_on_floor_only(), &delta);
        self.movements
            .handle_acceleration(self.state.state(), frame);
        self.update_camera(prev_vel);

        let v = self.movements.velocity;
        self.base_mut().set_velocity(v);
        self.base_mut().move_and_slide();

        // dbg!(self.state.state());
        // dbg!(&self.movements.velocity);

        if ent_physics::hit_ceiling(&mut self.to_gd(), &mut self.movements) {
            let input = InputHandler::handle(&Input::singleton(), self);
            self.transition_sm(&Event::HitCeiling(input));
        }
    }
}

#[godot_api]
impl MainCharacter {
    #[signal]
    pub fn player_health_changed(previous_health: u32, new_health: u32, damage_amount: u32);

    #[signal]
    fn parried_attack();

    fn new_frame(&self) -> PhysicsFrameData {
        let guard = self.base();
        PhysicsFrameData::new(
            *self.state.state(),
            self.movements.velocity,
            guard.is_on_floor(),
            guard.is_on_floor_only(),
            guard.is_on_wall(),
            guard.is_on_wall_only(),
            guard.is_on_ceiling(),
            guard.is_on_ceiling_only(),
            guard.get_physics_process_delta_time() as f32,
        )
    }

    fn wall_grab(&mut self) {
        if self.base().is_on_wall_only()
            && !matches!(
                self.state.state(),
                State::WallGrabLeft {} | State::WallGrabRight {}
            )
        {
            let left = self.hit_reg.left_wall_cast.as_ref().unwrap().is_colliding();
            let right = self
                .hit_reg
                .right_wall_cast
                .as_ref()
                .unwrap()
                .is_colliding();
            let input = InputHandler::handle(&Input::singleton(), self);
            match input.0 {
                Some(crate::utils::input_hanlder::MoveButton::Left) => {
                    if left {
                        self.transition_sm(&Event::GrabbedWall(input));
                    }
                }
                Some(crate::utils::input_hanlder::MoveButton::Right) => {
                    if right {
                        self.transition_sm(&Event::GrabbedWall(input));
                    }
                }
                _ => (),
            }
        }
    }

    // TODO: Finish this and remove parried signal.
    fn on_area_entered_hitbox(&mut self, area: Gd<Area2D>) {
        let hurtbox = area.cast::<Hurtbox>();
        // let data = hurtbox.bind().data.unwrap();
        if hurtbox.bind().data.as_ref().unwrap().parryable && self.parried_attack(&hurtbox) {}
        // if let Ok(h_box) = &area.try_cast::<Hurtbox>()
        //     && !self.parried_attack(h_box)
        // {
        //     self.timers.get_mut(&PT::HurtAnimation).unwrap().start();
        //     let damaging =
        //         DynGd::<Area2D, dyn Damaging>::from_godot(h_box.clone().upcast::<Area2D>());
        //     let target = self.to_gd().upcast::<Node2D>();
        //     let guard = self.base_mut();
        //     let damageable = DynGd::<Node2D, dyn Damageable>::from_godot(target);
        //     damaging.dyn_bind().do_damage(damageable);
        //     drop(guard);
        //     let mut camera = self.base().get_node_as::<PlayerCamera>("ShakyPlayerCamera");
        //     camera
        //         .bind_mut()
        //         .add_trauma(TraumaLevel::from(damaging.dyn_bind().damage_amount()));
        //     self.transition_sm(&Event::Hurt);
        // }
    }

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
            .animation_player
            .get_animation("dodge_right")
            .unwrap()
            .get_length()
            / 1.5) as f64;

        let attack_animation_length = self
            .animation_player
            .get_animation("attack_right")
            .unwrap()
            .get_length() as f64;

        let attack_2_animation_length = self
            .animation_player
            .get_animation("chainattack_right")
            .unwrap()
            .get_length() as f64;

        let healing_animation_length = self
            .animation_player
            .get_animation("heal_right")
            .unwrap()
            .get_length() as f64;

        let parry_animation_length = self
            .animation_player
            .get_animation("parry_right")
            .unwrap()
            .get_length() as f64;

        let hurt_animation_length = self
            .animation_player
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

        // Wall jump limit
        let mut timer = Timer::new_alloc();
        timer.set_wait_time(0.1);
        timer.set_one_shot(true);
        timer.set_timer_process_callback(TimerProcessCallback::PHYSICS);
        timer
            .signals()
            .timeout()
            .connect_other(this, Self::on_jump_limit_timeout);
        self.timers.insert(PlayerTimer::WallJumpLimit, timer);

        let mut pt = self.timers.clone();
        pt.values_mut().for_each(|timer| {
            timer.set_one_shot(true);
            self.base_mut().add_child(&timer.clone());
        });
    }

    fn update_camera(&mut self, previous_velocity: Vector2) {
        if previous_velocity != self.movements.velocity {
            if self.movements.velocity.x > 5.0 {
                self.camera.bind_mut().set_right(Some(true));
            } else if self.movements.velocity.x < -5.0 {
                self.camera.bind_mut().set_right(Some(false));
            }
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
