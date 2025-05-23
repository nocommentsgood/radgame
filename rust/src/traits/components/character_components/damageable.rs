use super::character_resources::CharacterResources;

/// Implement on entities that are capable of being damaged. See also: trait Damaging.
/// Implementor is responsible for providing their own 'destroy' function.
/// This trait is 'dyn compatible' and can be used with godot_dyn macro.
pub trait Damageable: CharacterResources {
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
