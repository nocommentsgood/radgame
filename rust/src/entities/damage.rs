use super::entity_stats::EntityResources;
use godot::{
    classes::{AnimationPlayer, Area2D, Node2D, Timer},
    obj::{DynGd, Gd, NewAlloc},
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

struct Damage;

struct AttackData<'a> {
    attack_name: String,
    damage: u32,
    defending_entity: DynGd<Node2D, dyn Damageable>,
    attack_ent: &'a dyn Damaging,
    animation: String,
    anim_player: &'a mut Gd<AnimationPlayer>,
    timer: &'a mut Gd<Timer>,
}

struct MockEnemy {
    anim_player: Gd<AnimationPlayer>,
    timer: Gd<Timer>,
}
impl Damaging for MockEnemy {
    fn damage_amount(&self) -> u32 {
        todo!()
    }
}

impl MockEnemy {
    fn on_hit(&self, def_enemy: Gd<Area2D>) {
        let mut player = self.anim_player.clone();
        let mut timer = self.timer.clone();
        Self::attack(AttackData {
            attack_name: "Some attaack".to_string(),
            damage: 16,
            defending_entity: def_enemy,
            attack_ent: self,
            timer: &mut timer,
            animation: "Some attack anim".to_string(),
            anim_player: &mut player,
        })
    }

    fn attack(data: AttackData) {
        data.anim_player.play_ex().name(&data.animation).done();
        data.timer.start();
        data.defending_entity.take_damage(data.damage);
    }
}
