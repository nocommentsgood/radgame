use std::collections::VecDeque;

use godot::{
    builtin::Vector2,
    classes::{IStaticBody2D, StaticBody2D, Timer},
    obj::{Base, Gd, OnReady, WithBaseField},
    prelude::{GodotClass, godot_api},
};

use crate::entities::damage::Ability;

#[derive(GodotClass, Debug)]
#[class(init, base=StaticBody2D)]
pub struct JumpPlatform {
    pub velocity: Vector2,
    pub target: Vector2,
    pub start: Vector2,
    collision_count: u32,
    #[init(node = "FreeTimer")]
    free_timer: OnReady<Gd<Timer>>,
    #[init(node = "ChangeTimer")]
    change_timer: OnReady<Gd<Timer>>,
    base: Base<StaticBody2D>,
}

#[godot_api]
impl IStaticBody2D for JumpPlatform {
    fn ready(&mut self) {
        self.free_timer.set_wait_time(4.0);
        self.free_timer
            .signals()
            .timeout()
            .connect_other(&self.to_gd(), Self::free);

        self.change_timer.set_wait_time(2.0);
        self.change_timer
            .signals()
            .timeout()
            .connect_other(&self.to_gd(), Self::change_dir);

        self.free_timer.start();
        self.change_timer.start();
        self.start = self.base().get_position();
    }

    fn physics_process(&mut self, delta: f32) {
        let velocity = self.velocity * 100.0;

        let kin = self.base_mut().move_and_collide(velocity * delta);
        if let Some(col) = kin
            && let Some(obj) = col.get_collider()
            && !obj.is_class("MainCharacter")
        {
            self.change_dir();
        }
    }
}

#[godot_api]
impl JumpPlatform {
    fn free(&mut self) {
        self.run_deferred(|this| this.base_mut().queue_free());
    }

    fn change_dir(&mut self) {
        self.collision_count += 1;
        if self.collision_count == 2 {
            self.free();
        }
        self.change_timer.stop();
        let cur_pos = self.base().get_position();
        self.velocity = cur_pos.direction_to(self.start);
        let lin_vel = self.base().get_constant_linear_velocity() * -1.0;
        self.base_mut().set_constant_linear_velocity(lin_vel);
    }
}

#[derive(Default, Clone, Debug)]
pub struct AbilityComp {
    /// The player's quick abilities. Limited to a capacity of 3 abilities.
    quick: VecDeque<Ability>,
}

impl AbilityComp {
    pub fn new() -> Self {
        let v = vec![Ability::JumpPlatform, Ability::TwinPillar];
        Self { quick: v.into() }
    }

    pub fn get_current(&self) -> Ability {
        *self.quick.front().unwrap()
    }

    pub fn rotate_right(&mut self) {
        self.quick.rotate_right(1);
    }

    pub fn rotate_left(&mut self) {
        self.quick.rotate_left(1);
    }
}
