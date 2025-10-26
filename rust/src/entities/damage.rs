use godot::{
    classes::{Node2D, PackedScene},
    obj::Gd,
    tools::load,
};

use crate::{entities::hit_reg::Hurtbox, utils::global_data_singleton::GlobalData};

#[derive(Clone, Copy, Debug)]
struct Resource {
    amount: i64,
    max: i64,
}

impl Resource {
    pub fn new(amount: i64, max: i64) -> Self {
        Self { amount, max }
    }
    pub fn amount(&self) -> &i64 {
        &self.amount
    }

    pub fn increase(&mut self, amount: i64) {
        let a = self.amount.saturating_add(amount);
        self.amount = a.clamp(0, self.max);
    }

    pub fn decrease(&mut self, amount: i64) {
        let a = self.amount.saturating_sub(amount);
        self.amount = a.clamp(0, self.max);
    }

    pub fn increase_max(&mut self, max: i64) {
        self.max = max;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Stamina(Resource);
impl Stamina {
    pub fn new(amount: i64, max: i64) -> Self {
        Self(Resource::new(amount, max))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Heal(i64);
impl Heal {
    pub fn new(amount: i64) -> Self {
        Self(amount)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Health(Resource, Heal);
impl Health {
    pub fn new(amount: i64, max: i64, heal: Heal) -> Self {
        Self(Resource::new(amount, max), heal)
    }

    pub fn take_damage(&mut self, damage: Damage) {
        self.0.decrease(damage.0);
    }

    pub fn is_dead(&self) -> bool {
        self.0.amount <= 0
    }

    pub fn heal(&mut self) {
        self.0.increase(self.1.0);
    }

    pub fn set_healing(&mut self, heal: Heal) {
        self.1 = heal;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Mana(Resource);
impl Mana {
    pub fn new(amount: i64, max: i64) -> Self {
        Self(Resource::new(amount, max))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Damage(pub i64);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Element {
    Magic,
    Poison,
    Lightning,
    Fire,
}

#[derive(Clone, Copy)]
pub enum Resistance {
    Physical(i64),
    Elemental(Element, i64),
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
    pub fn build(&self, player_level: i64) -> Attack {
        match self {
            PlayerAttacks::SimpleMelee => Attack {
                damage: Damage(player_level * 10),
                kind: AttackKind::Melee,
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
pub enum AttackResourceCost {
    Stamina(i64),
    Mana(i64),
}

#[derive(Debug, Clone, Copy)]
enum AttackKind {
    Melee,
    ElementalMelee(Element),
    ProjectileSpell(Element),
}

pub struct Defense {
    resistances: Vec<Resistance>,
}

impl Defense {
    pub fn new(resistances: Vec<Resistance>) -> Self {
        Self { resistances }
    }

    pub fn apply_resistances(&self, attack: Attack) -> Damage {
        let mut amount = attack.damage.0;

        for resistance in &self.resistances {
            match (&attack.kind, resistance) {
                (AttackKind::Melee, Resistance::Physical(val)) => {
                    amount -= val;
                }
                (AttackKind::ElementalMelee(_), Resistance::Physical(val)) => {
                    amount -= val;
                }
                (
                    AttackKind::ElementalMelee(attack_element),
                    Resistance::Elemental(resist_element, val),
                ) => {
                    if attack_element == resist_element {
                        amount -= val;
                    }
                }
                (AttackKind::ProjectileSpell(ele), Resistance::Elemental(res_ele, val)) => {
                    if ele == res_ele {
                        amount -= val;
                    }
                }
                _ => (),
            }
        }
        Damage(amount)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Spell {
    TwinPillar,
}

impl Spell {
    pub fn attack(&self, player_level: i64) -> Attack {
        match self {
            Spell::TwinPillar => Attack {
                damage: Damage(player_level * 5),
                kind: AttackKind::ProjectileSpell(Element::Lightning),
                resource_cost: vec![AttackResourceCost::Mana(10)],
                parryable: false,
            },
        }
    }

    pub fn init_scene(&self) -> Gd<Node2D> {
        match self {
            Spell::TwinPillar => {
                // BUG: Scene doesn't align with player's position.
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

        // TODO: This function should just update the attack's damage, not return the damage.
        attack.damage.0 = amount;
    }

    pub fn try_get_spell(&self, idx: HotSpellIndexer) -> Option<Spell> {
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

#[derive(Clone, Copy)]
pub struct CombatResources {
    health: Health,
    stam: Stamina,
    mana: Mana,
    stam_counter: f32,
    mana_counter: f32,
}

impl CombatResources {
    pub fn new(health: Health, stam: Stamina, mana: Mana) -> Self {
        Self {
            health,
            stam,
            mana,
            stam_counter: 0.0,
            mana_counter: 0.0,
        }
    }

    pub fn health(&self) -> &Health {
        &self.health
    }

    pub fn mana(&self) -> &Mana {
        &self.mana
    }

    pub fn stamina(&self) -> &Stamina {
        &self.stam
    }

    pub fn take_damage(&mut self, damage: Damage) {
        self.health.take_damage(damage);
    }

    pub fn heal(&mut self) {
        self.health.heal();
    }

    pub fn tick_resources(&mut self, delta: f32) {
        if self.mana.0.amount < self.mana.0.max {
            self.mana_counter += delta;
            if self.mana_counter >= 8.0 {
                self.mana_counter = 0.0;
                self.mana.0.increase(2);
            }
        }

        if self.stam.0.amount < self.stam.0.max {
            self.stam_counter += delta;
            if self.stam_counter > 3.0 {
                self.stam_counter = 0.0;
                self.stam.0.increase(5);
            }
        }
    }

    fn handle_attack_cost(&mut self, costs: &[AttackResourceCost]) -> Result<(), ()> {
        for cost in costs {
            match cost {
                AttackResourceCost::Stamina(val) => {
                    if self.stam.0.amount() >= val {
                        self.stam.0.decrease(*val);
                    } else {
                        return Err(());
                    }
                }

                AttackResourceCost::Mana(val) => {
                    if self.mana.0.amount() >= val {
                        self.mana.0.decrease(*val);
                    } else {
                        return Err(());
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use super::{
        Buff, CombatResources, Defense, Element, Heal, Health, Mana, Offense, PlayerAttacks,
        Resistance, Spell, Stamina,
    };

    struct Dummy {
        offense: Offense,
        defense: Defense,
        resource: CombatResources,
    }

    impl Dummy {
        fn new() -> Self {
            Self {
                offense: Offense {
                    buffs: vec![Buff::Physical(5), Buff::Elemental(Element::Fire, 5)],
                    hot_spells: [Some(Spell::TwinPillar), None, None],
                },
                defense: Defense {
                    resistances: vec![
                        Resistance::Physical(10),
                        Resistance::Elemental(Element::Magic, 5),
                    ],
                },
                resource: CombatResources::new(
                    Health::new(10, 10, Heal(2)),
                    Stamina::new(30, 30),
                    Mana::new(15, 15),
                ),
            }
        }
    }

    #[test]
    fn test_resouce_math() {
        use super::Resource;
        let mut resource = Resource::new(20, 30);
        resource.increase(11);
        assert!(resource.amount == 30);

        resource.decrease(10);
        assert_eq!(20, resource.amount);

        resource.decrease(21);
        assert_eq!(0, resource.amount);

        resource.increase_max(31);
        resource.increase(32);
        assert_eq!(31, resource.amount);
    }

    #[test]
    fn test_offense_handling() {
        let mut dummy = Dummy::new();
        if let Ok(mut attack) =
            Offense::try_attack(PlayerAttacks::SimpleMelee, &mut dummy.resource, 1)
        {
            dummy.offense.apply_buffs(&mut attack);
            assert_eq!(attack.damage.0, 15);
        }

        dummy.offense.buffs.push(Buff::Physical(1));

        if let Ok(mut attack) =
            Offense::try_attack(PlayerAttacks::SimpleMelee, &mut dummy.resource, 1)
        {
            dummy.offense.apply_buffs(&mut attack);
            assert_eq!(attack.damage.0, 16);
        }

        if let Ok(mut attack) =
            Offense::try_attack(PlayerAttacks::FireMelee, &mut dummy.resource, 1)
        {
            // base attack damage = 10
            // kind = ElementalMelee(Element::Fire)
            // buffs = Physical(1), Physical(5), Elemental::Fire(3)
            dummy.offense.apply_buffs(&mut attack);
            assert_eq!(attack.damage.0, 21);
        }
    }

    #[test]
    fn test_damage_handling() {
        use super::PlayerAttacks;

        let mut dummy = Dummy::new(); // health is 10
        let attack = PlayerAttacks::SimpleMelee.build(1); // 10
        let damage = dummy.defense.apply_resistances(attack); // dummy resistance is 10
        dummy.resource.health.take_damage(damage);
        assert_eq!(*dummy.resource.health.0.amount(), 10);

        let attack = PlayerAttacks::SimpleMelee.build(2); // 20
        let damage = dummy.defense.apply_resistances(attack);
        dummy.resource.health.take_damage(damage);
        assert_eq!(*dummy.resource.health.0.amount(), 0);
    }

    #[test]
    fn test_resource_consumption() {
        use super::PlayerAttacks;

        let mut attacker = Dummy::new();
        if Offense::try_attack(PlayerAttacks::SimpleMelee, &mut attacker.resource, 1).is_ok() {
            assert_eq!(attacker.resource.stam.0.amount(), &25);
        }
        if Offense::try_attack(PlayerAttacks::ChargedMelee, &mut attacker.resource, 1).is_ok() {
            assert_eq!(attacker.resource.stam.0.amount(), &15)
        }
        if Offense::try_attack(PlayerAttacks::FireMelee, &mut attacker.resource, 1).is_ok() {
            assert_eq!(attacker.resource.stam.0.amount(), &10);
            assert_eq!(attacker.resource.mana.0.amount(), &10);
        }

        assert!(Offense::try_attack(PlayerAttacks::FireSpell, &mut attacker.resource, 1).is_err());
    }
}
