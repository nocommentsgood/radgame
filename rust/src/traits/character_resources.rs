pub trait CharacterResources {
    fn get_health(&self) -> i32;

    fn set_health(&mut self, amount: i32);

    fn get_energy(&self) -> i32;

    fn set_energy(&mut self, amount: i32);

    fn get_mana(&self);

    fn set_mana(&mut self, amount: i32);
}
