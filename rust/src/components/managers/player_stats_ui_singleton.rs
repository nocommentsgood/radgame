use godot::prelude::*;

use crate::classes::characters::{health_bar::HealthBar, main_character::MainCharacter};

/// A singleton responsible for handling signals to and from the player character stats and their
/// relative UI.
#[derive(GodotClass)]
#[class(init, base=Node2D)]
struct PlayerStatsUIHandler {
    #[init(node = "/root/Node2D/TileMapLayer/MainCharacter")]
    player: OnReady<Gd<MainCharacter>>,
    #[init(node = "/root/Node2D/HealthBar")]
    player_ui: OnReady<Gd<HealthBar>>,
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for PlayerStatsUIHandler {
    fn ready(&mut self) {
        self.connect_signals();
    }
}

impl PlayerStatsUIHandler {
    fn connect_signals(&mut self) {
        self.player.signals().player_health_changed().connect_obj(
            &*self.player_ui,
            |s: &mut HealthBar, previous_health, current_health, amount| {
                s.on_player_health_changed(previous_health, current_health, amount);
            },
        );
    }
}
