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
    collectables: Vec<Option<Item>>,
    unlocked_relics: Vec<Option<Item>>,
    equipped_relics: Vec<Option<Item>>,
    quest_and_other: Vec<Option<Item>>,
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
            collectables: vec![None; 20],
            unlocked_relics: Vec::with_capacity(7),
            equipped_relics: Vec::with_capacity(3),
            quest_and_other: vec![None; 20],
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
            dbg!(self.try_equip_item(0));
        }
    }

    fn ready(&mut self) {
        self.collectables.push(Some(Item::new(
            ItemKind::Collectable,
            "test item 1".to_string(),
            Some("This is a test item".to_string()),
            "res://assets/icon.svg".to_string(),
        )));

        self.collectables.push(Some(Item::new(
            ItemKind::Collectable,
            "test item 2".to_string(),
            Some("This is another test item".to_string()),
            "res://assets/icon.svg".to_string(),
        )));

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
                    // Duplicates are OK on misc items.
                    super::item::ItemKind::Collectable => self.collectables.push(Some(bind)),
                    super::item::ItemKind::RosaryBead { effect: _ } => {
                        if self.unlocked_beads.iter().flatten().any(|i| i == &bind) {
                        } else if let Some(item) =
                            self.unlocked_beads.iter_mut().find(|i| i.is_none())
                        {
                            *item = Some(bind);
                        }
                    }
                    super::item::ItemKind::Relic { effect: _ } => {
                        if self.unlocked_relics.iter().flatten().any(|i| i == &bind) {
                        } else if let Some(relic) =
                            self.unlocked_relics.iter_mut().find(|i| i.is_none())
                        {
                            *relic = Some(bind);
                        }
                    }
                    _ => self.quest_and_other.push(Some(bind)),
                }
                self.signals().picked_up_item().emit(&item.clone());
                item.bind_mut().picked_up();
            }
        }
        self.in_item_area = false;
    }

    pub fn try_equip_item(&mut self, idx: usize) -> Result<(), EquipErr> {
        let item = self
            .unlocked_beads
            .get(idx)
            .or_else(|| self.unlocked_relics.get(idx));

        if let Some(i) = item {
            let kind = match i.as_ref() {
                Some(ty) => Ok(ty.kind.clone()),
                None => Err(EquipErr::IncorrectItemKind),
            };
        }
        let s_item = match self.unlocked_beads.get(idx).and_then(|opt| opt.as_ref()) {
            Some(item) => item.clone(),
            None => return Err(EquipErr::ItemNotFound),
        };

        // Check for capacity
        if self.equipped_beads.iter().all(|b| b.is_some()) {
            return Err(EquipErr::CapactiyReached);
        }

        // Validate item kind
        let modifier = match &s_item.kind {
            ItemKind::RosaryBead { effect } | ItemKind::Relic { effect } => {
                Gd::from_object(effect.clone())
            }
            _ => return Err(EquipErr::IncorrectItemKind),
        };
        // Unequip if already equipped
        if let Some(existing) = self
            .equipped_beads
            .iter()
            .filter_map(|o| o.as_ref())
            .find(|&i| *i == s_item)
        {
            self.try_unequip_item(&existing.clone())
        } else if let Some(slot) = self.equipped_beads.iter_mut().find(|i| i.is_none()) {
            *slot = Some(s_item.clone());
            self.signals().new_modifier().emit(&modifier);
            Ok(())
        } else {
            Err(EquipErr::CapactiyReached)
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
                    return Err(EquipErr::IncorrectItemKind);
                };
                let modifier = Gd::from_object(modifier.clone());
                self.signals().modifier_removed().emit(&modifier);
            }
            Ok(())
        } else {
            Err(EquipErr::ItemNotFound)
        }
    }
}
