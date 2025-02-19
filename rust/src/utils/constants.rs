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

// Enemy callables
pub const CALLABLE_DESTROY_ENEMY: &str = "destroy";
// End Enemies ====================================================
