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
            resources::{CombatResources, Heal, Health, Mana, ResourceChanged, Stamina},
        },
        enemies::projectile::Projectile,
        entity_stats::{EntityStats, Stat, StatModifier, StatVal},
        graphics::Graphics,
        hit_reg::{self, Hitbox, Hurtbox},
        movements::Direction,
        player::{
            character_state_machine::{self as csm, Timers},
            item_component::ItemComponent,
            shaky_player_camera::{PlayerCamera, TraumaLevel},
            time::PlayerTimers,
        },
    },
    utils::{
        global_data_singleton::GlobalData,
        input_hanlder::{DevInputHandler, InputHandler, Inputs},
        node_utils::ResetTimer,
    },
};

type State = csm::State;
type Event = csm::Event;

#[derive(GodotClass)]
#[class(init, base=CharacterBody2D)]
pub struct MainCharacter {
    inputs: Inputs,
    previous_state: State,
    pub state: StateMachine<csm::CharacterStateMachine>,
    pub stats: EntityStats,
    base: Base<CharacterBody2D>,

    #[init(val = OnReady::manual())]
    pub timer: OnReady<PlayerTimers>,
    #[init(val = physics::Movement::default())]
    movements: physics::Movement,
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
    #[init(val = CombatResources::new(
        Health::new(30, 30, Heal::new(5)), Stamina::new(30, 50), Mana::new(50, 50)))]
    pub resources: CombatResources,
}

#[godot_api]
impl ICharacterBody2D for MainCharacter {
    fn ready(&mut self) {
        self.movements.speeds = physics::Speeds {
            running: 180.0,
            jumping: 450.0,
            dodging: 800.0,
        };

        self.timer.init(PlayerTimers::new(
            &self.to_gd().upcast(),
            &mut self.graphics,
        ));

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
        let tick = self.resources.tick_resources(&delta);
        if let Ok(tick) = tick {
            match tick {
                ResourceChanged::Stamina { previous, new } => {
                    self.signals().stamina_changed().emit(previous, new)
                }
                ResourceChanged::Mana { previous, new } => {
                    self.signals().mana_changed().emit(previous, new)
                }
                ResourceChanged::Health { previous, new } => {
                    self.signals().player_health_changed().emit(previous, new)
                }
            }
        }

        // TODO: Get rid of this.
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

        if self.movements.not_on_floor(&frame) && self.state.state() != (&State::Jumping {}) {
            self.transition_sm(&Event::FailedFloorCheck(input));
        }

        if self.movements.landed(&frame) {
            self.timer.jump_limit.reset();
            self.transition_sm(&Event::Landed(input));
        }

        if physics::Movement::wall_grab(&frame, &input) {
            self.transition_sm(&Event::GrabbedWall(input));
        }

        if !matches!(self.state.state(), &State::WallGrab {} | &State::AirDash {}) {
            self.movements.apply_gravity(&frame);
        }

        let v = self.movements.velocity();
        self.update_camera(v);
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
    pub fn player_health_changed(previous: i64, new: i64);

    #[signal]
    pub fn stamina_changed(previous: i64, new: i64);

    #[signal]
    pub fn mana_changed(previous: i64, new: i64);

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
            let res = self.resources.take_damage(damage);
            self.signals().player_health_changed().emit(res.0, res.1);
            if self.resources.health().is_dead() {
                self.on_death();
            }
            self.camera
                .bind_mut()
                .add_trauma(TraumaLevel::from(damage.0));
            self.transition_sm(&Event::Hurt);
        }
    }

    fn on_parry_timeout(&mut self) {
        let input = InputHandler::handle(&Input::singleton(), self);
        self.transition_sm(&Event::TimerElapsed(Timers::ParryAnimation, input));
    }

    // TODO: Chain attacking.
    fn on_attack_timeout(&mut self) {
        let input = InputHandler::handle(&Input::singleton(), self);
        self.transition_sm(&Event::TimerElapsed(Timers::AttackAnimation, input));
    }

    fn on_attack_2_timeout(&mut self) {
        let input = InputHandler::handle(&Input::singleton(), self);
        self.transition_sm(&Event::TimerElapsed(Timers::Attack2Animation, input));
    }

    fn on_healing_timeout(&mut self) {
        let change = self.resources.heal();
        self.signals()
            .player_health_changed()
            .emit(change.0, change.1);
        let input = InputHandler::handle(&Input::singleton(), self);
        self.transition_sm(&Event::TimerElapsed(Timers::HealingAnimation, input));
    }

    fn on_dodge_animation_timeout(&mut self) {
        self.timer.dodge_cooldown.start();
        let input = InputHandler::handle(&Input::singleton(), self);
        self.transition_sm(&Event::TimerElapsed(Timers::DodgeAnimation, input));
    }

    fn on_hurt_animation_timeout(&mut self) {
        let input = InputHandler::handle(&Input::singleton(), self);
        self.transition_sm(&Event::TimerElapsed(Timers::HurtAnimation, input));
    }

    fn on_jump_limit_timeout(&mut self) {
        let input = InputHandler::handle(&Input::singleton(), self);
        self.transition_sm(&Event::TimerElapsed(Timers::JumpLimit, input));
    }

    fn on_charged_att_anim_timeout(&mut self) {
        let input = InputHandler::handle(&Input::singleton(), self);
        self.transition_sm(&Event::TimerElapsed(Timers::ChargedAttack, input));
    }

    fn on_cast_spell_anim_timeout(&mut self) {
        let input = InputHandler::handle(&Input::singleton(), self);
        self.transition_sm(&Event::TimerElapsed(Timers::CastSpellAnimation, input));
    }

    fn parried(&mut self) -> bool {
        if let State::Parry {} = self.state.state() {
            if self.timer.perfect_parry.get_time_left() > 0.0 {
                println!("Perfect parry");
                true
            } else if self.timer.parry.get_time_left() > 0.0 {
                println!("Normal parry");
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn transition_sm(&mut self, event: &Event) {
        let mut context = csm::SMContext::new(
            &mut self.timer,
            &mut self.resources,
            self.hit_reg.hurtbox.clone(),
            &self.off,
            &mut self.movements,
            &mut self.graphics,
        );
        self.state.handle_with_context(event, &mut context);
        self.graphics
            .update(self.state.state(), &self.movements.get_direction());
    }

    /// Sets timer lengths, timer callbacks, and adds timers as children of the player.
    fn init_timers(&mut self) {
        let this = &self.to_gd();
        self.timer.connect_signals(
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
        if previous_velocity != self.movements.velocity() {
            if self.movements.velocity().x > 5.0 {
                self.camera.bind_mut().set_right(Some(true));
            } else if self.movements.velocity().x < -5.0 {
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
        self.movements.get_direction()
    }

    fn on_death(&mut self) {
        self.base_mut().queue_free();
    }
}
