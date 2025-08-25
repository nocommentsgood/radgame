use crate::entities::movements::{AlternatingMovement, MoveRight, MovementBehavior};

use super::entity_stats::EntityResources;
use godot::{
    classes::{AnimationPlayer, Node2D, Timer},
    obj::{DynGd, Gd},
    prelude::*,
};

/// Implement on entities that are capable of being damaged. See also: trait Damaging.
/// Implementor is responsible for providing their own 'destroy' function.
/// This trait is 'dyn compatible' and can be used with godot_dyn macro.
pub trait Damageable: EntityResources {
    fn take_damage(&mut self, amount: u32) {
        let mut current_health = self.get_health();
        current_health = current_health.saturating_sub(amount);
        self.set_health(current_health);

        if self.is_dead() {
            self.destroy();
        }
    }

    fn is_dead(&self) -> bool {
        self.get_health() == 0
    }

    fn destroy(&mut self);
}

/// Implement this trait on anything that is capable of dealing damage
pub trait Damaging {
    fn damage_amount(&self) -> u32;
    fn do_damage(&self, mut target: DynGd<Node2D, dyn Damageable>) {
        let amount = self.damage_amount();
        let mut dyn_target = target.dyn_bind_mut();
        dyn_target.take_damage(amount);
    }
}

pub trait HasHealth: std::fmt::Debug {
    fn get_health(&self) -> u32;
    fn set_health(&mut self, amount: u32);
}

pub trait TestDamaging: std::fmt::Debug {
    fn deal_damage(&self, amount: u32, target: &mut dyn HasHealth) {
        let cur = target.get_health();
        println!("Previous health: {cur}");
        println!("Damage amount: {amount}");
        target.set_health(cur.saturating_sub(amount));
        println!("new health: {}", target.get_health());
    }
}

#[derive(GodotClass)]
#[class(base = Node2D, init)]
struct MockEnemy {
    #[init(val = 200)]
    health: u32,
    #[init(node = "AnimationPlayer")]
    anim_player: OnReady<Gd<AnimationPlayer>>,
    base: Base<Node2D>,
}

impl HasHealth for Gd<MockEnemy> {
    fn get_health(&self) -> u32 {
        self.bind().health
    }

    fn set_health(&mut self, amount: u32) {
        self.bind_mut().health = amount;
    }
}

pub fn test_damage(data: &mut AttackData) {
    data.attacking_unit
        .deal_damage(data.damage, data.defending_unit);
}

#[derive(Debug)]
pub struct AttackData<'a> {
    damage: u32,
    defending_unit: &'a mut dyn HasHealth,
    attacking_unit: &'a dyn TestDamaging,
}

impl<'a> AttackData<'a> {
    pub fn new(
        damage: u32,
        defending_unit: &'a mut dyn HasHealth,
        attacking_unit: &'a mut dyn TestDamaging,
    ) -> Self {
        Self {
            damage,
            defending_unit,
            attacking_unit,
        }
    }
}

// pub struct AttackSequence<'a> {
//     count: usize,
//     data: &'a mut [&'a mut AttackData<'a>],
// }
//
// impl<'a> AttackSequence<'a> {
//     pub fn new(sequence: &'a mut [&'a mut AttackData<'a>]) -> Self {
//         if sequence.is_empty() {
//             panic!("Data sequence is empty.")
//         } else {
//             Self {
//                 count: 0,
//                 data: sequence,
//             }
//         }
//     }
//
//     pub fn execute(&mut self) {
//         let count = self.count;
//         self.data[count].timer.start();
//         test_damage(self.data[count]);
//         if self.data[count].timer.get_time_left() == 0.0 {
//             println!("Timer timeout");
//             self.on_timer_timeout();
//         }
//     }
//
//     fn on_timer_timeout(&mut self) {
//         println!("On timer timeout");
//         self.count += 1;
//         if self.count <= self.data.len() {
//             self.execute();
//         }
//     }
// }
