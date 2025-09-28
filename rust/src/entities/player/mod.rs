pub mod abilities;
pub mod character_state_machine;
pub mod item_component;
pub mod main_character;
mod physics;
pub mod shaky_player_camera;

// TODO: Actually write tests.
#[cfg(test)]
mod tests {
    use crate::{
        entities::{
            entity_stats::{Stat, StatModifier},
            player::item_component::ItemComponent,
        },
        world::item::{Item, ItemKind},
    };

    #[test]
    fn test_inventory_vec_capacity() {
        let full_inv = ItemComponent::default();
        let bead_1 = Item::new(
            ItemKind::RosaryBead {
                effect: StatModifier::new(
                    Stat::Health,
                    crate::entities::entity_stats::ModifierKind::Flat(10),
                ),
            },
            "TestBead1".to_string(),
            Some("Test description 1".to_string()),
            "test".to_string(),
        );
    }
}
