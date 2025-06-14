use crate::classes::characters::character_stats::Stats;

use super::item::{GameItem, Item, ItemKind, ModifierKind, StatModifier};
use godot::{classes::InputEvent, prelude::*};

#[derive(Debug)]
pub enum EquipErr {
    CapactiyReached,
    AlreadyEquipped,
    ItemNotFound,
    IncorrectItemKind,
}

#[derive(GodotClass)]
#[class(base=Node)]
pub struct ItemComponent {
    pub unlocked_beads: Vec<Item>,
    equipped_beads: Vec<Item>,
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
            unlocked_beads: Vec::with_capacity(5),
            equipped_beads: Vec::with_capacity(3),
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
        if input.is_action_pressed("equip") {
            println!("equipping test bead");
            self.try_equip_item(1);
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
        self.unlocked_beads.push(test_bead_1);
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
                        self.unlocked_beads.push(bind)
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
        let item = self.unlocked_beads.get(idx).ok_or(EquipErr::ItemNotFound)?;
        println!("trying to equip item: {:?}", item);
        println!("unlocked beads: {:?}", self.unlocked_beads);
        println!("equipped beads: {:?}", self.equipped_beads);

        if self.equipped_beads.contains(item) {
            return Err(EquipErr::AlreadyEquipped);
        }
        if self.equipped_beads.len() >= self.equipped_beads.capacity() {
            return Err(EquipErr::CapactiyReached);
        }
        let ItemKind::RosaryBead { effect: modif } = &item.kind else {
            return Err(EquipErr::IncorrectItemKind);
        };

        let m = Gd::from_object(modif.clone());
        self.equipped_beads.push(item.clone());
        self.signals().new_modifier().emit(&m);
        println!("after equipping\n\n");
        println!("unlocked beads: {:?}", self.unlocked_beads);
        println!("equipped beads: {:?}", self.equipped_beads);
        println!("\n\n");
        Ok(())
    }

    fn unequip_bead(&mut self, bead: &super::item::Item) {
        if let Some(pos) = self.equipped_beads.iter().position(|i| i == bead) {
            self.equipped_beads.remove(pos);
        }
    }
}
