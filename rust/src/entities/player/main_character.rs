use std::{cell, collections, rc};

use godot::{
    classes::{
        Area2D, CharacterBody2D, ICharacterBody2D, Input, RayCast2D, Timer,
        timer::TimerProcessCallback,
    },
    obj::WithBaseField,
    prelude::*,
};
use statig::prelude::StateMachine;

use super::physics;
use crate::{
    entities::{
        damage::{CombatResources, Defense, Element, Health, Mana, Resistance, Stamina},
        enemies::projectile::Projectile,
        entity_stats::{EntityStats, Stat, StatModifier, StatVal},
        graphics::Graphics,
        hit_reg::{self, Hitbox, Hurtbox},
        movements::Direction,
        player::{
            abilities::AbilityComp,
            character_state_machine::{self as csm},
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
    previous_state: State,
    pub timers: collections::HashMap<PlayerTimer, Gd<Timer>>,
    pub state: StateMachine<csm::CharacterStateMachine>,

    pub stats: EntityStats,
    movements: physics::Movement,
    base: Base<CharacterBody2D>,

    #[init(val = OnReady::from_base_fn(|this| {
        hit_reg::HitReg::new(
            this.get_node_as::<Hitbox>("Hitbox"),
            this.get_node_as::<Hurtbox>("Hurtbox"),
        )
        }))]
    hit_reg: OnReady<hit_reg::HitReg>,

    #[init(node = "LeftWallCast")]
    left_wall_cast: OnReady<Gd<RayCast2D>>,
    #[init(node = "RightWallCast")]
    right_wall_cast: OnReady<Gd<RayCast2D>>,

    #[init(val = AbilityComp::new_test())]
    pub ability_comp: AbilityComp,

    #[init(node = "ItemComponent")]
    pub item_comp: OnReady<Gd<ItemComponent>>,

    #[init(val = OnReady::from_base_fn(|this|{ Graphics::new(this)}))]
    graphics: OnReady<Graphics>,

    #[init(node = "ShakyPlayerCamera")]
    pub camera: OnReady<Gd<PlayerCamera>>,

    #[init(val = Health::new(50, 50))]
    pub health: Health,

    #[init(val = Defense::new(vec![Resistance::Physical(5), Resistance::Elemental(Element::Fire, 10)]))]
    pub def: Defense,

    // statig SM lifetime support seems a bit limited, hence Rc<RefCell<T>>>. Could be user error
    // but may as well move on for now.
    #[init(val = rc::Rc::new(cell::RefCell::new(CombatResources::new(Stamina::new(30, 30), Mana::new(50, 50)))))]
    resources: rc::Rc<cell::RefCell<CombatResources>>,
}

#[godot_api]
impl ICharacterBody2D for MainCharacter {
    fn ready(&mut self) {
        self.movements.speeds = physics::Speeds {
            running: 180.0,
            jumping: 300.0,
            dodging: 250.0,
        };

        let hitbox = self.base().get_node_as::<Hitbox>("Hitbox");
        hitbox
            .signals()
            .area_entered()
            .connect_other(&self.to_gd(), Self::on_area_entered_hitbox);

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
    }

    fn physics_process(&mut self, delta: f32) {
        if let Ok(mut borrow) = self.resources.try_borrow_mut() {
            borrow.tick_resources(delta);
        }

        let frame = physics::PhysicsFrame::new(
            *self.state.state(),
            self.previous_state,
            self.base().is_on_floor(),
            self.base().is_on_floor_only(),
            self.base().is_on_wall_only(),
            self.left_wall_cast.is_colliding(),
            self.right_wall_cast.is_colliding(),
            delta,
        );
        let input = DevInputHandler::handle_unhandled(&Input::singleton(), self);

        if self.inputs != input {
            self.inputs = input;
            self.transition_sm(&Event::InputChanged(input));
        }

        if self.movements.not_on_floor(&frame) {
            self.transition_sm(&Event::FailedFloorCheck(input));
        }

        if self.movements.landed(&frame) {
            self.transition_sm(&Event::Landed(input));
        }

        if physics::Movement::wall_grab(&frame, &input) {
            self.transition_sm(&Event::GrabbedWall(input));
        }

        self.movements.apply_gravity(frame);
        self.movements.handle_acceleration(self.state.state());
        self.update_camera(self.movements.velocity);

        let v = self.movements.velocity;
        self.base_mut().set_velocity(v);
        self.base_mut().move_and_slide();

        if physics::hit_ceiling(&mut self.to_gd(), &mut self.movements) {
            self.transition_sm(&Event::HitCeiling(input));
        }
    }
}

#[godot_api]
impl MainCharacter {
    #[signal]
    pub fn player_health_changed(previous_health: u32, new_health: u32, damage_amount: u32);

    fn on_area_entered_hitbox(&mut self, area: Gd<Area2D>) {
        println!("Player Hitbox entered");
        let hurtbox = area.cast::<Hurtbox>();
        let attack = hurtbox.bind().attack.clone().unwrap();
        if attack.is_parryable() && self.parried() {
            if let Some(node) = hurtbox.get_parent()
                && let Ok(mut proj) = node.try_cast::<Projectile>()
            {
                proj.bind_mut().on_parried();
            }
        } else {
            let damage = self.def.apply_resistances(attack);
            self.health.take_damage(damage);
            self.camera
                .bind_mut()
                .add_trauma(TraumaLevel::from(damage.0));
            self.transition_sm(&Event::Hurt);
        }
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

    fn parried(&mut self) -> bool {
        if let State::ParryRight {} | State::ParryLeft {} = self.state.state() {
            if self.timers.get(&PT::PerfectParry).unwrap().get_time_left() > 0.0 {
                println!("Perfect parry");
                true
            } else if self.timers.get(&PT::Parry).unwrap().get_time_left() > 0.0 {
                println!("Normal parry");
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Transitions the state machine, checking and returning if the previous state is equal to the
    /// current state.
    pub fn transition_sm(&mut self, event: &Event) {
        let prev = *self.state.state();
        let mut context = csm::SMContext::new(
            self.timers.clone(),
            self.resources.clone(),
            self.hit_reg.hurtbox.clone(),
        );
        self.state.handle_with_context(event, &mut context);
        let new = *self.state.state();
        if prev != new {
            // TODO: Temporary solution. The direction isn't updated in time, so defer getting the
            // direction unti the velocity updates.
            self.run_deferred(|this| {
                this.graphics
                    .update(this.state.state(), &this.movements.get_direction())
            });
        }
    }

    /// Sets timer lengths, timer callbacks, and adds timers as children of the player.
    fn init_timers(&mut self) {
        let this = &self.to_gd();

        // Dodge animation
        let mut timer = Timer::new_alloc();
        timer.set_wait_time(self.graphics.get_animation_length("dodge_right"));
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
        timer.set_wait_time(self.graphics.get_animation_length("attack_right"));
        timer.set_one_shot(true);
        timer
            .signals()
            .timeout()
            .connect_other(this, Self::on_attack_timeout);
        self.timers.insert(PlayerTimer::AttackAnimation, timer);

        // Attack 2 animation
        let mut timer = Timer::new_alloc();
        timer.set_wait_time(self.graphics.get_animation_length("chainattack_right"));
        timer.set_one_shot(true);
        timer
            .signals()
            .timeout()
            .connect_other(this, Self::on_attack_2_timeout);
        self.timers.insert(PlayerTimer::Attack2Animation, timer);

        // Healing animation
        let mut timer = Timer::new_alloc();
        timer.set_wait_time(self.graphics.get_animation_length("heal_right"));
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
        timer.set_wait_time(self.graphics.get_animation_length("parry_right"));
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
        timer.set_wait_time(self.graphics.get_animation_length("hurt_right"));
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
        Direction::from_vel(&self.movements.velocity)
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
