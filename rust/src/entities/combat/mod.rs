pub mod defense;
pub mod offense;
pub mod resources;

#[cfg(test)]
mod test {

    use super::{
        defense::{Defense, Resistance},
        offense::{Buff, Element, Offense, PlayerAttacks, Spell},
        resources::{CombatResources, Heal, Health, Mana, Resource, Stamina},
    };

    struct Dummy {
        offense: Offense,
        defense: Defense,
        resource: CombatResources,
    }

    impl Dummy {
        fn new() -> Self {
            Self {
                offense: Offense::new(
                    vec![Buff::Physical(5), Buff::Elemental(Element::Fire, 5)],
                    [Some(Spell::TwinPillar), None, None],
                ),
                defense: Defense::new(vec![
                    Resistance::Physical(10),
                    Resistance::Elemental(Element::Magic, 5),
                ]),
                resource: CombatResources::new(
                    Health::new(10, 10, Heal::new(2)),
                    Stamina::new(30, 30),
                    Mana::new(15, 15),
                ),
            }
        }
    }

    #[test]
    fn test_resouce_math() {
        let mut resource = Resource::new(20, 30);
        resource.increase(11);
        assert!(resource.amount() == 30);

        resource.decrease(10);
        assert_eq!(20, resource.amount());

        resource.decrease(21);
        assert_eq!(0, resource.amount());

        resource.increase_max(31);
        resource.increase(32);
        assert_eq!(31, resource.amount());
    }

    #[test]
    fn test_offense_handling() {
        let mut dummy = Dummy::new();
        if let Ok(mut attack) =
            Offense::try_attack(PlayerAttacks::SimpleMelee, &mut dummy.resource, 1)
        {
            dummy.offense.apply_buffs(&mut attack);
            assert_eq!(attack.damage().0, 15);
        }

        dummy.offense.add_buff(Buff::Physical(1));

        if let Ok(mut attack) =
            Offense::try_attack(PlayerAttacks::SimpleMelee, &mut dummy.resource, 1)
        {
            dummy.offense.apply_buffs(&mut attack);
            assert_eq!(attack.damage().0, 16);
        }

        if let Ok(mut attack) =
            Offense::try_attack(PlayerAttacks::FireMelee, &mut dummy.resource, 1)
        {
            // base attack damage = 10
            // kind = ElementalMelee(Element::Fire)
            // buffs = Physical(1), Physical(5), Elemental::Fire(3)
            dummy.offense.apply_buffs(&mut attack);
            assert_eq!(attack.damage().0, 21);
        }
    }

    #[test]
    fn test_damage_handling() {
        let mut dummy = Dummy::new(); // health is 10
        let attack = PlayerAttacks::SimpleMelee.build(1); // 10
        let damage = dummy.defense.apply_resistances(&attack); // dummy resistance is 10
        dummy.resource.take_damage(damage);
        assert_eq!(dummy.resource.health().amount(), 10);

        let attack = PlayerAttacks::SimpleMelee.build(2); // 20
        let damage = dummy.defense.apply_resistances(&attack);
        dummy.resource.take_damage(damage);
        assert_eq!(dummy.resource.health().amount(), 0);
    }

    #[test]
    fn test_resource_consumption() {
        let mut attacker = Dummy::new();
        if Offense::try_attack(PlayerAttacks::SimpleMelee, &mut attacker.resource, 1).is_ok() {
            assert_eq!(attacker.resource.stamina().amount(), 25);
        }
        if Offense::try_attack(PlayerAttacks::ChargedMelee, &mut attacker.resource, 1).is_ok() {
            assert_eq!(attacker.resource.stamina().amount(), 15)
        }
        if Offense::try_attack(PlayerAttacks::FireMelee, &mut attacker.resource, 1).is_ok() {
            assert_eq!(attacker.resource.stamina().amount(), 10);
            assert_eq!(attacker.resource.mana().amount(), 10);
        }

        assert!(Offense::try_attack(PlayerAttacks::FireSpell, &mut attacker.resource, 1).is_err());
    }
}
