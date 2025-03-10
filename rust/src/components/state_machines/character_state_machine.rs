use godot::builtin::Vector2;
use statig::blocking::*;

use crate::classes::characters::main_character::MainCharacter;

#[derive(Default, Debug, Clone)]
pub struct CharacterStateMachine;

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Dodging {
                velocity: _,
                delta: _,
            } => write!(f, "dodge"),
            State::Moving {
                velocity: _,
                delta: _,
            } => write!(f, "run"),
            State::Attacking {
                velocity: _,
                delta: _,
            } => write!(f, "attack"),
            State::Jumping {
                velocity: _,
                delta: _,
            } => write!(f, "jumping"),
            State::Falling {
                velocity: _,
                delta: _,
            } => write!(f, "falling"),
            State::Idle {} => write!(f, "idle"),
            State::Handle {} => write!(f, "handled"),
        }
    }
}

#[derive(Debug)]
pub enum Event {
    Wasd { velocity: Vector2, delta: f64 },
    DodgeButton { velocity: Vector2, delta: f64 },
    AttackButton { velocity: Vector2, delta: f64 },
    JumpButton { velocity: Vector2, delta: f64 },
    None,
}

#[state_machine(initial = "State::idle()", state(derive(Debug, Clone)))]
impl CharacterStateMachine {
    #[state]
    fn idle(event: &Event, context: &mut MainCharacter) -> Response<State> {
        let response = context.idle(event);
        match response {
            State::Moving { velocity, delta } => {
                Response::Transition(State::moving(velocity, delta))
            }
            State::Attacking { velocity, delta } => {
                Response::Transition(State::attacking(velocity, delta))
            }
            State::Jumping { velocity, delta } => {
                Response::Transition(State::jumping(velocity, delta))
            }
            _ => Handled,
        }
    }

    #[state]
    fn moving(
        &self,
        event: &Event,
        velocity: &Vector2,
        delta: &f64,
        context: &mut MainCharacter,
    ) -> Response<State> {
        let response = context.move_character(event, *velocity, *delta);
        match response {
            State::Dodging { velocity, delta } => {
                Response::Transition(State::dodging(velocity, delta))
            }
            State::Moving { velocity, delta } => {
                Response::Transition(State::moving(velocity, delta))
            }
            State::Attacking { velocity, delta } => {
                Response::Transition(State::attacking(velocity, delta))
            }
            State::Jumping { velocity, delta } => {
                Response::Transition(State::jumping(velocity, delta))
            }
            State::Idle {} => Response::Transition(State::idle()),
            _ => Handled,
        }
    }

    #[state]
    fn dodging(
        &mut self,
        event: &Event,
        velocity: &Vector2,
        delta: &f64,
        context: &mut MainCharacter,
    ) -> Response<State> {
        let response = context.dodge(event, *velocity, *delta);
        match response {
            State::Idle {} => Response::Transition(State::idle()),
            State::Dodging { velocity, delta } => {
                Response::Transition(State::dodging(velocity, delta))
            }
            State::Moving { velocity, delta } => {
                Response::Transition(State::moving(velocity, delta))
            }
            _ => Handled,
        }
    }

    #[state]
    fn attacking(
        event: &Event,
        velocity: &Vector2,
        delta: &f64,
        context: &mut MainCharacter,
    ) -> Response<State> {
        let response = context.attack(event, *velocity, *delta);

        match response {
            State::Moving { velocity, delta } => {
                Response::Transition(State::moving(velocity, delta))
            }
            State::Idle {} => Response::Transition(State::idle()),
            State::Dodging { velocity, delta } => {
                Response::Transition(State::dodging(velocity, delta))
            }
            _ => Handled,
        }
    }

    #[state]
    fn jumping(
        event: &Event,
        velocity: &Vector2,
        delta: &f64,
        context: &mut MainCharacter,
    ) -> Response<State> {
        let response = context.jump(event, *velocity, *delta);

        match response {
            State::Jumping { velocity, delta } => {
                Response::Transition(State::jumping(velocity, delta))
            }
            State::Falling { velocity, delta } => {
                Response::Transition(State::falling(velocity, delta))
            }
            State::Attacking { velocity, delta } => {
                Response::Transition(State::attacking(velocity, delta))
            }
            State::Moving { velocity, delta } => {
                Response::Transition(State::moving(velocity, delta))
            }
            State::Dodging { velocity, delta } => {
                Response::Transition(State::dodging(velocity, delta))
            }
            State::Idle {} => Response::Transition(State::idle()),
            _ => Handled,
        }
    }

    #[state]
    fn falling(
        event: &Event,
        velocity: &Vector2,
        delta: &f64,
        context: &mut MainCharacter,
    ) -> Response<State> {
        let response = context.fall(event, *velocity, *delta);

        match response {
            State::Falling { velocity, delta } => {
                Response::Transition(State::falling(velocity, delta))
            }
            State::Idle {} => Response::Transition(State::idle()),
            State::Moving { velocity, delta } => {
                Response::Transition(State::moving(velocity, delta))
            }
            State::Attacking { velocity, delta } => {
                Response::Transition(State::attacking(velocity, delta))
            }
            State::Dodging { velocity, delta } => {
                Response::Transition(State::dodging(velocity, delta))
            }
            _ => Handled,
        }
    }

    #[state]
    fn handle() -> Response<State> {
        Handled
    }
}
