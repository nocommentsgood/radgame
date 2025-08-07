use godot::prelude::GodotClass;

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

pub trait EntityResources {
    fn get_health(&self) -> u32;

    fn set_health(&mut self, amount: u32);

    fn get_energy(&self) -> u32;

    fn set_energy(&mut self, amount: u32);

    fn get_mana(&self) -> u32;

    fn set_mana(&mut self, amount: u32);
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

// TODO: Emit in signals without using Gd as parameter.
#[derive(GodotClass, Clone, Debug, PartialEq)]
#[class(no_init)]
pub struct StatModifier {
    pub stat: Stats,
    pub modifier: ModifierKind,
}

impl StatModifier {
    pub fn new(stat: Stats, modifier: ModifierKind) -> Self {
        Self { stat, modifier }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ModifierKind {
    Flat(u32),
    Percent(f32),
}
