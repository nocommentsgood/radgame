use godot::{
    classes::{Area2D, CollisionShape2D, RectangleShape2D, Sprite2D, Texture2D},
    prelude::*,
};

use crate::{
    classes::characters::character_hitbox::CharacterHitbox,
    utils::collision_layers::CollisionLayers,
};

pub enum Tier {
    One,
    Two,
    Three,
}

pub enum SpEffect {
    IncreaseDashInvul(Tier),
    DecreaseAttackCooldown(Tier),
}

#[derive(Default)]
pub enum ItemType {
    #[default]
    Misc,
    RosaryKnot,
    Quest,
    Relic {
        effect: SpEffect,
        equipped: bool,
    },
    RosaryBead {
        effect: SpEffect,
        equipped: bool,
    },
}

#[derive(Default)]
pub struct Item {
    pub ty: ItemType,
    pub name: String,
    pub desc: Option<String>,
}

impl Item {
    pub fn new(ty: ItemType, name: String, desc: Option<String>) -> Self {
        Self { ty, name, desc }
    }
}

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct GameItem {
    item: Item,
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

        area.set_collision_layer_value(CollisionLayers::LayerOne.into(), false);
        area.set_collision_mask_value(CollisionLayers::LayerOne.into(), false);
        area.set_collision_layer_value(CollisionLayers::Items.into(), true);
        area.set_collision_mask_value(CollisionLayers::PlayerHitbox.into(), true);

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
            self.signals().player_entered_item_area().emit(this);
        }
    }

    fn on_area_exited(&mut self, area: Gd<Area2D>) {
        if let Ok(_area) = area.try_cast::<CharacterHitbox>() {
            self.signals().player_exited_item_area().emit();
        }
    }
}
