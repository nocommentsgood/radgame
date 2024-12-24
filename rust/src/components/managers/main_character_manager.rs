use godot::prelude::*;

use crate::{
    components::characters::main_character::MainCharacter, traits::player_moveable::PlayerMoveable,
};

#[derive(GodotClass)]
#[class(init, base=Node)]
struct MainCharacterManager {
    main_character: Option<Gd<MainCharacter>>,
    base: Base<Node>,
}

#[godot_api]
impl INode for MainCharacterManager {
    // fn init(base: Base<Node>) -> Self {
    //     Self { base }
    // }

    fn ready(&mut self) {
        let char = self
            .base()
            .try_get_node_as::<MainCharacter>("MainCharacter");
        self.main_character = char;
    }

    fn physics_process(&mut self, delta: f64) {
        if let Some(main) = &mut self.main_character {
            godot_print!("{}", main);
        }
        let input_direction = self.get_input_direction();
        if let Some(main) = &mut self.main_character {
            main.bind_mut().move_character(input_direction);
        }
    }
}

#[godot_api]
impl MainCharacterManager {
    #[func]
    fn get_input_direction(&self) -> Vector2 {
        Input::singleton().get_vector("left", "down", "up", "right")
    }
}
