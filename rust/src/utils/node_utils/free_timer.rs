use godot::{
    classes::{ITimer, Timer},
    obj::{Base, WithBaseField, WithUserSignals},
    prelude::{GodotClass, godot_api},
};

/// A oneshot, autostarting timer that frees the parent of the FreeTimer on timeout.
#[derive(GodotClass)]
#[class(base = Timer, init)]
pub struct FreeTimer {
    base: Base<Timer>,
}

#[godot_api]
impl ITimer for FreeTimer {
    fn ready(&mut self) {
        self.base_mut().set_autostart(true);
        self.base_mut().set_one_shot(true);
        self.signals().timeout().connect_self(Self::on_timeout);
    }
}

#[godot_api]
impl FreeTimer {
    fn on_timeout(&mut self) {
        let mut parent = self.base().get_parent().unwrap();
        parent.queue_free();
    }
}
