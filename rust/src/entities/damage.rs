use godot::{
    classes::{AnimationPlayer, Area2D, KinematicCollision2D, Node2D, PhysicsBody2D},
    obj::{DynGd, Gd, WithBaseField},
    prelude::*,
};

use crate::entities::{
    entity_hitbox::{EntityHitbox, Hurtbox},
    entity_stats::ModifierKind,
};

pub trait HasHealth {
    fn get_health(&self) -> u32;
    fn set_health(&mut self, amount: u32);
    fn on_death(&mut self);
}

    fn destroy(&mut self);
#[derive(Clone, Copy)]
pub struct Damage {
    pub raw: u32,
    pub d_type: DamageType,
}

/// Implement this trait on anything that is capable of dealing damage
pub trait Damaging {
    // fn damage_amount(&self) -> u32;
    // fn do_damage(&self, mut target: DynGd<Node2D, dyn Damageable>) {
    //     let amount = self.damage_amount();
    //     let mut dyn_target = target.dyn_bind_mut();
    //     dyn_target.take_damage(amount);
    // }
    fn get_hurtbox(&self) -> &Gd<Hurtbox>;

    fn connect_hurtbox_sig(&mut self)
    where
        Self: WithBaseField,
    {
        self.get_hurtbox()
            .signals()
            .area_entered()
            .connect_other(&self.to_gd(), Self::on_hurtbox_entered_hitbox);
    }
#[derive(Clone, Copy)]
pub enum DamageType {
    Elemental(ElementType),
    Physical,
}

    fn on_hurtbox_entered_hitbox(&mut self, area: Gd<Area2D>)
    where
        Self: Sized,
    {
        if let Ok(mut hitbox) = area.try_cast::<EntityHitbox>() {
            println!("Hurtbox hit hitbox");
            let h = &mut hitbox.bind_mut().parent;
            let data = AttackData::new(
                Damage {
                    raw: 10,
                    d_type: DamageType::Physical,
                },
                &mut **h.as_mut().unwrap(),
                self,
            );
            CombatSystem::resolve(data);
            println!("Resolved combat");
        }
    }
#[derive(Clone, Copy)]
pub enum ElementType {
    Magic,
    Poison,
    Lightning,
    Fire,
}

struct Health {
    pub value: u32,
#[derive(Clone, Copy)]
pub enum Resistance {
    Physical(ModifierKind),
    Elemental(ElementType, ModifierKind),
}

pub trait HasHealth {
    fn get_health(&self) -> u32;
    fn set_health(&mut self, amount: u32);
#[derive(GodotClass, Clone)]
#[class(no_init)]
pub struct AttackData {
    pub hurtbox: Gd<Hurtbox>,
    pub parryable: bool,
    pub damage: Damage,
}

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
    #[init(val = Health { value: 10})]
    health: Health,
    #[init(node = "AnimationPlayer")]
    anim_player: OnReady<Gd<AnimationPlayer>>,

    #[init(node = "EntityHitbox")]
    hitbox: OnReady<Gd<EntityHitbox>>,

    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for MockEnemy {
    fn ready(&mut self) {
        self.hitbox.bind_mut().parent = Some(Box::new(self.to_gd()));
    }
}

impl HasHealth for Gd<MockEnemy> {
    fn get_health(&self) -> u32 {
        self.bind().health.value
    }

    fn set_health(&mut self, amount: u32) {
        self.bind_mut().health.value = amount;
    }

    fn on_death(&mut self) {
        self.queue_free();
    }
}

    }
}
