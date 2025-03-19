use statig::blocking::*;

#[derive(Default, Debug, Clone)]
pub struct CharacterStateMachine;

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Attacking {} => write!(f, "attack"),
            State::Dodging {} => write!(f, "dodge"),
            State::Idle {} => write!(f, "idle"),
            State::Moving {} => write!(f, "run"),
            State::Falling {} => write!(f, "falling"),
            State::Jumping {} => write!(f, "jumping"),
            State::Grappling {} => write!(f, "grapple"),
        }
    }
}

#[derive(Debug, Default)]
pub enum Event {
    Wasd,
    WasdJustPressed,
    DodgeButton,
    AttackButton,
    JumpButton,
    GrabbedLedge,
    FailedFloorCheck,
    ActionReleasedEarly,
    TimerElapsed,
    TimerInProgress,
    OnFloor,
    #[default]
    None,
}

#[state_machine(initial = "State::idle()", state(derive(Debug, Clone)))]
impl CharacterStateMachine {
    #[state]
    fn idle(event: &Event) -> Response<State> {
        match event {
            Event::Wasd => Response::Transition(State::moving()),
            Event::AttackButton => Response::Transition(State::attacking()),
            Event::JumpButton => Response::Transition(State::jumping()),
            Event::FailedFloorCheck => Response::Transition(State::falling()),
            _ => Handled,
        }
    }

    #[state]
    fn moving(&self, event: &Event) -> Response<State> {
        match event {
            Event::Wasd => Response::Transition(State::moving()),
            Event::DodgeButton => Response::Transition(State::dodging()),
            Event::AttackButton => Response::Transition(State::attacking()),
            Event::JumpButton => Response::Transition(State::jumping()),
            Event::FailedFloorCheck => Response::Transition(State::falling()),
            Event::None => Response::Transition(State::idle()),
            _ => Handled,
        }
    }

    #[state]
    fn dodging(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed => Response::Transition(State::idle()),
            Event::TimerInProgress => Response::Transition(State::idle()),
            Event::FailedFloorCheck => Response::Transition(State::falling()),
            _ => Handled,
        }
    }

    #[state]
    fn attacking(event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed => Response::Transition(State::idle()),
            _ => Handled,
        }
    }

    #[state]
    fn jumping(event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed => Response::Transition(State::falling()),
            Event::ActionReleasedEarly => Response::Transition(State::falling()),
            Event::GrabbedLedge => Response::Transition(State::grappling()),
            _ => Handled,
        }
    }

    #[state]
    fn falling(event: &Event) -> Response<State> {
        match event {
            Event::OnFloor => Response::Transition(State::moving()),
            Event::GrabbedLedge => Response::Transition(State::grappling()),
            _ => Handled,
        }
    }

    #[state]
    fn grappling(event: &Event) -> Response<State> {
        match event {
            Event::WasdJustPressed => Response::Transition(State::falling()),
            Event::JumpButton => Response::Transition(State::jumping()),
            _ => Handled,
        }
    }
}
