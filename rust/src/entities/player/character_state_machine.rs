use std::{cell::RefCell, rc::Rc};

use godot::obj::Gd;
use statig::blocking::*;

use crate::{
    entities::{
        damage::{CombatResources, Offense, PlayerAttacks},
        hit_reg::Hurtbox,
        player::time::PlayerTimers,
    },
    utils::input_hanlder::{Inputs, ModifierButton, MoveButton},
};

pub struct SMContext {
    timers: Rc<RefCell<PlayerTimers>>,
    resources: Rc<RefCell<CombatResources>>,
    hurtbox: Gd<Hurtbox>,
}

impl SMContext {
    pub fn new(
        timers: Rc<RefCell<PlayerTimers>>,
        resources: Rc<RefCell<CombatResources>>,
        hurtbox: Gd<Hurtbox>,
    ) -> Self {
        Self {
            timers,
            resources,
            hurtbox,
        }
    }
}
#[derive(Default, Debug, Clone)]
pub struct CharacterStateMachine {
    #[allow(unused)]
    chain_attacked: bool,
}

// Animation player uses the implementation of `Display` for animation names.
impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Right variants
            State::HurtRight {} => write!(f, "hurt"),
            State::AttackingRight {} => write!(f, "attack"),
            State::AttackRight2 {} => write!(f, "chainattack"),
            State::DodgingRight {} => write!(f, "dodge"),
            State::IdleRight {} | State::ForcedDisabledRight {} => write!(f, "idle"),
            State::MoveRight {} => write!(f, "run"),
            State::FallingRight {} | State::MoveFallingRight {} => write!(f, "falling"),
            State::JumpingRight {} | State::MoveJumpingRight {} => write!(f, "jumping"),
            State::HealingRight {} => write!(f, "heal"),
            State::ParryRight {} => write!(f, "parry"),
            State::AirAttackRight {} | State::MoveRightAirAttack {} => write!(f, "airattack"),
            State::WallGrabRight {} => write!(f, "idle"),

            // Left variants
            State::HurtLeft {} => write!(f, "hurt"),
            State::AttackingLeft {} => write!(f, "attack"),
            State::AttackLeft2 {} => write!(f, "chainattack"),
            State::DodgingLeft {} => write!(f, "dodge"),
            State::IdleLeft {} | State::ForcedDisabledLeft {} => write!(f, "idle"),
            State::MoveLeft {} => write!(f, "run"),
            State::FallingLeft {} | State::MoveFallingLeft {} => write!(f, "falling"),
            State::JumpingLeft {} | State::MoveJumpingLeft {} => {
                write!(f, "jumping")
            }
            State::WallGrabLeft {} => write!(f, "idle"),
            State::HealingLeft {} => write!(f, "heal"),
            State::ParryLeft {} => write!(f, "parry"),
            State::AirAttackLeft {} | State::MoveLeftAirAttack {} => write!(f, "airattack"),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        State::IdleRight {}
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
    initial = "State::idle_right()",
    state(derive(Debug, Clone, PartialEq, Copy))
)]
impl CharacterStateMachine {
    #[state]
    fn idle_right(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::InputChanged(inputs) => {
                match (&inputs.0, &inputs.1) {
                    // Moving
                    (Some(MoveButton::Left), None) => Response::Transition(State::move_left()),
                    (Some(MoveButton::Right), None) => Response::Transition(State::move_right()),

                    // Dodging
                    (Some(MoveButton::Left), Some(ModifierButton::Dodge)) => {
                        Response::Transition(State::dodging_left())
                    }
                    (Some(MoveButton::Right), Some(ModifierButton::Dodge))
                        if context.timers.borrow().dodge_cooldown.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().dodge_cooldown.start();
                        Response::Transition(State::dodging_right())
                    }
                    (None, Some(ModifierButton::Dodge))
                        if context.timers.borrow().dodge_anim.get_time_left() == 0.0
                            && context.timers.borrow().dodge_cooldown.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().dodge_anim.start();
                        Response::Transition(State::dodging_right())
                    }

                    // Jumping
                    (Some(MoveButton::Right), Some(ModifierButton::Jump)) => try_jump(
                        context,
                        || Response::Transition(State::move_jumping_right()),
                    ),
                    (Some(MoveButton::Left), Some(ModifierButton::Jump)) => {
                        try_jump(context, || Response::Transition(State::move_jumping_left()))
                    }
                    (None, Some(ModifierButton::Jump)) => {
                        try_jump(context, || Response::Transition(State::jumping_right()))
                    }

                    // Healing
                    (Some(MoveButton::Right), Some(ModifierButton::Heal))
                        if context.timers.borrow().healing_anim.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().healing_anim.start();
                        Response::Transition(State::healing_right())
                    }
                    (Some(MoveButton::Left), Some(ModifierButton::Heal))
                        if context.timers.borrow().healing_anim.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().healing_anim.start();
                        Response::Transition(State::healing_left())
                    }
                    (None, Some(ModifierButton::Heal))
                        if context.timers.borrow().healing_anim.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().healing_anim.start();
                        Response::Transition(State::healing_right())
                    }

                    // Attacking
                    (Some(MoveButton::Right), Some(ModifierButton::Attack))
                        if context.timers.borrow().attack_anim.get_time_left() == 0.0
                            && let Ok(attack) = Offense::try_attack(
                                PlayerAttacks::SimpleMelee,
                                &mut context.resources.borrow_mut(),
                                1,
                            ) =>
                    {
                        context.hurtbox.bind_mut().set_attack(attack);
                        context.timers.borrow_mut().attack_anim.start();
                        Response::Transition(State::attacking_right())
                    }
                    (Some(MoveButton::Left), Some(ModifierButton::Attack))
                        if let Ok(attack) = Offense::try_attack(
                            PlayerAttacks::SimpleMelee,
                            &mut context.resources.borrow_mut(),
                            1,
                        ) && context.timers.borrow().attack_anim.get_time_left() == 0.0 =>
                    {
                        context.hurtbox.bind_mut().set_attack(attack);
                        context.timers.borrow_mut().attack_anim.start();
                        Response::Transition(State::attacking_left())
                    }
                    (None, Some(ModifierButton::Attack))
                        if let Ok(attack) = Offense::try_attack(
                            PlayerAttacks::SimpleMelee,
                            &mut context.resources.borrow_mut(),
                            1,
                        ) && context.timers.borrow().attack_anim.get_time_left() == 0.0 =>
                    {
                        context.hurtbox.bind_mut().set_attack(attack);
                        context.timers.borrow_mut().attack_anim.start();
                        Response::Transition(State::attacking_right())
                    }

                    // Parry
                    (Some(MoveButton::Right), Some(ModifierButton::Parry))
                        if context.timers.borrow().parry_anim.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().parry_anim.start();
                        Response::Transition(State::parry_right())
                    }
                    (Some(MoveButton::Left), Some(ModifierButton::Parry))
                        if context.timers.borrow().parry_anim.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().parry_anim.start();
                        Response::Transition(State::parry_left())
                    }
                    (None, Some(ModifierButton::Parry))
                        if context.timers.borrow().parry_anim.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().parry_anim.start();
                        Response::Transition(State::parry_right())
                    }
                    _ => Handled,
                }
            }

            // Falling
            Event::FailedFloorCheck(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Right), None) => {
                    Response::Transition(State::move_falling_right())
                }
                (Some(MoveButton::Left), None) => Response::Transition(State::move_falling_left()),
                (None, None) => Response::Transition(State::falling_right()),
                _ => Response::Transition(State::falling_right()),
            },

            // Hurt
            Event::Hurt => {
                context.timers.borrow_mut().hurt_anim.start();
                Response::Transition(State::hurt_right())
            }
            Event::ForceDisabled => Response::Transition(State::forced_disabled_right()),
            _ => Handled,
        }
    }

    #[state]
    fn idle_left(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::InputChanged(inputs) => {
                match (&inputs.0, &inputs.1) {
                    // Moving
                    (Some(MoveButton::Left), None) => Response::Transition(State::move_left()),
                    (Some(MoveButton::Right), None) => Response::Transition(State::move_right()),

                    // Dodging
                    (Some(MoveButton::Left), Some(ModifierButton::Dodge))
                        if context.timers.borrow().dodge_anim.get_time_left() == 0.0
                            && context.timers.borrow().dodge_cooldown.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().dodge_anim.start();
                        Response::Transition(State::dodging_left())
                    }
                    (Some(MoveButton::Right), Some(ModifierButton::Dodge))
                        if context.timers.borrow().dodge_anim.get_time_left() == 0.0
                            && context.timers.borrow().dodge_cooldown.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().dodge_anim.start();
                        Response::Transition(State::dodging_right())
                    }
                    (None, Some(ModifierButton::Dodge))
                        if context.timers.borrow().dodge_anim.get_time_left() == 0.0
                            && context.timers.borrow().dodge_cooldown.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().dodge_anim.start();
                        Response::Transition(State::dodging_left())
                    }

                    // Jumping
                    (Some(MoveButton::Right), Some(ModifierButton::Jump)) => try_jump(
                        context,
                        || Response::Transition(State::move_jumping_right()),
                    ),
                    (Some(MoveButton::Left), Some(ModifierButton::Jump)) => {
                        try_jump(context, || Response::Transition(State::move_jumping_left()))
                    }
                    (None, Some(ModifierButton::Jump)) => {
                        try_jump(context, || Response::Transition(State::jumping_left()))
                    }

                    // Healing
                    (Some(MoveButton::Right), Some(ModifierButton::Heal))
                        if context.timers.borrow().healing_anim.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().healing_anim.start();
                        Response::Transition(State::healing_right())
                    }
                    (Some(MoveButton::Left), Some(ModifierButton::Heal))
                        if context.timers.borrow().healing_anim.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().healing_anim.start();
                        Response::Transition(State::healing_left())
                    }
                    (None, Some(ModifierButton::Heal))
                        if context.timers.borrow().healing_anim.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().healing_anim.start();
                        Response::Transition(State::healing_left())
                    }

                    // Attacking
                    (Some(MoveButton::Right), Some(ModifierButton::Attack))
                        if let Ok(attack) = Offense::try_attack(
                            PlayerAttacks::SimpleMelee,
                            &mut context.resources.borrow_mut(),
                            1,
                        ) && context.timers.borrow().attack_anim.get_time_left() == 0.0 =>
                    {
                        context.hurtbox.bind_mut().set_attack(attack);
                        context.timers.borrow_mut().attack_anim.start();
                        Response::Transition(State::attacking_right())
                    }
                    (Some(MoveButton::Left), Some(ModifierButton::Attack))
                        if let Ok(attack) = Offense::try_attack(
                            PlayerAttacks::SimpleMelee,
                            &mut context.resources.borrow_mut(),
                            1,
                        ) && context.timers.borrow().attack_anim.get_time_left() == 0.0 =>
                    {
                        context.hurtbox.bind_mut().set_attack(attack);
                        context.timers.borrow_mut().attack_anim.start();
                        Response::Transition(State::attacking_left())
                    }
                    (None, Some(ModifierButton::Attack))
                        if let Ok(attack) = Offense::try_attack(
                            PlayerAttacks::SimpleMelee,
                            &mut context.resources.borrow_mut(),
                            1,
                        ) && context.timers.borrow().attack_anim.get_time_left() == 0.0 =>
                    {
                        context.hurtbox.bind_mut().set_attack(attack);
                        context.timers.borrow_mut().attack_anim.start();
                        Response::Transition(State::attacking_left())
                    }

                    // Parry
                    (Some(MoveButton::Right), Some(ModifierButton::Parry))
                        if context.timers.borrow().parry_anim.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().parry_anim.start();
                        Response::Transition(State::parry_right())
                    }
                    (Some(MoveButton::Left), Some(ModifierButton::Parry))
                        if context.timers.borrow().parry_anim.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().parry_anim.start();
                        Response::Transition(State::parry_left())
                    }
                    (None, Some(ModifierButton::Parry))
                        if context.timers.borrow().parry_anim.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().parry_anim.start();
                        Response::Transition(State::parry_left())
                    }

                    _ => Handled,
                }
            }

            // Falling
            Event::FailedFloorCheck(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Right), None) => {
                    Response::Transition(State::move_falling_right())
                }
                (Some(MoveButton::Left), None) => Response::Transition(State::move_falling_left()),
                (None, None) => Response::Transition(State::falling_left()),
                _ => Response::Transition(State::falling_left()),
            },

            // Hurt
            Event::Hurt => {
                context.timers.borrow_mut().hurt_anim.start();
                Response::Transition(State::hurt_left())
            }
            Event::ForceDisabled => Response::Transition(State::forced_disabled_left()),
            _ => Handled,
        }
    }

    #[state]
    fn move_right(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::InputChanged(input) => {
                match (&input.0, &input.1) {
                    // Moving
                    (Some(MoveButton::Left), None) => Response::Transition(State::move_left()),

                    // Jumping
                    (Some(MoveButton::Right), Some(ModifierButton::Jump)) => try_jump(
                        context,
                        || Response::Transition(State::move_jumping_right()),
                    ),
                    (Some(MoveButton::Left), Some(ModifierButton::Jump)) => {
                        try_jump(context, || Response::Transition(State::move_jumping_left()))
                    }
                    (None, Some(ModifierButton::Jump)) => {
                        try_jump(context, || Response::Transition(State::jumping_right()))
                    }

                    // Dodging
                    (Some(MoveButton::Left), Some(ModifierButton::Dodge))
                        if context.timers.borrow().dodge_anim.get_time_left() == 0.0
                            && context.timers.borrow().dodge_cooldown.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().dodge_anim.start();
                        Response::Transition(State::dodging_left())
                    }
                    (Some(MoveButton::Right), Some(ModifierButton::Dodge))
                        if context.timers.borrow().dodge_anim.get_time_left() == 0.0
                            && context.timers.borrow().dodge_cooldown.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().dodge_anim.start();
                        Response::Transition(State::dodging_right())
                    }
                    (None, Some(ModifierButton::Dodge))
                        if context.timers.borrow().dodge_anim.get_time_left() == 0.0
                            && context.timers.borrow().dodge_cooldown.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().dodge_anim.start();
                        Response::Transition(State::dodging_right())
                    }

                    // Healing
                    (Some(MoveButton::Right), Some(ModifierButton::Heal))
                        if context.timers.borrow().healing_anim.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().healing_anim.start();
                        Response::Transition(State::healing_right())
                    }
                    (Some(MoveButton::Left), Some(ModifierButton::Heal))
                        if context.timers.borrow().healing_anim.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().healing_anim.start();
                        Response::Transition(State::healing_left())
                    }
                    (None, Some(ModifierButton::Heal))
                        if context.timers.borrow().healing_anim.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().healing_anim.start();
                        Response::Transition(State::healing_right())
                    }

                    // Attacking
                    (Some(MoveButton::Right), Some(ModifierButton::Attack))
                        if let Ok(attack) = Offense::try_attack(
                            PlayerAttacks::SimpleMelee,
                            &mut context.resources.borrow_mut(),
                            1,
                        ) && context.timers.borrow().attack_anim.get_time_left() == 0.0 =>
                    {
                        context.hurtbox.bind_mut().set_attack(attack);
                        context.timers.borrow_mut().attack_anim.start();
                        Response::Transition(State::attacking_right())
                    }
                    (Some(MoveButton::Left), Some(ModifierButton::Attack))
                        if let Ok(attack) = Offense::try_attack(
                            PlayerAttacks::SimpleMelee,
                            &mut context.resources.borrow_mut(),
                            1,
                        ) && context.timers.borrow().attack_anim.get_time_left() == 0.0 =>
                    {
                        context.hurtbox.bind_mut().set_attack(attack);
                        context.timers.borrow_mut().attack_anim.start();
                        Response::Transition(State::attacking_left())
                    }
                    (None, Some(ModifierButton::Attack))
                        if let Ok(attack) = Offense::try_attack(
                            PlayerAttacks::SimpleMelee,
                            &mut context.resources.borrow_mut(),
                            1,
                        ) && context.timers.borrow().attack_anim.get_time_left() == 0.0 =>
                    {
                        context.hurtbox.bind_mut().set_attack(attack);
                        context.timers.borrow_mut().attack_anim.start();
                        Response::Transition(State::attacking_right())
                    }

                    // Parry
                    (Some(MoveButton::Right), Some(ModifierButton::Parry))
                        if context.timers.borrow().parry_anim.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().parry_anim.start();
                        Response::Transition(State::parry_right())
                    }
                    (Some(MoveButton::Left), Some(ModifierButton::Parry))
                        if context.timers.borrow().parry_anim.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().parry_anim.start();
                        Response::Transition(State::parry_left())
                    }
                    (None, Some(ModifierButton::Parry))
                        if context.timers.borrow().parry_anim.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().parry_anim.start();
                        Response::Transition(State::parry_right())
                    }

                    // Idle
                    (None, None) => Response::Transition(State::idle_right()),
                    _ => Handled,
                }
            }

            // Falling
            Event::FailedFloorCheck(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Right), None) => {
                    Response::Transition(State::move_falling_right())
                }
                (Some(MoveButton::Left), None) => Response::Transition(State::move_falling_left()),
                (None, None) => Response::Transition(State::falling_right()),
                _ => Response::Transition(State::falling_right()),
            },

            // Hurt
            Event::Hurt => Response::Transition(State::hurt_right()),
            Event::ForceDisabled => Response::Transition(State::forced_disabled_right()),
            _ => Handled,
        }
    }
    #[state]
    fn move_left(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::InputChanged(input) => {
                match (&input.0, &input.1) {
                    // Moving
                    (Some(MoveButton::Right), None) => Response::Transition(State::move_right()),

                    // Jumping
                    (Some(MoveButton::Right), Some(ModifierButton::Jump)) => try_jump(
                        context,
                        || Response::Transition(State::move_jumping_right()),
                    ),
                    (Some(MoveButton::Left), Some(ModifierButton::Jump)) => {
                        try_jump(context, || Response::Transition(State::move_jumping_left()))
                    }
                    (None, Some(ModifierButton::Jump)) => {
                        try_jump(context, || Response::Transition(State::jumping_left()))
                    }

                    // Dodging
                    (Some(MoveButton::Left), Some(ModifierButton::Dodge))
                        if context.timers.borrow().dodge_anim.get_time_left() == 0.0
                            && context.timers.borrow().dodge_cooldown.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().dodge_anim.start();
                        Response::Transition(State::dodging_left())
                    }
                    (Some(MoveButton::Right), Some(ModifierButton::Dodge))
                        if context.timers.borrow().dodge_anim.get_time_left() == 0.0
                            && context.timers.borrow().dodge_cooldown.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().dodge_anim.start();
                        Response::Transition(State::dodging_right())
                    }

                    // Healing
                    (Some(MoveButton::Right), Some(ModifierButton::Heal))
                        if context.timers.borrow().healing_anim.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().healing_anim.start();
                        Response::Transition(State::healing_right())
                    }
                    (Some(MoveButton::Left), Some(ModifierButton::Heal))
                        if context.timers.borrow().healing_anim.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().healing_anim.start();
                        Response::Transition(State::healing_left())
                    }
                    (None, Some(ModifierButton::Heal))
                        if context.timers.borrow().healing_anim.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().healing_anim.start();
                        Response::Transition(State::healing_left())
                    }

                    // Attacking
                    (Some(MoveButton::Right), Some(ModifierButton::Attack))
                        if let Ok(attack) = Offense::try_attack(
                            PlayerAttacks::SimpleMelee,
                            &mut context.resources.borrow_mut(),
                            1,
                        ) && context.timers.borrow().attack_anim.get_time_left() == 0.0 =>
                    {
                        context.hurtbox.bind_mut().set_attack(attack);
                        context.timers.borrow_mut().attack_anim.start();
                        Response::Transition(State::attacking_right())
                    }
                    (Some(MoveButton::Left), Some(ModifierButton::Attack))
                        if let Ok(attack) = Offense::try_attack(
                            PlayerAttacks::SimpleMelee,
                            &mut context.resources.borrow_mut(),
                            1,
                        ) && context.timers.borrow().attack_anim.get_time_left() == 0.0 =>
                    {
                        context.hurtbox.bind_mut().set_attack(attack);
                        context.timers.borrow_mut().attack_anim.start();
                        Response::Transition(State::attacking_left())
                    }
                    (None, Some(ModifierButton::Attack))
                        if let Ok(attack) = Offense::try_attack(
                            PlayerAttacks::SimpleMelee,
                            &mut context.resources.borrow_mut(),
                            1,
                        ) && context.timers.borrow().attack_anim.get_time_left() == 0.0 =>
                    {
                        context.hurtbox.bind_mut().set_attack(attack);
                        context.timers.borrow_mut().attack_anim.start();
                        Response::Transition(State::attacking_left())
                    }

                    // Parry
                    (Some(MoveButton::Right), Some(ModifierButton::Parry))
                        if context.timers.borrow().parry_anim.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().parry_anim.start();
                        Response::Transition(State::parry_right())
                    }
                    (Some(MoveButton::Left), Some(ModifierButton::Parry))
                        if context.timers.borrow().parry_anim.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().parry_anim.start();
                        Response::Transition(State::parry_left())
                    }
                    (None, Some(ModifierButton::Parry))
                        if context.timers.borrow().parry_anim.get_time_left() == 0.0 =>
                    {
                        context.timers.borrow_mut().parry_anim.start();
                        Response::Transition(State::parry_left())
                    }

                    // Idle
                    (None, None) => Response::Transition(State::idle_left()),
                    _ => Handled,
                }
            }

            // Falling
            Event::FailedFloorCheck(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Right), None) => {
                    Response::Transition(State::move_falling_right())
                }
                (Some(MoveButton::Left), None) => Response::Transition(State::move_falling_left()),
                (None, None) => Response::Transition(State::falling_left()),
                _ => Response::Transition(State::falling_left()),
            },

            // Hurt
            Event::Hurt => Response::Transition(State::hurt_left()),
            Event::ForceDisabled => Response::Transition(State::forced_disabled_left()),
            _ => Handled,
        }
    }
    #[state]
    fn dodging_right(&mut self, event: &Event) -> Response<State> {
        match event {
            // Moving
            Event::TimerElapsed(inputs) => match (&inputs.0, &inputs.1) {
                (None, _) => Response::Transition(State::idle_right()),
                (Some(MoveButton::Left), _) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), _) => Response::Transition(State::move_right()),
            },

            // Falling
            Event::FailedFloorCheck(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Right), _) => Response::Transition(State::move_falling_right()),
                (Some(MoveButton::Left), _) => Response::Transition(State::move_falling_left()),
                (None, _) => Response::Transition(State::falling_right()),
            },
            Event::ForceDisabled => Response::Transition(State::forced_disabled_right()),
            _ => Handled,
        }
    }

    #[state]
    fn dodging_left(&mut self, event: &Event) -> Response<State> {
        match event {
            // Moving
            Event::TimerElapsed(inputs) => match (&inputs.0, &inputs.1) {
                (None, _) => Response::Transition(State::idle_left()),
                (Some(MoveButton::Left), _) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), _) => Response::Transition(State::move_right()),
            },

            // Dodging
            Event::FailedFloorCheck(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Right), _) => Response::Transition(State::move_falling_right()),
                (Some(MoveButton::Left), _) => Response::Transition(State::move_falling_left()),
                (None, _) => Response::Transition(State::falling_left()),
            },
            Event::ForceDisabled => Response::Transition(State::forced_disabled_left()),
            _ => Handled,
        }
    }

    #[state]
    fn move_jumping_right(&mut self, event: &Event) -> Response<State> {
        match event {
            // Wall grab
            Event::GrabbedWall(inputs) => match (&inputs.0, inputs.1) {
                (Some(MoveButton::Right), _) => Response::Transition(State::wall_grab_right()),
                (_, _) => Handled,
            },

            Event::InputChanged(inputs) => match (&inputs.0, &inputs.1) {
                // Attacking
                (Some(MoveButton::Left), Some(ModifierButton::JumpAttack)) => {
                    Response::Transition(State::move_left_air_attack())
                }
                (Some(MoveButton::Right), Some(ModifierButton::JumpAttack)) => {
                    Response::Transition(State::move_right_air_attack())
                }
                (None, Some(ModifierButton::JumpAttack)) => {
                    Response::Transition(State::air_attack_right())
                }

                // Jumping
                (Some(MoveButton::Left), Some(ModifierButton::Jump)) => {
                    Response::Transition(State::move_jumping_left())
                }
                (None, Some(ModifierButton::Jump)) => Response::Transition(State::jumping_right()),

                // Released jump button
                (Some(MoveButton::Right), _) => Response::Transition(State::move_falling_right()),
                (Some(MoveButton::Left), _) => Response::Transition(State::move_falling_left()),
                _ => Response::Transition(State::falling_right()),
            },

            Event::TimerElapsed(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Left), _) => Response::Transition(State::move_falling_left()),
                (Some(MoveButton::Right), _) => Response::Transition(State::move_falling_right()),
                _ => Response::Transition(State::falling_right()),
            },

            // On floor
            Event::Landed(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Right), _) => Response::Transition(State::move_right()),
                (Some(MoveButton::Left), _) => Response::Transition(State::move_left()),
                (None, _) => Response::Transition(State::idle_right()),
            },
            Event::HitCeiling(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Right), _) => Response::Transition(State::move_falling_right()),
                (Some(MoveButton::Left), _) => Response::Transition(State::move_falling_left()),
                (None, _) => Response::Transition(State::falling_right()),
            },
            Event::ForceDisabled => Response::Transition(State::forced_disabled_right()),
            _ => Handled,
        }
    }

    #[state]
    fn move_jumping_left(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::InputChanged(inputs) => match (&inputs.0, &inputs.1) {
                // Attacking
                (Some(MoveButton::Left), Some(ModifierButton::JumpAttack)) => {
                    Response::Transition(State::move_left_air_attack())
                }
                (Some(MoveButton::Right), Some(ModifierButton::JumpAttack)) => {
                    Response::Transition(State::move_right_air_attack())
                }
                (None, Some(ModifierButton::JumpAttack)) => {
                    Response::Transition(State::air_attack_left())
                }

                // Jumping
                (Some(MoveButton::Right), Some(ModifierButton::Jump)) => {
                    Response::Transition(State::move_jumping_right())
                }
                (None, Some(ModifierButton::Jump)) => Response::Transition(State::jumping_left()),

                // Released jump button
                (Some(MoveButton::Right), _) => Response::Transition(State::move_falling_right()),
                (Some(MoveButton::Left), _) => Response::Transition(State::move_falling_left()),
                _ => Response::Transition(State::falling_left()),
            },

            Event::TimerElapsed(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Left), _) => Response::Transition(State::move_falling_left()),
                (Some(MoveButton::Right), _) => Response::Transition(State::move_falling_right()),
                _ => Response::Transition(State::falling_left()),
            },

            // On floor
            Event::Landed(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Right), _) => Response::Transition(State::move_right()),
                (Some(MoveButton::Left), _) => Response::Transition(State::move_left()),
                (None, _) => Response::Transition(State::idle_left()),
            },
            Event::HitCeiling(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Right), _) => Response::Transition(State::move_falling_right()),
                (Some(MoveButton::Left), _) => Response::Transition(State::move_falling_left()),
                (None, _) => Response::Transition(State::falling_left()),
            },
            Event::ForceDisabled => Response::Transition(State::forced_disabled_right()),
            _ => Handled,
        }
    }

    #[state]
    fn jumping_right(&mut self, event: &Event) -> Response<State> {
        match event {
            // Wall grab
            Event::GrabbedWall(inputs) => match (&inputs.0, inputs.1) {
                (Some(MoveButton::Right), _) => Response::Transition(State::wall_grab_right()),
                (_, _) => Handled,
            },

            Event::InputChanged(inputs) => match (&inputs.0, &inputs.1) {
                // Attacking
                (Some(MoveButton::Left), Some(ModifierButton::JumpAttack)) => {
                    Response::Transition(State::move_left_air_attack())
                }
                (Some(MoveButton::Right), Some(ModifierButton::JumpAttack)) => {
                    Response::Transition(State::move_right_air_attack())
                }
                (None, Some(ModifierButton::JumpAttack)) => {
                    Response::Transition(State::air_attack_right())
                }

                // Jumping
                (Some(MoveButton::Left), Some(ModifierButton::Jump)) => {
                    Response::Transition(State::move_jumping_left())
                }
                (Some(MoveButton::Right), Some(ModifierButton::Jump)) => {
                    Response::Transition(State::move_jumping_right())
                }

                // Released jump button
                (Some(MoveButton::Left), _) => Response::Transition(State::move_falling_left()),
                (Some(MoveButton::Right), _) => Response::Transition(State::move_falling_right()),
                _ => Response::Transition(State::falling_right()),
            },

            Event::TimerElapsed(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Left), _) => Response::Transition(State::move_falling_left()),
                (Some(MoveButton::Right), _) => Response::Transition(State::move_falling_right()),
                _ => Response::Transition(State::falling_right()),
            },

            // On floor
            Event::Landed(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Right), _) => Response::Transition(State::move_right()),
                (Some(MoveButton::Left), _) => Response::Transition(State::move_left()),
                (None, _) => Response::Transition(State::idle_right()),
            },
            Event::HitCeiling(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Right), _) => Response::Transition(State::move_falling_right()),
                (Some(MoveButton::Left), _) => Response::Transition(State::move_falling_left()),
                (None, _) => Response::Transition(State::falling_right()),
            },
            Event::ForceDisabled => Response::Transition(State::forced_disabled_right()),
            _ => Handled,
        }
    }

    #[state]
    fn jumping_left(&mut self, event: &Event) -> Response<State> {
        match event {
            // Wall grab
            Event::GrabbedWall(inputs) => match (&inputs.0, inputs.1) {
                (Some(MoveButton::Left), _) => Response::Transition(State::wall_grab_left()),
                // (Some(MoveButton::Right), _) => Response::Transition(State::wall_jump_right()),
                (_, _) => Handled,
            },

            Event::InputChanged(inputs) => match (&inputs.0, &inputs.1) {
                // Attacking
                (Some(MoveButton::Left), Some(ModifierButton::JumpAttack)) => {
                    Response::Transition(State::move_left_air_attack())
                }
                (Some(MoveButton::Right), Some(ModifierButton::JumpAttack)) => {
                    Response::Transition(State::move_right_air_attack())
                }
                (None, Some(ModifierButton::JumpAttack)) => {
                    Response::Transition(State::air_attack_left())
                }

                // Jumping
                (Some(MoveButton::Left), Some(ModifierButton::Jump)) => {
                    Response::Transition(State::move_jumping_left())
                }
                (Some(MoveButton::Right), Some(ModifierButton::Jump)) => {
                    Response::Transition(State::move_jumping_right())
                }

                // Released jump button
                (Some(MoveButton::Right), _) => Response::Transition(State::move_falling_right()),
                (Some(MoveButton::Left), _) => Response::Transition(State::move_falling_left()),
                _ => Response::Transition(State::falling_left()),
            },

            Event::TimerElapsed(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Left), _) => Response::Transition(State::move_falling_left()),
                (Some(MoveButton::Right), _) => Response::Transition(State::move_falling_right()),
                _ => Response::Transition(State::falling_left()),
            },

            // On floor
            Event::Landed(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Right), _) => Response::Transition(State::move_right()),
                (Some(MoveButton::Left), _) => Response::Transition(State::move_left()),
                (None, _) => Response::Transition(State::idle_left()),
            },
            Event::HitCeiling(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Right), _) => Response::Transition(State::move_falling_right()),
                (Some(MoveButton::Left), _) => Response::Transition(State::move_falling_left()),
                (None, _) => Response::Transition(State::falling_left()),
            },
            Event::ForceDisabled => Response::Transition(State::forced_disabled_left()),
            _ => Handled,
        }
    }

    /// Player is falling and providing an x axis value of 0.0.
    #[state]
    fn falling_right(&mut self, event: &Event) -> Response<State> {
        match event {
            // Wall grab
            Event::GrabbedWall(inputs) => match (&inputs.0, inputs.1) {
                (Some(MoveButton::Right), _) => Response::Transition(State::wall_grab_right()),
                (_, _) => Handled,
            },

            Event::InputChanged(inputs) => match (&inputs.0, &inputs.1) {
                // Air attack
                (
                    Some(MoveButton::Left),
                    Some(ModifierButton::Attack | ModifierButton::JumpAttack),
                ) => Response::Transition(State::move_left_air_attack()),
                (
                    Some(MoveButton::Right),
                    Some(ModifierButton::Attack | ModifierButton::JumpAttack),
                ) => Response::Transition(State::move_right_air_attack()),
                (None, Some(ModifierButton::Attack | ModifierButton::JumpAttack)) => {
                    Response::Transition(State::air_attack_right())
                }

                (Some(MoveButton::Left), _) => Response::Transition(State::move_falling_left()),
                (Some(MoveButton::Right), _) => Response::Transition(State::move_falling_right()),
                _ => Handled,
            },

            // On floor
            Event::Landed(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Left), _) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), _) => Response::Transition(State::move_right()),
                (None, _) => Response::Transition(State::idle_right()),
            },
            Event::ForceDisabled => Response::Transition(State::forced_disabled_right()),
            _ => Handled,
        }
    }

    /// Player is falling and providing an x axis value of 0.0.
    #[state]
    fn falling_left(&mut self, event: &Event) -> Response<State> {
        match event {
            // Wall grab
            Event::GrabbedWall(inputs) => match (&inputs.0, inputs.1) {
                (Some(MoveButton::Left), _) => Response::Transition(State::wall_grab_left()),
                (_, _) => Handled,
            },

            Event::InputChanged(inputs) => match (&inputs.0, &inputs.1) {
                // Air attack
                (
                    Some(MoveButton::Left),
                    Some(ModifierButton::Attack | ModifierButton::JumpAttack),
                ) => Response::Transition(State::move_left_air_attack()),
                (
                    Some(MoveButton::Right),
                    Some(ModifierButton::Attack | ModifierButton::JumpAttack),
                ) => Response::Transition(State::move_right_air_attack()),
                (None, Some(ModifierButton::Attack | ModifierButton::JumpAttack)) => {
                    Response::Transition(State::air_attack_left())
                }
                (Some(MoveButton::Left), _) => Response::Transition(State::move_falling_left()),
                (Some(MoveButton::Right), _) => Response::Transition(State::move_falling_right()),
                _ => Handled,
            },

            // On floor
            Event::Landed(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Left), _) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), _) => Response::Transition(State::move_right()),
                (None, _) => Response::Transition(State::idle_left()),
            },
            Event::ForceDisabled => Response::Transition(State::forced_disabled_left()),
            _ => Handled,
        }
    }

    /// Player is falling and providing positive X axis movement.
    #[state]
    fn move_falling_right(&mut self, event: &Event) -> Response<State> {
        match event {
            // Wall grab
            Event::GrabbedWall(inputs) => match (&inputs.0, inputs.1) {
                (Some(MoveButton::Right), _) => Response::Transition(State::wall_grab_right()),
                (_, _) => Handled,
            },
            Event::InputChanged(inputs) => match (&inputs.0, &inputs.1) {
                // Air attack
                (
                    Some(MoveButton::Left),
                    Some(ModifierButton::Attack | ModifierButton::JumpAttack),
                ) => Response::Transition(State::move_left_air_attack()),

                (
                    Some(MoveButton::Right),
                    Some(ModifierButton::Attack | ModifierButton::JumpAttack),
                ) => Response::Transition(State::move_right_air_attack()),

                (None, Some(ModifierButton::Attack | ModifierButton::JumpAttack)) => {
                    Response::Transition(State::air_attack_right())
                }

                (Some(MoveButton::Left), _) => Response::Transition(State::move_falling_left()),
                (None, _) => Response::Transition(State::falling_right()),

                _ => Handled,
            },

            // On floor
            Event::Landed(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Left), _) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), _) => Response::Transition(State::move_right()),
                (None, _) => Response::Transition(State::idle_right()),
            },
            Event::ForceDisabled => Response::Transition(State::forced_disabled_right()),
            _ => Handled,
        }
    }

    /// Player is falling and providing negative X axis movement.
    #[state]
    fn move_falling_left(&mut self, event: &Event) -> Response<State> {
        match event {
            // Wall grab
            Event::GrabbedWall(inputs) => match (&inputs.0, inputs.1) {
                (Some(MoveButton::Left), _) => Response::Transition(State::wall_grab_left()),
                (_, _) => Handled,
            },

            Event::InputChanged(inputs) => match (&inputs.0, &inputs.1) {
                // Air attack
                (
                    Some(MoveButton::Left),
                    Some(ModifierButton::Attack | ModifierButton::JumpAttack),
                ) => Response::Transition(State::move_left_air_attack()),
                (
                    Some(MoveButton::Right),
                    Some(ModifierButton::Attack | ModifierButton::JumpAttack),
                ) => Response::Transition(State::move_right_air_attack()),
                (None, Some(ModifierButton::Attack | ModifierButton::JumpAttack)) => {
                    Response::Transition(State::air_attack_right())
                }

                (Some(MoveButton::Right), _) => Response::Transition(State::move_falling_right()),
                (None, _) => Response::Transition(State::falling_left()),
                _ => Handled,
            },

            // On floor
            Event::Landed(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Left), _) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), _) => Response::Transition(State::move_right()),
                (None, _) => Response::Transition(State::idle_left()),
            },
            Event::ForceDisabled => Response::Transition(State::forced_disabled_left()),
            _ => Handled,
        }
    }

    // TODO: Chain attacking.
    #[state]
    fn attacking_right(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::TimerElapsed(inputs) => match (&inputs.0, &inputs.1) {
                // Chain attacking
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
                    Response::Transition(State::attack_left_2())
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
                    Response::Transition(State::attack_right_2())
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
                    Response::Transition(State::attack_right_2())
                }

                // Moving
                (Some(MoveButton::Left), _) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), _) => Response::Transition(State::move_right()),
                (_, _) => Response::Transition(State::idle_right()),
            },

            Event::Hurt => Response::Transition(State::hurt_right()),
            Event::ForceDisabled => Response::Transition(State::forced_disabled_right()),
            _ => Handled,
        }
    }

    // TODO: Chain attacking.
    #[state]
    fn attacking_left(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::TimerElapsed(inputs) => match (&inputs.0, &inputs.1) {
                // Chain attacking
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
                    Response::Transition(State::attack_left_2())
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
                    Response::Transition(State::attack_right_2())
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
                    Response::Transition(State::attack_right_2())
                }

                // Moving
                (Some(MoveButton::Left), _) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), _) => Response::Transition(State::move_right()),
                (_, _) => Response::Transition(State::idle_left()),
            },

            Event::Hurt => Response::Transition(State::hurt_left()),
            Event::ForceDisabled => Response::Transition(State::forced_disabled_left()),
            _ => Handled,
        }
    }

    // TODO: Chain attacking
    #[state]
    fn attack_right_2(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Right), _) => Response::Transition(State::move_right()),
                (Some(MoveButton::Left), _) => Response::Transition(State::move_left()),
                (None, _) => Response::Transition(State::idle_right()),
            },
            Event::Hurt => Response::Transition(State::hurt_right()),
            Event::ForceDisabled => Response::Transition(State::forced_disabled_right()),
            _ => Handled,
        }
    }

    // TODO: Chain attacking
    #[state]
    fn attack_left_2(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Right), _) => Response::Transition(State::move_right()),
                (Some(MoveButton::Left), _) => Response::Transition(State::move_left()),
                (None, _) => Response::Transition(State::idle_left()),
            },
            Event::Hurt => Response::Transition(State::hurt_right()),
            Event::ForceDisabled => Response::Transition(State::forced_disabled_left()),
            _ => Handled,
        }
    }

    #[state]
    fn hurt_right(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed(input) => match (&input.0, &input.1) {
                // Moving
                (Some(MoveButton::Left), _) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), _) => Response::Transition(State::move_right()),

                // Idle
                (None, _) => Response::Transition(State::idle_right()),
            },
            Event::ForceDisabled => Response::Transition(State::forced_disabled_right()),
            _ => Handled,
        }
    }

    #[state]
    fn hurt_left(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed(input) => match (&input.0, &input.1) {
                // Moving
                (Some(MoveButton::Left), _) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), _) => Response::Transition(State::move_right()),

                // Idle
                (None, _) => Response::Transition(State::idle_left()),
            },
            Event::ForceDisabled => Response::Transition(State::forced_disabled_left()),
            _ => Handled,
        }
    }

    #[state]
    fn air_attack_right(&mut self, event: &Event) -> Response<State> {
        match event {
            // Moving
            Event::TimerElapsed(input) => match (&input.0, &input.1) {
                // Air attacking
                (Some(MoveButton::Right), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::move_right_air_attack())
                }
                (Some(MoveButton::Left), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::move_left_air_attack())
                }
                (None, Some(ModifierButton::Attack)) => {
                    Response::Transition(State::air_attack_right())
                }

                // Moving
                (Some(MoveButton::Left), _) => Response::Transition(State::move_falling_left()),
                (Some(MoveButton::Right), _) => Response::Transition(State::move_falling_right()),
                (None, _) => Response::Transition(State::falling_right()),
            },

            // On floor
            Event::Landed(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Left), _) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), _) => Response::Transition(State::move_right()),
                (None, _) => Response::Transition(State::idle_right()),
            },
            Event::ForceDisabled => Response::Transition(State::forced_disabled_right()),
            _ => Handled,
        }
    }

    #[state]
    fn move_right_air_attack(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed(input) => match (&input.0, &input.1) {
                // Air attacking
                (Some(MoveButton::Right), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::move_right_air_attack())
                }
                (Some(MoveButton::Left), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::move_left_air_attack())
                }
                (None, Some(ModifierButton::Attack)) => {
                    Response::Transition(State::air_attack_right())
                }

                // Moving
                (Some(MoveButton::Left), _) => Response::Transition(State::move_falling_left()),
                (Some(MoveButton::Right), _) => Response::Transition(State::move_falling_right()),
                (None, _) => Response::Transition(State::falling_right()),
            },

            // On floor
            Event::Landed(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Left), _) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), _) => Response::Transition(State::move_right()),
                (None, _) => Response::Transition(State::idle_right()),
            },
            Event::ForceDisabled => Response::Transition(State::forced_disabled_right()),
            _ => Handled,
        }
    }

    #[state]
    fn air_attack_left(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed(input) => match (&input.0, &input.1) {
                // Air attacking
                (Some(MoveButton::Right), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::move_right_air_attack())
                }
                (Some(MoveButton::Left), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::move_left_air_attack())
                }
                (None, Some(ModifierButton::Attack)) => {
                    Response::Transition(State::air_attack_left())
                }

                // Moving
                (Some(MoveButton::Left), _) => Response::Transition(State::move_falling_left()),
                (Some(MoveButton::Right), _) => Response::Transition(State::move_falling_right()),
                (None, _) => Response::Transition(State::falling_left()),
            },

            // On floor
            Event::Landed(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Left), _) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), _) => Response::Transition(State::move_right()),
                (None, _) => Response::Transition(State::idle_left()),
            },
            Event::ForceDisabled => Response::Transition(State::forced_disabled_left()),
            _ => Handled,
        }
    }

    #[state]
    fn move_left_air_attack(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed(input) => match (&input.0, &input.1) {
                // Air attacking
                (Some(MoveButton::Right), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::move_right_air_attack())
                }
                (Some(MoveButton::Left), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::move_left_air_attack())
                }
                (None, Some(ModifierButton::Attack)) => {
                    Response::Transition(State::air_attack_left())
                }

                // Moving
                (Some(MoveButton::Left), _) => Response::Transition(State::move_falling_left()),
                (Some(MoveButton::Right), _) => Response::Transition(State::move_falling_right()),
                (None, _) => Response::Transition(State::falling_left()),
            },

            // On floor
            Event::Landed(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Left), _) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), _) => Response::Transition(State::move_right()),
                (None, _) => Response::Transition(State::idle_left()),
            },
            Event::ForceDisabled => Response::Transition(State::forced_disabled_left()),
            _ => Handled,
        }
    }

    #[state]
    fn healing_right(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::TimerElapsed(inputs) => {
                context.resources.borrow_mut().heal();

                match (&inputs.0, &inputs.1) {
                    // Moving
                    (Some(MoveButton::Left), _) => Response::Transition(State::move_left()),
                    (Some(MoveButton::Right), _) => Response::Transition(State::move_right()),
                    (None, _) => Response::Transition(State::idle_right()),
                }
            }
            Event::ForceDisabled => Response::Transition(State::forced_disabled_right()),
            _ => Handled,
        }
    }

    #[state]
    fn healing_left(&mut self, event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::TimerElapsed(inputs) => {
                context.resources.borrow_mut().heal();

                match (&inputs.0, &inputs.1) {
                    // Moving
                    (Some(MoveButton::Left), _) => Response::Transition(State::move_left()),
                    (Some(MoveButton::Right), _) => Response::Transition(State::move_right()),
                    (None, _) => Response::Transition(State::idle_left()),
                }
            }
            Event::ForceDisabled => Response::Transition(State::forced_disabled_left()),
            _ => Handled,
        }
    }

    #[state]
    fn parry_right(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed(inputs) => match (&inputs.0, &inputs.1) {
                // Moving
                (Some(MoveButton::Left), _) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), _) => Response::Transition(State::move_right()),
                (None, _) => Response::Transition(State::idle_right()),
            },
            Event::ForceDisabled => Response::Transition(State::forced_disabled_right()),
            _ => Handled,
        }
    }

    #[state]
    fn parry_left(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed(inputs) => match (&inputs.0, &inputs.1) {
                // Moving
                (Some(MoveButton::Left), _) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), _) => Response::Transition(State::move_right()),
                (None, _) => Response::Transition(State::idle_left()),
            },
            Event::ForceDisabled => Response::Transition(State::forced_disabled_left()),
            _ => Handled,
        }
    }

    #[state]
    fn forced_disabled_right(event: &Event) -> Response<State> {
        match event {
            Event::ForceEnabled => Response::Transition(State::idle_right()),
            _ => Handled,
        }
    }

    #[state]
    fn forced_disabled_left(event: &Event) -> Response<State> {
        match event {
            Event::ForceEnabled => Response::Transition(State::idle_left()),
            _ => Handled,
        }
    }

    #[state]
    fn wall_grab_left(event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::InputChanged(inputs) => match (&inputs.0, inputs.1) {
                // Jumping
                (Some(MoveButton::Left), Some(ModifierButton::Jump))
                    if context.timers.borrow().wall_jump.get_time_left() == 0.0 =>
                {
                    context.timers.borrow_mut().wall_jump.start();
                    Response::Transition(State::move_jumping_right())
                }

                // Falling
                (Some(MoveButton::Right), _) => Response::Transition(State::move_falling_right()),
                _ => Response::Transition(State::falling_left()),
            },

            // Landed
            Event::Landed(inputs) => match (&inputs.0, inputs.1) {
                (Some(MoveButton::Left), _) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), _) => Response::Transition(State::move_right()),
                (None, _) => Response::Transition(State::idle_left()),
            },

            Event::Hurt => Response::Transition(State::falling_left()),

            _ => Handled,
        }
    }

    #[state]
    fn wall_grab_right(event: &Event, context: &mut SMContext) -> Response<State> {
        match event {
            Event::InputChanged(inputs) => match (&inputs.0, inputs.1) {
                // Jumping
                (Some(MoveButton::Right), Some(ModifierButton::Jump))
                    if context.timers.borrow().wall_jump.get_time_left() == 0.0 =>
                {
                    context.timers.borrow_mut().wall_jump.start();
                    Response::Transition(State::move_jumping_left())
                }

                // Falling
                (Some(MoveButton::Left), _) => Response::Transition(State::move_falling_left()),
                _ => Response::Transition(State::falling_right()),
            },

            // Landed
            Event::Landed(inputs) => match (&inputs.0, inputs.1) {
                (Some(MoveButton::Left), _) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), _) => Response::Transition(State::move_right()),
                (None, _) => Response::Transition(State::idle_left()),
            },

            Event::Hurt => Response::Transition(State::falling_left()),

            _ => Handled,
        }
    }
}

fn try_jump<F>(context: &mut SMContext, completed: F) -> Response<State>
where
    F: FnOnce() -> Response<State>,
{
    if context.timers.borrow().jump_limit.get_time_left() == 0.0 {
        context.timers.borrow_mut().jump_limit.start();
        completed()
    } else {
        Handled
    }
}
