use std::collections::HashMap;

use super::item::{GameItem, Item};
use godot::{classes::InputEvent, prelude::*};

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct ItemComponent {
    unlocked_items: Vec<Item>,
    item_map: HashMap<i32, Item>,
    unlocked_beads: Vec<Item>,
    equipped_beads: Vec<Item>,
    unlocked_misc: Vec<Item>,
    equipped_misc: Vec<Item>,
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
        if input.is_action_pressed("equip") {
            // TODO: This is just for testing
            let bead = self.unlocked_items.first().unwrap().clone();
            println!("trying to equip bead: {:?}", bead);
            self.equip_bead(&bead);
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
                let bind = item.bind().item.clone();
                match bind.ty {
                    super::item::ItemType::Misc => self.unlocked_misc.push(bind),
                    super::item::ItemType::RosaryBead {
                        effect: _,
                        equipped: false,
                    } => self.unlocked_beads.push(bind),
                    _ => (),
                }
                self.signals().picked_up_item().emit(item.clone());
                item.bind_mut().picked_up();
            }
        }
        self.in_item_area = false;
    }

    fn equip_bead(&mut self, bead: &super::item::Item) {
        if self.unlocked_items.len() < self.unlocked_items.capacity() {
            for item in &self.unlocked_items {
                match &item.ty {
                    super::item::ItemType::RosaryBead {
                        effect: _,
                        equipped: false,
                    } => self.equipped_beads.push(item.clone()),
                    _ => panic!("Bead of type: {bead:?} is not unlocked, or item of type RosaryBead not received."),
                }
            }
        } else {
            panic!("Number of equipped beads is already at capactiy");
        }
        println!("equipped beads: {:?}", self.equipped_beads);
    }

    fn unequip_bead(&mut self, bead: &super::item::Item) {
        if let Some(pos) = self.equipped_beads.iter().position(|i| i == bead) {
            self.equipped_beads.remove(pos);
        }
    }
}
