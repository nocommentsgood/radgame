use godot::{builtin::Vector2, obj::Gd};
use statig::blocking::*;

use crate::classes::characters::main_character::MainCharacter;

#[derive(Default, Debug, Clone)]
pub struct CharacterStateMachine {
    running_speed: godot::builtin::real,
    dodging_speed: godot::builtin::real,
    just_dodged: bool,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Dodging {
                velocity: _,
                delta: _,
            } => write!(f, "run"),
            State::Moving {
                velocity: _,
                delta: _,
            } => write!(f, "run"),
            State::Idle {} => write!(f, "idle"),
        }
    }
}

#[derive(Debug)]
pub enum Event {
    Wasd { velocity: Vector2, delta: f64 },
    DodgeButton { velocity: Vector2, delta: f64 },
    AttackButton,
    None,
}

#[state_machine(initial = "State::idle()", state(derive(Debug, Clone)))]
impl CharacterStateMachine {
    #[state]
    fn idle(event: &Event) -> Response<State> {
        match event {
            Event::Wasd {
                velocity: vel,
                delta,
            } => Response::Transition(State::moving(*vel, *delta)),
            _ => Handled,
        }
    }

    #[state(entry_action = "entered_moving")]
    fn moving(
        &self,
        velocity: &Vector2,
        delta: &f64,
        event: &Event,
        context: &mut Gd<MainCharacter>,
    ) -> Response<State> {
        context.bind_mut().set_velocity(velocity.to_owned());
        context.set_velocity(velocity.to_owned() * self.running_speed * delta.to_owned() as f32);
        context.move_and_slide();

        match event {
            Event::None => Response::Transition(State::idle()),
            Event::Wasd {
                velocity: vel,
                delta,
            } => Response::Transition(State::moving(*vel, *delta)),
            Event::DodgeButton { velocity, delta } => {
                Response::Transition(State::dodging(*velocity, *delta))
            }
            _ => Handled,
        }
    }

    #[action]
    fn entered_moving(&mut self) {
        self.running_speed = 7000.0;
    }

    #[state(entry_action = "entered_dodging", exit_action = "leaving_dodging")]
    fn dodging(
        &self,
        event: &Event,
        velocity: &Vector2,
        delta: &f64,
        context: &mut Gd<MainCharacter>,
    ) -> Response<State> {
        let mut time = 2.0;
        while time > 0.0 {
            context
                .set_velocity(velocity.to_owned() * self.dodging_speed * delta.to_owned() as f32);
            context.move_and_slide();
            time -= delta;
        }
        let mut cooldown = context.bind_mut().get_dodging_cooldown_timer();
        cooldown.start();
        match event {
            Event::None => Response::Transition(State::idle()),
            Event::Wasd {
                velocity: vel,
                delta,
            } => Response::Transition(State::moving(*vel, *delta)),
            _ => Handled,
        }
    }

    #[action]
    fn leaving_dodging(&mut self) {
        self.just_dodged = true;
    }

    #[action]
    fn entered_dodging(&mut self) {
        self.just_dodged = true;
        self.dodging_speed = 500.0;
    }
}
