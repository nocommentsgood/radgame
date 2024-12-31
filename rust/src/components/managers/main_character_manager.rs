use godot::{
    classes::{AnimatedSprite2D, ProjectSettings},
    prelude::*,
};

use crate::{
    classes::characters::main_character::MainCharacter,
    components::state_machines::main_character_state::CharacterState,
};

use super::input_manager::InputManager;

#[derive(GodotClass)]
#[class(init, base=Node)]
struct MainCharacterManager {
    main_character: Option<Gd<MainCharacter>>,
    main_character_state: CharacterState,
    input_manager: Option<Gd<InputManager>>,
    base: Base<Node>,
}

#[godot_api]
impl INode for MainCharacterManager {
    fn ready(&mut self) {
        let char = self
            .base()
            .try_get_node_as::<MainCharacter>("MainCharacter");
        self.main_character = char;

        let input_handler = self.base().get_node_as("InputManager");
        self.input_manager = Some(input_handler);
    }

    fn process(&mut self, _delta: f64) {
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
        let terminal_velocity = 600;
        let velocity = self.get_input_direction();
        let mut animated_sprite = self
            .base()
            .get_node_as::<AnimatedSprite2D>("MainCharacter/AnimatedSprite2D");
        let animation;

        // TODO: add 8 way animations
        if velocity.x > 0.0 {
            animated_sprite.set_flip_h(false);
            animation = "run_right";
        } else if velocity.x < 0.0 {
            animated_sprite.set_flip_h(true);
            animation = "run_left";
        } else if velocity.y < 0.0 {
            animation = "jump";
        } else {
            animation = "idle";
        }

        if let Some(main) = &mut self.main_character {
            let velocity = main.bind().get_speed() * velocity;

            if !main.bind().base().is_on_floor() {
                main.set_velocity(Vector2::DOWN);
            }

            main.bind_mut().base_mut().set_velocity(velocity);
            main.bind_mut().base_mut().move_and_slide();

            animated_sprite.play_ex().name(animation).done();
            self.main_character_state = CharacterState::Default;
        }
    }

    fn jump(&mut self) {}
}
