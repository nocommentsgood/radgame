use godot::{
    classes::Node2D,
    obj::{Base, Gd, OnReady},
    prelude::GodotClass,
};

use super::health_bar::HealthBar;

/// An autoload responsible for handling signals to and from the player character stats and their
/// relative UI.
#[derive(GodotClass)]
#[class(init, base=Node2D)]
struct PlayerStatsUIHandler {
    #[init(node = "/root/Main/HealthBar")]
    player_ui: OnReady<Gd<HealthBar>>,
    base: Base<Node2D>,
}
