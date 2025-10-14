use godot::{
    classes::{AnimationPlayer, Node2D},
    obj::{Gd, WithBaseField},
    prelude::*,
};

use crate::entities::{entity_stats::ModifierKind, hit_reg::Hitbox};

pub trait HasHealth {
    fn get_health(&self) -> u32;
    fn set_health(&mut self, amount: u32);
    fn on_death(&mut self);
}

// TODO: Add resistance calculations to Damageable entities.
//
/// Implement on entities that can take damage. Requires the entity to have a Hitbox.
pub trait Damageable: HasHealth {
    /// Decreases a Damageable's health and checks if the Damageable's health is zero.
    fn take_damage(&mut self, amount: u32) {
        self.set_health(self.get_health().saturating_sub(amount));
        if self.get_health() == 0 {
            self.on_death();
        }
    }

    /// Handles the `AttackData` given by a `Hurtbox`. This should handle attack damage,
    /// resistances of the defender, attack types, etc.
    fn handle_attack(&mut self, attack: AttackData);
}

#[derive(Clone, Copy)]
pub struct Damage {
    pub raw: u32,
    pub d_type: DamageType,
}

#[derive(Clone, Copy)]
pub enum DamageType {
    Elemental(Element),
    Physical,
}

#[derive(Clone, Copy)]
pub enum Element {
    Magic,
    Poison,
    Lightning,
    Fire,
}

#[derive(Clone, Copy)]
pub enum Resistance {
    Physical(ModifierKind),
    Elemental(Element, ModifierKind),
}

#[derive(Clone)]
pub struct AttackData {
    pub parryable: bool,
    pub damage: Damage,
}

pub struct Health {
    amount: i64,
    max: i64,
}

impl Health {
    pub fn new(amount: i64, max: i64) -> Self {
        Self { amount, max }
    }
    pub fn amount(&self) -> &i64 {
        &self.amount
    }

    pub fn heal(&mut self, amount: i64) {
        let a = self.amount.saturating_add(amount);
        self.amount = a.clamp(0, self.max);
    }

    pub fn damage(&mut self, amount: i64) {
        let a = self.amount.saturating_sub(amount);
        self.amount = a.clamp(0, self.max);
    }

    pub fn increase_max(&mut self, max: i64) {
        self.max = max;
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn health_math() {
        use super::Health;
        let mut health = Health::new(20, 30);
        health.heal(11);
        assert!(health.amount == 30);

        health.damage(10);
        assert_eq!(20, health.amount);

        health.damage(21);
        assert_eq!(0, health.amount);

        health.increase_max(31);
        health.heal(32);
        assert_eq!(31, health.amount);
    }
}
