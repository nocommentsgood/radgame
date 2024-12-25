use godot::prelude::*;

use crate::{
    classes::characters::main_character::MainCharacter,
    components::state_machines::main_character_state::CharacterState,
    traits::player_moveable::PlayerMoveable,
};

#[derive(GodotClass)]
#[class(init, base=Node)]
struct MainCharacterManager {
    main_character: Option<Gd<MainCharacter>>,
    main_character_state: CharacterState,
    base: Base<Node>,
}

#[godot_api]
impl INode for MainCharacterManager {
    fn ready(&mut self) {
        let char = self
            .base()
            .try_get_node_as::<MainCharacter>("MainCharacter");
        self.main_character = char;
    }

    fn physics_process(&mut self, delta: f64) {
        self.move_main_character();
    }
}

#[godot_api]
impl MainCharacterManager {
    #[func]
    fn get_input_direction(&self) -> Vector2 {
        Input::singleton().get_vector("left", "right", "up", "down")
    }

    // TODO: input handling should be moved to a singleton
    #[func]
    fn move_main_character(&mut self) {
        let input_direction = self.get_input_direction();
        if let Some(main) = &mut self.main_character {
            let velocity = main.bind().get_speed() * input_direction;
            main.bind_mut().move_character(velocity);
            self.main_character_state = CharacterState::MOVING;
        }
    }
}
