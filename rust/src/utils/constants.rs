// TODO: This could probably be broken up into modules

// Player ====================================================
// Player character signals
pub const SIGNAL_PLAYER_DAMAGED: &str = "player_damaged";
pub const SIGNAL_HURTBOX_BODY_ENTERED: &str = "body_entered";

// Player character callables
pub const CALLABLE_BODY_ENTERED_HURTBOX: &str = "on_body_entered_hurtbox";

// Player child nodes
pub const PLAYER_HITBOX: &str = "Hitbox";
pub const PLAYER_HURTBOX: &str = "Hurtbox";
// End Player ====================================================

// Enemies ====================================================
pub const SIGNAL_TESTENEMY_DIED: &str = "test_enemy_died";
pub const SIGNAL_ENEMY_DETECTS_PLAYER: &str = "body_entered";

// Enemy callables
pub const CALLABLE_DESTROY_ENEMY: &str = "destroy";
pub const CALLABLE_ENEMY_SENSES_PLAYER: &str = "on_enemy_senses_player";

// Enemy child nodes
pub const PLAYER_SENSORS: &str = "PlayerSensors";

// End Enemies ====================================================
