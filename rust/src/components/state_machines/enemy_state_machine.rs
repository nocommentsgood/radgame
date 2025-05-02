use godot::obj::Gd;
use statig::Response::Handled;
use statig::{state_machine, Response};

use crate::classes::characters::main_character::MainCharacter;

#[derive(Default, Debug, Clone)]
pub struct EnemyStateMachine {
    just_chain_attacked: bool,
}

#[derive(Default, Debug)]
pub enum EnemyEvent {
    FoundPlayer {
        player: Gd<MainCharacter>,
    },
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

#[state_machine(initial = "State::idle()", state(derive(Debug, Clone)))]
impl EnemyStateMachine {
    #[state(superstate = "passive")]
    fn idle(event: &EnemyEvent) -> Response<State> {
        match event {
            EnemyEvent::TimerElapsed => Response::Transition(State::patrol()),
            EnemyEvent::FoundPlayer { player: _player } => Response::Super,
            EnemyEvent::FailedFloorCheck => Response::Transition(State::falling()),
            _ => Response::Handled,
        }
    }

    #[state(superstate = "passive")]
    fn patrol(event: &EnemyEvent) -> Response<State> {
        match event {
            EnemyEvent::TimerElapsed => Response::Transition(State::idle()),
            EnemyEvent::FoundPlayer { player: _player } => Response::Super,
            EnemyEvent::FailedFloorCheck => Response::Transition(State::falling()),
            _ => Handled,
        }
    }

    #[state(superstate = "aggresive")]
    fn chase_player(&mut self, event: &EnemyEvent, player: &Gd<MainCharacter>) -> Response<State> {
        match event {
            EnemyEvent::LostPlayer => Response::Super,
            EnemyEvent::FailedFloorCheck => Response::Transition(State::falling()),
            EnemyEvent::InAttackRange => {
                if self.just_chain_attacked {
                    self.just_chain_attacked = false;
                    Response::Transition(State::attack(player.clone()))
                } else {
                    self.just_chain_attacked = true;
                    Response::Transition(State::attack_2(player.clone()))
                }
            }
            _ => Handled,
        }
    }

    #[state(superstate = "aggresive")]
    fn attack(event: &EnemyEvent, player: &Gd<MainCharacter>) -> Response<State> {
        match event {
            EnemyEvent::FailedFloorCheck => Response::Transition(State::falling()),
            EnemyEvent::LostPlayer => Response::Super,
            EnemyEvent::TimerElapsed => Response::Transition(State::chase_player(player.clone())),
            _ => Handled,
        }
    }

    #[state(superstate = "aggresive")]
    fn attack_2(event: &EnemyEvent, player: &Gd<MainCharacter>) -> Response<State> {
        match event {
            EnemyEvent::FailedFloorCheck => Response::Transition(State::falling()),
            EnemyEvent::LostPlayer => Response::Super,
            EnemyEvent::TimerElapsed => Response::Transition(State::chase_player(player.clone())),
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
            EnemyEvent::FoundPlayer { player } => {
                Response::Transition(State::chase_player(player.to_owned()))
            }
            _ => Response::Super,
        }
    }
}
