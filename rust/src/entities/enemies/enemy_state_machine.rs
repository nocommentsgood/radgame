use statig::Response::Handled;
use statig::{Response, state_machine};

use crate::entities::enemies::time::EnemyTimers;

#[derive(Clone)]
pub enum EnemySMType {
    Basic(statig::blocking::StateMachine<EnemyStateMachine>),
}

impl EnemySMType {
    pub fn handle(&mut self, event: &EnemyEvent) {
        match self {
            EnemySMType::Basic(state_machine) => state_machine.handle(event),
        }
    }

    pub fn state(&self) -> &State {
        match self {
            EnemySMType::Basic(state_machine) => state_machine.state(),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct EnemyStateMachine {
    just_chain_attacked: bool,
    disable_movement: bool,
}

#[derive(Default, Debug)]
pub enum EnemyEvent {
    FoundPlayer,
    FailedFloorCheck,
    OnFloor,
    LostPlayer,
    InAttackRange,
    RayCastNotColliding,
    TimerElapsed(EnemyTimers),
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

#[state_machine(initial = "State::idle()", state(derive(Debug, Clone, PartialEq)))]
impl EnemyStateMachine {
    #[state(superstate = "passive")]
    fn idle(&self, event: &EnemyEvent) -> Response<State> {
        match event {
            EnemyEvent::TimerElapsed(EnemyTimers::Idle) if !self.disable_movement => {
                Response::Transition(State::patrol())
            }
            EnemyEvent::FoundPlayer => Response::Super,
            EnemyEvent::FailedFloorCheck => Response::Transition(State::falling()),
            _ => Response::Handled,
        }
    }

    #[state(superstate = "passive")]
    fn patrol(&mut self, event: &EnemyEvent) -> Response<State> {
        match event {
            EnemyEvent::TimerElapsed(EnemyTimers::Patrol) => Response::Transition(State::idle()),
            EnemyEvent::RayCastNotColliding => {
                self.disable_movement = true;
                Response::Transition(State::idle())
            }
            EnemyEvent::FoundPlayer => Response::Super,
            EnemyEvent::FailedFloorCheck => Response::Transition(State::falling()),
            _ => Handled,
        }
    }

    #[state(superstate = "aggresive")]
    fn chase_player(&mut self, event: &EnemyEvent) -> Response<State> {
        match event {
            EnemyEvent::RayCastNotColliding if self.disable_movement => {
                Response::Transition(State::idle())
            }
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
            EnemyEvent::TimerElapsed(EnemyTimers::Attack) => {
                Response::Transition(State::chase_player())
            }
            _ => Handled,
        }
    }

    #[state(superstate = "aggresive")]
    fn attack_2(event: &EnemyEvent) -> Response<State> {
        match event {
            EnemyEvent::FailedFloorCheck => Response::Transition(State::falling()),
            EnemyEvent::LostPlayer => Response::Super,
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

    pub fn new() -> statig::blocking::StateMachine<Self> {
        statig::blocking::StateMachine::default()
    }
}
