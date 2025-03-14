use godot::obj::Gd;
use statig::Response::Handled;
use statig::{state_machine, Response};

use crate::classes::characters::main_character::MainCharacter;

// TODO: State machine does not need to be aware of the player, which is currently being
// passed in. We can move all of the pointer passing to the TestEnemy class, so the
// TestEnemy handles all of that logic and just 'listens' to it's state, and performs the
// corresponding actions.

#[derive(Default, Debug, Clone)]
pub struct EnemyStateMachine;

#[derive(Default, Debug)]
pub enum EnemyEvent {
    FoundPlayer {
        player: Gd<MainCharacter>,
    },
    LostPlayer,
    InAttackRange,
    DamagedByPlayer,
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
            _ => Response::Handled,
        }
    }

    #[state(superstate = "passive")]
    fn patrol(event: &EnemyEvent) -> Response<State> {
        match event {
            EnemyEvent::TimerElapsed => Response::Transition(State::idle()),
            EnemyEvent::FoundPlayer { player: _player } => Response::Super,
            _ => Handled,
        }
    }

    #[allow(unused_variables)]
    #[state(superstate = "aggresive")]
    fn chase_player(event: &EnemyEvent, player: &Gd<MainCharacter>) -> Response<State> {
        match event {
            EnemyEvent::LostPlayer => Response::Super,
            EnemyEvent::InAttackRange => Response::Transition(State::attack(player.clone())),
            _ => Handled,
        }
    }

    #[state(superstate = "aggresive")]
    fn attack(event: &EnemyEvent, player: &Gd<MainCharacter>) -> Response<State> {
        match event {
            EnemyEvent::LostPlayer => Response::Super,
            EnemyEvent::TimerElapsed => Response::Transition(State::chase_player(player.clone())),
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
