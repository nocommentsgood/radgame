use godot::{
    classes::{Input, InputEvent, Timer},
    obj::{Gd, WithBaseField, WithUserSignals},
};

use crate::entities::{
    entity_stats::{StatVal, Stats},
    player::{
        character_state_machine::{Event, State},
        main_character::MainCharacter,
    },
    time::PlayerTimer,
};

type PT = PlayerTimer;

#[derive(Default, Clone)]
pub struct InputHandler;

impl InputHandler {
    pub fn get_movement(input: &Gd<Input>) -> Inputs {
        let mut inputs = Inputs::default();
        if input.is_action_pressed("east") {
            inputs.0 = Some(MoveButton::Right);
        } else if input.is_action_pressed("west") {
            inputs.0 = Some(MoveButton::Left);
        } else {
            inputs.0 = None;
        }
        inputs
    }

    pub fn handle(input: &Gd<Input>, entity: &mut MainCharacter) -> Inputs {
        let timer_ok = |timer: Option<&Gd<Timer>>| {
            timer.is_some_and(|t| t.get_time_left() == 0.0 && t.is_stopped())
        };
        let mut inputs = Self::get_movement(input);

        if input.is_action_pressed("attack") && timer_ok(entity.timers.get(&PT::AttackAnimation)) {
            entity.timers.get_mut(&PT::AttackAnimation).unwrap().start();
            inputs.1 = Some(ModifierButton::Attack);
        }
        if input.is_action_pressed("jump") {
            inputs.1 = Some(ModifierButton::Jump);
        }
        if input.is_action_pressed("dodge")
            && timer_ok(entity.timers.get(&PT::DodgeAnimation))
            && timer_ok(entity.timers.get(&PT::DodgeCooldown))
        {
            entity.timers.get_mut(&PT::DodgeAnimation).unwrap().start();
            inputs.1 = Some(ModifierButton::Dodge);
        }
        if input.is_action_pressed("heal")
            && timer_ok(entity.timers.get(&PT::HealingAnimation))
            && timer_ok(entity.timers.get(&PT::HealingCooldown))
        {
            entity
                .timers
                .get_mut(&PT::HealingAnimation)
                .unwrap()
                .start();
            inputs.1 = Some(ModifierButton::Heal);
            entity.transition_sm(&Event::InputChanged(inputs));

            // TODO: This isn't the best place to apply healing...
            //
            // If the state machine changed to a `healing` state, heal the player.
            if *entity.state.state() == (State::HealingLeft {})
                || *entity.state.state() == (State::HealingRight {})
            {
                let get_stat = |stat: Option<&StatVal>| stat.unwrap().0;
                let cur = get_stat(entity.stats.get(&Stats::Health));
                let max = get_stat(entity.stats.get(&Stats::MaxHealth));
                let amount = get_stat(entity.stats.get(&Stats::HealAmount));

                if cur < max {
                    entity.stats.get_mut(&Stats::Health).unwrap().0 += amount;
                    let new = get_stat(entity.stats.get(&Stats::Health));
                    entity
                        .signals()
                        .player_health_changed()
                        .emit(cur, new, amount);
                }
            }

            if input.is_action_pressed("parry") && timer_ok(entity.timers.get(&PT::ParryAnimation))
            {
                entity.timers.get_mut(&PT::ParryAnimation).unwrap().start();
                entity.timers.get_mut(&PT::PerfectParry).unwrap().start();
                inputs.1 = Some(ModifierButton::Parry);
            }
        }
        inputs
    }
}

/// Developer input handling.
pub struct DevInputHandler;

impl DevInputHandler {
    pub fn handle_unhandled(event: &Gd<Input>, entity: &mut MainCharacter) -> Inputs {
        let inputs = InputHandler::handle(&Input::singleton(), entity);
        if event.is_action_pressed("dev_teleport") {
            let pos = entity
                .base()
                .get_viewport()
                .unwrap()
                .get_camera_2d()
                .unwrap()
                .get_global_mouse_position();
            entity.base_mut().set_global_position(pos);
        }

        if event.is_action_just_pressed("dev_increase_level")
            && let Some(x) = entity.stats.get_mut(&Stats::Level)
        {
            x.0 += 1;
            println!(
                "DevTools: Increased player level... Current level: {}",
                entity.stats.get(&Stats::Level).unwrap().0
            );
        }

        if event.is_action_just_pressed("dev_decrease_level")
            && let Some(x) = entity.stats.get_mut(&Stats::Level)
            && x.0 > 1
        {
            x.0 -= 1;
            println!(
                "DevTools: Decreased player level... Current level: {}",
                entity.stats.get(&Stats::Level).unwrap().0
            );
        }
        inputs
    }
}
/// Horizontal movement.
#[derive(Clone, PartialEq, Eq, Debug, Copy)]
pub enum MoveButton {
    Left,
    Right,
}

/// Action buttons pressed by the player.
#[derive(Clone, PartialEq, Eq, Debug, Copy)]
pub enum ModifierButton {
    Dodge,
    Jump,
    Attack,
    Heal,
    Parry,
}

/// Represents player input actions.
#[derive(Default, Clone, PartialEq, Eq, Debug, Copy)]
pub struct Inputs(pub Option<MoveButton>, pub Option<ModifierButton>);
