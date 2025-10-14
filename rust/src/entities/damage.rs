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

pub enum AttackResult {
    Hit,
    Killed,
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
