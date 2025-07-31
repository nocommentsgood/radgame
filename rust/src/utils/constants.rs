// TODO: This could probably be broken up into modules

use godot::obj::Gd;

use crate::components::managers::global_data_singleton::GlobalData;

// Player ====================================================
// Player child nodes
pub const PLAYER_HITBOX: &str = "Hitbox";
pub const PLAYER_HURTBOX: &str = "Hurtbox";
// End Player ====================================================

// Enemies ====================================================
// Enemy child nodes
pub const ENEMY_SENSORS: &str = "EnemySensors";
// End Enemies ====================================================

// Globals ====================================================
pub const GLOBAL_DATA: &str = "GlobalData";

pub fn get_world_data() -> Gd<GlobalData> {
    godot::classes::Engine::singleton()
        .get_singleton(GLOBAL_DATA)
        .expect("Couldn't get GlobalData object")
        .cast::<GlobalData>()
}
