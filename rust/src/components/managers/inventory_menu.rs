use godot::{
    classes::{Control, IControl, InputEvent, ItemList, Label, TabContainer},
    prelude::*,
};

use super::item_component::ItemComponent;

#[derive(GodotClass)]
#[class(base=Control, init)]
struct InventoryMenu {
    #[init(node = "PanelContainer/MarginContainer/TabContainer")]
    tab_container: OnReady<Gd<TabContainer>>,

    #[init(
        node = "PanelContainer/MarginContainer/TabContainer/MarginContainer/VBoxContainer/ItemList"
    )]
    bead_item_list: OnReady<Gd<ItemList>>,

    #[init(
        node = "PanelContainer/MarginContainer/TabContainer/MarginContainer/VBoxContainer/ItemDescriptionLabel"
    )]
    item_desc: OnReady<Gd<Label>>,

    #[init(node = "../../TileMapLayer/LevelManager/MainCharacter/ItemComponent")]
    item_comp: OnReady<Gd<ItemComponent>>,
    base: Base<Control>,
}

#[godot_api]
impl IControl for InventoryMenu {
    fn ready(&mut self) {
        self.base_mut().set_visible(false);
        let this = self.to_gd();
        self.bead_item_list
            .signals()
            .item_selected()
            .connect_other(&this, Self::on_bead_selected);

        self.tab_container.set_tab_title(0, "RosaryBeads");
    }

    fn unhandled_input(&mut self, event: Gd<InputEvent>) {
        if event.is_action_pressed("inventory") && !self.base().is_visible() {
            self.base_mut().set_visible(true);
            self.base_mut().grab_focus();
            self.populate_unlocked_beads();
        } else if event.is_action_pressed("inventory") && self.base().is_visible() {
            self.base_mut().set_visible(false);
        }
    }
}

#[godot_api]
impl InventoryMenu {
    fn populate_unlocked_beads(&mut self) {
        for (idx, item) in self.item_comp.bind().unlocked_beads.iter().enumerate() {
            if let Some(item) = item {
                let icon = load::<godot::classes::Texture2D>(&item.icon_path.clone());
                self.bead_item_list.set_item_icon(idx as i32, &icon);
                if let Some(desc) = &item.desc {
                    self.item_desc.set_text(desc);
                } else {
                    self.item_desc.set_text("");
                }
            } else {
                self.item_desc.set_text("");
            }
        }
    }

    fn on_bead_selected(&mut self, idx: i64) {
        if let Err(e) = self.item_comp.bind_mut().try_equip_bead(idx as usize) {
            dbg!(&e);
        }
    }
}
