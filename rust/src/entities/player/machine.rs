use godot::builtin::Vector2;
use statig::blocking::*;

// enum HMove {
//     Left,
//     Right,
// }
//
// enum VMove {
//     Up,
//     Down,
// }
//
// struct Move(Option<HMove>, Option<VMove>);
//
// enum Modifier {
//     Dodge,
//     Jump,
//     Attack,
//     Heal,
// }
// enum MoveButton {
//     Left,
//     Right,
// }
//
// enum ModifierButton {
//     Dodge,
//     Jump,
//     Attack,
//     Heal,
// }
//
// struct Inputs(Option<MoveButton>, Option<ModifierButton>);
//
// enum Event {
//     InputChanged(Inputs),
//     TimerElapsed(Inputs),
//     FailedFloorCheck(Inputs),
// }
//
// #[derive(Default, Debug, Clone)]
// pub struct Machine;
//
// #[state_machine(
//     initial = "State::idle_right()",
//     state(derive(Debug, Clone, PartialEq, Copy))
// )]
// impl Machine {
//     #[state]
//     fn idle_right(event: &Event) -> Response<State> {
//         match event {
//             Event::InputChanged(input) => match (input.0, input.1) {
//                 (Some(MoveButton::Right), None) => todo!("run right"),
//                 _ => Handled,
//             },
//             Event::FailedFloorCheck(input) => match (&input.0, &input.1) {
//                 (Some(MoveButton::Right), None) => todo!("fall right"),
//                 _ => Handled,
//             },
//         }
//     }
//
//     #[state]
//     fn falling_right(event: &Event) -> Response<State> {
//         match event {
//
//         }
//     }
// }
