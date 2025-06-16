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

pub struct StatVal(pub u32, Option<u32>);

impl StatVal {
    pub fn new(val: u32) -> Self {
        StatVal(val, None)
    }
    pub fn apply_modifier(&mut self, modifier: StatModifier) {
        if let ModifierKind::Flat(val) = modifier.modifier {
            println!("prev stat val: {}", self.0);
            let amount = self.0 * val;
            self.1 = Some(self.0);
            self.0 = amount;
            println!("new stat val: {}", self.0);
        }

        if let ModifierKind::Percent(val) = modifier.modifier {
            println!("prev stat val: {}", self.0);
            let sum = (self.0 as f32 * val).round_ties_even() as u32;
            self.1 = Some(self.0);
            self.0 = sum;
            println!("new stat val: {}", self.0);
        }
    }

    pub fn remove_modifier(&mut self, _modifier: StatModifier) {
        if let Some(v) = self.1 {
            println!("Removing modifier... Previous val: {}", self.0);
            self.0 = v;
            self.1 = None;
            println!("Removed modifier... Current val: {}", self.0);
        }
    }
}
