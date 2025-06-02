use godot::prelude::*;

use crate::{
    classes::characters::{character_stats::Stats, main_character::MainCharacter},
    components::managers::item::GameItem,
};

use super::{
    item::{Item, ItemKind, ModifierKind, StatModifier},
    item_component::ItemComponent,
};

#[derive(GodotClass)]
#[class(init, base=Node)]
struct LevelManager {
    #[init(val = 0)]
    kill_count: u8,
    #[init(val = OnReady::from_loaded("res://main_character.tscn"))]
    enemy_scene: OnReady<Gd<PackedScene>>,
    #[init(val = OnReady::from_loaded("res://wave_2.tscn"))]
    wave_2: OnReady<Gd<PackedScene>>,
    #[init(node = "MainCharacter")]
    player: OnReady<Gd<MainCharacter>>,
    base: Base<Node>,
}

#[godot_api]
impl INode for LevelManager {
    fn ready(&mut self) {
        let item = GameItem::new_from_fn(
            Item::new(
                ItemKind::RosaryBead {
                    effect: StatModifier::new(Stats::AttackDamage, ModifierKind::Flat(2.0)),
                    equipped: false,
                },
                "inc_damage".to_string(),
                None,
                "res://assets/icon.svg".to_string(),
            ),
            Vector2i::new(-400, 250),
        );

        self.base_mut().add_child(&item);
        self.connect_to_signals();
    }
}

impl LevelManager {
    fn connect_to_signals(&mut self) {
        let enemy = self
            .base()
            .get_node_as::<crate::classes::enemies::test_enemy::TestEnemy>("TestEnemy");
        let mut this = self.to_gd();
        enemy
            .signals()
            .test_enemy_died()
            .connect(move || this.bind_mut().on_enemy_died());

        let items = self.base().get_tree().unwrap().get_nodes_in_group("items");
        for item in items.iter_shared() {
            let item = item.cast::<GameItem>();
            item.signals().player_entered_item_area().connect_other(
                &*self.player.bind().item_comp,
                ItemComponent::set_in_item_area,
            );

            item.signals().player_exited_item_area().connect_other(
                &*self.player.bind().item_comp,
                ItemComponent::set_exited_item_area,
            );
        }
    }

    fn increment_kills(&mut self) {
        self.kill_count += 1;

        if self.kill_count < 3 {
            let wave_2 = self
                .wave_2
                .instantiate_as::<crate::classes::scenes::wave_2::Wave2>();
            self.base_mut().add_child(&wave_2);
        }
    }

    fn on_enemy_died(&mut self) {
        self.increment_kills();

        if self.kill_count < 4 {
            let mut enemy = self
                .enemy_scene
                .instantiate_as::<crate::classes::enemies::test_enemy::TestEnemy>();
            let varg = enemy.to_variant();
            self.base_mut().call_deferred("add_child", &[varg]);

            let spawn_position = Vector2::new(400.0, 230.0);
            enemy.set_position(spawn_position);

            let mut this = self.to_gd();
            enemy
                .signals()
                .test_enemy_died()
                .connect(move || this.bind_mut().on_enemy_died());
        }
    }
}
