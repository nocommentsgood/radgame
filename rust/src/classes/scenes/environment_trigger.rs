use godot::{
    classes::{Area2D, IArea2D},
    prelude::*,
};

use crate::{
    classes::characters::entity_hitbox::EntityHitbox, utils::collision_layers::CollisionLayers,
};

#[derive(GodotClass)]
#[class(init, base = Area2D)]
pub struct EnvironmentTrigger {
    base: Base<Area2D>,
}

#[godot_api]
impl IArea2D for EnvironmentTrigger {
    fn ready(&mut self) {
        self.base_mut()
            .set_collision_layer_value(CollisionLayers::WorldEffects as i32, true);
        self.base_mut()
            .set_collision_mask_value(CollisionLayers::PlayerHitbox as i32, true);
        self.signals()
            .area_entered()
            .connect_self(Self::on_player_enters_area);
    }
}

#[godot_api]
impl EnvironmentTrigger {
    #[signal]
    pub fn prepare_arena();

    fn on_player_enters_area(&mut self, area: Gd<Area2D>) {
        if let Ok(h_box) = area.try_cast::<EntityHitbox>() {
            if let Some(_player) = h_box.get_owner() {
                println!("Emitting area signal");
                self.signals().prepare_arena().emit();
                self.base_mut().queue_free();
            }
        } else {
            println!("Not a player");
        }
    }
}
