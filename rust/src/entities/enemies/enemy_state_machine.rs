use std::fmt::Display;

use statig::Response::Handled;
use statig::prelude::StateMachine;
use statig::{Response, state_machine};

use crate::entities::enemies::time::EnemyTimers;
use crate::entities::movements::Direction;

#[derive(Clone)]
pub enum EnemySMType {
    Basic(statig::blocking::StateMachine<EnemyStateMachine>),
}

impl EnemySMType {
    pub fn handle(&mut self, event: &EnemyEvent) {
        dbg!(&event);
        match self {
            EnemySMType::Basic(state_machine) => state_machine.handle(event),
        }
    }

    pub fn state(&self) -> &State {
        match self {
            EnemySMType::Basic(state_machine) => state_machine.state(),
        }
    }

    pub fn inner_mut(&mut self) -> &mut StateMachine<EnemyStateMachine> {
        match self {
            EnemySMType::Basic(state_machine) => state_machine,
        }
    }
}

#[derive(Debug, Clone, Default)]
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
    RayCastFailed(Direction),
    WallCastRecovered,
    TimerElapsed(EnemyTimers),
    #[default]
    None,
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Patrol { .. } | State::RecoverLeft {} | State::RecoverRight {} => {
                write!(f, "patrol")
            }
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

#[state_machine(initial = "State::idle()", state(derive(Debug, Clone, PartialEq)))]
impl EnemyStateMachine {
    #[state]
    fn idle(&mut self, event: &EnemyEvent) -> Response<State> {
        match event {
            EnemyEvent::TimerElapsed(EnemyTimers::Idle) => Response::Transition(State::patrol()),
            EnemyEvent::FoundPlayer => Response::Transition(State::chase_player()),
            EnemyEvent::FailedFloorCheck => Response::Transition(State::falling()),
            _ => Response::Handled,
        }
    }

    #[state]
    fn patrol(&mut self, event: &EnemyEvent) -> Response<State> {
        match event {
            EnemyEvent::RayCastFailed(dir) => {
                if let Direction::Left = dir {
                    Response::Transition(State::recover_left())
                } else {
                    Response::Transition(State::recover_right())
                }
            }
            EnemyEvent::TimerElapsed(EnemyTimers::Patrol) => Response::Transition(State::idle()),
            EnemyEvent::FoundPlayer => Response::Transition(State::chase_player()),
            EnemyEvent::FailedFloorCheck => Response::Transition(State::falling()),
            _ => Handled,
        }
    }

    #[state]
    fn chase_player(&mut self, event: &EnemyEvent) -> Response<State> {
        match event {
            EnemyEvent::RayCastFailed(dir) => {
                if let Direction::Left = dir {
                    Response::Transition(State::recover_left())
                } else {
                    Response::Transition(State::recover_right())
                }
            }
            EnemyEvent::LostPlayer => Response::Transition(State::idle()),
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

    #[state]
    fn attack(event: &EnemyEvent) -> Response<State> {
        match event {
            EnemyEvent::FailedFloorCheck => Response::Transition(State::falling()),
            EnemyEvent::LostPlayer => Response::Transition(State::idle()),
            EnemyEvent::TimerElapsed(EnemyTimers::Attack) => {
                Response::Transition(State::chase_player())
            }
            _ => Handled,
        }
    }

    #[state]
    fn attack_2(event: &EnemyEvent) -> Response<State> {
        match event {
            EnemyEvent::FailedFloorCheck => Response::Transition(State::falling()),
            EnemyEvent::LostPlayer => Response::Transition(State::idle()),
            EnemyEvent::TimerElapsed(EnemyTimers::Attack) => {
                Response::Transition(State::chase_player())
            }
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

    #[state]
    fn recover_left(event: &EnemyEvent) -> Response<State> {
        match event {
            EnemyEvent::WallCastRecovered => Response::Transition(State::idle()),
            _ => Handled,
        }
    }

    #[state]
    fn recover_right(event: &EnemyEvent) -> Response<State> {
        match event {
            EnemyEvent::WallCastRecovered => Response::Transition(State::idle()),
            _ => Handled,
        }
    }
}
