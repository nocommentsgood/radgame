use godot::{obj::Gd, prelude::GodotClass};

use crate::entities::damage::HasHealth;

#[derive(GodotClass)]
#[class(init, base = Area2D)]
pub struct EntityHitbox {
    #[init(val = 20)]
    health: u32,
}

// impl super::damage::Damageable for EntityHitbox {
//     fn destroy(&mut self) {
//         todo!()
//     }
// }

impl super::damage::Damageable for Gd<EntityHitbox> {
    fn destroy(&mut self) {
        todo!()
    }
}
impl super::entity_stats::EntityResources for Gd<EntityHitbox> {
    fn get_health(&self) -> u32 {
        self.bind().health
    }

    fn set_health(&mut self, amount: u32) {
        self.bind_mut().health = amount;
    }

    fn get_energy(&self) -> u32 {
        todo!()
    }

    fn set_energy(&mut self, amount: u32) {
        todo!()
    }

    fn get_mana(&self) -> u32 {
        todo!()
    }

    fn set_mana(&mut self, amount: u32) {
        todo!()
    }
}

impl HasHealth for Gd<EntityHitbox> {
    fn get_health(&self) -> u32 {
        self.bind().health
    }

    fn set_health(&mut self, amount: u32) {
        self.bind_mut().health = amount;
    }
}
