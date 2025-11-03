use std::{cell::RefCell, rc::Rc};

use godot::obj::Gd;
use statig::blocking::*;

use crate::{
    entities::{
        combat::{
            offense::{HotSpellIndexer, Offense, PlayerAttacks},
            resources::CombatResources,
        },
        graphics::Graphics,
        hit_reg::Hurtbox,
        movements::Direction,
        player::{physics::Movement, time::PlayerTimers},
    },
    utils::{
        global_data_singleton::GlobalData,
        input_hanlder::{Inputs, ModifierButton, MoveButton},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// Prevents the SM from switching states due to irrelavent timers emitting their `timeout` signal.
pub enum Timers {
    DodgeAnimation,
    AttackAnimation,
    Attack2Animation,
    HealingAnimation,
    HurtAnimation,
    ParryAnimation,
    JumpLimit,
    ChargedAttack,
    CastSpellAnimation,
}

pub struct SMContext {
    timers: Rc<RefCell<PlayerTimers>>,
    resources: Rc<RefCell<CombatResources>>,
    hurtbox: Gd<Hurtbox>,
    off: Offense,
    movement: Rc<RefCell<Movement>>,
    graphics: Rc<RefCell<Graphics>>,
}

impl SMContext {
    pub fn new(
        timers: Rc<RefCell<PlayerTimers>>,
        resources: Rc<RefCell<CombatResources>>,
        hurtbox: Gd<Hurtbox>,
        off: Offense,
        movement: Rc<RefCell<Movement>>,
        graphics: Rc<RefCell<Graphics>>,
    ) -> Self {
        Self {
            timers,
            resources,
            hurtbox,
            off,
            movement,
            graphics,
        }
    }
}
#[derive(Default, Debug, Clone)]
pub struct CharacterStateMachine;

// Animation player uses the implementation of `Display` for animation names.
impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Idle {} | State::WallGrab {} | State::ForcedDisabled {} => write!(f, "idle"),
            State::Run {} => write!(f, "run"),
            State::Dodging {} => write!(f, "dodge"),
            State::Jumping {} => write!(f, "jumping"),
            State::Falling {} => write!(f, "falling"),
            State::Attacking {} => write!(f, "attack"),
            State::Chargedattack {} => write!(f, "chargedattack"),
            State::ChainAttack {} => write!(f, "chainattack"),
            State::Hurt {} => write!(f, "hurt"),
            State::Healing {} => write!(f, "heal"),
            State::Parry {} => write!(f, "parry"),
            State::CastSpell {} => write!(f, "cast_spell"),
            State::AirDash {} => write!(f, "air_dash"),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        State::Idle {}
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub enum Event {
    InputChanged(Inputs),
    TimerElapsed(Timers, Inputs),
    FailedFloorCheck(Inputs),
    Landed(Inputs),
    HitCeiling(Inputs),
    GrabbedWall(Inputs),
    Hurt,
    ForceDisabled,
    ForceEnabled,
    #[default]
    None,
}

#[state_machine(
    initial = "State::idle()",
    state(derive(Debug, Clone, PartialEq, Copy))
)]
impl CharacterStateMachine {
    #[state]
    fn idle(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::InputChanged(inputs) => {
                if let Ok(res) = Self::try_dodging(inputs, context) {
                    return res;
                }
                if let Ok(res) = Self::try_jumping(inputs, context) {
                    return res;
                }
                if let Ok(res) = Self::try_healing(inputs, context) {
                    return res;
                }
                if let Ok(res) = Self::try_attacking(inputs, context) {
                    return res;
                }
                if let Ok(res) = Self::try_casting_spell(inputs, context) {
                    return res;
                }
                if let Ok(res) = Self::try_charged_attack(inputs, context) {
                    return res;
                }
                if let Ok(res) = Self::try_parry(inputs, context) {
                    res
                } else {
                    Self::to_moving(inputs, context)
                }
            }
            Event::FailedFloorCheck(inputs) => Self::to_falling(inputs, context),
            Event::Hurt => {
                context.timers.borrow_mut().hurt_anim.start();
                Response::Transition(State::hurt())
            }
            Event::ForceDisabled => Response::Transition(State::forced_disabled()),
            _ => Handled,
        }
    }

    #[state]
    fn run(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::InputChanged(inputs) => {
                if let Ok(res) = Self::try_dodging(inputs, context) {
                    return res;
                }
                if let Ok(res) = Self::try_jumping(inputs, context) {
                    return res;
                }
                if let Ok(res) = Self::try_healing(inputs, context) {
                    return res;
                }
                if let Ok(res) = Self::try_attacking(inputs, context) {
                    return res;
                }
                if let Ok(res) = Self::try_casting_spell(inputs, context) {
                    return res;
                }
                if let Ok(res) = Self::try_charged_attack(inputs, context) {
                    return res;
                }
                if let Ok(res) = Self::try_parry(inputs, context) {
                    res
                } else {
                    Self::to_moving(inputs, context)
                }
            }
            Event::FailedFloorCheck(inputs) => Self::to_falling(inputs, context),
            Event::Hurt => Response::Transition(State::hurt()),
            Event::ForceDisabled => Response::Transition(State::forced_disabled()),
            _ => Handled,
        }
    }

    #[state]
    fn dodging(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::TimerElapsed(timer, inputs) if *timer == Timers::DodgeAnimation => {
                Self::to_moving(inputs, context)
            }
            Event::FailedFloorCheck(inputs) => Self::to_falling(inputs, context),
            Event::ForceDisabled => Response::Transition(State::forced_disabled()),
            _ => Handled,
        }
    }

    #[state]
    fn jumping(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::GrabbedWall(inputs) => Self::handle_wall_grab(inputs, context),
            Event::InputChanged(inputs) => {
                if let Ok(res) = Self::try_air_dash(inputs, context) {
                    res
                } else if let Ok(res) = Self::try_airborne_attack(inputs, context) {
                    res
                } else {
                    match (&inputs.0, &inputs.1) {
                        (Some(MoveButton::Left), Some(ModifierButton::Jump)) => {
                            context.movement.borrow_mut().jump_left();
                            Handled
                        }
                        (Some(MoveButton::Right), Some(ModifierButton::Jump)) => {
                            context.movement.borrow_mut().jump_right();
                            Handled
                        }
                        (_, Some(ModifierButton::Jump)) => {
                            context.movement.borrow_mut().jump();
                            Handled
                        }
                        _ => Self::to_falling(inputs, context),
                    }
                }
            }
            Event::TimerElapsed(timer, inputs) if *timer == Timers::JumpLimit => {
                Self::to_falling(inputs, context)
            }
            Event::Landed(inputs) => Self::to_moving(inputs, context),
            Event::HitCeiling(inputs) => Self::to_falling(inputs, context),
            Event::ForceDisabled => Response::Transition(State::forced_disabled()),
            _ => Handled,
        }
    }

    #[state]
    fn falling(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::GrabbedWall(inputs) => Self::handle_wall_grab(inputs, context),
            Event::InputChanged(inputs) => {
                if let Ok(res) = Self::try_air_dash(inputs, context) {
                    res
                } else if let Ok(res) = Self::try_airborne_attack(inputs, context) {
                    res
                } else {
                    Self::handled_movement_input(inputs, context)
                }
            }
            Event::Landed(inputs) => Self::to_moving(inputs, context),
            Event::ForceDisabled => Response::Transition(State::forced_disabled()),
            _ => Handled,
        }
    }

    #[state]
    fn attacking(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::TimerElapsed(timer, inputs) if *timer == Timers::AttackAnimation => {
                match (&inputs.0, &inputs.1) {
                    (_, Some(ModifierButton::Attack))
                        if context.timers.borrow().attack_2_anim.get_time_left() == 0.0
                            && let Ok(attack) = Offense::try_attack(
                                PlayerAttacks::ChargedMelee,
                                &mut context.resources.borrow_mut(),
                                1,
                            ) =>
                    {
                        context.hurtbox.bind_mut().set_attack(attack);
                        context.timers.borrow_mut().attack_2_anim.start();
                        Response::Transition(State::chain_attack())
                    }
                    _ => Self::to_moving(inputs, context),
                }
            }
            Event::InputChanged(inputs) => Self::handled_movement_input(inputs, context),
            Event::Landed(inputs) => Self::to_moving(inputs, context),
            Event::Hurt => Response::Transition(State::hurt()),
            Event::ForceDisabled => Response::Transition(State::forced_disabled()),
            _ => Handled,
        }
    }

    #[state]
    fn chargedattack(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::TimerElapsed(timer, inputs) if *timer == Timers::ChargedAttack => {
                Self::to_moving(inputs, context)
            }
            _ => Handled,
        }
    }

    #[state]
    fn chain_attack(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::TimerElapsed(timer, inputs) if *timer == Timers::Attack2Animation => {
                Self::to_moving(inputs, context)
            }
            Event::Hurt => Response::Transition(State::hurt()),
            Event::ForceDisabled => Response::Transition(State::forced_disabled()),
            _ => Handled,
        }
    }

    #[state]
    fn hurt(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::TimerElapsed(timer, inputs) if *timer == Timers::HurtAnimation => {
                Self::to_moving(inputs, context)
            }
            Event::ForceDisabled => Response::Transition(State::forced_disabled()),
            _ => Handled,
        }
    }

    #[state]
    fn healing(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::TimerElapsed(timer, inputs) if *timer == Timers::HealingAnimation => {
                context.resources.borrow_mut().heal();
                Self::to_moving(inputs, context)
            }
            Event::ForceDisabled => Response::Transition(State::forced_disabled()),
            _ => Handled,
        }
    }

    #[state]
    fn parry(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::TimerElapsed(timer, inputs) if *timer == Timers::ParryAnimation => {
                Self::to_moving(inputs, context)
            }
            Event::ForceDisabled => Response::Transition(State::forced_disabled()),
            _ => Handled,
        }
    }

    #[state]
    fn forced_disabled(event: &Event) -> Response<State> {
        match event {
            Event::ForceEnabled => Response::Transition(State::idle()),
            _ => Handled,
        }
    }

    #[state]
    fn wall_grab(event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::InputChanged(inputs) => match (&inputs.0, inputs.1) {
                (Some(MoveButton::Left), Some(ModifierButton::Jump))
                    if context.timers.borrow().wall_jump.get_time_left() == 0.0 =>
                {
                    context.timers.borrow_mut().wall_jump.start();
                    context.movement.borrow_mut().jump_right();
                    Response::Transition(State::jumping())
                }
                (Some(MoveButton::Right), Some(ModifierButton::Jump))
                    if context.timers.borrow().wall_jump.get_time_left() == 0.0 =>
                {
                    context.timers.borrow_mut().wall_jump.start();
                    context.movement.borrow_mut().jump_left();
                    Response::Transition(State::jumping())
                }
                (Some(MoveButton::Right), _) => Response::Transition(State::falling()),
                _ => Self::to_falling(inputs, context),
            },
            Event::Landed(inputs) => Self::to_moving(inputs, context),
            Event::Hurt => Response::Transition(State::falling()),
            _ => Handled,
        }
    }

    #[state]
    fn cast_spell(event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::InputChanged(inputs) => Self::to_moving(inputs, context),
            _ => Handled,
        }
    }

    #[state]
    fn air_dash(event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::GrabbedWall(inputs) => Self::handle_wall_grab(inputs, context),
            Event::TimerElapsed(timer, inputs) if *timer == Timers::DodgeAnimation => {
                Self::to_falling(inputs, context)
            }
            Event::Landed(inputs) => Self::to_moving(inputs, context),
            _ => Handled,
        }
    }

    fn try_cast_spell(context: &mut SMContext, hot_spell_index: HotSpellIndexer) -> Result<(), ()> {
        let spell = context.off.get_spell(hot_spell_index);
        if let Some(spell) = spell {
            let mut attack = spell.attack(1);
            if Offense::check_resources(attack.cost(), &mut context.resources.borrow_mut()).is_ok()
            {
                let scene = spell.init_scene();
                context.off.apply_buffs(&mut attack);
                GlobalData::singleton()
                    .bind_mut()
                    .get_player_mut()
                    .unwrap()
                    .add_sibling(&scene);
                context.timers.borrow_mut().spell_cooldown.start();
                Ok(())
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
    fn try_casting_spell(inputs: &Inputs, context: &mut SMContext) -> Result<Response<State>, ()> {
        match (&inputs.0, &inputs.1) {
            (_, Some(ModifierButton::Ability1))
                if context.timers.borrow().spell_cooldown.get_time_left() == 0.0
                    && Self::try_cast_spell(context, HotSpellIndexer::Ability1).is_ok() =>
            {
                context.timers.borrow_mut().spell_cooldown.start();
                context.timers.borrow_mut().cast_spell_anim.start();
                Ok(Response::Transition(State::cast_spell()))
            }
            (_, Some(ModifierButton::Ability2))
                if context.timers.borrow().spell_cooldown.get_time_left() == 0.0
                    && Self::try_cast_spell(context, HotSpellIndexer::Ability2).is_ok() =>
            {
                context.timers.borrow_mut().spell_cooldown.start();
                context.timers.borrow_mut().cast_spell_anim.start();
                Ok(Response::Transition(State::cast_spell()))
            }
            (_, Some(ModifierButton::Ability3))
                if context.timers.borrow().spell_cooldown.get_time_left() == 0.0
                    && Self::try_cast_spell(context, HotSpellIndexer::Ability3).is_ok() =>
            {
                context.timers.borrow_mut().spell_cooldown.start();
                context.timers.borrow_mut().cast_spell_anim.start();
                Ok(Response::Transition(State::cast_spell()))
            }
            _ => Err(()),
        }
    }

    fn try_dodging(inputs: &Inputs, context: &mut SMContext) -> Result<Response<State>, ()> {
        match (&inputs.0, &inputs.1) {
            (Some(MoveButton::Left), Some(ModifierButton::Dodge))
                if context.timers.borrow().dodge_cooldown.get_time_left() == 0.0 =>
            {
                context.timers.borrow_mut().dodge_cooldown.start();
                context.movement.borrow_mut().dodge_left();
                Ok(Response::Transition(State::dodging()))
            }
            (Some(MoveButton::Right), Some(ModifierButton::Dodge))
                if context.timers.borrow().dodge_cooldown.get_time_left() == 0.0 =>
            {
                context.timers.borrow_mut().dodge_cooldown.start();
                context.movement.borrow_mut().dodge_right();
                Ok(Response::Transition(State::dodging()))
            }
            (None, Some(ModifierButton::Dodge))
                if context.timers.borrow().dodge_cooldown.get_time_left() == 0.0 =>
            {
                let dir = context.movement.borrow_mut().get_direction();
                match dir {
                    Direction::Right => context.movement.borrow_mut().dodge_right(),
                    Direction::Left => context.movement.borrow_mut().dodge_left(),
                }
                context.timers.borrow_mut().dodge_cooldown.start();
                context.timers.borrow_mut().dodge_anim.start();
                Ok(Response::Transition(State::dodging()))
            }

            _ => Err(()),
        }
    }

    fn try_jumping(inputs: &Inputs, context: &mut SMContext) -> Result<Response<State>, ()> {
        if context.timers.borrow().jump_limit.get_time_left() == 0.0 {
            match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Right), Some(ModifierButton::Jump)) => {
                    context.movement.borrow_mut().jump_right();
                    context.timers.borrow_mut().jump_limit.start();
                    Ok(Response::Transition(State::jumping()))
                }
                (Some(MoveButton::Left), Some(ModifierButton::Jump)) => {
                    context.movement.borrow_mut().jump_left();
                    context.timers.borrow_mut().jump_limit.start();
                    Ok(Response::Transition(State::jumping()))
                }
                (None, Some(ModifierButton::Jump)) => {
                    context.movement.borrow_mut().jump();
                    context.timers.borrow_mut().jump_limit.start();
                    Ok(Response::Transition(State::jumping()))
                }
                _ => Err(()),
            }
        } else {
            Err(())
        }
    }

    fn try_healing(inputs: &Inputs, context: &mut SMContext) -> Result<Response<State>, ()> {
        if context.timers.borrow().healing_anim.get_time_left() == 0.0
            && context.timers.borrow().healing_cooldown.get_time_left() == 0.0
        {
            match (&inputs.0, &inputs.1) {
                (_, Some(ModifierButton::Heal)) => {
                    context.timers.borrow_mut().healing_anim.start();
                    context.timers.borrow_mut().healing_cooldown.start();
                    context.movement.borrow_mut().stop_x();
                    Ok(Response::Transition(State::healing()))
                }
                _ => Err(()),
            }
        } else {
            Err(())
        }
    }
    fn try_attacking(inputs: &Inputs, context: &mut SMContext) -> Result<Response<State>, ()> {
        match (&inputs.0, &inputs.1) {
            (_, Some(ModifierButton::Attack))
                if context.timers.borrow().attack_anim.get_time_left() == 0.0
                    && let Ok(attack) = Offense::try_attack(
                        PlayerAttacks::SimpleMelee,
                        &mut context.resources.borrow_mut(),
                        1,
                    ) =>
            {
                context.hurtbox.bind_mut().set_attack(attack);
                context.timers.borrow_mut().attack_anim.start();
                Ok(Response::Transition(State::attacking()))
            }
            _ => Err(()),
        }
    }

    fn try_parry(inputs: &Inputs, context: &mut SMContext) -> Result<Response<State>, ()> {
        match (&inputs.0, &inputs.1) {
            (_, Some(ModifierButton::Parry))
                if context.timers.borrow().parry_anim.get_time_left() == 0.0 =>
            {
                context.movement.borrow_mut().stop_x();
                context.timers.borrow_mut().parry_anim.start();
                Ok(Response::Transition(State::parry()))
            }
            _ => Err(()),
        }
    }

    fn try_charged_attack(inputs: &Inputs, context: &mut SMContext) -> Result<Response<State>, ()> {
        match (&inputs.0, &inputs.1) {
            (_, Some(ModifierButton::ChargedAttack))
                if context.timers.borrow().charged_attack_anim.get_time_left() == 0.0
                    && let Ok(attack) = Offense::try_attack(
                        PlayerAttacks::ChargedMelee,
                        &mut context.resources.borrow_mut(),
                        1,
                    ) =>
            {
                context.hurtbox.bind_mut().set_attack(attack);
                context.timers.borrow_mut().charged_attack_anim.start();
                Ok(Response::Transition(State::chargedattack()))
            }
            _ => Err(()),
        }
    }

    fn try_air_dash(inputs: &Inputs, context: &mut SMContext) -> Result<Response<State>, ()> {
        match (&inputs.0, &inputs.1) {
            (Some(MoveButton::Left), Some(ModifierButton::Dodge))
                if context.timers.borrow().dodge_cooldown.get_time_left() == 0.0 =>
            {
                context.timers.borrow_mut().dodge_cooldown.start();
                context.timers.borrow_mut().dodge_anim.start();
                context.movement.borrow_mut().air_dash_left();
                Ok(Response::Transition(State::air_dash()))
            }
            (Some(MoveButton::Right), Some(ModifierButton::Dodge))
                if context.timers.borrow().dodge_cooldown.get_time_left() == 0.0 =>
            {
                context.movement.borrow_mut().air_dash_right();
                context.timers.borrow_mut().dodge_cooldown.start();
                context.timers.borrow_mut().dodge_anim.start();
                Ok(Response::Transition(State::air_dash()))
            }
            _ => Err(()),
        }
    }

    fn to_falling(inputs: &Inputs, context: &mut SMContext) -> Response<State> {
        match (&inputs.0, &inputs.1) {
            (Some(MoveButton::Left), _) => {
                context.movement.borrow_mut().run_left();
                Response::Transition(State::falling())
            }
            (Some(MoveButton::Right), _) => {
                context.movement.borrow_mut().run_right();
                Response::Transition(State::falling())
            }
            (None, _) => {
                context.movement.borrow_mut().stop_x();
                Response::Transition(State::falling())
            }
        }
    }

    /// Transitions the SM after checking movement input.
    fn to_moving(inputs: &Inputs, context: &mut SMContext) -> Response<State> {
        match (&inputs.0, &inputs.1) {
            (Some(MoveButton::Left), _) => {
                context.movement.borrow_mut().run_left();
                Response::Transition(State::run())
            }
            (Some(MoveButton::Right), _) => {
                context.movement.borrow_mut().run_right();
                Response::Transition(State::run())
            }
            (_, _) => {
                context.movement.borrow_mut().stop_x();
                Response::Transition(State::idle())
            }
        }
    }

    /// Checks inputs and updates velocity without changing state.
    fn handled_movement_input(inputs: &Inputs, context: &mut SMContext) -> Response<State> {
        match (&inputs.0, &inputs.1) {
            (Some(MoveButton::Left), _) => {
                context.movement.borrow_mut().run_left();
                Handled
            }
            (Some(MoveButton::Right), _) => {
                context.movement.borrow_mut().run_right();
                Handled
            }
            (_, _) => {
                context.movement.borrow_mut().stop_x();
                Handled
            }
        }
    }

    fn try_airborne_attack(
        inputs: &Inputs,
        context: &mut SMContext,
    ) -> Result<Response<State>, ()> {
        match (&inputs.0, &inputs.1, &inputs.2) {
            (_, Some(ModifierButton::Attack), Some(ModifierButton::Jump))
            | (_, Some(ModifierButton::Attack), None) => {
                if context.timers.borrow().air_attack_anim.get_time_left() == 0.0
                    && let Ok(attack) = Offense::try_attack(
                        PlayerAttacks::SimpleMelee,
                        &mut context.resources.borrow_mut(),
                        1,
                    )
                {
                    let anim = format!("attack_{}", context.movement.borrow_mut().get_direction());
                    context.graphics.borrow_mut().play_then_resume(&anim);
                    context.hurtbox.bind_mut().set_attack(attack);
                    context.timers.borrow_mut().air_attack_anim.start();
                    Ok(Handled)
                } else {
                    Ok(Handled)
                }
            }
            _ => Err(()),
        }
    }

    fn handle_wall_grab(inputs: &Inputs, context: &mut SMContext) -> Response<State> {
        match &inputs.0 {
            Some(MoveButton::Left) => {
                context.movement.borrow_mut().wall_grab_velocity();
                Response::Transition(State::wall_grab())
            }
            Some(MoveButton::Right) => {
                context.movement.borrow_mut().wall_grab_velocity();
                Response::Transition(State::wall_grab())
            }
            _ => Handled,
        }
    }
}
