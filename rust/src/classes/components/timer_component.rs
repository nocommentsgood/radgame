#[derive(Default, Clone)]
pub struct Timer {
    pub value: f64,
    init_value: f64,
}

impl Timer {
    pub fn new(value: f64) -> Self {
        Timer {
            value,
            init_value: value,
        }
    }

    pub fn reset(&mut self) {
        self.value = self.init_value;
    }

    pub fn initial_value(&self) -> f64 {
        self.init_value
    }
}

#[derive(Default)]
pub struct PlayerTimers {
    pub attack_chain_timer: Timer,
    pub dodging_animation_timer: Timer,
    pub jumping_animation_timer: Timer,
    pub attack_animation_timer: Timer,
    pub attack_animation_timer_2: Timer,
    pub healing_animation_timer: Timer,
    pub parry_animation_timer: Timer,
    pub parry_timer: Timer,
    pub perfect_parry_timer: Timer,
}

impl PlayerTimers {
    // stinky
    pub fn new(
        attack_chain_timer: f64,
        dodging_animation_timer: f64,
        jumping_animation_timer: f64,
        attack_animation_timer: f64,
        attack_animation_timer_2: f64,
        healing_animation_timer: f64,
        parry_animation_timer: f64,
        parry_timer: f64,
        perfect_parry_timer: f64,
    ) -> Self {
        PlayerTimers {
            attack_chain_timer: Timer::new(attack_chain_timer),
            dodging_animation_timer: Timer::new(dodging_animation_timer),
            jumping_animation_timer: Timer::new(jumping_animation_timer),
            attack_animation_timer: Timer::new(attack_animation_timer),
            attack_animation_timer_2: Timer::new(attack_animation_timer_2),
            healing_animation_timer: Timer::new(healing_animation_timer),
            parry_animation_timer: Timer::new(parry_animation_timer),
            parry_timer: Timer::new(parry_timer),
            perfect_parry_timer: Timer::new(perfect_parry_timer),
        }
    }
}

#[derive(Default, Clone)]
pub struct EnemyTimers {
    pub attack_animation: Timer,
    pub attack_cooldown: Timer,
    pub chain_attack: Timer,
    pub idle: Timer,
    pub patrol: Timer,
}

impl EnemyTimers {
    pub fn new(
        attack_animation: f64,
        attack_cooldown: f64,
        chain_attack: f64,
        idle: f64,
        patrol: f64,
    ) -> Self {
        EnemyTimers {
            attack_animation: Timer::new(attack_animation),
            attack_cooldown: Timer::new(attack_cooldown),
            chain_attack: Timer::new(chain_attack),
            idle: Timer::new(idle),
            patrol: Timer::new(patrol),
        }
    }
}
