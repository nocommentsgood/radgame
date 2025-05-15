use godot::{builtin::Vector2, obj::Gd};

use crate::classes::characters::main_character::MainCharacter;

pub trait PlayerMoveable {
    fn get_movement_animation(&mut self) -> String;

    fn move_main_character(mut main: Gd<MainCharacter>, velocity: Vector2) {
        main.set_velocity(velocity);
        main.bind_mut().set_velocity(velocity);
        main.move_and_slide();
    }

    // fn move_character_set_animation(
    //     mut main: Gd<MainCharacter>,
    //     velocity: Vector2,
    //     direction: Directions,
    // ) {
    //     main.set_velocity(velocity);
    //     main.bind_mut().set_velocity(velocity);
    //     let animation = main.bind_mut().get_movement_animation();
    //     {
    //         let guard = main.bind_mut().base_mut();
    //     }
    // }
}
