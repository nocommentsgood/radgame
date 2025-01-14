use super::character_resources::CharacterResources;

pub trait Damageable: CharacterResources {
    fn take_damage(&mut self, amount: i32) {
        let mut current_health = self.get_health();
        current_health = current_health.saturating_sub(amount);
        self.set_health(current_health);
    }
}
