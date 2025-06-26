#[derive(Clone, Copy)]
pub enum PlayerTimer {
    AttackChain,
    DodgeAnimation,
    JumpingAnimation,
    AttackAnimation,
    AttackAnimation2,
    HealingAnimation,
    ParryAnimation,
    Parry,
    PerfectParry,
}

#[derive(Clone, Copy)]
pub enum EnemyTimer {
    AttackAnimation,
    AttackChain,
    AttackCooldown,
    Idle,
    Patrol,
}

pub trait TimerIndex {
    fn index(&self) -> usize;
}

impl TimerIndex for PlayerTimer {
    fn index(&self) -> usize {
        *self as usize
    }
}
impl TimerIndex for EnemyTimer {
    fn index(&self) -> usize {
        *self as usize
    }
}

#[derive(Default)]
pub struct Time(pub f32, f32);

impl Time {
    pub fn new(value: f32) -> Self {
        Time(value, value)
    }

    // TODO: This is public until `projectile.rs` gets updated.
    pub fn reset(&mut self) {
        self.0 = self.1;
    }
}

#[derive(Default)]
pub struct Timers(pub Vec<Time>);

impl Timers {
    pub fn get<T: TimerIndex>(&self, name: &T) -> f32 {
        self.0.get(name.index()).unwrap().0
    }

    pub fn set<T: TimerIndex>(&mut self, name: &T, value: f32) {
        self.0.get_mut(name.index()).unwrap().0 = value;
    }
    pub fn get_init<T: TimerIndex>(&self, name: &T) -> f32 {
        self.0.get(name.index()).unwrap().1
    }

    pub fn reset<T: TimerIndex>(&mut self, name: &T) {
        self.0.get_mut(name.index()).unwrap().reset();
    }
}
