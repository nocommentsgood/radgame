use crate::entities::combat::offense::Damage;

#[derive(Clone, Copy, Debug)]
pub struct Resource {
    amount: i64,
    max: i64,
}

impl Resource {
    pub fn new(amount: i64, max: i64) -> Self {
        Self { amount, max }
    }
    pub fn amount(&self) -> i64 {
        self.amount
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

    pub fn amount(&self) -> i64 {
        self.0.amount
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

    pub fn amount(&self) -> i64 {
        self.0.amount
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Mana(Resource);
impl Mana {
    pub fn new(amount: i64, max: i64) -> Self {
        Self(Resource::new(amount, max))
    }

    pub fn amount(&self) -> i64 {
        self.0.amount
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AttackResourceCost {
    Stamina(i64),
    Mana(i64),
}

#[derive(Clone, Copy, Debug)]
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

    pub fn tick_resources(&mut self, delta: &f32) {
        if self.mana.0.amount < self.mana.0.max {
            self.mana_counter += delta;
            if self.mana_counter >= 8.0 {
                self.mana_counter = 0.0;
                self.mana.0.increase(10);
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

    pub fn handle_attack_cost(&mut self, costs: &[AttackResourceCost]) -> Result<(), ()> {
        for cost in costs {
            match cost {
                AttackResourceCost::Stamina(val) => {
                    if &self.stam.0.amount() >= val {
                        self.stam.0.decrease(*val);
                    } else {
                        return Err(());
                    }
                }

                AttackResourceCost::Mana(val) => {
                    if &self.mana.0.amount() >= val {
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
