use godot::obj::Gd;
use statig::Response::Handled;
use statig::{Response, state_machine};

use crate::classes::characters::main_character::MainCharacter;

#[derive(Default, Debug, Clone)]
pub struct EnemyStateMachine {
    just_chain_attacked: bool,
}

#[derive(Default, Debug)]
pub enum EnemyEvent {
    FoundPlayer,
    FailedFloorCheck,
    OnFloor,
    LostPlayer,
    InAttackRange,
    TimerElapsed,
    #[default]
    None,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Patrol { .. } => write!(f, "patrol"),
            State::Idle {} => write!(f, "idle"),
            State::ChasePlayer { .. } => write!(f, "patrol"),
            State::Attack { .. } => write!(f, "attack"),
            State::Attack2 { .. } => write!(f, "chain_attack"),
            State::Falling { .. } => write!(f, "fall"),
        }
    }
}

impl State {
    pub fn as_discriminant(&self) -> std::mem::Discriminant<Self> {
        std::mem::discriminant(self)
    }
}

pub fn to_discriminant(state: &State) -> std::mem::Discriminant<State> {
    match state {
        State::Patrol {} => std::mem::discriminant(&State::Patrol {}),
        State::Idle {} => std::mem::discriminant(&State::Idle {}),
        State::ChasePlayer {} => std::mem::discriminant(&State::ChasePlayer {}),
        State::Attack {} => std::mem::discriminant(&State::Attack {}),
        State::Attack2 {} => std::mem::discriminant(&State::Attack2 {}),
        State::Falling {} => std::mem::discriminant(&State::Falling {}),
    }
}

#[state_machine(initial = "State::idle()", state(derive(Debug, Clone)))]
impl EnemyStateMachine {
    #[state(superstate = "passive")]
    fn idle(event: &EnemyEvent) -> Response<State> {
        match event {
            EnemyEvent::TimerElapsed => Response::Transition(State::patrol()),
            EnemyEvent::FoundPlayer => Response::Super,
            EnemyEvent::FailedFloorCheck => Response::Transition(State::falling()),
            _ => Response::Handled,
        }
    }

    #[state(superstate = "passive")]
    fn patrol(event: &EnemyEvent) -> Response<State> {
        match event {
            EnemyEvent::TimerElapsed => Response::Transition(State::idle()),
            EnemyEvent::FoundPlayer => Response::Super,
            EnemyEvent::FailedFloorCheck => Response::Transition(State::falling()),
            _ => Handled,
        }
    }

    #[state(superstate = "aggresive")]
    fn chase_player(&mut self, event: &EnemyEvent) -> Response<State> {
        match event {
            EnemyEvent::LostPlayer => Response::Super,
            EnemyEvent::FailedFloorCheck => Response::Transition(State::falling()),
            EnemyEvent::InAttackRange => {
                if self.just_chain_attacked {
                    self.just_chain_attacked = false;
                    Response::Transition(State::attack())
                } else {
                    self.just_chain_attacked = true;
                    Response::Transition(State::attack_2())
                }
            }
            _ => Handled,
        }
    }

    #[state(superstate = "aggresive")]
    fn attack(event: &EnemyEvent) -> Response<State> {
        match event {
            EnemyEvent::FailedFloorCheck => Response::Transition(State::falling()),
            EnemyEvent::LostPlayer => Response::Super,
            EnemyEvent::TimerElapsed => Response::Transition(State::chase_player()),
            _ => Handled,
        }
    }

    #[state(superstate = "aggresive")]
    fn attack_2(event: &EnemyEvent) -> Response<State> {
        match event {
            EnemyEvent::FailedFloorCheck => Response::Transition(State::falling()),
            EnemyEvent::LostPlayer => Response::Super,
            EnemyEvent::TimerElapsed => Response::Transition(State::chase_player()),
            _ => Handled,
        }
    }

    #[state]
    fn falling(event: &EnemyEvent) -> Response<State> {
        match event {
            EnemyEvent::OnFloor => Response::Transition(State::idle()),
            _ => Handled,
        }
    }

    #[superstate]
    fn aggresive(event: &EnemyEvent) -> Response<State> {
        match event {
            EnemyEvent::LostPlayer => Response::Transition(State::idle()),
            _ => Response::Super,
        }
    }

    #[superstate]
    fn passive(event: &EnemyEvent) -> Response<State> {
        match event {
            EnemyEvent::FoundPlayer => Response::Transition(State::chase_player()),
            _ => Response::Super,
        }
    }
}
