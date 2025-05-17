use godot::{classes::Node2D, obj::DynGd};

use super::damageable::Damageable;

// TODO: This trait could perhaps be replaced with a wrapper Area2D type. Currently, it is used on
// in-game objects that deal damage. Though, all of these objects have an Area2D. Instead of
// implementing Damaging for all Hurtboxes, we could just have a wrapper Hurtbox and rely on
// physics layers.

/// Implement this trait on anything that is capable of dealing damage
pub trait Damaging {
    fn damage_amount(&self) -> u32;

    fn do_damage(&self, mut target: DynGd<Node2D, dyn Damageable>) {
        let amount = self.damage_amount();
        let mut dyn_target = target.dyn_bind_mut();
        dyn_target.take_damage(amount);
    }
}
