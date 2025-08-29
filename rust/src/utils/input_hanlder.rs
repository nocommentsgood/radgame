use godot::{
    classes::{Input, InputEvent, Timer},
    obj::{Gd, WithBaseField, WithUserSignals},
};

use crate::entities::{
    entity_stats::{StatVal, Stats},
    player::{character_state_machine::Event, main_character::MainCharacter},
    time::PlayerTimer,
};

type PT = PlayerTimer;

#[derive(Default, Clone)]
pub struct InputHandler;

impl InputHandler {
    pub fn handle_input(input: &Gd<Input>) -> Inputs {
        if input.is_action_pressed("east") {
            Inputs(Some(MoveButton::Right), None)
        } else if input.is_action_pressed("west") {
            Inputs(Some(MoveButton::Left), None)
        } else {
            Inputs(None, None)
        }
    }

    pub fn handle_unhandled(event: &Gd<InputEvent>, entity: &mut MainCharacter) {
        let timer_ok = |timer: Option<&Gd<Timer>>| timer.is_some_and(|t| t.get_time_left() == 0.0);

        if event.is_action_pressed("attack") && timer_ok(entity.timers.get(&PT::AttackAnimation)) {
            entity.timers.get_mut(&PT::AttackAnimation).unwrap().start();
            entity.transition_sm(&Event::InputChanged(Inputs(
                InputHandler::handle_input(&Input::singleton()).0,
                Some(ModifierButton::Attack),
            )));
        }
        if event.is_action_pressed("jump") && entity.base().is_on_floor() {
            entity.transition_sm(&Event::InputChanged(Inputs(
                InputHandler::handle_input(&Input::singleton()).0,
                Some(ModifierButton::Jump),
            )));
        }
        if event.is_action_pressed("dodge")
            && timer_ok(entity.timers.get(&PT::DodgeAnimation))
            && timer_ok(entity.timers.get(&PT::DodgeCooldown))
        {
            entity.timers.get_mut(&PT::DodgeAnimation).unwrap().start();
            entity.transition_sm(&Event::InputChanged(Inputs(
                InputHandler::handle_input(&Input::singleton()).0,
                Some(ModifierButton::Dodge),
            )));
        }

        if event.is_action_pressed("heal")
            && timer_ok(entity.timers.get(&PT::HealingAnimation))
            && timer_ok(entity.timers.get(&PT::HealingCooldown))
        {
            entity
                .timers
                .get_mut(&PT::HealingAnimation)
                .unwrap()
                .start();
            entity.transition_sm(&Event::InputChanged(Inputs(
                InputHandler::handle_input(&Input::singleton()).0,
                Some(ModifierButton::Heal),
            )));
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

        if event.is_action_pressed("parry") && timer_ok(entity.timers.get(&PT::ParryAnimation)) {
            entity.timers.get_mut(&PT::ParryAnimation).unwrap().start();
            entity.timers.get_mut(&PT::PerfectParry).unwrap().start();
            entity.transition_sm(&Event::InputChanged(Inputs(
                InputHandler::handle_input(&Input::singleton()).0,
                Some(ModifierButton::Parry),
            )));
        }

        if event.is_action_pressed("rotate_abilities_left") {
            entity.ability_comp.quick.rotate_left(1);
        }

        if event.is_action_pressed("rotate_abilities_right") {
            entity.ability_comp.quick.rotate_right(1);
        }

        if event.is_action_pressed("ability")
            && let Some(Some(ability)) = entity.ability_comp.clone().quick.front()
        {
            ability.execute(entity);
        }
    }
}

pub struct DevInputHandler;

impl DevInputHandler {
    pub fn handle_unhandled(event: &Gd<InputEvent>, entity: &mut MainCharacter) {
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
