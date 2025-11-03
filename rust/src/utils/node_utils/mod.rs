use godot::{classes::Timer, obj::Gd};

pub mod free_timer;

pub trait ResetTimer {
    fn reset(&mut self);
}

impl ResetTimer for Gd<Timer> {
    fn reset(&mut self) {
        self.stop();
        self.start();
        self.stop();
    }
}
