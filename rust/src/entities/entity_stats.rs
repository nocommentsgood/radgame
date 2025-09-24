use std::{collections::HashMap, error::Error};

use godot::prelude::GodotClass;

#[derive(Default)]
pub struct EntityStats(HashMap<Stat, StatVal>);

impl EntityStats {
    /// Inserts the given key-value.
    /// Performs no operations if the given key already exists.
    pub fn add(&mut self, stat: (Stat, StatVal)) {
        self.0.try_insert(stat.0, stat.1);
    }

    /// Inserts the given keys and values.
    /// If a given key already exists, it will not be added, nor will it's corresponding value.
    pub fn add_slice(&mut self, stats: &[(Stat, StatVal)]) {
        for s in stats {
            self.0.try_insert(s.0, s.1);
        }
    }

    pub fn update(&mut self, stat: (Stat, StatVal)) {
        if let Some(val) = self.0.get_mut(&stat.0) {
            *val = stat.1;
        }
    }

    /// Panics
    /// Panics if the given `Stat` is not present.
    pub fn get(&self, stat: Stat) -> &StatVal {
        self.0.get(&stat).unwrap()
    }

    /// Panics
    /// Panics if the given `Stat` is not present.
    pub fn get_raw(&self, stat: Stat) -> u32 {
        self.0.get(&stat).unwrap().0
    }

    /// Panics
    /// Panics if the given `Stat` is not present.
    pub fn get_mut(&mut self, stat: Stat) -> &mut StatVal {
        self.0.get_mut(&stat).unwrap()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub enum Stat {
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
    Level,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
// The optional second tuple value is used for "caching" the value when a modification is applied.
// When the modification is removed, the values are swapped and the second value is 'Option::None`.
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
    pub stat: Stat,
    pub modifier: ModifierKind,
}

impl StatModifier {
    pub fn new(stat: Stat, modifier: ModifierKind) -> Self {
        Self { stat, modifier }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ModifierKind {
    Flat(u32),
    Percent(f32),
}
