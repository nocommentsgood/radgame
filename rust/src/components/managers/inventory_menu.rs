use godot::{
    classes::{Control, IControl, InputEvent, ItemList, Label, TabContainer},
    prelude::*,
};

use super::{item::Item, item_component::ItemComponent};

#[derive(GodotClass)]
#[class(base=Control, init)]
struct InventoryMenu {
    #[init(node = "PanelContainer/MarginContainer/TabContainer")]
    tab_container: OnReady<Gd<TabContainer>>,

    #[init(
        node = "PanelContainer/MarginContainer/TabContainer/MarginContainer/VBoxContainer/ItemList"
    )]
    item_list: OnReady<Gd<ItemList>>,

    #[init(
        node = "PanelContainer/MarginContainer/TabContainer/MarginContainer/VBoxContainer/ItemDescriptionLabel"
    )]
    item_desc: OnReady<Gd<Label>>,

    #[init(node = "../../LevelManager/MainCharacter/ItemComponent")]
    item_comp: OnReady<Gd<ItemComponent>>,
    base: Base<Control>,
}

#[godot_api]
impl IControl for InventoryMenu {
    fn ready(&mut self) {
        self.base_mut().set_visible(false);
        let this = self.to_gd();
        self.item_list
            .signals()
            .item_selected()
            .connect_other(&this, Self::on_item_selected);
    }

    fn unhandled_input(&mut self, event: Gd<InputEvent>) {
        if event.is_action_pressed("inventory") && !self.base().is_visible() {
            println!("setting vis true");
            self.base_mut().set_visible(true);
            self.base_mut().grab_focus();
        } else if event.is_action_pressed("inventory") && self.base().is_visible() {
            println!("setting vis false");
            self.base_mut().set_visible(false);
        }
    }

    // fn gui_input(&mut self, event: Gd<InputEvent>) {
    //     if event.is_action_pressed("inventory") {
    //         println!("got inv input from gui");
    //         self.base_mut().grab_focus();
    //     }
    //     if event.is_action_pressed("inventory") && !self.base().is_visible() {
    //         self.base_mut().set_visible(true);
    //         self.base_mut().grab_focus();
    //     }
    //     if event.is_action_pressed("inventory") && self.base().is_visible() {
    //         self.base_mut().set_visible(false);
    //     }
    // }
}

#[godot_api]
impl InventoryMenu {
    fn get_unlocked_beads(&self) -> Vec<Item> {
        self.item_comp.bind().unlocked_beads.clone()
    }

    fn on_item_selected(&mut self, idx: i64) {
        if let Some(item) = self.item_comp.bind().unlocked_beads.get(idx as usize) {
            if let Some(desc) = &item.desc {
                self.item_desc.set_text(desc);
            }
        }
    }
}
