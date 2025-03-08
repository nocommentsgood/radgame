// TODO: This could probably be broken up into modules

// Player ====================================================
// Player character signals
pub const SIGNAL_PLAYER_DAMAGED: &str = "player_damaged";
pub const SIGNAL_HURTBOX_BODY_ENTERED: &str = "body_entered";
pub const SIGNAL_PLAYER_DIED: &str = "player_died";

// Player character callables
pub const CALLABLE_BODY_ENTERED_HURTBOX: &str = "on_body_entered_hurtbox";

// Player child nodes
pub const PLAYER_HITBOX: &str = "Hitbox";
pub const PLAYER_HURTBOX: &str = "Hurtbox";
// End Player ====================================================

// Enemies ====================================================

// Enemy signals
pub const SIGNAL_TESTENEMY_DIED: &str = "test_enemy_died";
pub const SIGNAL_AGRRO_AREA_ENTERED: &str = "body_entered";
pub const SIGNAL_AGGRO_AREA_EXITED: &str = "body_exited";
pub const SIGNAL_PLAYER_ENTERED_ATTACK_RANGE: &str = "body_entered";
pub const SIGNAL_HURTBOX_ENTERED: &str = "body_entered";

// Enemy callables
pub const CALLABLE_DESTROY_ENEMY: &str = "destroy";
pub const CALLABLE_ON_AGGRO_AREA_ENTERED: &str = "on_aggro_area_entered";
pub const CALLABLE_PLAYER_ENTERED_ATTACK_RANGE: &str = "on_player_enters_attack_range";
pub const CALLABLE_ON_AGGRO_AREA_EXITED: &str = "on_aggro_area_exited";
pub const CALLABLE_ON_HURTBOX_ENTERED: &str = "on_hurtbox_entered";

// Enemy child nodes
pub const ENEMY_SENSORS: &str = "EnemySensors";

// End Enemies ====================================================
