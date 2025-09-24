use godot::{
    classes::{Input, Timer},
    obj::{Gd, WithBaseField, WithUserSignals},
};

use crate::entities::{
    entity_stats::Stat,
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

    // TODO: Move timer handling to state machine.
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
            if inputs.1.is_some_and(|btn| btn == ModifierButton::Attack) {
                inputs.1 = Some(ModifierButton::JumpAttack);
            } else {
                inputs.1 = Some(ModifierButton::Jump);
            }
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
                let cur = entity.stats.get_raw(Stat::Health);
                let max = entity.stats.get_raw(Stat::MaxHealth);
                let amount = entity.stats.get_raw(Stat::HealAmount);

                if cur < max {
                    entity.stats.get_mut(Stat::Health).0 += amount;
                    let new = entity.stats.get(Stat::Health).0;
                    entity
                        .signals()
                        .player_health_changed()
                        .emit(cur, new, amount);
                }
            }
        }
        if input.is_action_pressed("parry") && timer_ok(entity.timers.get(&PT::ParryAnimation)) {
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

        if event.is_action_just_pressed("dev_increase_level") {
            entity.stats.get_mut(Stat::Level).0 += 1;
            println!(
                "DevTools: Increased player level... Current level: {}",
                entity.stats.get(Stat::Level).0
            );
        }

        if event.is_action_just_pressed("dev_decrease_level") {
            if entity.stats.get(Stat::Level).0 > 1 {
                entity.stats.get_mut(Stat::Level).0 -= 1;
            }
            println!(
                "DevTools: Decreased player level... Current level: {}",
                entity.stats.get(Stat::Level).0
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
    JumpAttack,
    Heal,
    Parry,
}

/// Represents player input actions.
#[derive(Default, Clone, PartialEq, Eq, Debug, Copy)]
pub struct Inputs(pub Option<MoveButton>, pub Option<ModifierButton>);
