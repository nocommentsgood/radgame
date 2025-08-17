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

pub trait HasHealth {
    fn get_health(&self) -> u32;
    fn set_health(&mut self, amount: u32);
}

pub trait TestDamaging {
    fn deal_damage(&self, amount: u32, target: &mut dyn HasHealth) {
        let cur = target.get_health();
        println!("Previous health: {cur}");
        target.set_health(cur - amount);
        println!("new health: {}", target.get_health());
    }
}

pub trait TestDamageSystem {
    fn test_deal_damage();

    fn test_damage(data: &mut AttackData) {
        data.anim_player.play_ex().name(data.animation).done();
        data.timer.start();
        data.attacking_unit
            .deal_damage(data.damage, data.defending_unit);
    }
}

#[derive(GodotClass)]
#[class(base = Node2D, init)]
struct MockEnemy {
    #[init(node = "AnimationPlayer")]
    anim_player: OnReady<Gd<AnimationPlayer>>,
    #[init(node = "Timer")]
    timer: OnReady<Gd<Timer>>,
    #[init(val = 20)]
    health: u32,
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

// fn test_deal_damage();

pub fn test_damage(data: &mut AttackData) {
    data.anim_player.play_ex().name(data.animation).done();
    data.timer.start();
    data.attacking_unit
        .deal_damage(data.damage, data.defending_unit);
}

pub struct AttackData<'a> {
    attack_name: &'a str,
    damage: u32,
    defending_unit: &'a mut dyn HasHealth,
    attacking_unit: &'a dyn TestDamaging,
    animation: &'a str,
    anim_player: &'a mut Gd<AnimationPlayer>,
    timer: &'a mut Gd<Timer>,
}

impl<'a> AttackData<'a> {
    pub fn new(
        attack_name: &'a str,
        damage: u32,
        defending_unit: &'a mut dyn HasHealth,
        attacking_unit: &'a mut dyn TestDamaging,
        animation: &'a str,
        anim_player: &'a mut Gd<AnimationPlayer>,
        timer: &'a mut Gd<Timer>,
    ) -> Self {
        Self {
            attack_name,
            damage,
            defending_unit,
            attacking_unit,
            animation,
            anim_player,
            timer,
        }
    }
}

pub struct AttackSequence<'a> {
    count: usize,
    data: &'a mut [&'a mut AttackData<'a>],
}

impl<'a> AttackSequence<'a> {
    pub fn new(sequence: &'a mut [&'a mut AttackData<'a>]) -> Self {
        if sequence.is_empty() {
            panic!("Data sequence is empty.")
        } else {
            Self {
                count: 0,
                data: sequence,
            }
        }
    }

    pub fn execute(&mut self) {
        let count = self.count;
        self.data[count].timer.start();
        test_damage(self.data[count]);
        if self.data[count].timer.get_time_left() == 0.0 {
            println!("Timer timeout");
            self.on_timer_timeout();
        }
    }

    fn on_timer_timeout(&mut self) {
        println!("On timer timeout");
        self.count += 1;
        if self.count <= self.data.len() {
            self.execute();
        }
    }
}
