use godot::{
    classes::{Area2D, CollisionShape2D, RectangleShape2D, Sprite2D, Texture2D, Timer},
    obj::WithBaseField,
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

/// This struct is initialized by Godot in `Main::init_game_items`.
/// To set base node properties, do so in the aforementioned function.
#[derive(GodotClass, Clone)]
#[class(base=Node2D)]
pub struct GameItem {
    pub item: Item,
    pub sig_handler: Option<Gd<GameItemSignalHandler>>,
}

#[godot_api]
impl INode2D for GameItem {
    fn init(_base: Base<Node2D>) -> Self {
        Self {
            item: Item::default(),
            sig_handler: None,
        }
    }
}

#[godot_api]
impl GameItem {
    pub fn picked_up(&mut self) {
        println!("Picked up");
        if let Some(base) = &self.sig_handler {
            self.sig_handler.as_mut().unwrap().bind_mut().destroy();
        }
    }

    pub fn on_area_entered(&mut self, area: Gd<Area2D>) {
        if let Ok(_area) = area.try_cast::<EntityHitbox>() {
            println!("PLayer entered item area");
            let b = self.sig_handler.as_ref().unwrap();
            let this = Gd::from_object(self.clone());
            b.signals().player_entered_item_area().emit(&this);
        }
    }

    pub fn on_area_exited(&mut self, area: Gd<Area2D>) {
        if let Ok(_area) = area.try_cast::<EntityHitbox>() {
            println!("PLayer exited item area");
            let b = self.sig_handler.as_ref().unwrap();
            b.signals().player_exited_item_area().emit();
        }
    }
}

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct GameItemSignalHandler {
    base: Base<Node>,
}

#[godot_api]
impl GameItemSignalHandler {
    #[signal]
    pub fn player_entered_item_area(item: Gd<GameItem>);

    #[signal]
    pub fn player_exited_item_area();

    pub fn destroy(&mut self) {
        self.base().get_parent().unwrap().queue_free();
    }
}
