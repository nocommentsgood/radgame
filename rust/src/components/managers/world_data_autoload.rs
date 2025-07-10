use godot::prelude::*;

use crate::classes::characters::main_character::MainCharacter;

#[derive(GodotClass)]
#[class(base=Node, init)]
pub struct WorldData {
    #[init(val = OnReady::manual())]
    pub player_path: OnReady<String>,

    base: Base<Node>,
}

#[godot_api]
impl INode for WorldData {
    fn ready(&mut self) {
        self.player_path
            .init("/root/Main/World/MainCharacter".to_string());

        let this = self.to_gd();
        let tree = self.base().get_tree().unwrap();
        tree.signals()
            .node_added()
            .connect_other(&this, Self::on_player_tree_position_changed);
    }
}

#[godot_api]
impl WorldData {
    fn on_player_tree_position_changed(&mut self, player: Gd<Node>) {
        if let Ok(player) = player.try_cast::<MainCharacter>() {
            println!("Player scenetree path has changed");
            *self.player_path = String::from(player.get_path());
        }
    }
}
