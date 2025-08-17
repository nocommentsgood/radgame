use statig::blocking::*;

use crate::utils::input_hanlder::{Inputs, ModifierButton, MoveButton};

#[derive(Default, Debug, Clone)]
pub struct CharacterStateMachine {
    chain_attacked: bool,
}

// Animation player uses the implementation of `Display` for animation names.
impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Right variants
            State::HurtRight {} => write!(f, "hurt_right"),
            State::AttackingRight {} => write!(f, "attack_right"),
            State::AttackRight2 {} => write!(f, "chainattack_right"),
            State::DodgingRight {} => write!(f, "dodge_right"),
            State::IdleRight {} => write!(f, "idle_right"),
            State::MoveRight {} => write!(f, "run_right"),
            State::FallingRight {} | State::MoveFallingRight {} => write!(f, "falling_right"),
            State::JumpingRight {} => write!(f, "jumping_right"),
            // State::GrapplingRight {} => write!(f, "grapple_right"),
            State::HealingRight {} => write!(f, "heal_right"),
            State::ParryRight {} => write!(f, "parry_right"),
            State::AirAttackRight {} | State::MoveRightAirAttack {} => write!(f, "airattack_right"),

            // Left variants
            State::HurtLeft {} => write!(f, "hurt_left"),
            State::AttackingLeft {} => write!(f, "attack_left"),
            State::AttackLeft2 {} => write!(f, "chainattack_left"),
            State::DodgingLeft {} => write!(f, "dodge_left"),
            State::IdleLeft {} => write!(f, "idle_left"),
            State::MoveLeft {} => write!(f, "run_left"),
            State::FallingLeft {} | State::MoveFallingLeft {} => write!(f, "falling_left"),
            State::JumpingLeft {} => write!(f, "jumping_left"),
            // State::GrapplingLeft {} => write!(f, "grapple_left"),
            State::HealingLeft {} => write!(f, "heal_left"),
            State::ParryLeft {} => write!(f, "parry_left"),
            State::AirAttackLeft {} | State::MoveLeftAirAttack {} => write!(f, "airattack_left"),
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
    Hurt,
    #[default]
    None,
}

#[state_machine(
    initial = "State::idle_right()",
    state(derive(Debug, Clone, PartialEq, Copy))
)]
impl CharacterStateMachine {
    #[state]
    fn idle_right(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::InputChanged(inputs) => match (&inputs.0, &inputs.1) {
                // Moving
                (Some(MoveButton::Left), None) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), None) => Response::Transition(State::move_right()),

                // Dodging
                (Some(MoveButton::Left), Some(ModifierButton::Dodge)) => {
                    Response::Transition(State::dodging_left())
                }
                (Some(MoveButton::Right), Some(ModifierButton::Dodge)) => {
                    Response::Transition(State::dodging_right())
                }
                (None, Some(ModifierButton::Dodge)) => Response::Transition(State::dodging_right()),

                // Jumping
                (Some(MoveButton::Right), Some(ModifierButton::Jump)) => {
                    Response::Transition(State::jumping_right())
                }
                (Some(MoveButton::Left), Some(ModifierButton::Jump)) => {
                    Response::Transition(State::jumping_left())
                }
                (None, Some(ModifierButton::Jump)) => Response::Transition(State::jumping_right()),

                // Healing
                (Some(MoveButton::Right), Some(ModifierButton::Heal)) => {
                    Response::Transition(State::healing_right())
                }
                (Some(MoveButton::Left), Some(ModifierButton::Heal)) => {
                    Response::Transition(State::healing_left())
                }
                (None, Some(ModifierButton::Heal)) => Response::Transition(State::healing_right()),

                // Attacking
                (Some(MoveButton::Right), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::attacking_right())
                }
                (Some(MoveButton::Left), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::attacking_left())
                }
                (None, Some(ModifierButton::Attack)) => {
                    Response::Transition(State::attacking_right())
                }

                // Parry
                (Some(MoveButton::Right), Some(ModifierButton::Parry)) => {
                    Response::Transition(State::parry_right())
                }
                (Some(MoveButton::Left), Some(ModifierButton::Parry)) => {
                    Response::Transition(State::parry_left())
                }
                (None, Some(ModifierButton::Parry)) => Response::Transition(State::parry_right()),
                _ => Handled,
            },

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
            _ => Handled,
        }
    }

    #[state]
    fn idle_left(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::InputChanged(inputs) => match (&inputs.0, &inputs.1) {
                // Moving
                (Some(MoveButton::Left), None) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), None) => Response::Transition(State::move_right()),

                // Dodging
                (Some(MoveButton::Left), Some(ModifierButton::Dodge)) => {
                    Response::Transition(State::dodging_left())
                }
                (Some(MoveButton::Right), Some(ModifierButton::Dodge)) => {
                    Response::Transition(State::dodging_right())
                }
                (None, Some(ModifierButton::Dodge)) => Response::Transition(State::dodging_left()),

                // Jumping
                (Some(MoveButton::Right), Some(ModifierButton::Jump)) => {
                    Response::Transition(State::jumping_right())
                }
                (Some(MoveButton::Left), Some(ModifierButton::Jump)) => {
                    Response::Transition(State::jumping_left())
                }
                (None, Some(ModifierButton::Jump)) => Response::Transition(State::jumping_left()),

                // Healing
                (Some(MoveButton::Right), Some(ModifierButton::Heal)) => {
                    Response::Transition(State::healing_right())
                }
                (Some(MoveButton::Left), Some(ModifierButton::Heal)) => {
                    Response::Transition(State::healing_left())
                }
                (None, Some(ModifierButton::Heal)) => Response::Transition(State::healing_left()),

                // Attacking
                (Some(MoveButton::Right), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::attacking_right())
                }
                (Some(MoveButton::Left), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::attacking_left())
                }
                (None, Some(ModifierButton::Attack)) => {
                    Response::Transition(State::attacking_left())
                }

                // Parry
                (Some(MoveButton::Right), Some(ModifierButton::Parry)) => {
                    Response::Transition(State::parry_right())
                }
                (Some(MoveButton::Left), Some(ModifierButton::Parry)) => {
                    Response::Transition(State::parry_left())
                }
                (None, Some(ModifierButton::Parry)) => Response::Transition(State::parry_left()),
                _ => Handled,
            },

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
            _ => Handled,
        }
    }

    #[state]
    fn move_right(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::InputChanged(input) => match (&input.0, &input.1) {
                // Moving
                (Some(MoveButton::Left), None) => Response::Transition(State::move_left()),

                // Jumping
                (Some(MoveButton::Right), Some(ModifierButton::Jump)) => {
                    Response::Transition(State::jumping_right())
                }
                (Some(MoveButton::Left), Some(ModifierButton::Jump)) => {
                    Response::Transition(State::jumping_left())
                }
                (None, Some(ModifierButton::Jump)) => Response::Transition(State::jumping_right()),

                // Dodging
                (Some(MoveButton::Left), Some(ModifierButton::Dodge)) => {
                    Response::Transition(State::dodging_left())
                }
                (Some(MoveButton::Right), Some(ModifierButton::Dodge)) => {
                    Response::Transition(State::dodging_right())
                }
                (None, Some(ModifierButton::Dodge)) => Response::Transition(State::dodging_right()),

                // Healing
                (Some(MoveButton::Right), Some(ModifierButton::Heal)) => {
                    Response::Transition(State::healing_right())
                }
                (Some(MoveButton::Left), Some(ModifierButton::Heal)) => {
                    Response::Transition(State::healing_left())
                }
                (None, Some(ModifierButton::Heal)) => Response::Transition(State::healing_right()),

                // Attacking
                (Some(MoveButton::Right), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::attacking_right())
                }
                (Some(MoveButton::Left), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::attacking_left())
                }
                (None, Some(ModifierButton::Attack)) => {
                    Response::Transition(State::attacking_right())
                }

                // Parry
                (Some(MoveButton::Right), Some(ModifierButton::Parry)) => {
                    Response::Transition(State::parry_right())
                }
                (Some(MoveButton::Left), Some(ModifierButton::Parry)) => {
                    Response::Transition(State::parry_left())
                }
                (None, Some(ModifierButton::Parry)) => Response::Transition(State::parry_right()),

                // Idle
                (None, None) => Response::Transition(State::idle_right()),
                _ => Handled,
            },

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
            _ => Handled,
        }
    }

    #[state]
    fn move_left(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::InputChanged(input) => match (&input.0, &input.1) {
                // Moving
                (Some(MoveButton::Right), None) => Response::Transition(State::move_right()),

                // Jumping
                (Some(MoveButton::Right), Some(ModifierButton::Jump)) => {
                    Response::Transition(State::jumping_right())
                }
                (Some(MoveButton::Left), Some(ModifierButton::Jump)) => {
                    Response::Transition(State::jumping_left())
                }
                (None, Some(ModifierButton::Jump)) => Response::Transition(State::jumping_left()),

                // Dodging
                (Some(MoveButton::Left), Some(ModifierButton::Dodge)) => {
                    Response::Transition(State::dodging_left())
                }
                (Some(MoveButton::Right), Some(ModifierButton::Dodge)) => {
                    Response::Transition(State::dodging_right())
                }

                // Healing
                (Some(MoveButton::Right), Some(ModifierButton::Heal)) => {
                    Response::Transition(State::healing_right())
                }
                (Some(MoveButton::Left), Some(ModifierButton::Heal)) => {
                    Response::Transition(State::healing_left())
                }
                (None, Some(ModifierButton::Heal)) => Response::Transition(State::healing_left()),

                // Attacking
                (Some(MoveButton::Right), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::attacking_right())
                }
                (Some(MoveButton::Left), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::attacking_left())
                }
                (None, Some(ModifierButton::Attack)) => {
                    Response::Transition(State::attacking_left())
                }

                // Parry
                (Some(MoveButton::Right), Some(ModifierButton::Parry)) => {
                    Response::Transition(State::parry_right())
                }
                (Some(MoveButton::Left), Some(ModifierButton::Parry)) => {
                    Response::Transition(State::parry_left())
                }
                (None, Some(ModifierButton::Parry)) => Response::Transition(State::parry_left()),

                // Idle
                (None, None) => Response::Transition(State::idle_left()),
                _ => Handled,
            },

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
            _ => Handled,
        }
    }

    #[state]
    fn dodging_right(&mut self, event: &Event) -> Response<State> {
        match event {
            // Moving
            Event::TimerElapsed(inputs) => match (&inputs.0, &inputs.1) {
                (None, None) => Response::Transition(State::idle_right()),
                (Some(MoveButton::Left), None) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), None) => Response::Transition(State::move_right()),
                _ => Handled,
            },

            // Falling
            Event::FailedFloorCheck(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Right), None) => {
                    Response::Transition(State::move_falling_right())
                }
                (Some(MoveButton::Left), None) => Response::Transition(State::move_falling_left()),
                (None, None) => Response::Transition(State::falling_right()),
                _ => Response::Transition(State::falling_right()),
            },
            _ => Handled,
        }
    }

    #[state]
    fn dodging_left(&mut self, event: &Event) -> Response<State> {
        match event {
            // Moving
            Event::TimerElapsed(inputs) => match (&inputs.0, &inputs.1) {
                (None, None) => Response::Transition(State::idle_left()),
                (Some(MoveButton::Left), None) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), None) => Response::Transition(State::move_right()),
                _ => Handled,
            },

            // Dodging
            Event::FailedFloorCheck(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Right), None) => {
                    Response::Transition(State::move_falling_right())
                }
                (Some(MoveButton::Left), None) => Response::Transition(State::move_falling_left()),
                (None, None) => Response::Transition(State::falling_left()),
                _ => Response::Transition(State::falling_left()),
            },
            _ => Handled,
        }
    }

    #[state]
    fn jumping_right(&mut self, event: &Event) -> Response<State> {
        match event {
            // Jumping
            Event::InputChanged(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Left), None) => Response::Transition(State::jumping_left()),
                _ => Handled,
            },

            // Falling
            Event::FailedFloorCheck(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Right), None) => {
                    Response::Transition(State::move_falling_right())
                }
                (Some(MoveButton::Left), None) => Response::Transition(State::move_falling_left()),
                (None, None) => Response::Transition(State::falling_right()),
                _ => Response::Transition(State::falling_right()),
            },
            _ => Handled,
        }
    }

    #[state]
    fn jumping_left(&mut self, event: &Event) -> Response<State> {
        match event {
            // Jumping
            Event::InputChanged(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Right), None) => Response::Transition(State::jumping_right()),
                _ => Handled,
            },

            // Falling
            Event::FailedFloorCheck(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Right), None) => {
                    Response::Transition(State::move_falling_right())
                }
                (Some(MoveButton::Left), None) => Response::Transition(State::move_falling_left()),
                (None, None) => Response::Transition(State::falling_left()),
                _ => Response::Transition(State::falling_left()),
            },
            _ => Handled,
        }
    }

    /// Player is falling and providing an x axis value of 0.0.
    #[state]
    fn falling_right(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::InputChanged(inputs) => match (&inputs.0, &inputs.1) {
                // Still falling
                (Some(MoveButton::Left), None) => Response::Transition(State::move_falling_left()),
                (Some(MoveButton::Right), None) => {
                    Response::Transition(State::move_falling_right())
                }

                // Air attack
                (Some(MoveButton::Left), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::move_left_air_attack())
                }
                (Some(MoveButton::Right), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::move_right_air_attack())
                }
                (None, Some(ModifierButton::Attack)) => {
                    Response::Transition(State::air_attack_right())
                }
                _ => Handled,
            },

            // On floor
            Event::Landed(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Left), None) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), None) => Response::Transition(State::move_right()),
                (None, None) => Response::Transition(State::idle_right()),
                _ => Handled,
            },
            _ => Handled,
        }
    }

    /// Player is falling and providing an x axis value of 0.0.
    #[state]
    fn falling_left(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::InputChanged(inputs) => match (&inputs.0, &inputs.1) {
                // Still falling
                (Some(MoveButton::Right), None) => {
                    Response::Transition(State::move_falling_right())
                }
                (Some(MoveButton::Left), None) => Response::Transition(State::move_falling_left()),

                // Air attack
                (Some(MoveButton::Left), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::move_left_air_attack())
                }
                (Some(MoveButton::Right), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::move_right_air_attack())
                }
                (None, Some(ModifierButton::Attack)) => {
                    Response::Transition(State::air_attack_left())
                }
                _ => Handled,
            },

            // On floor
            Event::Landed(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Left), None) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), None) => Response::Transition(State::move_right()),
                (None, None) => Response::Transition(State::idle_left()),
                _ => Handled,
            },
            _ => Handled,
        }
    }

    /// Player is falling and providing positive x axis movement.
    #[state]
    fn move_falling_right(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::InputChanged(inputs) => match (&inputs.0, &inputs.1) {
                // Still falling
                (Some(MoveButton::Left), None) => Response::Transition(State::move_falling_left()),
                (None, None) => Response::Transition(State::falling_right()),

                // Air attack
                (Some(MoveButton::Left), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::move_left_air_attack())
                }
                (Some(MoveButton::Right), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::move_right_air_attack())
                }
                (None, Some(ModifierButton::Attack)) => {
                    Response::Transition(State::air_attack_right())
                }
                _ => Handled,
            },

            // On floor
            Event::Landed(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Left), None) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), None) => Response::Transition(State::move_right()),
                (None, None) => Response::Transition(State::idle_right()),
                _ => Handled,
            },
            _ => Handled,
        }
    }

    /// Player is falling and providing negative x axis movement.
    #[state]
    fn move_falling_left(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::InputChanged(inputs) => match (&inputs.0, &inputs.1) {
                // Still falling
                (Some(MoveButton::Right), None) => {
                    Response::Transition(State::move_falling_right())
                }
                (None, None) => Response::Transition(State::falling_left()),

                // Air attack
                (Some(MoveButton::Left), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::move_left_air_attack())
                }
                (Some(MoveButton::Right), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::move_right_air_attack())
                }
                (None, Some(ModifierButton::Attack)) => {
                    Response::Transition(State::air_attack_right())
                }
                _ => Handled,
            },

            // On floor
            Event::Landed(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Left), None) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), None) => Response::Transition(State::move_right()),
                (None, None) => Response::Transition(State::idle_left()),
                _ => Handled,
            },
            _ => Handled,
        }
    }

    // TODO: Chain attacking.
    #[state]
    fn attacking_right(&mut self, event: &Event) -> Response<State> {
        let chain_attacked = match event {
            Event::InputChanged(inputs) => matches!(
                (&inputs.0, &inputs.1),
                (Some(MoveButton::Right), Some(ModifierButton::Attack))
                    | (Some(MoveButton::Left), Some(ModifierButton::Attack))
                    | (None, Some(ModifierButton::Attack))
            ),
            _ => false,
        };

        match (event, chain_attacked) {
            // Attack chaining
            (Event::TimerElapsed(inputs), true) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Right), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::attack_right_2())
                }
                (Some(MoveButton::Left), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::attack_left_2())
                }
                (None, Some(ModifierButton::Attack)) => {
                    Response::Transition(State::attack_left_2())
                }
                _ => Handled,
            },

            // Moving
            (Event::TimerElapsed(inputs), false) => match (&inputs.0, &inputs.1) {
                // Moving
                (Some(MoveButton::Right), None) => Response::Transition(State::move_right()),
                (Some(MoveButton::Left), None) => Response::Transition(State::move_left()),
                (None, None) => Response::Transition(State::idle_right()),

                _ => Handled,
            },

            // Hurt
            (Event::Hurt, _) => Response::Transition(State::hurt_right()),
            (_, _) => Handled,
        }
    }

    // TODO: Chain attacking.
    #[state]
    fn attacking_left(&mut self, event: &Event) -> Response<State> {
        let chain_attacked = match event {
            Event::InputChanged(inputs) => matches!(
                (&inputs.0, &inputs.1),
                (Some(MoveButton::Right), Some(ModifierButton::Attack))
                    | (Some(MoveButton::Left), Some(ModifierButton::Attack))
                    | (None, Some(ModifierButton::Attack))
            ),
            _ => false,
        };

        match (event, chain_attacked) {
            // Attack chaining
            (Event::TimerElapsed(inputs), true) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Right), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::attack_right_2())
                }
                (Some(MoveButton::Left), Some(ModifierButton::Attack)) => {
                    Response::Transition(State::attack_left_2())
                }
                (None, Some(ModifierButton::Attack)) => {
                    Response::Transition(State::attack_left_2())
                }
                _ => Handled,
            },

            // Moving
            (Event::TimerElapsed(inputs), false) => match (&inputs.0, &inputs.1) {
                // Moving
                (Some(MoveButton::Right), None) => Response::Transition(State::move_right()),
                (Some(MoveButton::Left), None) => Response::Transition(State::move_left()),
                (None, None) => Response::Transition(State::idle_right()),

                _ => Handled,
            },

            // Hurt
            (Event::Hurt, _) => Response::Transition(State::hurt_right()),
            (_, _) => Handled,
        }
    }

    // TODO: Chain attacking
    #[state]
    fn attack_right_2(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Right), None) => Response::Transition(State::move_right()),
                (Some(MoveButton::Left), None) => Response::Transition(State::move_left()),
                (None, None) => Response::Transition(State::idle_right()),
                _ => Handled,
            },
            Event::Hurt => Response::Transition(State::hurt_right()),
            _ => Handled,
        }
    }

    // TODO: Chain attacking
    #[state]
    fn attack_left_2(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Right), None) => Response::Transition(State::move_right()),
                (Some(MoveButton::Left), None) => Response::Transition(State::move_left()),
                (None, None) => Response::Transition(State::idle_left()),
                _ => Handled,
            },
            Event::Hurt => Response::Transition(State::hurt_right()),
            _ => Handled,
        }
    }

    #[state]
    fn hurt_right(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed(input) => match (&input.0, &input.1) {
                // Moving
                (Some(MoveButton::Left), None) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), None) => Response::Transition(State::move_right()),

                // Idle
                (None, None) => Response::Transition(State::idle_right()),
                _ => Handled,
            },
            _ => Handled,
        }
    }

    #[state]
    fn hurt_left(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed(input) => match (&input.0, &input.1) {
                // Moving
                (Some(MoveButton::Left), None) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), None) => Response::Transition(State::move_right()),

                // Idle
                (None, None) => Response::Transition(State::idle_left()),
                _ => Handled,
            },
            _ => Handled,
        }
    }

    #[state]
    fn air_attack_right(&mut self, event: &Event) -> Response<State> {
        match event {
            // Moving
            Event::TimerElapsed(input) => match (&input.0, &input.1) {
                (Some(MoveButton::Left), None) => Response::Transition(State::move_falling_left()),
                (Some(MoveButton::Right), None) => {
                    Response::Transition(State::move_falling_right())
                }
                (None, None) => Response::Transition(State::falling_right()),

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
                _ => Handled,
            },

            // On floor
            Event::Landed(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Left), None) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), None) => Response::Transition(State::move_right()),
                (None, None) => Response::Transition(State::idle_right()),
                _ => Handled,
            },
            _ => Handled,
        }
    }

    #[state]
    fn move_right_air_attack(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed(input) => match (&input.0, &input.1) {
                // Moving
                (Some(MoveButton::Left), None) => Response::Transition(State::move_falling_left()),
                (Some(MoveButton::Right), None) => {
                    Response::Transition(State::move_falling_right())
                }
                (None, None) => Response::Transition(State::falling_right()),

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
                _ => Handled,
            },

            // On floor
            Event::Landed(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Left), None) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), None) => Response::Transition(State::move_right()),
                (None, None) => Response::Transition(State::idle_right()),
                _ => Handled,
            },
            _ => Handled,
        }
    }

    #[state]
    fn air_attack_left(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed(input) => match (&input.0, &input.1) {
                (Some(MoveButton::Left), None) => Response::Transition(State::move_falling_left()),
                (Some(MoveButton::Right), None) => {
                    Response::Transition(State::move_falling_right())
                }
                (None, None) => Response::Transition(State::falling_left()),

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
                _ => Handled,
            },

            // On floor
            Event::Landed(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Left), None) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), None) => Response::Transition(State::move_right()),
                (None, None) => Response::Transition(State::idle_left()),
                _ => Handled,
            },
            _ => Handled,
        }
    }

    #[state]
    fn move_left_air_attack(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed(input) => match (&input.0, &input.1) {
                // Moving
                (Some(MoveButton::Left), None) => Response::Transition(State::move_falling_left()),
                (Some(MoveButton::Right), None) => {
                    Response::Transition(State::move_falling_right())
                }
                (None, None) => Response::Transition(State::falling_left()),

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
                _ => Handled,
            },

            // On floor
            Event::Landed(inputs) => match (&inputs.0, &inputs.1) {
                (Some(MoveButton::Left), None) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), None) => Response::Transition(State::move_right()),
                (None, None) => Response::Transition(State::idle_left()),
                _ => Handled,
            },
            _ => Handled,
        }
    }

    #[state]
    fn healing_right(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed(inputs) => match (&inputs.0, &inputs.1) {
                // Moving
                (Some(MoveButton::Left), None) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), None) => Response::Transition(State::move_right()),
                (None, None) => Response::Transition(State::idle_right()),
                _ => Handled,
            },
            _ => Handled,
        }
    }

    #[state]
    fn healing_left(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed(inputs) => match (&inputs.0, &inputs.1) {
                // Moving
                (Some(MoveButton::Left), None) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), None) => Response::Transition(State::move_right()),
                (None, None) => Response::Transition(State::idle_left()),
                _ => Handled,
            },
            _ => Handled,
        }
    }

    #[state]
    fn parry_right(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed(inputs) => match (&inputs.0, &inputs.1) {
                // Moving
                (Some(MoveButton::Left), None) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), None) => Response::Transition(State::move_right()),
                (None, None) => Response::Transition(State::idle_right()),
                _ => Handled,
            },
            _ => Handled,
        }
    }

    #[state]
    fn parry_left(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed(inputs) => match (&inputs.0, &inputs.1) {
                // Moving
                (Some(MoveButton::Left), None) => Response::Transition(State::move_left()),
                (Some(MoveButton::Right), None) => Response::Transition(State::move_right()),
                (None, None) => Response::Transition(State::idle_left()),
                _ => Handled,
            },
            _ => Handled,
        }
    }
}
