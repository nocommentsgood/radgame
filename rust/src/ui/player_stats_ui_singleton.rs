use godot::prelude::*;

use super::health_bar::HealthBar;
use crate::entities::player::main_character::MainCharacter;

/// A singleton responsible for handling signals to and from the player character stats and their
/// relative UI.
#[derive(GodotClass)]
#[class(init, base=Node2D)]
struct PlayerStatsUIHandler {
    #[init(node = "/root/Main/World/MainCharacter")]
    player: OnReady<Gd<MainCharacter>>,
    #[init(node = "/root/Main/HealthBar")]
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
        self.player.signals().player_health_changed().connect_other(
            &*self.player_ui,
            |s: &mut HealthBar, previous_health, current_health, amount| {
                s.on_player_damaged(previous_health, current_health, amount);
            },
        );
    }
}
