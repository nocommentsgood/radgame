use godot::prelude::*;

use crate::classes::characters::{
    main_character::MainCharacter,
    shaky_player_camera::{ShakyPlayerCamera, TraumaLevel},
};

#[derive(GodotClass)]
#[class(init, base=Node)]
struct PlayerManager {
    player: Option<Gd<MainCharacter>>,
    base: Base<Node>,
}

#[godot_api]
impl INode for PlayerManager {
    fn enter_tree(&mut self) {
        let tree = self.base().get_tree().unwrap();
        let this = &self.to_gd();
        tree.signals()
            .node_added()
            .connect_other(this, Self::on_player_enters_tree);
    }
}

#[godot_api]
impl PlayerManager {
    #[signal]
    fn dummy();

    fn on_player_enters_tree(&mut self, player: Gd<Node>) {
        if let Ok(player) = player.clone().try_cast::<MainCharacter>() {
            self.player = Some(player);
            let this = &self.to_gd();
            if let Some(player) = &self.player {
                player
                    .signals()
                    .ready()
                    .connect_other(this, Self::on_player_ready);
            }
        }
    }

    fn on_player_ready(&mut self) {
        if let Some(player) = &self.player {
            let camera = player.get_node_as::<ShakyPlayerCamera>("ShakyPlayerCamera");
            player.signals().player_health_changed().connect_other(
                &camera,
                |camera, prev, new, _am| {
                    if new < prev {
                        camera.add_trauma(TraumaLevel::Low);
                    }
                },
            );
        }
    }
}
