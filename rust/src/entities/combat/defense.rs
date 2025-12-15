use crate::entities::combat::offense::{Attack, AttackKind, Damage, Element};

#[derive(Clone, Copy)]
pub enum Resistance {
    Physical(i64),
    Elemental(Element, i64),
}

pub struct Defense {
    resistances: Vec<Resistance>,
}

impl Defense {
    pub fn new(resistances: Vec<Resistance>) -> Self {
        Self { resistances }
    }

    pub fn apply_resistances(&self, attack: &Attack) -> Damage {
        let mut amount = attack.damage().0;

        for resistance in &self.resistances {
            match (&attack.kind(), resistance) {
                (AttackKind::Melee | AttackKind::ElementalMelee(_), Resistance::Physical(val)) => {
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
