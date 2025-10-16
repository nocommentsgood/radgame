use std::collections::HashMap;

use godot::{
    classes::{AnimationPlayer, Node2D},
    obj::{Gd, WithBaseField},
    prelude::*,
};

use crate::entities::{
    entity::{Entity, ID},
    entity_stats::ModifierKind,
    hit_reg::Hitbox,
};

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

struct Resource {
    amount: i64,
    max: i64,
}

impl Resource {
    pub fn new(amount: i64, max: i64) -> Self {
        Self { amount, max }
    }
    pub fn amount(&self) -> &i64 {
        &self.amount
    }

    pub fn increase(&mut self, amount: i64) {
        let a = self.amount.saturating_add(amount);
        self.amount = a.clamp(0, self.max);
    }

    pub fn decrease(&mut self, amount: i64) {
        let a = self.amount.saturating_sub(amount);
        self.amount = a.clamp(0, self.max);
    }

    pub fn increase_max(&mut self, max: i64) {
        self.max = max;
    }
}

pub struct Stamina(Resource);
impl Stamina {
    pub fn new(amount: i64, max: i64) -> Self {
        Self(Resource::new(amount, max))
    }
}

pub struct Health(Resource);
impl Health {
    pub fn new(amount: i64, max: i64) -> Self {
        Self(Resource::new(amount, max))
    }

    pub fn take_damage(&mut self, damage: Damage) {
        self.0.decrease(damage.0);
    }
}

pub struct Mana(Resource);
impl Mana {
    pub fn new(amount: i64, max: i64) -> Self {
        Self(Resource::new(amount, max))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Damage(i64);

#[derive(Clone, Copy)]
pub enum DamageType {
    Elemental(Element),
    Physical,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Element {
    Magic,
    Poison,
    Lightning,
    Fire,
}

#[derive(Clone, Copy)]
pub enum Resistance {
    Physical(i64),
    Elemental(Element, i64),
}

#[derive(Clone)]
pub struct AttackData {
    pub parryable: bool,
    pub damage: Damage,
}

#[derive(Debug)]
struct Attack {
    damage: Damage,
    kind: AttackKind,
    resource_cost: AttackResourceCost,
    parryable: bool,
}

#[derive(Debug)]
enum PlayerAttacks {
    SimpleMelee,
    ChargedMelee,
    FireSpell,
}

impl PlayerAttacks {
    // TODO: Refactor player_level
    pub fn build(&self, player_level: i64) -> Attack {
        match self {
            PlayerAttacks::SimpleMelee => Attack {
                damage: Damage(player_level * 10),
                kind: AttackKind::Melee,
                resource_cost: AttackResourceCost::Stamina(5),
                parryable: true,
            },

            PlayerAttacks::ChargedMelee => Attack {
                damage: Damage(player_level * 15),
                kind: AttackKind::Melee,
                resource_cost: AttackResourceCost::Stamina(10),
                parryable: true,
            },

            _ => todo!(),
        }
    }
}

#[derive(Debug)]
enum AttackResourceCost {
    Stamina(i64),
    Mana(i64),
}

#[derive(Debug)]
enum AttackKind {
    Melee,
    ElementalMelee(Element),
    OffensiveSpell(Element),
}

enum AttackResult {
    AppliedDamage { amount: i64, killed: bool },

    // Due to resistances and/or defense, the defender took no damage.
    Absorbed,
}

enum EntityTypes {
    Player(Gd<super::player::main_character::MainCharacter>),
}

struct Defense {
    resistances: Vec<Resistance>,
}

impl Defense {
    pub fn apply_resistances(&self, attack: Attack) -> Damage {
        let mut amount = attack.damage.0;

        for resistance in &self.resistances {
            match (&attack.kind, resistance) {
                (AttackKind::Melee, Resistance::Physical(val)) => {
                    amount -= val;
                }
                (AttackKind::ElementalMelee(_), Resistance::Physical(val)) => {
                    amount -= val;
                }
                (
                    AttackKind::ElementalMelee(attack_element),
                    Resistance::Elemental(resist_element, val),
                ) => {
                    if attack_element == resist_element {
                        amount -= val;
                    }
                }
                _ => (),
            }
        }
        Damage(amount)
    }
}

struct CombatSystem {
    attackers: HashMap<ID, EntityTypes>,
}
impl CombatSystem {
    pub fn handle_attack(&self, attacker_id: &super::entity::ID, attack: Attack, defense: Defense) {
        let raw_amount = defense.apply_resistances(attack);
        let Some(attacker) = self.attackers.get(attacker_id) else {
            return println!("Couldn't find attacker");
        };

        // if let Ok(()) = attacker.can_attack(attack.resource_cost) {
        //     match attack.kind {
        //         AttackKind::Melee { parryable } => {
        //             let resistances = defender.get_melee_resistances();
        //             let raw_damage = defender.defense.apply_resistances();
        //             defender.handle(AttackResult);
        //         }
        //         AttackKind::ElementalMelee { parryable, element } => todo!(),
        //         AttackKind::OffensiveSpell => todo!(),
        //     }
        // }
    }
}
                }
                AttackKind::ElementalMelee { parryable, element } => todo!(),
                AttackKind::OffensiveSpell => todo!(),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::Resource;
    #[test]
    fn resouce_math() {
        let mut resource = Resource::new(20, 30);
        resource.increase(11);
        assert!(resource.amount == 30);

        resource.decrease(10);
        assert_eq!(20, resource.amount);

        resource.decrease(21);
        assert_eq!(0, resource.amount);

        resource.increase_max(31);
        resource.increase(32);
        assert_eq!(31, resource.amount);
    }
}
