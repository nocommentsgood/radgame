use super::item::GameItem;
use godot::{classes::InputEvent, prelude::*};

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct ItemComponent {
    items: Vec<Gd<GameItem>>,
    in_item_area: bool,
    item: Option<Gd<GameItem>>,
    base: Base<Node>,
}

#[godot_api]
impl INode for ItemComponent {
    fn unhandled_input(&mut self, input: Gd<InputEvent>) {
        if input.is_action_pressed("interact") {
            self.pickup_item();
        }
    }
}

#[godot_api]
impl ItemComponent {
    #[signal]
    fn new_item_added(item: Gd<GameItem>);

    #[signal]
    fn picked_up_item(item: Gd<GameItem>);

    pub fn set_in_item_area(&mut self, item: Gd<GameItem>) {
        self.in_item_area = true;
        self.item = Some(item);
    }

    pub fn set_exited_item_area(&mut self) {
        self.in_item_area = false;
        self.item = None;
    }

    pub fn pickup_item(&mut self) {
        if self.in_item_area {
            let item = self.item.clone();
            if let Some(mut item) = item {
                self.items.push(item.clone());
                self.signals().picked_up_item().emit(item.clone());
                item.bind_mut().picked_up();
                self.in_item_area = false;
            }
        }
    }
}
