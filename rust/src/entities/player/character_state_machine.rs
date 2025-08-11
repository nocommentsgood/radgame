use statig::blocking::*;

#[derive(Default, Debug, Clone)]
pub struct CharacterStateMachine {
    pub can_dodge: bool,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Hurt {} => write!(f, "hurt"),
            State::Attacking {} => write!(f, "attack"),
            State::Attack2 {} => write!(f, "chainattack"),
            State::Dodging {} => write!(f, "dodge"),
            State::Idle {} => write!(f, "idle"),
            State::Moving {} => write!(f, "run"),
            State::Falling {} => write!(f, "falling"),
            State::Jumping {} => write!(f, "jumping"),
            State::Grappling {} => write!(f, "grapple"),
            State::Healing {} => write!(f, "heal"),
            State::Parry {} => write!(f, "parry"),
            State::AirAttack {} => write!(f, "airattack"),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        State::Idle {}
    }
}

impl State {
    pub fn as_descriminant(&self) -> std::mem::Discriminant<Self> {
        std::mem::discriminant(self)
    }
}

pub fn to_descriminant(state: &State) -> std::mem::Discriminant<State> {
    match state {
        State::Hurt {} => std::mem::discriminant(&State::Hurt {}),
        State::Jumping {} => std::mem::discriminant(&State::Jumping {}),
        State::Moving {} => std::mem::discriminant(&State::Moving {}),
        State::Grappling {} => std::mem::discriminant(&State::Grappling {}),
        State::Dodging {} => std::mem::discriminant(&State::Dodging {}),
        State::Attacking {} => std::mem::discriminant(&State::Attacking {}),
        State::Healing {} => std::mem::discriminant(&State::Healing {}),
        State::Attack2 {} => std::mem::discriminant(&State::Attack2 {}),
        State::Falling {} => std::mem::discriminant(&State::Falling {}),
        State::Parry {} => std::mem::discriminant(&State::Parry {}),
        State::Idle {} => std::mem::discriminant(&State::Idle {}),
        State::AirAttack {} => std::mem::discriminant(&State::AirAttack {}),
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub enum Event {
    Wasd,
    WasdJustPressed,
    DodgeButton,
    AttackButton,
    JumpButton,
    ParryButton,
    GrabbedLedge,
    HealingButton,
    FailedFloorCheck,
    ActionReleasedEarly,
    TimerElapsed,
    ActionInterrupt,
    OnFloor,
    MovingToIdle,
    Hurt,
    #[default]
    None,
}

#[state_machine(initial = "State::idle()", state(derive(Debug, Clone)))]
impl CharacterStateMachine {
    #[state]
    fn idle(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::Wasd => Response::Transition(State::moving()),
            Event::AttackButton => Response::Transition(State::attacking()),
            Event::JumpButton => Response::Transition(State::jumping()),
            Event::HealingButton => Response::Transition(State::healing()),
            Event::ParryButton => Response::Transition(State::parry()),
            Event::FailedFloorCheck => Response::Transition(State::falling()),
            Event::Hurt => Response::Transition(State::hurt()),
            _ => Handled,
        }
    }
    #[state]
    fn moving(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::Wasd => Response::Transition(State::moving()),
            Event::DodgeButton => Response::Transition(State::dodging()),
            Event::AttackButton => Response::Transition(State::attacking()),
            Event::ParryButton => Response::Transition(State::parry()),
            Event::JumpButton => Response::Transition(State::jumping()),
            Event::HealingButton => Response::Transition(State::healing()),
            Event::FailedFloorCheck => Response::Transition(State::falling()),
            Event::Hurt => Response::Transition(State::hurt()),
            Event::None => Response::Transition(State::idle()),
            _ => Handled,
        }
    }

    #[state]
    fn dodging(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed => Response::Transition(State::moving()),
            Event::MovingToIdle => Response::Transition(State::idle()),
            Event::ActionInterrupt => Response::Transition(State::idle()),
            Event::FailedFloorCheck => Response::Transition(State::falling()),
            _ => Handled,
        }
    }

    #[state]
    fn attacking(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::AttackButton => Response::Transition(State::attack_2()),
            Event::MovingToIdle => Response::Transition(State::idle()),
            Event::TimerElapsed => Response::Transition(State::moving()),
            Event::ParryButton => Response::Transition(State::parry()),
            Event::Hurt => Response::Transition(State::hurt()),
            _ => Handled,
        }
    }

    #[state]
    fn attack_2(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed => Response::Transition(State::idle()),
            Event::FailedFloorCheck => Response::Transition(State::falling()),
            Event::Hurt => Response::Transition(State::hurt()),
            _ => Handled,
        }
    }

    #[state]
    fn hurt(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed => Response::Transition(State::idle()),
            _ => Handled,
        }
    }

    #[allow(unused_variables)]
    #[state]
    fn jumping(&mut self, event: &Event) -> Response<State> {
        Response::Transition(State::falling())
    }

    #[state]
    fn air_attack(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed => Response::Transition(State::falling()),
            _ => Handled,
        }
    }

    #[state]
    fn falling(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::AttackButton => Response::Transition(State::air_attack()),
            Event::MovingToIdle => Response::Transition(State::idle()),
            Event::OnFloor => Response::Transition(State::moving()),
            Event::GrabbedLedge => Response::Transition(State::grappling()),
            _ => Handled,
        }
    }

    #[state]
    fn grappling(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::WasdJustPressed => Response::Transition(State::falling()),
            Event::JumpButton => Response::Transition(State::jumping()),
            _ => Handled,
        }
    }

    #[state]
    fn healing(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed => Response::Transition(State::idle()),
            _ => Handled,
        }
    }

    #[state]
    fn parry(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed => Response::Transition(State::idle()),
            _ => Handled,
        }
    }

    #[action]
    fn enter_dodging(&mut self) {
        self.can_dodge = false;
    }
}
