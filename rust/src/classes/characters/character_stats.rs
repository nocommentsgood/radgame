use godot::builtin::real;

pub struct CharacterStats {
    pub health: i32,
    pub max_health: i32,
    pub healing_amount: i32,
    pub energy: i32,
    pub mana: i32,
    pub attack_damage: i32,
    pub running_speed: real,
    pub jumping_speed: real,
    pub falling_speed: real,
    pub dodging_speed: real,
    pub attacking_speed: real,
    pub parry_length: f64,
    pub perfect_parry_length: f64,
}

impl Default for CharacterStats {
    fn default() -> Self {
        Self {
            health: 30,
            max_health: 40,
            healing_amount: 5,
            energy: Default::default(),
            mana: Default::default(),
            attack_damage: 10,
            running_speed: 90.0,
            jumping_speed: 200.0,
            falling_speed: 200.0,
            dodging_speed: 150.0,
            attacking_speed: 10.0,
            parry_length: 0.7,
            perfect_parry_length: 0.3,
        }
    }
}
impl CharacterStats {
    fn healable(&self) -> bool {
        self.health < self.max_health
    }

    pub fn heal(&mut self) {
        if self.healable() {
            self.health += self.healing_amount;
        }
    }
}
