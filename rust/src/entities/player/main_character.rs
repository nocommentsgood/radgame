use std::{
    cell::{self, RefCell},
    rc::{self, Rc},
};

use godot::{
    classes::{Area2D, CharacterBody2D, ICharacterBody2D, Input, RayCast2D},
    obj::WithBaseField,
    prelude::*,
};
use statig::prelude::StateMachine;

use super::physics;
use crate::{
    entities::{
        combat::{
            defense::{Defense, Resistance},
            offense::{Buff, Element, Offense, Spell},
            resources::{CombatResources, Heal, Health, Mana, Stamina},
        },
        enemies::projectile::Projectile,
        entity_stats::{EntityStats, Stat, StatModifier, StatVal},
        graphics::Graphics,
        hit_reg::{self, Hitbox, Hurtbox},
        movements::Direction,
        player::{
            character_state_machine::{self as csm},
            item_component::ItemComponent,
            shaky_player_camera::{PlayerCamera, TraumaLevel},
            time::PlayerTimers,
        },
    },
    utils::{
        global_data_singleton::GlobalData,
        input_hanlder::{DevInputHandler, InputHandler, Inputs},
    },
};

type State = csm::State;
type Event = csm::Event;

#[derive(GodotClass)]
#[class(init, base=CharacterBody2D)]
pub struct MainCharacter {
    inputs: Inputs,
    previous_state: State,

    #[init(val = OnReady::manual())]
    pub timer: OnReady<rc::Rc<cell::RefCell<PlayerTimers>>>,

    pub state: StateMachine<csm::CharacterStateMachine>,

    pub stats: EntityStats,

    #[init(val = Rc::new(RefCell::new(physics::Movement::default())))]
    movements: rc::Rc<cell::RefCell<physics::Movement>>,
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

    #[init(node = "ItemComponent")]
    pub item_comp: OnReady<Gd<ItemComponent>>,

    #[init(val = OnReady::from_base_fn(|this|{ Graphics::new(this)}))]
    graphics: OnReady<Graphics>,

    #[init(node = "ShakyPlayerCamera")]
    pub camera: OnReady<Gd<PlayerCamera>>,

    #[init(val = Offense::new(
        vec![Buff::Physical(2)],
        [Some(Spell::ProjectileSpell), Some(Spell::TwinPillar), None],
        ))]
    off: Offense,

    #[init(val = Defense::new(vec![Resistance::Physical(5), Resistance::Elemental(Element::Fire, 10)]))]
    pub def: Defense,

    // statig SM lifetime support seems a bit limited, hence Rc<RefCell<T>>>. Could be user error
    // but may as well move on for now.
    #[init(val = rc::Rc::new(cell::RefCell::new(CombatResources::new(
        Health::new(30, 30, Heal::new(5)), Stamina::new(30, 30), Mana::new(50, 50)))))]
    resources: rc::Rc<cell::RefCell<CombatResources>>,
}

#[godot_api]
impl ICharacterBody2D for MainCharacter {
    fn ready(&mut self) {
        self.movements.borrow_mut().speeds = physics::Speeds {
            running: 180.0,
            jumping: 300.0,
            dodging: 250.0,
        };

        self.timer
            .init(rc::Rc::new(cell::RefCell::new(PlayerTimers::new(
                &self.to_gd().upcast(),
                &mut self.graphics,
            ))));

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

        self.previous_state = State::Idle {};
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

        if self.movements.borrow().not_on_floor(&frame) {
            self.transition_sm(&Event::FailedFloorCheck(input));
        }

        if self.movements.borrow_mut().landed(&frame) {
            self.transition_sm(&Event::Landed(input));
        }

        if physics::Movement::wall_grab(&frame, &input) {
            self.transition_sm(&Event::GrabbedWall(input));
        }

        self.movements.borrow_mut().apply_gravity(frame);
        // self.movements
        //     .borrow_mut()
        //     .handle_acceleration(self.state.state());
        // self.update_camera(self.movements.borrow().velocity);

        let v = self.movements.borrow().velocity;
        self.update_camera(v);
        self.base_mut().set_velocity(v);
        self.base_mut().move_and_slide();

        if physics::hit_ceiling(&mut self.to_gd(), &mut self.movements.borrow_mut()) {
            self.transition_sm(&Event::HitCeiling(input));
        }
    }
}

#[godot_api]
impl MainCharacter {
    #[signal]
    pub fn player_health_changed(previous_health: u32, new_health: u32, damage_amount: u32);

    fn on_area_entered_hitbox(&mut self, area: Gd<Area2D>) {
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
            self.resources.borrow_mut().take_damage(damage);
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
        self.timer.borrow_mut().healing_cooldown.start();
        let input = InputHandler::handle(&Input::singleton(), self);
        self.transition_sm(&Event::TimerElapsed(input));
    }

    fn on_dodge_animation_timeout(&mut self) {
        self.timer.borrow_mut().dodge_cooldown.start();
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

    fn on_charged_att_anim_timeout(&mut self) {
        let input = InputHandler::handle(&Input::singleton(), self);
        self.transition_sm(&Event::TimerElapsed(input));
    }

    fn on_cast_spell_anim_timeout(&mut self) {
        let input = InputHandler::handle(&Input::singleton(), self);
        self.transition_sm(&Event::TimerElapsed(input));
    }

    fn parried(&mut self) -> bool {
        if let State::Parry {} = self.state.state() {
            if self.timer.borrow_mut().perfect_parry.get_time_left() > 0.0 {
                println!("Perfect parry");
                true
            } else if self.timer.borrow_mut().parry.get_time_left() > 0.0 {
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
        let mut context = csm::SMContext::new(
            self.timer.clone(),
            self.resources.clone(),
            self.hit_reg.hurtbox.clone(),
            self.off.clone(),
            self.movements.clone(),
        );
        self.state.handle_with_context(event, &mut context);
        self.graphics.update(
            self.state.state(),
            &self.movements.borrow_mut().get_direction(),
        );
    }

    /// Sets timer lengths, timer callbacks, and adds timers as children of the player.
    fn init_timers(&mut self) {
        let this = &self.to_gd();
        self.timer.borrow_mut().connect_signals(
            {
                let mut this = this.clone();
                move || this.bind_mut().on_jump_limit_timeout()
            },
            {
                let mut this = this.clone();
                move || this.bind_mut().on_dodge_animation_timeout()
            },
            {
                let mut this = this.clone();
                move || this.bind_mut().on_attack_timeout()
            },
            {
                let mut this = this.clone();
                move || this.bind_mut().on_attack_2_timeout()
            },
            {
                let mut this = this.clone();
                move || this.bind_mut().on_healing_timeout()
            },
            {
                let mut this = this.clone();
                move || this.bind_mut().on_hurt_animation_timeout()
            },
            {
                let mut this = this.clone();
                move || this.bind_mut().on_parry_timeout()
            },
            {
                let mut this = this.clone();
                move || this.bind_mut().on_jump_limit_timeout()
            },
            {
                let mut this = this.clone();
                move || this.bind_mut().on_charged_att_anim_timeout()
            },
            {
                let mut this = this.clone();
                move || this.bind_mut().on_cast_spell_anim_timeout()
            },
        );
    }

    fn update_camera(&mut self, previous_velocity: Vector2) {
        if previous_velocity != self.movements.borrow().velocity {
            if self.movements.borrow().velocity.x > 5.0 {
                self.camera.bind_mut().set_right(Some(true));
            } else if self.movements.borrow().velocity.x < -5.0 {
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

    /// Transitions state machine from it's current state to `disabled`.
    /// Effectively disables input handling.
    pub fn force_disabled(&mut self) {
        self.transition_sm(&csm::Event::ForceDisabled);
    }

    /// Transitions state machine from `disabled` to `idle`.
    pub fn force_enabled(&mut self) {
        self.transition_sm(&csm::Event::ForceEnabled);
    }

    pub fn get_direction(&mut self) -> Direction {
        self.movements.borrow_mut().get_direction()
    }
}
