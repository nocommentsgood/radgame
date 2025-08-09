use super::entity_stats::EntityResources;
use godot::{classes::Node2D, obj::DynGd};

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
