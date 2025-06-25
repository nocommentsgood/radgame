use godot::{
    classes::{Area2D, CollisionShape2D, RectangleShape2D, Sprite2D, Texture2D},
    prelude::*,
};

use crate::{
    classes::characters::{character_hitbox::CharacterHitbox, character_stats::Stats},
    utils::collision_layers::CollisionLayers,
};

#[derive(GodotClass, Clone, Debug, PartialEq)]
#[class(no_init)]
pub struct StatModifier {
    pub stat: Stats,
    pub modifier: ModifierKind,
}

impl StatModifier {
    pub fn new(stat: Stats, modifier: ModifierKind) -> Self {
        Self { stat, modifier }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ModifierKind {
    Flat(u32),
    Percent(f32),
}

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
    location: Vector2i,
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for GameItem {
    fn enter_tree(&mut self) {
        self.base_mut().add_to_group("items");
    }

    fn ready(&mut self) {
        let pos = self.location.cast_float();
        let texture = godot::prelude::load::<Texture2D>("res://assets/bullet.webp");
        let mut sprite = Sprite2D::new_alloc();
        sprite.set_texture(&texture);

        let mut area = Area2D::new_alloc();
        let mut shape = CollisionShape2D::new_alloc();
        let mut rect = RectangleShape2D::new_gd();
        rect.set_size(Vector2::new(30.0, 30.0));
        shape.set_shape(&rect);
        area.add_child(&shape);

        area.set_collision_layer_value(CollisionLayers::Items as i32, true);
        area.set_collision_mask_value(CollisionLayers::PlayerHitbox as i32, true);

        let mut base = self.base_mut();
        base.set_as_top_level(true);
        base.set_global_position(pos);
        base.add_child(&sprite);
        base.add_child(&area);
        drop(base);

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

    pub fn new_from_fn(item: Item, location: Vector2i) -> Gd<Self> {
        Gd::from_init_fn(|base| Self {
            item,
            location,
            base,
        })
    }

    fn on_area_entered(&mut self, area: Gd<Area2D>) {
        if let Ok(_area) = area.try_cast::<CharacterHitbox>() {
            let this = self.to_gd();
            self.signals().player_entered_item_area().emit(&this);
        }
    }

    fn on_area_exited(&mut self, area: Gd<Area2D>) {
        if let Ok(_area) = area.try_cast::<CharacterHitbox>() {
            self.signals().player_exited_item_area().emit();
        }
    }
}
