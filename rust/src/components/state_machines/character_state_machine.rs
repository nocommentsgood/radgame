use godot::{builtin::Vector2, obj::Gd};
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

    #[state]
    fn moving(
        &self,
        velocity: &Vector2,
        delta: &f64,
        event: &Event,
        context: &mut Gd<MainCharacter>,
    ) -> Response<State> {
        let speed = context.bind().get_running_speed();
        context.bind_mut().set_velocity(velocity.to_owned());
        context.set_velocity(velocity.to_owned() * speed);
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

    #[state]
    fn dodging(
        &mut self,
        event: &Event,
        velocity: &Vector2,
        delta: &f64,
        context: &mut Gd<MainCharacter>,
    ) -> Response<State> {
        let mut cooldown_timer = context.bind_mut().get_dodging_cooldown_timer();
        if cooldown_timer.get_time_left() > 0.0 {
            Response::Transition(State::moving(*velocity, *delta))
        } else {
            let speed = context.bind().get_dodging_speed();
            let time = context.bind().get_dodging_animation_timer();

            context.set_velocity(velocity.to_owned() * speed);
            context.move_and_slide();
            context.bind_mut().set_dodging_animation_timer(time - delta);

            if time <= 0.0 {
                context.bind_mut().reset_dodging_animation_timer();
                cooldown_timer.start();
                match event {
                    Event::None => Response::Transition(State::idle()),
                    Event::Wasd {
                        velocity: vel,
                        delta,
                    } => Response::Transition(State::moving(*vel, *delta)),
                    _ => Handled,
                }
            } else {
                Handled
            }
        }
    }
}
