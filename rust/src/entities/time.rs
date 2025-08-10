use godot::{
    classes::{Timer, timer::TimerProcessCallback},
    obj::{Gd, NewAlloc},
};

#[derive(Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum EnemyTimer {
    AttackAnimation,
    AttackChainCooldown,
    AttackCooldown,
    Idle,
    Patrol,
    #[default]
    None,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum PlayerTimer {
    AttackChain,
    DodgeAnimation,
    JumpingAnimation,
    AttackAnimation,
    AttackAnimation2,
    HealingAnimation,
    HurtAnimation,
    ParryAnimation,
    Parry,
    PerfectParry,
    Coyote,
    DodgeCooldown,
}

trait TimerIndex {
    fn index(self) -> usize;
}

impl TimerIndex for PlayerTimer {
    fn index(self) -> usize {
        self as usize
    }
}

#[derive(Default, Clone)]
pub struct PlayerTimers(Vec<Gd<Timer>>);

impl PlayerTimers {
    pub fn add(&mut self, time: f32) {
        let mut timer = Timer::new_alloc();
        timer.set_wait_time(time as f64);
        self.0.push(timer);
    }
    pub fn get<T: TimerIndex>(&self, name: T) -> &Gd<Timer> {
        &self.0[name.index()]
        // self.0.get(name.index()).unwrap()
    }

    pub fn get_mut<T: TimerIndex>(&mut self, name: T) -> &mut Gd<Timer> {
        &mut self.0[name.index()]
    }

    pub fn iter(&self) -> impl Iterator<Item = &Gd<Timer>> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Gd<Timer>> {
        self.0.iter_mut()
    }
}

// impl IntoIterator for PlayerTimers {
//     type Gd<Timer>;
//
//     type IntoIter = std::vec::IntoIter<Gd<Timer>>;
//
//     fn into_iter(self) -> Self::IntoIter {
//         todo!()
//     }
// }
