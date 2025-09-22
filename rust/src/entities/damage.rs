use godot::{
    classes::{AnimationPlayer, Node2D},
    obj::{Gd, WithBaseField},
    prelude::*,
};

use crate::entities::{
    entity_stats::ModifierKind,
    hit_reg::{Hitbox, Hurtbox},
};

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
    Elemental(ElementType),
    Physical,
}

#[derive(Clone, Copy)]
pub enum ElementType {
    Magic,
    Poison,
    Lightning,
    Fire,
}

#[derive(Clone, Copy)]
pub enum Resistance {
    Physical(ModifierKind),
    Elemental(ElementType, ModifierKind),
}

#[derive(GodotClass, Clone)]
#[class(no_init)]
pub struct AttackData {
    pub hurtbox: Gd<Hurtbox>,
    pub parryable: bool,
    pub damage: Damage,
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

#[derive(GodotClass)]
#[class(base = Node2D, init)]
struct MockEnemy {
    #[init(node = "AnimationPlayer")]
    anim_player: OnReady<Gd<AnimationPlayer>>,

    #[init(val = 10)]
    health: u32,

    #[init(val = OnReady::manual())]
    hitbox: OnReady<Gd<Hitbox>>,

    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for MockEnemy {
    fn ready(&mut self) {
        let mut hitbox = self.base().get_node_as::<Hitbox>("EntityHitbox");
        let this = self.to_gd();
        hitbox.bind_mut().damageable_parent = Some(Box::new(this));
        self.hitbox.init(hitbox);
    }
}

impl HasHealth for Gd<MockEnemy> {
    fn get_health(&self) -> u32 {
        self.bind().health
    }

    fn set_health(&mut self, amount: u32) {
        self.bind_mut().health = amount;
    }

    fn on_death(&mut self) {
        self.queue_free();
    }
}

impl Damageable for Gd<MockEnemy> {
    fn handle_attack(&mut self, attack: AttackData) {
        self.take_damage(attack.damage.raw);
    }
}
