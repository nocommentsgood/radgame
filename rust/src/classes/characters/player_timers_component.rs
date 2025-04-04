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
pub struct TimerComponent {
    pub attack_chain_timer: Timer,
    pub dodging_animation_timer: Timer,
    pub jumping_animation_timer: Timer,
    pub attack_animation_timer: Timer,
    pub attack_animation_timer_2: Timer,
    pub healing_animation_timer: Timer,
}

impl TimerComponent {
    pub fn new(
        attack_chain_timer: f64,
        dodging_animation_timer: f64,
        jumping_animation_timer: f64,
        attack_animation_timer: f64,
        attack_animation_timer_2: f64,
        healing_animation_timer: f64,
    ) -> Self {
        TimerComponent {
            attack_chain_timer: Timer::new(attack_chain_timer),
            dodging_animation_timer: Timer::new(dodging_animation_timer),
            jumping_animation_timer: Timer::new(jumping_animation_timer),
            attack_animation_timer: Timer::new(attack_animation_timer),
            attack_animation_timer_2: Timer::new(attack_animation_timer_2),
            healing_animation_timer: Timer::new(healing_animation_timer),
        }
    }
}
