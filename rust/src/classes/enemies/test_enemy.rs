use godot::{
    classes::{AnimationPlayer, CharacterBody2D, ICharacterBody2D, Timer},
    prelude::*,
};

use crate::traits::components::character_components::{
    character_resources::CharacterResources, damageable::Damageable, damaging::Damaging,
};

#[derive(GodotClass)]
#[class(init, base=CharacterBody2D)]
pub struct TestEnemy {
    #[var]
    speed: real,
    #[var]
    health: i32,
    #[var]
    energy: i32,
    #[var]
    mana: i32,
    velocity: Vector2,

    #[init(node = "AnimationPlayer2")]
    animation_player: OnReady<Gd<AnimationPlayer>>,

    #[init(node = "MovementTimer")]
    movement_timer: OnReady<Gd<Timer>>,
    running_speed: real,
    base: Base<CharacterBody2D>,
}

#[godot_api]
impl ICharacterBody2D for TestEnemy {
    // fn physics_process(&mut self, delta: f64) {
    //     let idle_vel = Vector2::ZERO;
    //     let left_vel = Vector2::LEFT;
    //     let right_vel = Vector2::RIGHT;
    //
    //     self.run(left_vel * self.running_speed * delta as f32);
    //     self.set_direction();
    //     self.get_movement_animation();
    //     let animation = self.get_movement_animation();
    //     let mut animate = self
    //         .base_mut()
    //         .get_node_as::<AnimationPlayer>("AnimationPlayer2");
    //     animate.play_ex().name(&animation).done();
    //     self.run(right_vel);
    //
    //     self.velocity = idle_vel;
    //     self.set_direction();
    //     self.get_movement_animation();
    //     let animation = self.get_movement_animation();
    //     animate.play_ex().name(&animation).done();
    //     self.run(idle_vel);
    // }
}

#[godot_api]
impl TestEnemy {
    // fn get_movement_animation(&mut self) -> String {
    //     let dir = self.direction.to_string();
    //     let mut state = self.state.to_string();
    //
    //     state.push('_');
    //     format!("{}{}", state, dir)
    // }
    //
    // fn set_direction(&mut self) {
    //     self.direction = Directions::from_velocity(&self.base().get_velocity());
    // }

    // fn run(&mut self, vel: Vector2) {
    //     if vel.length() == 0.0 {
    //         self.velocity = vel;
    //         self.state = CharacterState::Idle {
    //             direction: Directions::from_velocity(&self.velocity),
    //         };
    //         return;
    //     }
    //
    //     if vel.x != 0.0 {
    //         self.velocity = vel;
    //         self.base_mut().set_velocity(vel);
    //         self.set_direction();
    //         self.base_mut().move_and_slide();
    //     }
    //
    //     self.movement_timer.start();
    // }
}

#[godot_dyn]
impl CharacterResources for TestEnemy {
    fn get_health(&self) -> i32 {
        self.health
    }

    fn set_health(&mut self, amount: i32) {
        self.health = amount;
    }

    fn get_energy(&self) -> i32 {
        self.energy
    }

    fn set_energy(&mut self, amount: i32) {
        self.energy = amount;
    }

    fn get_mana(&self) -> i32 {
        self.mana
    }

    fn set_mana(&mut self, amount: i32) {
        self.mana = amount;
    }
}

#[godot_dyn]
impl Damageable for TestEnemy {}

#[godot_dyn]
impl Damaging for TestEnemy {}
