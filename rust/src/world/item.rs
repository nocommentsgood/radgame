use godot::{
    classes::{Area2D, CollisionShape2D, RectangleShape2D, Sprite2D, Texture2D},
    prelude::*,
};

use crate::{
    entities::{entity_hitbox::EntityHitbox, entity_stats::StatModifier},
    utils::collision_layers::CollisionLayers,
};

#[derive(Default, Clone, Debug, PartialEq)]
pub enum ItemKind {
    #[default]
    Collectable,
    RosaryKnot,
    Quest,
    Relic {
        effect: StatModifier,
    },
    RosaryBead {
        effect: StatModifier,
    },
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct Item {
    pub kind: ItemKind,
    pub name: String,
    pub desc: Option<String>,
    pub icon_path: String,
}

impl Item {
    pub fn new(kind: ItemKind, name: String, desc: Option<String>, icon_path: String) -> Self {
        Self {
            kind,
            name,
            desc,
            icon_path,
        }
    }
}

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct GameItem {
    pub item: Item,
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for GameItem {
    fn enter_tree(&mut self) {
        self.base_mut().add_to_group("items");
    }

    fn ready(&mut self) {
        let mut area = self.base().get_node_as::<Area2D>("Area2D");
        self.base_mut().set_as_top_level(true);

        let mut this = self.to_gd();
        area.signals()
            .area_entered()
            .connect(move |area| this.bind_mut().on_area_entered(area));

        let mut this = self.to_gd();
        area.signals()
            .area_exited()
            .connect(move |area| this.bind_mut().on_area_exited(area));
    }
}

#[godot_api]
impl GameItem {
    #[signal]
    pub fn player_entered_item_area(item: Gd<GameItem>);

    #[signal]
    pub fn player_exited_item_area();

    pub fn picked_up(&mut self) {
        self.base_mut().queue_free();
    }

    // Should this be entity hitbox?
    fn on_area_entered(&mut self, area: Gd<Area2D>) {
        if let Ok(_area) = area.try_cast::<EntityHitbox>() {
            println!("Player entered item");
            let this = self.to_gd();
            self.signals().player_entered_item_area().emit(&this);
        }
    }

    fn on_area_exited(&mut self, area: Gd<Area2D>) {
        if let Ok(_area) = area.try_cast::<EntityHitbox>() {
            println!("Player exited item");
            self.signals().player_exited_item_area().emit();
        }
    }
}
