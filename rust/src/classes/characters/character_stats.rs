use godot::builtin::real;

pub struct CharacterStats {
    pub health: i32,
    pub energy: i32,
    pub mana: i32,
    pub attack_damage: i32,
    pub running_speed: real,
    pub jumping_speed: real,
    pub falling_speed: real,
    pub dodging_speed: real,
    pub attacking_speed: real,
}

impl Default for CharacterStats {
    fn default() -> Self {
        Self {
            health: 30,
            energy: Default::default(),
            mana: Default::default(),
            attack_damage: 10,
            running_speed: 60.0,
            jumping_speed: 90.0,
            falling_speed: 110.0,
            dodging_speed: 80.0,
            attacking_speed: 10.0,
        }
    }
}
