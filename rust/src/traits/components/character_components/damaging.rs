use godot::{classes::Node2D, obj::DynGd};

use super::damageable::Damageable;

/// Implement this trait on anything that is capable of dealing damage
pub trait Damaging {
    fn do_damage(&self, amount: i32, mut target: DynGd<Node2D, dyn Damageable>) {
        let mut dyn_target = target.dyn_bind_mut();
        dyn_target.take_damage(amount);
    }
}
