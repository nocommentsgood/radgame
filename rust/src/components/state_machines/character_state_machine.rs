use statig::blocking::*;

#[derive(Default, Debug, Clone)]
pub struct CharacterStateMachine {
    pub new_state: State,
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
    TimerInProgress,
    OnFloor,
    MovingToIdle,
    Hurt,
    #[default]
    None,
}

#[state_machine(initial = "State::idle()", state(derive(Debug, Clone)))]
impl CharacterStateMachine {
    fn transition_to(&mut self, next: State) -> Response<State> {
        self.new_state = next.clone();
        Response::Transition(next)
    }

    #[state]
    fn idle(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::Wasd => self.transition_to(State::moving()),
            Event::AttackButton => self.transition_to(State::attacking()),
            Event::JumpButton => self.transition_to(State::jumping()),
            Event::HealingButton => self.transition_to(State::healing()),
            Event::ParryButton => self.transition_to(State::parry()),
            Event::FailedFloorCheck => self.transition_to(State::falling()),
            Event::Hurt => self.transition_to(State::hurt()),
            _ => Handled,
        }
    }
    #[state]
    fn moving(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::Wasd => self.transition_to(State::moving()),
            Event::DodgeButton => self.transition_to(State::dodging()),
            Event::AttackButton => self.transition_to(State::attacking()),
            Event::ParryButton => self.transition_to(State::parry()),
            Event::JumpButton => self.transition_to(State::jumping()),
            Event::HealingButton => self.transition_to(State::healing()),
            Event::FailedFloorCheck => self.transition_to(State::falling()),
            Event::Hurt => self.transition_to(State::hurt()),
            Event::None => self.transition_to(State::idle()),
            _ => Handled,
        }
    }

    #[state]
    fn dodging(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed => self.transition_to(State::moving()),
            Event::MovingToIdle => self.transition_to(State::idle()),
            Event::TimerInProgress => self.transition_to(State::idle()),
            Event::FailedFloorCheck => self.transition_to(State::falling()),
            _ => Handled,
        }
    }

    #[state]
    fn attacking(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::AttackButton => self.transition_to(State::attack_2()),
            Event::MovingToIdle => self.transition_to(State::idle()),
            Event::TimerElapsed => self.transition_to(State::moving()),
            Event::ParryButton => self.transition_to(State::parry()),
            Event::Hurt => self.transition_to(State::hurt()),
            _ => Handled,
        }
    }

    #[state]
    fn attack_2(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed => self.transition_to(State::idle()),
            Event::FailedFloorCheck => self.transition_to(State::falling()),
            Event::Hurt => self.transition_to(State::hurt()),
            _ => Handled,
        }
    }

    #[state]
    fn hurt(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed => self.transition_to(State::idle()),
            _ => Handled,
        }
    }

    #[allow(unused_variables)]
    #[state]
    fn jumping(&mut self, event: &Event) -> Response<State> {
        self.transition_to(State::falling())
    }

    #[state]
    fn air_attack(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed => self.transition_to(State::falling()),
            _ => Handled,
        }
    }

    #[state]
    fn falling(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::AttackButton => self.transition_to(State::air_attack()),
            Event::MovingToIdle => self.transition_to(State::idle()),
            Event::OnFloor => self.transition_to(State::moving()),
            Event::GrabbedLedge => self.transition_to(State::grappling()),
            _ => Handled,
        }
    }

    #[state]
    fn grappling(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::WasdJustPressed => self.transition_to(State::falling()),
            Event::JumpButton => self.transition_to(State::jumping()),
            _ => Handled,
        }
    }

    #[state]
    fn healing(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed => self.transition_to(State::idle()),
            _ => Handled,
        }
    }

    #[state]
    fn parry(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed => self.transition_to(State::idle()),
            _ => Handled,
        }
    }
}
