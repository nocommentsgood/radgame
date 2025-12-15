use godot::{
    classes::{Node2D, PackedScene},
    obj::{Gd, NewAlloc},
    tools::load,
};

use crate::{
    entities::{
        combat::resources::{AttackResourceCost, CombatResources},
        hit_reg::Hurtbox,
        movements::{Direction, MoveLeft, MoveRight},
    },
    utils::global_data_singleton::GlobalData,
};

#[derive(Clone, Copy, Debug)]
pub struct Damage(pub i64);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Element {
    Magic,
    Poison,
    Lightning,
    Fire,
}

#[derive(Clone, Copy, Debug)]
pub enum Buff {
    Physical(i64),
    Elemental(Element, i64),
}

#[derive(Debug, Clone)]
pub struct Attack {
    damage: Damage,
    kind: AttackKind,
    resource_cost: Vec<AttackResourceCost>,
    parryable: bool,
}

impl Attack {
    pub fn cost(&self) -> &[AttackResourceCost] {
        &self.resource_cost
    }
    pub fn is_parryable(&self) -> bool {
        self.parryable
    }

    pub fn kind(&self) -> AttackKind {
        self.kind
    }

    pub fn damage(&self) -> Damage {
        self.damage
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PlayerAttacks {
    SimpleMelee,
    ChargedMelee,
    FireSpell,
    FireMelee,
}

impl PlayerAttacks {
    // TODO: Refactor player_level
    pub fn build(self, player_level: i64) -> Attack {
        match self {
            PlayerAttacks::SimpleMelee => Attack {
                damage: Damage(player_level * 10),
                kind: AttackKind::Melee,
                // TODO: Stamina cost is 0 for testing.
                resource_cost: vec![AttackResourceCost::Stamina(5)],
                parryable: true,
            },

            PlayerAttacks::ChargedMelee => Attack {
                damage: Damage(player_level * 15),
                kind: AttackKind::Melee,
                resource_cost: vec![AttackResourceCost::Stamina(10)],
                parryable: true,
            },

            PlayerAttacks::FireMelee => Attack {
                damage: Damage(player_level * 10),
                kind: AttackKind::ElementalMelee(Element::Fire),
                resource_cost: vec![AttackResourceCost::Stamina(5), AttackResourceCost::Mana(5)],
                parryable: true,
            },

            PlayerAttacks::FireSpell => Attack {
                damage: Damage(player_level * 20),
                kind: AttackKind::ProjectileSpell(Element::Fire),
                resource_cost: vec![AttackResourceCost::Mana(20)],
                parryable: false,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AttackKind {
    Melee,
    ElementalMelee(Element),
    ProjectileSpell(Element),
}

#[derive(Debug, Clone, Copy)]
pub enum Spell {
    TwinPillar,
    ProjectileSpell,
}

impl Spell {
    pub fn attack(self, player_level: i64) -> Attack {
        match self {
            Spell::TwinPillar => Attack {
                damage: Damage(player_level * 5),
                kind: AttackKind::ProjectileSpell(Element::Lightning),
                resource_cost: vec![AttackResourceCost::Mana(10)],
                parryable: false,
            },

            Spell::ProjectileSpell => Attack {
                damage: Damage(player_level * 15),
                kind: AttackKind::ProjectileSpell(Element::Poison),
                resource_cost: vec![AttackResourceCost::Mana(20)],
                parryable: false,
            },
        }
    }

    pub fn init_scene(self) -> Gd<Node2D> {
        match self {
            Spell::TwinPillar => {
                let player_pos = GlobalData::singleton().bind().player_pos;
                let attack = self.attack(1);
                let mut scene =
                    load::<PackedScene>("uid://dnfo3s5ywpq6m").instantiate_as::<Node2D>();
                let mut left = scene.get_node_as::<Hurtbox>("LeftPillar");
                let mut right = scene.get_node_as::<Hurtbox>("RightPillar");
                left.bind_mut().set_attack(attack.clone());
                right.bind_mut().set_attack(attack);
                scene.set_global_position(player_pos);
                scene
            }

            Spell::ProjectileSpell => {
                let dir = GlobalData::singleton().bind().player_dir;
                let dir_node: Gd<Node2D> = match dir {
                    Direction::Right => {
                        let mut right = MoveRight::new_alloc();
                        right.bind_mut().speed = 350.0;
                        right.upcast()
                    }
                    Direction::Left => {
                        let mut left = MoveLeft::new_alloc();
                        left.bind_mut().set_speed(350.0);
                        left.upcast()
                    }
                };

                let player_pos = GlobalData::singleton().bind().player_pos;
                let attack = self.attack(1);
                let mut scene =
                    load::<PackedScene>("res://entities/player/abilities/projectile_spell.tscn")
                        .instantiate_as::<Node2D>();
                let mut hurtbox = scene.get_node_as::<Hurtbox>("Hurtbox");
                hurtbox.bind_mut().set_attack(attack);
                scene.set_global_position(player_pos);
                scene.add_child(&dir_node);
                scene
            }
        }
    }
}

pub enum HotSpellIndexer {
    Ability1,
    Ability2,
    Ability3,
}

#[derive(Clone, Debug)]
pub struct Offense {
    buffs: Vec<Buff>,
    hot_spells: [Option<Spell>; 3],
}

impl Offense {
    pub fn new(buffs: Vec<Buff>, hot_spells: [Option<Spell>; 3]) -> Self {
        Self { buffs, hot_spells }
    }

    pub fn buffs(&self) -> &[Buff] {
        &self.buffs
    }

    pub fn add_buff(&mut self, buff: Buff) {
        self.buffs.push(buff);
    }

    pub fn apply_buffs(&self, attack: &mut Attack) {
        let mut amount = attack.damage.0;

        for buff in &self.buffs {
            match (&attack.kind, buff) {
                (AttackKind::Melee, Buff::Physical(val)) => {
                    amount += val;
                }

                (AttackKind::ElementalMelee(ele), Buff::Elemental(buff_ele, val)) => {
                    if ele == buff_ele {
                        amount += val;
                    }
                }
                (AttackKind::ElementalMelee(_ele), Buff::Physical(val)) => {
                    amount += val;
                }

                // TODO: Offense spell currently applies straight damage.
                _ => (),
            }
        }

        attack.damage.0 = amount;
    }

    pub fn get_spell(&self, idx: HotSpellIndexer) -> Option<Spell> {
        self.hot_spells[idx as usize]
    }

    pub fn check_resources(
        costs: &[AttackResourceCost],
        resource: &mut CombatResources,
    ) -> Result<(), ()> {
        resource.handle_attack_cost(costs)
    }

    pub fn try_attack(
        attack: PlayerAttacks,
        resources: &mut CombatResources,
        level: i64,
    ) -> Result<Attack, ()> {
        let attack = attack.build(level);
        if resources.handle_attack_cost(&attack.resource_cost).is_ok() {
            Ok(attack)
        } else {
            Err(())
        }
    }
}
