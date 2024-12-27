use godot::{classes::InputEvent, prelude::*};

use crate::components::state_machines::movements::Movements;

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct InputManager {
    pub current_action: Option<Movements>,
}

#[godot_api]
impl INode for InputManager {
    fn unhandled_input(&mut self, event: Gd<InputEvent>) {
        if event.is_action_pressed("up") {
            self.current_action = Some(Movements::WALK_UP);
        } else if event.is_action_pressed("down") {
            self.current_action = Some(Movements::WALK_DOWN);
        } else if event.is_action_pressed("left") {
            godot_print!("got left");
            self.current_action = Some(Movements::WALK_LEFT);
        } else if event.is_action_pressed("right") {
            godot_print!("got right");
            self.current_action = Some(Movements::WALK_RIGHT);
            // } else if event.is_action_pressed("jump") {
            //     self.current_action = Some(Movements::JUMP);
        }
    }
}
