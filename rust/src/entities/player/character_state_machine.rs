use godot::obj::Gd;
use statig::prelude::*;

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

pub struct SMContext<'a> {
    timers: &'a mut PlayerTimers,
    resources: &'a mut CombatResources,
    hurtbox: Gd<Hurtbox>,
    off: &'a Offense,
    movement: &'a mut Movement,
    graphics: &'a mut Graphics,
}

impl<'a> SMContext<'a> {
    pub fn new(
        timers: &'a mut PlayerTimers,
        resources: &'a mut CombatResources,
        hurtbox: Gd<Hurtbox>,
        off: &'a Offense,
        movement: &'a mut Movement,
        graphics: &'a mut Graphics,
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

#[state_machine(
    initial = "State::idle()",
    state(derive(Debug, Clone, PartialEq, Copy))
)]
impl CharacterStateMachine {
    #[state]
    fn idle(event: &Event, context: &mut SMContext<'_>) -> Response<State> {
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
                context.timers.hurt_anim.start();
                Response::Transition(State::hurt())
            }
            Event::ForceDisabled => Response::Transition(State::forced_disabled()),
            _ => Handled,
        }
    }

    #[state]
    fn run(event: &Event, context: &mut SMContext<'_>) -> Response<State> {
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
    fn dodging(event: &Event, context: &mut SMContext<'_>) -> Response<State> {
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
    fn jumping(event: &Event, context: &mut SMContext<'_>) -> Response<State> {
        match event {
            Event::GrabbedWall(inputs) => Self::handle_wall_grab(inputs, context),
            Event::InputChanged(inputs) => {
                if let Ok(res) = Self::try_casting_spell(inputs, context) {
                    return res;
                }
                if let Ok(res) = Self::try_air_dash(inputs, context) {
                    res
                } else if let Ok(res) = Self::try_airborne_attack(inputs, context) {
                    res
                } else {
                    match (&inputs.0, &inputs.1) {
                        (_, Some(ModifierButton::ReleasedJump(time))) => {
                            context.movement.apply_early_gravity(*time);
                            Response::Transition(State::falling())
                        }

                        (Some(MoveButton::Left), Some(ModifierButton::Jump)) => {
                            context.movement.jump_left();
                            Handled
                        }
                        (Some(MoveButton::Right), Some(ModifierButton::Jump)) => {
                            context.movement.jump_right();
                            Handled
                        }
                        (_, Some(ModifierButton::Jump)) => {
                            context.movement.jump();
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
    fn falling(event: &Event, context: &mut SMContext<'_>) -> Response<State> {
        match event {
            Event::GrabbedWall(inputs) => Self::handle_wall_grab(inputs, context),
            Event::InputChanged(inputs) => {
                if let Ok(res) = Self::try_casting_spell(inputs, context) {
                    return res;
                }
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
    fn attacking(event: &Event, context: &mut SMContext<'_>) -> Response<State> {
        match event {
            Event::TimerElapsed(timer, inputs) if *timer == Timers::AttackAnimation => {
                match (&inputs.0, &inputs.1) {
                    (_, Some(ModifierButton::Attack))
                        if context.timers.attack_2_anim.is_stopped()
                            && let Ok(attack) = Offense::try_attack(
                                PlayerAttacks::ChargedMelee,
                                context.resources,
                                1,
                            ) =>
                    {
                        context.hurtbox.bind_mut().set_attack(attack);
                        context.timers.attack_2_anim.start();
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
    fn chargedattack(event: &Event, context: &mut SMContext<'_>) -> Response<State> {
        match event {
            Event::TimerElapsed(timer, inputs) if *timer == Timers::ChargedAttack => {
                Self::to_moving(inputs, context)
            }
            _ => Handled,
        }
    }

    #[state]
    fn chain_attack(event: &Event, context: &mut SMContext<'_>) -> Response<State> {
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
    fn hurt(event: &Event, context: &mut SMContext<'_>) -> Response<State> {
        match event {
            Event::TimerElapsed(timer, inputs) if *timer == Timers::HurtAnimation => {
                Self::to_moving(inputs, context)
            }
            Event::ForceDisabled => Response::Transition(State::forced_disabled()),
            _ => Handled,
        }
    }

    #[state]
    fn healing(event: &Event, context: &mut SMContext<'_>) -> Response<State> {
        match event {
            Event::TimerElapsed(timer, inputs) if *timer == Timers::HealingAnimation => {
                context.timers.healing_cooldown.start();
                Self::to_moving(inputs, context)
            }
            Event::ForceDisabled => Response::Transition(State::forced_disabled()),
            _ => Handled,
        }
    }

    #[state]
    fn parry(event: &Event, context: &mut SMContext<'_>) -> Response<State> {
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
    fn wall_grab(event: &Event, context: &mut SMContext<'_>) -> Response<State> {
        match event {
            Event::InputChanged(inputs) => match (&inputs.0, inputs.1) {
                (Some(MoveButton::Left), Some(ModifierButton::Jump))
                    if context.timers.wall_jump.is_stopped() =>
                {
                    context.timers.wall_jump.start();
                    context.movement.jump_right();
                    Response::Transition(State::jumping())
                }
                (Some(MoveButton::Right), Some(ModifierButton::Jump))
                    if context.timers.wall_jump.is_stopped() =>
                {
                    context.timers.wall_jump.start();
                    context.movement.jump_left();
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
    fn cast_spell(event: &Event, context: &mut SMContext<'_>) -> Response<State> {
        context.movement.stop_x();
        context.movement.stop_y();
        match event {
            Event::TimerElapsed(timer, inputs) if *timer == Timers::CastSpellAnimation => {
                Self::to_moving(inputs, context)
            }
            _ => Handled,
        }
    }

    #[state]
    fn air_dash(event: &Event, context: &mut SMContext<'_>) -> Response<State> {
        match event {
            Event::GrabbedWall(inputs) => Self::handle_wall_grab(inputs, context),
            Event::TimerElapsed(timer, inputs) if *timer == Timers::DodgeAnimation => {
                Self::to_falling(inputs, context)
            }
            Event::Landed(inputs) => Self::to_moving(inputs, context),
            _ => Handled,
        }
    }

    fn try_cast_spell(
        context: &mut SMContext<'_>,
        hot_spell_index: HotSpellIndexer,
    ) -> Result<(), ()> {
        let spell = context.off.get_spell(hot_spell_index);
        if let Some(spell) = spell {
            let mut attack = spell.attack(1);
            if Offense::check_resources(attack.cost(), context.resources).is_ok() {
                let scene = spell.init_scene();
                context.off.apply_buffs(&mut attack);
                GlobalData::singleton()
                    .bind_mut()
                    .get_player_mut()
                    .unwrap()
                    .add_sibling(&scene);
                context.timers.spell_cooldown.start();
                Ok(())
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    fn try_casting_spell(
        inputs: &Inputs,
        context: &mut SMContext<'_>,
    ) -> Result<Response<State>, ()> {
        match (&inputs.1, &inputs.2) {
            (
                Some(ModifierButton::Ability1 | ModifierButton::Jump),
                Some(ModifierButton::Jump | ModifierButton::Ability1),
            ) if context.timers.spell_cooldown.is_stopped()
                && Self::try_cast_spell(context, HotSpellIndexer::Ability1).is_ok() =>
            {
                context.timers.spell_cooldown.start();
                context.timers.cast_spell_anim.start();
                Ok(Response::Transition(State::cast_spell()))
            }
            (
                Some(ModifierButton::Ability2 | ModifierButton::Jump),
                Some(ModifierButton::Jump | ModifierButton::Ability2),
            ) if context.timers.spell_cooldown.is_stopped()
                && Self::try_cast_spell(context, HotSpellIndexer::Ability2).is_ok() =>
            {
                context.timers.spell_cooldown.start();
                context.timers.cast_spell_anim.start();
                Ok(Response::Transition(State::cast_spell()))
            }
            (
                Some(ModifierButton::Ability3 | ModifierButton::Jump),
                Some(ModifierButton::Jump | ModifierButton::Ability3),
            ) if context.timers.spell_cooldown.is_stopped()
                && Self::try_cast_spell(context, HotSpellIndexer::Ability3).is_ok() =>
            {
                context.timers.spell_cooldown.start();
                context.timers.cast_spell_anim.start();
                Ok(Response::Transition(State::cast_spell()))
            }
            _ => Err(()),
        }
    }

    fn try_dodging(inputs: &Inputs, context: &mut SMContext<'_>) -> Result<Response<State>, ()> {
        match (&inputs.0, &inputs.1) {
            (Some(MoveButton::Left), Some(ModifierButton::Dodge))
                if context.timers.dodge_cooldown.is_stopped() =>
            {
                context.timers.dodge_cooldown.start();
                context.movement.dodge_left();
                Ok(Response::Transition(State::dodging()))
            }
            (Some(MoveButton::Right), Some(ModifierButton::Dodge))
                if context.timers.dodge_cooldown.is_stopped() =>
            {
                context.timers.dodge_cooldown.start();
                context.movement.dodge_right();
                Ok(Response::Transition(State::dodging()))
            }
            (None, Some(ModifierButton::Dodge)) if context.timers.dodge_cooldown.is_stopped() => {
                let dir = context.movement.get_direction();
                match dir {
                    Direction::Right => context.movement.dodge_right(),
                    Direction::Left => context.movement.dodge_left(),
                }
                context.timers.dodge_cooldown.start();
                context.timers.dodge_anim.start();
                Ok(Response::Transition(State::dodging()))
            }

            _ => Err(()),
        }
    }

    fn try_jumping(inputs: &Inputs, context: &mut SMContext<'_>) -> Result<Response<State>, ()> {
        if context.timers.jump_limit.is_stopped() {
            match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Right), Some(ModifierButton::Jump)) => {
                    context.movement.jump_right();
                    context.timers.jump_limit.start();
                    Ok(Response::Transition(State::jumping()))
                }
                (Some(MoveButton::Left), Some(ModifierButton::Jump)) => {
                    context.movement.jump_left();
                    context.timers.jump_limit.start();
                    Ok(Response::Transition(State::jumping()))
                }
                (None, Some(ModifierButton::Jump)) => {
                    context.movement.jump();
                    context.timers.jump_limit.start();
                    Ok(Response::Transition(State::jumping()))
                }
                _ => Err(()),
            }
        } else {
            Err(())
        }
    }

    fn try_healing(inputs: &Inputs, context: &mut SMContext<'_>) -> Result<Response<State>, ()> {
        if context.timers.healing_anim.is_stopped()
            && context.timers.healing_cooldown.is_stopped()
            && context.resources.health().amount() < context.resources.health().max()
        {
            match (&inputs.0, &inputs.1) {
                (_, Some(ModifierButton::Heal)) => {
                    context.timers.healing_anim.start();
                    context.timers.healing_cooldown.start();
                    context.movement.stop_x();
                    Ok(Response::Transition(State::healing()))
                }
                _ => Err(()),
            }
        } else {
            Err(())
        }
    }
    fn try_attacking(inputs: &Inputs, context: &mut SMContext<'_>) -> Result<Response<State>, ()> {
        match (&inputs.0, &inputs.1) {
            (_, Some(ModifierButton::Attack))
                if context.timers.attack_anim.is_stopped()
                    && let Ok(attack) =
                        Offense::try_attack(PlayerAttacks::SimpleMelee, context.resources, 1) =>
            {
                context.hurtbox.bind_mut().set_attack(attack);
                context.timers.attack_anim.start();
                Ok(Response::Transition(State::attacking()))
            }
            _ => Err(()),
        }
    }

    fn try_parry(inputs: &Inputs, context: &mut SMContext<'_>) -> Result<Response<State>, ()> {
        match (&inputs.0, &inputs.1) {
            (_, Some(ModifierButton::Parry)) if context.timers.parry_anim.is_stopped() => {
                context.movement.stop_x();
                context.timers.parry_anim.start();
                context.timers.perfect_parry.start();
                context.timers.parry.start();
                Ok(Response::Transition(State::parry()))
            }
            _ => Err(()),
        }
    }

    fn try_charged_attack(
        inputs: &Inputs,
        context: &mut SMContext<'_>,
    ) -> Result<Response<State>, ()> {
        match (&inputs.0, &inputs.1) {
            (_, Some(ModifierButton::ChargedAttack))
                if context.timers.charged_attack_anim.is_stopped()
                    && let Ok(attack) =
                        Offense::try_attack(PlayerAttacks::ChargedMelee, context.resources, 1) =>
            {
                context.hurtbox.bind_mut().set_attack(attack);
                context.timers.charged_attack_anim.start();
                Ok(Response::Transition(State::chargedattack()))
            }
            _ => Err(()),
        }
    }

    fn try_air_dash(inputs: &Inputs, context: &mut SMContext<'_>) -> Result<Response<State>, ()> {
        match (&inputs.0, &inputs.1) {
            (Some(MoveButton::Left), Some(ModifierButton::Dodge))
                if context.timers.dodge_cooldown.is_stopped() =>
            {
                context.timers.dodge_cooldown.start();
                context.timers.dodge_anim.start();
                context.movement.air_dash_left();
                Ok(Response::Transition(State::air_dash()))
            }
            (Some(MoveButton::Right), Some(ModifierButton::Dodge))
                if context.timers.dodge_cooldown.is_stopped() =>
            {
                context.movement.air_dash_right();
                context.timers.dodge_cooldown.start();
                context.timers.dodge_anim.start();
                Ok(Response::Transition(State::air_dash()))
            }
            _ => Err(()),
        }
    }

    fn to_falling(inputs: &Inputs, context: &mut SMContext<'_>) -> Response<State> {
        match (&inputs.0, &inputs.1) {
            (Some(MoveButton::Left), _) => {
                context.movement.run_left();
                Response::Transition(State::falling())
            }
            (Some(MoveButton::Right), _) => {
                context.movement.run_right();
                Response::Transition(State::falling())
            }
            (None, _) => {
                context.movement.stop_x();
                Response::Transition(State::falling())
            }
        }
    }

    /// Transitions the SM after checking movement input.
    fn to_moving(inputs: &Inputs, context: &mut SMContext<'_>) -> Response<State> {
        match (&inputs.0, &inputs.1) {
            (Some(MoveButton::Left), _) => {
                context.movement.run_left();
                Response::Transition(State::run())
            }
            (Some(MoveButton::Right), _) => {
                context.movement.run_right();
                Response::Transition(State::run())
            }
            (_, _) => {
                context.movement.stop_x();
                Response::Transition(State::idle())
            }
        }
    }

    /// Checks inputs and updates velocity without changing state.
    fn handled_movement_input(inputs: &Inputs, context: &mut SMContext<'_>) -> Response<State> {
        match (&inputs.0, &inputs.1) {
            (Some(MoveButton::Left), _) => {
                context.movement.run_left();
                Handled
            }
            (Some(MoveButton::Right), _) => {
                context.movement.run_right();
                Handled
            }
            (_, _) => {
                context.movement.stop_x();
                Handled
            }
        }
    }

    fn try_airborne_attack(
        inputs: &Inputs,
        context: &mut SMContext<'_>,
    ) -> Result<Response<State>, ()> {
        match (&inputs.0, &inputs.1, &inputs.2) {
            (_, Some(ModifierButton::Attack), Some(ModifierButton::Jump) | None) => {
                if context.timers.air_attack_anim.is_stopped()
                    && let Ok(attack) =
                        Offense::try_attack(PlayerAttacks::SimpleMelee, context.resources, 1)
                {
                    let anim = format!("attack_{}", context.movement.get_direction());
                    context.graphics.play_then_resume(&anim);
                    context.hurtbox.bind_mut().set_attack(attack);
                    context.timers.air_attack_anim.start();
                    Ok(Handled)
                } else {
                    Ok(Handled)
                }
            }
            _ => Err(()),
        }
    }

    fn handle_wall_grab(inputs: &Inputs, context: &mut SMContext<'_>) -> Response<State> {
        match &inputs.0 {
            Some(MoveButton::Left | MoveButton::Right) => {
                context.movement.wall_grab_velocity();
                Response::Transition(State::wall_grab())
            }
            _ => Handled,
        }
    }
}
