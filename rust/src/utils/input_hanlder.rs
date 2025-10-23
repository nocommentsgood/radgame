use godot::obj::Singleton;
use godot::{
    classes::Input,
    obj::{Gd, WithBaseField},
};

use crate::entities::{entity_stats::Stat, player::main_character::MainCharacter};

/// Horizontal movement buttons.
#[derive(Clone, PartialEq, Eq, Debug, Copy)]
pub enum MoveButton {
    Left,
    Right,
}

/// Action buttons.
#[derive(Clone, PartialEq, Eq, Debug, Copy)]
pub enum ModifierButton {
    Dodge,
    Jump,
    Attack,
    JumpAttack,
    Heal,
    Parry,
    Spell,
}

/// Player inputs, used by the state machine.
#[derive(Default, Clone, PartialEq, Eq, Debug, Copy)]
pub struct Inputs(pub Option<MoveButton>, pub Option<ModifierButton>);

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

    pub fn handle(input: &Gd<Input>, player: &mut MainCharacter) -> Inputs {
        let mut inputs = Self::get_movement(input);

        if input.is_action_pressed("attack") {
            inputs.1 = Some(ModifierButton::Attack);
        }

        if input.is_action_pressed("jump") {
            if inputs.1.is_some_and(|btn| btn == ModifierButton::Attack) {
                inputs.1 = Some(ModifierButton::JumpAttack);
            } else {
                inputs.1 = Some(ModifierButton::Jump);
            }
        }

        if input.is_action_just_pressed("ability") {
            inputs.1 = Some(ModifierButton::Spell);
            println!("TODO: Implement ability usage.");
        }

        if input.is_action_just_pressed("rotate_abilities_right") {
            dbg!(player.ability_comp.quick.rotate_right(1));
        }

        if input.is_action_just_pressed("rotate_abilities_left") {
            dbg!(player.ability_comp.quick.rotate_left(1));
        }

        if input.is_action_pressed("dodge") {
            inputs.1 = Some(ModifierButton::Dodge);
        }

        if input.is_action_just_pressed("heal") {
            inputs.1 = Some(ModifierButton::Heal);
        }

        if input.is_action_pressed("parry") {
            {
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
