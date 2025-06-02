use crate::classes::characters::character_stats::Stats;

use super::item::{GameItem, Item, ItemKind, ModifierKind, StatModifier};
use godot::{classes::InputEvent, prelude::*};

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
            // TODO: This is just for testing
            // let bead = self.unlocked_items.first().unwrap().clone();
            // println!("trying to equip bead: {:?}", bead);
            // self.equip_bead(&bead);

            println!("equipping test bead");
            let test_bead = self.unlocked_beads.first().unwrap().clone();
            self.equip_item(&test_bead);
        }
    }

    fn ready(&mut self) {
        self.push_test_items();
        println!(
            "equp beads cap: {} \n equp beads len: {}",
            self.equipped_beads.capacity(),
            self.equipped_beads.len()
        );
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

    fn push_test_items(&mut self) {
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
                effect: StatModifier::new(Stats::RunningSpeed, ModifierKind::Flat(2.0)),
                equipped: false,
            },
            "TestBead1".to_string(),
            Some("A test bead that increases movement speed".to_string()),
            "res://assets/icon.svg".to_string(),
        );
        self.unlocked_beads.push(test_bead_1);
    }

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
                    super::item::ItemKind::RosaryBead {
                        effect: _,
                        equipped: false,
                    } => self.unlocked_beads.push(bind),
                    _ => (),
                }
                self.signals().picked_up_item().emit(&item.clone());
                item.bind_mut().picked_up();
            }
        }
        self.in_item_area = false;
    }

    fn equip_item(&mut self, item: &super::item::Item) {
        if let ItemKind::RosaryBead {
            effect: modif,
            equipped: false,
        } = &item.kind
        {
            println!(
                "equipped beads len: {} \n equipped beads cap: {}",
                self.equipped_beads.len(),
                self.equipped_beads.capacity()
            );
            if self.equipped_beads.len() < self.equipped_beads.capacity() {
                if let Ok(bead) = item.clone().set_equipped() {
                    self.equipped_beads.push(bead);
                }
                let modif = Gd::from_object(modif.clone());
                println!("emitting mod sig");
                self.signals().new_modifier().emit(&modif);
            } else {
                panic!("Maximum number of equipped beads reached.");
            }
        }
        if let ItemKind::Relic {
            effect: modif,
            equipped: false,
        } = &item.kind
        {
            println!(
                "equipped relics len: {} \n equipped relics cap: {}",
                self.equipped_relics.len(),
                self.equipped_relics.capacity()
            );
            if self.equipped_relics.len() < self.equipped_relics.capacity() {
                if let Ok(relic) = item.clone().set_equipped() {
                    self.equipped_beads.push(relic);
                }
                let modif = Gd::from_object(modif.clone());
                println!("emitting mod sig");
                self.signals().new_modifier().emit(&modif);
            } else {
                panic!("Maximum number of equipped relics reached.");
            }
        }
        // } else {
        //     panic!("Can not equip item {:?}\nEnsure item type is equipable and allowed number of equipped items is not reached.", item);
        // }
    }

    fn unequip_bead(&mut self, bead: &super::item::Item) {
        if let Some(pos) = self.equipped_beads.iter().position(|i| i == bead) {
            self.equipped_beads.remove(pos);
        }
    }
}
