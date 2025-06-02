use godot::builtin::real;

use crate::components::managers::item::StatModifier;

pub struct CharacterStats {
    pub health: u32,
    pub max_health: u32,
    pub healing_amount: u32,
    pub energy: u32,
    pub mana: u32,
    pub attack_damage: u32,
    pub running_speed: real,
    pub jumping_speed: real,
    pub dodging_speed: real,
    pub attacking_speed: real,
    pub parry_length: f32,
    pub perfect_parry_length: f32,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub enum Stats {
    Health,
    MaxHealth,
    HealAmount,
    AttackDamage,
    RunningSpeed,
    JumpingSpeed,
    DodgingSpeed,
    AttackingSpeed,
    ParryLength,
    PerfectParryLength,
}

pub struct StatVal(pub f32);

impl StatVal {
    pub fn apply_modifier(&mut self, modifier: StatModifier) {
        let modif = match modifier.modifier {
            crate::components::managers::item::ModifierKind::Flat(val) => val,
            crate::components::managers::item::ModifierKind::Percent(val) => val,
        };

        println!("prev stat val: {}", self.0);
        let sum = self.0 * modif;
        self.0 += sum;
        println!("new stat val: {}", self.0);
    }
}

impl Default for CharacterStats {
    fn default() -> Self {
        Self {
            health: 50,
            max_health: 50,
            healing_amount: 20,
            energy: Default::default(),
            mana: Default::default(),
            attack_damage: 10,
            running_speed: 150.0,
            jumping_speed: 300.0,
            dodging_speed: 250.0,
            attacking_speed: 10.0,
            parry_length: 0.3,
            perfect_parry_length: 0.15,
        }
    }
}

impl CharacterStats {
    fn can_heal(&self) -> bool {
        self.health < self.max_health
    }

    pub fn heal(&mut self) {
        if self.can_heal() {
            self.health += self.healing_amount;
        }
    }
}
