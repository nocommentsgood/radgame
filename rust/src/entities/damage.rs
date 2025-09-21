use godot::{
    classes::{AnimationPlayer, Area2D, KinematicCollision2D, Node2D, PhysicsBody2D},
    obj::{DynGd, Gd, WithBaseField},
    prelude::*,
};

use crate::entities::{
    entity_hitbox::{EntityHitbox, Hurtbox},
    entity_stats::ModifierKind,
};

/// Implement on entities that are capable of being damaged. See also: trait Damaging.
pub trait Damageable {
    fn take_damage(&mut self, amount: u32) {
        //     let mut current_health = self.get_health();
        //     current_health = current_health.saturating_sub(amount);
        //     self.set_health(current_health);
        //
        //     if self.is_dead() {
        //         self.destroy();
        //     }
    }
    //
    fn is_dead(&self) -> bool {
        //     self.get_health() == 0
        false
    }

    fn destroy(&mut self);
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
}

struct Health {
    pub value: u32,
}

pub trait HasHealth {
    fn get_health(&self) -> u32;
    fn set_health(&mut self, amount: u32);

    fn take_damage(&mut self, amount: u32) {
        self.set_health(self.get_health().saturating_sub(amount));

        if self.get_health() == 0 {
            self.on_death();
        }
    }

    fn on_death(&mut self);
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

impl HasHealth for MockEnemy {
    fn get_health(&self) -> u32 {
        self.health.value
    }

    fn set_health(&mut self, amount: u32) {
        self.health.value = amount;
    }

    fn on_death(&mut self) {
        self.base_mut().queue_free();
    }
}

pub struct AttackData<'a> {
    damage: Damage,
    defending_unit: &'a mut dyn HasHealth,
    attacking_unit: &'a dyn Damaging,
}

impl<'a> AttackData<'a> {
    pub fn new(
        damage: Damage,
        defending_unit: &'a mut dyn HasHealth,
        attacking_unit: &'a mut dyn Damaging,
    ) -> Self {
        Self {
            damage,
            defending_unit,
            attacking_unit,
        }
    }
}

pub struct Damage {
    pub raw: u32,
    pub d_type: DamageType,
}

pub enum DamageType {
    Elemental(ElementType),
    Physical,
}

pub enum ElementType {
    Magic,
    Poison,
    Lightning,
    Fire,
}

pub enum Resistance {
    Physical(ModifierKind),
    Elemental(ElementType, ModifierKind),
}

pub struct CombatSystem;

impl CombatSystem {
    pub fn resolve(data: AttackData) {
        let health = data.defending_unit.get_health();
        data.defending_unit.set_health(health - data.damage.raw);
    }
}
