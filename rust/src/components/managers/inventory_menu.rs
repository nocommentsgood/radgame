use godot::{
    classes::{
        CanvasLayer, GridContainer, ICanvasLayer, InputEvent, ItemList, Label, TabContainer,
        Texture2D, TextureRect,
    },
    prelude::*,
};

use crate::classes::characters::main_character::MainCharacter;
use crate::utils::constants;

use super::item_component::ItemComponent;

#[derive(GodotClass)]
#[class(base=CanvasLayer, init)]
struct InventoryMenu {
    #[export]
    tab_container: OnEditor<Gd<TabContainer>>,
    #[export]
    bead_item_list: OnEditor<Gd<ItemList>>,
    #[export]
    relic_item_list: OnEditor<Gd<ItemList>>,
    #[export]
    item_desc: OnEditor<Gd<Label>>,
    #[export]
    equipped_item_grid: OnEditor<Gd<GridContainer>>,

    item_comp: Option<Gd<ItemComponent>>,
    base: Base<CanvasLayer>,
}

#[godot_api]
impl ICanvasLayer for InventoryMenu {
    fn ready(&mut self) {
        self.item_comp = self
            .base()
            .get_node_as::<MainCharacter>(
                constants::get_world_data()
                    .bind()
                    .paths
                    .player
                    .as_ref()
                    .expect("Expected player path from GlobalData"),
            )
            .try_get_node_as::<ItemComponent>("ItemComponent");

        self.item_desc.set_text("");
        self.base_mut().set_visible(false);
        let this = self.to_gd();
        self.bead_item_list
            .signals()
            .item_activated()
            .connect_other(&this, Self::on_bead_activated);

        self.bead_item_list
            .signals()
            .item_selected()
            .connect_other(&this, Self::on_bead_selected);

        self.relic_item_list
            .signals()
            .item_activated()
            .connect_other(&this, Self::on_relic_activated);

        self.relic_item_list
            .signals()
            .item_selected()
            .connect_other(&this, Self::on_relic_selected);
        self.tab_container.set_tab_title(0, "RosaryBeads");
        self.tab_container.set_tab_title(1, "Relics");
    }

    fn unhandled_input(&mut self, event: Gd<InputEvent>) {
        if event.is_action_pressed("inventory") && !self.base().is_visible() {
            self.base_mut().set_visible(true);
            self.set_bead_list_icons();
        } else if event.is_action_pressed("inventory") && self.base().is_visible() {
            self.base_mut().set_visible(false);
        } else if event.is_action_pressed("equip") && self.base().is_visible() {
            //             // TODO: Used for testing. Remove later.
            println!("Testing equipping relic. Remove me.");
            self.on_relic_activated(0);
        }
    }
}

// TODO: This will have to be extended further to support setting icons in the equipped items/item
// grid.
//
// TODO: The Godot scene tabs shouldn't need to have their own labels, equipped icon section, etc.
#[godot_api]
impl InventoryMenu {
    fn set_bead_list_icons(&mut self) {
        if let Some(item_comp) = &self.item_comp {
            for (idx, item) in item_comp.bind().unlocked_beads.iter().enumerate() {
                if let Some(item) = item {
                    let icon = load::<godot::classes::Texture2D>(&item.icon_path.clone());
                    self.bead_item_list.set_item_icon(idx as i32, &icon);
                }
            }
            for (idx, item) in item_comp.bind().unlocked_relics.iter().enumerate() {
                if let Some(item) = item {
                    let icon = load::<godot::classes::Texture2D>(&item.icon_path.clone());
                    self.relic_item_list.set_item_icon(idx as i32, &icon);
                }
            }
        }
    }

    fn on_bead_selected(&mut self, idx: i64) {
        let some = self.item_comp.as_ref().unwrap();
        let bind = some.bind();
        let item = bind.unlocked_beads.get(idx as usize);
        if let Some(Some(item)) = item {
            if let Some(text) = &item.desc {
                self.item_desc.set_text(text);
            } else {
                self.item_desc.set_text("");
            }
        } else {
            self.item_desc.set_text("");
        }
    }

    fn on_relic_selected(&mut self, idx: i64) {
        let some = self.item_comp.as_ref().unwrap();
        let bind = some.bind();
        let relic = bind.unlocked_relics.get(idx as usize);
        if let Some(Some(relic)) = relic {
            if let Some(text) = &relic.desc {
                dbg!();
                self.item_desc.set_text(text);
            } else {
                dbg!();
                self.item_desc.set_text("");
            }
        } else {
            dbg!();
            self.item_desc.set_text("");
        }
    }

    fn on_bead_activated(&mut self, idx: i64) {
        let item_c = self.item_comp.as_mut().unwrap();
        let unlocked = item_c.bind().unlocked_beads.clone();
        let mut equipped = item_c.bind().equipped_beads.clone();
        let res = item_c
            .bind_mut()
            .try_equip_item(&unlocked, &mut equipped, idx as usize);

        match res {
            Ok(item) => {
                item_c.bind_mut().unlocked_beads = unlocked;
                item_c.bind_mut().equipped_beads = equipped;
                let icon = load::<Texture2D>(&item.icon_path);
                self.equipped_item_grid
                    .get_node_as::<TextureRect>("TextureRect")
                    .set_texture(&icon);
            }
            Err(e) => {
                dbg!(e);
            }
        }
    }

    fn on_relic_activated(&mut self, idx: i64) {
        let some = self.item_comp.as_mut().unwrap();
        let unlocked = some.bind().unlocked_relics.clone();
        let mut equipped = some.bind().equipped_relics.clone();
        let res = some
            .bind_mut()
            .try_equip_item(&unlocked, &mut equipped, idx as usize);

        match res {
            Ok(_item) => {
                some.bind_mut().unlocked_relics = unlocked;
                some.bind_mut().equipped_relics = equipped;
                // TODO: Add equipped relic grid. See above comment about scene modularity.
                // let icon = load::<Texture2D>(&item.icon_path);
                // self.equipped_relic_grid
                //     .get_node_as::<TextureRect>("TextureRect")
                //     .set_texture(&icon);
            }
            Err(e) => {
                dbg!(e);
            }
        }
    }
}
