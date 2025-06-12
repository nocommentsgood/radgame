use crate::components::managers::item::{ModifierKind, StatModifier};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub enum Stats {
    Health,
    MaxHealth,
    HealAmount,
    Energy,
    Mana,
    AttackDamage,
    RunningSpeed,
    JumpingSpeed,
    DodgingSpeed,
    AttackingSpeed,
}

pub struct StatVal(pub u32);

impl StatVal {
    pub fn apply_modifier(&mut self, modifier: StatModifier) {
        if let ModifierKind::Flat(val) = modifier.modifier {
            println!("prev stat val: {}", self.0);
            let amount = self.0 * val;
            self.0 += amount;
            println!("new stat val: {}", self.0);
        }

        if let ModifierKind::Percent(val) = modifier.modifier {
            println!("prev stat val: {}", self.0);
            let sum = (self.0 as f32 * val).round_ties_even() as u32;
            self.0 += sum;
            println!("new stat val: {}", self.0);
        }
    }
}
