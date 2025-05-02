pub trait CharacterResources {
    fn get_health(&self) -> u32;

    fn set_health(&mut self, amount: u32);

    fn get_energy(&self) -> u32;

    fn set_energy(&mut self, amount: u32);

    fn get_mana(&self) -> u32;

    fn set_mana(&mut self, amount: u32);
}
