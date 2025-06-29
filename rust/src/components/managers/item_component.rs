use crate::classes::characters::character_stats::Stats;

use super::item::{GameItem, Item, ItemKind, ModifierKind, StatModifier};
use godot::{classes::InputEvent, prelude::*};

/// Error type used for equipping and unequipping
#[derive(Debug)]
pub enum EquipErr {
    CapacityReached,
    ItemNotFound,
    IncorrectItemKind,
    OutOfBounds,
}

#[derive(GodotClass)]
#[class(base=Node)]
pub struct ItemComponent {
    pub unlocked_beads: Vec<Option<Item>>,
    pub equipped_beads: Vec<Option<Item>>,
    collectables: Vec<Option<Item>>,
    pub unlocked_relics: Vec<Option<Item>>,
    pub equipped_relics: Vec<Option<Item>>,
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
            unlocked_relics: vec![None; 5],
            equipped_relics: vec![None; 3],
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

        let test_bead_2 = Item::new(
            ItemKind::RosaryBead {
                effect: StatModifier::new(Stats::RunningSpeed, ModifierKind::Flat(2)),
            },
            "TestBead1 WOW".to_string(),
            Some("A test bead that also increases movement speed".to_string()),
            "res://assets/bullet.webp".to_string(),
        );

        self.unlocked_beads.insert(1, Some(test_bead_2));
        let relic = Item::new(
            ItemKind::Relic {
                effect: StatModifier::new(Stats::MaxHealth, ModifierKind::Flat(2)),
            },
            "Relic Increase Max Health".to_string(),
            Some("A relic which, when equipped, increases max health".to_string()),
            "res://assets/bullet.webp".to_string(),
        );
        self.unlocked_relics.insert(0, Some(relic));
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

    pub fn try_equip_item(
        &mut self,
        unlocked_items: &[Option<Item>],
        equipped_items: &mut [Option<Item>],
        idx: usize,
    ) -> Result<Item, EquipErr> {
        let item = unlocked_items.get(idx);
        if let Some(None) | None = item {
            return Err(EquipErr::ItemNotFound);
        }

        if let Some(item) = item {
            if equipped_items.contains(item) {
                return self.unequip_item(equipped_items, item.as_ref().unwrap());
            }
            if equipped_items.iter().all(|slot| slot.is_some()) {
                return Err(EquipErr::CapacityReached);
            }
            let (ItemKind::RosaryBead { effect: modifier } | ItemKind::Relic { effect: modifier }) =
                item.clone().unwrap().kind
            else {
                return Err(EquipErr::IncorrectItemKind);
            };
            if let Some(slot) = equipped_items.iter_mut().find(|slot| slot.is_none()) {
                *slot = item.clone();
                self.signals()
                    .new_modifier()
                    .emit(&Gd::from_object(modifier.clone()));
                Ok(item.to_owned().unwrap())
            } else {
                Err(EquipErr::CapacityReached)
            }
        } else {
            Err(EquipErr::OutOfBounds)
        }
    }

    fn unequip_item(
        &mut self,
        equipped: &mut [Option<Item>],
        item: &super::item::Item,
    ) -> Result<Item, EquipErr> {
        if let Some(slot) = equipped.iter_mut().find(|i| i.as_ref() == Some(item)) {
            if let Some(item) = slot.take() {
                let (ItemKind::RosaryBead { effect: modifier }
                | ItemKind::Relic { effect: modifier }) = &item.kind
                else {
                    return Err(EquipErr::IncorrectItemKind);
                };
                self.signals()
                    .modifier_removed()
                    .emit(&Gd::from_object(modifier.clone()));
                Ok(item)
            } else {
                Err(EquipErr::ItemNotFound)
            }
        } else {
            Err(EquipErr::ItemNotFound)
        }
    }
}
