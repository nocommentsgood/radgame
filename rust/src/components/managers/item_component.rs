use std::collections::HashMap;

use crate::classes::characters::character_stats::Stats;

use super::item::{GameItem, Item, ItemKind, ModifierKind, StatModifier};
use godot::{classes::InputEvent, prelude::*};

/// Error type used for equipping and unequipping
#[derive(Debug)]
pub enum EquipErr {
    CapactiyReached,
    ItemNotFound,
    IncorrectItemKind,
}

#[derive(GodotClass)]
#[class(base=Node)]
pub struct ItemComponent {
    pub unlocked_beads: Vec<Option<Item>>,
    equipped_beads: Vec<Option<Item>>,
    test_unlocked: HashMap<u32, Item>,
    test_equipped: HashMap<u32, Item>,
    unlocked_misc: Vec<Item>,
    equipped_misc: Vec<Item>,
    unlocked_relics: Vec<Item>,
    equipped_relics: Vec<Item>,
    in_item_area: bool,
    item: Option<Gd<GameItem>>,
    base: Base<Node>,
}

#[godot_api]
impl INode for ItemComponent {
    fn init(base: Base<Node>) -> Self {
        Self {
            unlocked_beads: vec![None; 9],
            equipped_beads: vec![None; 3],
            test_unlocked: HashMap::new(),
            test_equipped: HashMap::new(),
            unlocked_misc: Vec::with_capacity(3),
            equipped_misc: Vec::with_capacity(3),
            unlocked_relics: Vec::with_capacity(3),
            equipped_relics: Vec::with_capacity(3),
            in_item_area: false,
            item: None,
            base,
        }
    }
    fn unhandled_input(&mut self, input: Gd<InputEvent>) {
        if input.is_action_pressed("interact") {
            self.pickup_item();
        }

        // TODO: Used for testing. Remove.
        if input.is_action_pressed("equip") {
            println!("equipping test bead");
            dbg!(self.try_equip_item(0));
        }
    }

    fn ready(&mut self) {
        self.unlocked_misc.push(Item::new(
            ItemKind::Misc,
            "test item 1".to_string(),
            Some("This is a test item".to_string()),
            "res://assets/icon.svg".to_string(),
        ));

        self.unlocked_misc.push(Item::new(
            ItemKind::Misc,
            "test item 2".to_string(),
            Some("This is another test item".to_string()),
            "res://assets/icon.svg".to_string(),
        ));

        let test_bead_1 = Item::new(
            ItemKind::RosaryBead {
                effect: StatModifier::new(Stats::RunningSpeed, ModifierKind::Flat(2)),
            },
            "TestBead1 WOW".to_string(),
            Some("A test bead that increases movement speed".to_string()),
            "res://assets/icon.svg".to_string(),
        );
        self.unlocked_beads.insert(0, Some(test_bead_1));
    }
}

#[godot_api]
impl ItemComponent {
    #[signal]
    fn new_item_added(item: Gd<GameItem>);

    #[signal]
    fn picked_up_item(item: Gd<GameItem>);

    #[signal]
    pub fn new_modifier(modifier: Gd<StatModifier>);

    #[signal]
    pub fn modifier_removed(modifier: Gd<StatModifier>);

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
                match bind.kind {
                    super::item::ItemKind::Misc => self.unlocked_misc.push(bind),
                    super::item::ItemKind::RosaryBead { effect: _ } => {
                        // if let Some(item) = self.unlocked_beads.iter_mut().find(|i| i.is_none()) {
                        //     *item = Some(bind);
                        //     }
                        //
                        //
                        if !self.test_unlocked.values().any(|v| v == &bind) {
                            self.test_unlocked.insert()
                        }
                    }
                    _ => (),
                }
                self.signals().picked_up_item().emit(&item.clone());
                item.bind_mut().picked_up();
            }
        }
        self.in_item_area = false;
    }

    pub fn try_equip_item(&mut self, idx: usize) -> Result<(), EquipErr> {
        // dbg!(&self.equipped_beads.get(idx).unwrap());
        // dbg!(&self.unlocked_beads.get(idx).unwrap());
        let item = self.unlocked_beads.get(idx);
        if let Some(s_item) = item {
            let i = s_item.as_ref();
            if let Some(Some(res)) = self.equipped_beads.iter().find(|i| *i == **item) {
                let r = res.clone();
                self.try_unequip_item(&res.as_ref());
            }
            if self.equipped_beads.iter().all(|b| b.is_some()) {
                dbg!("133");
                Err(EquipErr::CapactiyReached)
            } else {
                let (ItemKind::RosaryBead { effect: modif } | ItemKind::Relic { effect: modif }) =
                    &item.kind
                else {
                    dbg!("138");
                    return Err(EquipErr::IncorrectItemKind);
                };
                let m = Gd::from_object(modif.clone());
                dbg!(&self.equipped_beads);
                if let Some(i) = self.equipped_beads.iter_mut().find(|i| i.is_none()) {
                    *i = Some(item.to_owned());
                    self.signals().new_modifier().emit(&m);
                }
                dbg!(&self.equipped_beads);
                Ok(())
            }
        } else {
            dbg!("148");
            Err(EquipErr::ItemNotFound)
        }
    }

    fn try_unequip_item(&mut self, item: &super::item::Item) -> Result<(), EquipErr> {
        if let Some(pos) = self
            .equipped_beads
            .iter()
            .position(|i| *i == Some(item.clone()))
        {
            if let Some(removed) = self.equipped_beads.remove(pos) {
                self.equipped_beads.insert(pos, None);
                let (ItemKind::RosaryBead { effect: modifier }
                | ItemKind::Relic { effect: modifier }) = &removed.kind
                else {
                    dbg!("Line 163");
                    return Err(EquipErr::IncorrectItemKind);
                };
                let modifier = Gd::from_object(modifier.clone());
                self.signals().modifier_removed().emit(&modifier);
            }
            Ok(())
        } else {
            dbg!("line 171");
            Err(EquipErr::ItemNotFound)
        }
    }
}
