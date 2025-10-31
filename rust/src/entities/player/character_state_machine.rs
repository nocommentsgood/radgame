use std::{cell::RefCell, rc::Rc};

use godot::{builtin::Vector2, obj::Gd};
use statig::blocking::*;

use crate::{
    entities::{
        combat::{
            offense::{HotSpellIndexer, Offense, PlayerAttacks},
            resources::CombatResources,
        },
        hit_reg::Hurtbox,
        movements::Direction,
        player::{physics::Movement, time::PlayerTimers},
    },
    utils::{
        global_data_singleton::GlobalData,
        input_hanlder::{Inputs, ModifierButton, MoveButton},
    },
};

pub struct SMContext {
    timers: Rc<RefCell<PlayerTimers>>,
    resources: Rc<RefCell<CombatResources>>,
    hurtbox: Gd<Hurtbox>,
    off: Offense,
    movement: Rc<RefCell<Movement>>,
}

impl SMContext {
    pub fn new(
        timers: Rc<RefCell<PlayerTimers>>,
        resources: Rc<RefCell<CombatResources>>,
        hurtbox: Gd<Hurtbox>,
        off: Offense,
        movement: Rc<RefCell<Movement>>,
    ) -> Self {
        Self {
            timers,
            resources,
            hurtbox,
            off,
            movement,
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
            State::Jumping {} | State::MoveJumping {} => write!(f, "jumping"),
            State::Falling {} => write!(f, "falling"),
            State::Attacking {} => write!(f, "attack"),
            State::Chargedattack {} => write!(f, "chargedattack"),
            State::ChainAttack {} => write!(f, "chainattack"),
            State::Hurt {} => write!(f, "hurt"),
            State::AirAttack {} => write!(f, "airattack"),
            State::Healing {} => write!(f, "heal"),
            State::Parry {} => write!(f, "parry"),
            State::CastSpell {} => write!(f, "cast_spell"),
            State::MovingAirAttack {} => write!(f, "airattack"),
            State::AirDash {} => write!(f, "air_dash"),
            State::MoveFalling {} => write!(f, "falling"),
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
    TimerElapsed(Inputs),
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
                if let Ok(res) = Self::check_dodging(inputs, context) {
                    return res;
                }
                if let Ok(res) = Self::check_jumping(inputs, context) {
                    return res;
                }
                if let Ok(res) = Self::check_healing(inputs, context) {
                    return res;
                }
                if let Ok(res) = Self::check_attacking(inputs, context) {
                    return res;
                }
                if let Ok(res) = Self::check_casting_spell(inputs, context) {
                    return res;
                }
                if let Ok(res) = Self::check_charged_attack(inputs, context) {
                    return res;
                }
                if let Ok(res) = Self::check_parry(inputs, context) {
                    res
                } else {
                    Self::unchecked_handle_movement(inputs, context)
                }
            }
            Event::FailedFloorCheck(inputs) => Self::check_falling(inputs),
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
                if let Ok(res) = Self::check_dodging(inputs, context) {
                    return res;
                }
                if let Ok(res) = Self::check_jumping(inputs, context) {
                    return res;
                }
                if let Ok(res) = Self::check_healing(inputs, context) {
                    return res;
                }
                if let Ok(res) = Self::check_attacking(inputs, context) {
                    return res;
                }
                if let Ok(res) = Self::check_casting_spell(inputs, context) {
                    return res;
                }
                if let Ok(res) = Self::check_charged_attack(inputs, context) {
                    return res;
                }
                if let Ok(res) = Self::check_parry(inputs, context) {
                    res
                } else {
                    Self::unchecked_handle_movement(inputs, context)
                }
            }
            Event::FailedFloorCheck(inputs) => Self::check_falling(inputs),
            Event::Hurt => Response::Transition(State::hurt()),
            Event::ForceDisabled => Response::Transition(State::forced_disabled()),
            _ => Handled,
        }
    }

    #[state]
    fn dodging(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::TimerElapsed(inputs) => Self::unchecked_handle_movement(inputs, context),
            Event::FailedFloorCheck(inputs) => Self::check_falling(inputs),
            Event::ForceDisabled => Response::Transition(State::forced_disabled()),
            _ => Handled,
        }
    }

    #[state]
    fn move_jumping(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::GrabbedWall(inputs) => match (&inputs.0, inputs.1) {
                (Some(MoveButton::Right), _) => Response::Transition(State::wall_grab()),
                (_, _) => Handled,
            },

            Event::InputChanged(inputs) => {
                if let Ok(res) = Self::check_air_attack(inputs, context) {
                    return res;
                }
                match (&inputs.0, &inputs.1) {
                    (_, Some(ModifierButton::Dodge))
                        if context.timers.borrow().dodge_cooldown.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().dodge_cooldown.start();
                        Response::Transition(State::air_dash())
                    }

                    (Some(MoveButton::Left), Some(ModifierButton::Jump)) => {
                        Response::Transition(State::move_jumping())
                    }
                    (None, Some(ModifierButton::Jump)) => Response::Transition(State::jumping()),

                    _ => Self::check_falling(inputs),
                }
            }
            Event::TimerElapsed(inputs) => Self::check_falling(inputs),
            Event::Landed(inputs) => Self::unchecked_handle_movement(inputs, context),
            Event::HitCeiling(inputs) => Self::check_falling(inputs),
            Event::ForceDisabled => Response::Transition(State::forced_disabled()),
            _ => Handled,
        }
    }

    #[state]
    fn jumping(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::GrabbedWall(inputs) => match (&inputs.0, inputs.1) {
                (Some(MoveButton::Right), _) => Response::Transition(State::wall_grab()),
                (_, _) => Handled,
            },

            Event::InputChanged(inputs) => {
                if let Ok(res) = Self::check_air_attack(inputs, context) {
                    return res;
                }
                match (&inputs.0, &inputs.1) {
                    (Some(MoveButton::Left), Some(ModifierButton::Jump)) => {
                        context.movement.borrow_mut().velocity.x = Vector2::LEFT.x;
                        Response::Transition(State::move_jumping())
                    }
                    (Some(MoveButton::Right), Some(ModifierButton::Jump)) => {
                        context.movement.borrow_mut().velocity.x = Vector2::RIGHT.x;
                        Response::Transition(State::move_jumping())
                    }

                    _ => Self::check_falling(inputs),
                }
            }
            Event::TimerElapsed(inputs) => Self::check_falling(inputs),
            Event::Landed(inputs) => Self::unchecked_handle_movement(inputs, context),
            Event::HitCeiling(inputs) => Self::check_falling(inputs),
            Event::ForceDisabled => Response::Transition(State::forced_disabled()),
            _ => Handled,
        }
    }

    #[state]
    fn falling(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::GrabbedWall(inputs) => match (&inputs.0, inputs.1) {
                (Some(MoveButton::Right), _) => Response::Transition(State::wall_grab()),
                (_, _) => Handled,
            },
            Event::InputChanged(inputs) => {
                if let Ok(res) = Self::check_air_attack(inputs, context) {
                    res
                } else {
                    Self::check_falling(inputs)
                }
            }
            Event::Landed(inputs) => Self::unchecked_handle_movement(inputs, context),
            Event::ForceDisabled => Response::Transition(State::forced_disabled()),
            _ => Handled,
        }
    }

    /// Player is falling and providing positive X axis movement.
    #[state]
    fn move_falling(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::GrabbedWall(inputs) => match (&inputs.0, inputs.1) {
                (Some(MoveButton::Right), _) => Response::Transition(State::wall_grab()),
                (_, _) => Handled,
            },
            Event::InputChanged(inputs) => {
                if let Ok(res) = Self::check_air_attack(inputs, context) {
                    res
                } else {
                    Self::check_falling(inputs)
                }
            }
            Event::Landed(inputs) => Self::unchecked_handle_movement(inputs, context),
            Event::ForceDisabled => Response::Transition(State::forced_disabled()),
            _ => Handled,
        }
    }

    #[state]
    fn attacking(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::TimerElapsed(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Left), Some(ModifierButton::Attack))
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
                (Some(MoveButton::Right), Some(ModifierButton::Attack))
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
                (None, Some(ModifierButton::Attack))
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

                _ => Self::unchecked_handle_movement(inputs, context),
            },

            Event::Hurt => Response::Transition(State::hurt()),
            Event::ForceDisabled => Response::Transition(State::forced_disabled()),
            _ => Handled,
        }
    }

    #[state]
    fn chargedattack(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::TimerElapsed(inputs) => Self::unchecked_handle_movement(inputs, context),
            _ => Handled,
        }
    }

    #[state]
    fn chain_attack(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::TimerElapsed(inputs) => Self::unchecked_handle_movement(inputs, context),
            Event::Hurt => Response::Transition(State::hurt()),
            Event::ForceDisabled => Response::Transition(State::forced_disabled()),
            _ => Handled,
        }
    }

    #[state]
    fn hurt(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::TimerElapsed(inputs) => Self::unchecked_handle_movement(inputs, context),
            Event::ForceDisabled => Response::Transition(State::forced_disabled()),
            _ => Handled,
        }
    }

    #[state]
    fn air_attack(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::TimerElapsed(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Right), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::moving_air_attack())
                }
                (Some(MoveButton::Left), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::moving_air_attack())
                }
                (None, Some(ModifierButton::Attack)) => Response::Transition(State::air_attack()),

                _ => Self::check_falling(inputs),
            },
            Event::Landed(inputs) => Self::unchecked_handle_movement(inputs, context),
            Event::ForceDisabled => Response::Transition(State::forced_disabled()),
            _ => Handled,
        }
    }

    #[state]
    fn moving_air_attack(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::TimerElapsed(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Right), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::moving_air_attack())
                }
                (Some(MoveButton::Left), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::moving_air_attack())
                }
                (None, Some(ModifierButton::Attack)) => Response::Transition(State::air_attack()),
                _ => Self::check_falling(inputs),
            },
            Event::Landed(inputs) => Self::unchecked_handle_movement(inputs, context),
            Event::ForceDisabled => Response::Transition(State::forced_disabled()),
            _ => Handled,
        }
    }

    #[state]
    fn healing(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::TimerElapsed(inputs) => {
                context.resources.borrow_mut().heal();
                Self::unchecked_handle_movement(inputs, context)
            }
            Event::ForceDisabled => Response::Transition(State::forced_disabled()),
            _ => Handled,
        }
    }

    #[state]
    fn parry(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::TimerElapsed(inputs) => Self::unchecked_handle_movement(inputs, context),
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
                    Response::Transition(State::move_jumping())
                }
                (Some(MoveButton::Right), _) => Response::Transition(State::move_falling()),
                _ => Response::Transition(State::falling()),
            },
            Event::Landed(inputs) => Self::unchecked_handle_movement(inputs, context),
            Event::Hurt => Response::Transition(State::falling()),
            _ => Handled,
        }
    }

    #[state]
    fn cast_spell(event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::InputChanged(inputs) => Self::unchecked_handle_movement(inputs, context),
            _ => Handled,
        }
    }

    #[state]
    fn air_dash(event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed(inputs) => Self::check_falling(inputs),
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
    fn check_casting_spell(
        inputs: &Inputs,
        context: &mut SMContext,
    ) -> Result<Response<State>, ()> {
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

    fn check_dodging(inputs: &Inputs, context: &mut SMContext) -> Result<Response<State>, ()> {
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

    fn check_jumping(inputs: &Inputs, context: &mut SMContext) -> Result<Response<State>, ()> {
        if context.timers.borrow().jump_limit.get_time_left() == 0.0 {
            match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Right), Some(ModifierButton::Jump)) => {
                    context.movement.borrow_mut().jump_right();
                    context.timers.borrow_mut().jump_limit.start();
                    Ok(Response::Transition(State::move_jumping()))
                }
                (Some(MoveButton::Left), Some(ModifierButton::Jump)) => {
                    context.movement.borrow_mut().jump_left();
                    context.timers.borrow_mut().jump_limit.start();
                    Ok(Response::Transition(State::move_jumping()))
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

    fn check_healing(inputs: &Inputs, context: &mut SMContext) -> Result<Response<State>, ()> {
        if context.timers.borrow().healing_anim.get_time_left() == 0.0
            && context.timers.borrow().healing_cooldown.get_time_left() == 0.0
        {
            match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Right), Some(ModifierButton::Heal)) => {
                    context.timers.borrow_mut().healing_anim.start();
                    context.timers.borrow_mut().healing_cooldown.start();
                    Ok(Response::Transition(State::healing()))
                }
                (Some(MoveButton::Left), Some(ModifierButton::Heal)) => {
                    context.timers.borrow_mut().healing_anim.start();
                    context.timers.borrow_mut().healing_cooldown.start();
                    Ok(Response::Transition(State::healing()))
                }
                (None, Some(ModifierButton::Heal)) => {
                    context.timers.borrow_mut().healing_anim.start();
                    context.timers.borrow_mut().healing_cooldown.start();
                    Ok(Response::Transition(State::healing()))
                }
                _ => Err(()),
            }
        } else {
            Err(())
        }
    }
    fn check_attacking(inputs: &Inputs, context: &mut SMContext) -> Result<Response<State>, ()> {
        match (&inputs.0, &inputs.1) {
            (Some(MoveButton::Right), Some(ModifierButton::Attack))
                if context.timers.borrow().attack_anim.get_time_left() == 0.0
                    && let Ok(attack) = Offense::try_attack(
                        PlayerAttacks::SimpleMelee,
                        &mut context.resources.borrow_mut(),
                        1,
                    ) =>
            {
                context.movement.borrow_mut().stop_x();
                context.hurtbox.bind_mut().set_attack(attack);
                context.timers.borrow_mut().attack_anim.start();
                Ok(Response::Transition(State::attacking()))
            }
            (Some(MoveButton::Left), Some(ModifierButton::Attack))
                if let Ok(attack) = Offense::try_attack(
                    PlayerAttacks::SimpleMelee,
                    &mut context.resources.borrow_mut(),
                    1,
                ) && context.timers.borrow().attack_anim.get_time_left() == 0.0 =>
            {
                context.movement.borrow_mut().stop_x();
                context.hurtbox.bind_mut().set_attack(attack);
                context.timers.borrow_mut().attack_anim.start();
                Ok(Response::Transition(State::attacking()))
            }
            (None, Some(ModifierButton::Attack))
                if let Ok(attack) = Offense::try_attack(
                    PlayerAttacks::SimpleMelee,
                    &mut context.resources.borrow_mut(),
                    1,
                ) && context.timers.borrow().attack_anim.get_time_left() == 0.0 =>
            {
                context.movement.borrow_mut().stop_x();
                context.hurtbox.bind_mut().set_attack(attack);
                context.timers.borrow_mut().attack_anim.start();
                Ok(Response::Transition(State::attacking()))
            }
            _ => Err(()),
        }
    }

    fn check_parry(inputs: &Inputs, context: &mut SMContext) -> Result<Response<State>, ()> {
        match (&inputs.0, &inputs.1) {
            (Some(MoveButton::Right), Some(ModifierButton::Parry))
                if context.timers.borrow().parry_anim.get_time_left() == 0.0 =>
            {
                context.timers.borrow_mut().parry_anim.start();
                Ok(Response::Transition(State::parry()))
            }
            (Some(MoveButton::Left), Some(ModifierButton::Parry))
                if context.timers.borrow().parry_anim.get_time_left() == 0.0 =>
            {
                context.timers.borrow_mut().parry_anim.start();
                Ok(Response::Transition(State::parry()))
            }
            (None, Some(ModifierButton::Parry))
                if context.timers.borrow().parry_anim.get_time_left() == 0.0 =>
            {
                context.timers.borrow_mut().parry_anim.start();
                Ok(Response::Transition(State::parry()))
            }

            _ => Err(()),
        }
    }

    fn check_charged_attack(
        inputs: &Inputs,
        context: &mut SMContext,
    ) -> Result<Response<State>, ()> {
        match (&inputs.0, &inputs.1) {
            (Some(MoveButton::Left), Some(ModifierButton::ChargedAttack))
                if context.timers.borrow().attack_2_anim.get_time_left() == 0.0
                    && let Ok(attack) = Offense::try_attack(
                        PlayerAttacks::ChargedMelee,
                        &mut context.resources.borrow_mut(),
                        1,
                    ) =>
            {
                context.hurtbox.bind_mut().set_attack(attack);
                context.timers.borrow_mut().attack_2_anim.start();
                Ok(Response::Transition(State::chargedattack()))
            }
            (Some(MoveButton::Right), Some(ModifierButton::ChargedAttack))
                if context.timers.borrow().attack_2_anim.get_time_left() == 0.0
                    && let Ok(attack) = Offense::try_attack(
                        PlayerAttacks::ChargedMelee,
                        &mut context.resources.borrow_mut(),
                        1,
                    ) =>
            {
                context.hurtbox.bind_mut().set_attack(attack);
                context.timers.borrow_mut().attack_2_anim.start();
                Ok(Response::Transition(State::chargedattack()))
            }
            (None, Some(ModifierButton::ChargedAttack))
                if context.timers.borrow().attack_2_anim.get_time_left() == 0.0
                    && let Ok(attack) = Offense::try_attack(
                        PlayerAttacks::ChargedMelee,
                        &mut context.resources.borrow_mut(),
                        1,
                    ) =>
            {
                context.hurtbox.bind_mut().set_attack(attack);
                context.timers.borrow_mut().attack_2_anim.start();
                Ok(Response::Transition(State::chargedattack()))
            }
            _ => Err(()),
        }
    }

    fn check_air_attack(inputs: &Inputs, _context: &mut SMContext) -> Result<Response<State>, ()> {
        match (&inputs.0, &inputs.1) {
            (Some(MoveButton::Left), Some(ModifierButton::Attack | ModifierButton::JumpAttack)) => {
                Ok(Response::Transition(State::moving_air_attack()))
            }

            (
                Some(MoveButton::Right),
                Some(ModifierButton::Attack | ModifierButton::JumpAttack),
            ) => Ok(Response::Transition(State::moving_air_attack())),

            (None, Some(ModifierButton::Attack | ModifierButton::JumpAttack)) => {
                Ok(Response::Transition(State::air_attack()))
            }
            _ => Err(()),
        }
    }

    fn check_falling(inputs: &Inputs) -> Response<State> {
        match (&inputs.0, &inputs.1) {
            (Some(MoveButton::Left), _) => Response::Transition(State::move_falling()),
            (Some(MoveButton::Right), _) => Response::Transition(State::move_falling()),
            (None, _) => Response::Transition(State::falling()),
        }
    }

    fn unchecked_handle_movement(inputs: &Inputs, context: &mut SMContext) -> Response<State> {
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
}
