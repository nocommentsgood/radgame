use crate::{
    classes::{characters::character_stats::CharacterStats, enemies},
    traits::components::character_components::{
        character_resources::CharacterResources, damageable::Damageable, damaging::Damaging,
    },
};
use godot::{classes::Area2D, prelude::*};

use crate::classes::components::timer_component::Timer;

const BULLET_SPEED: real = 500.0;

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct ProjectileEnemy {
    movement_limit: Vector2,
    shoot_cooldown: Timer,
    stats: CharacterStats,
    base: Base<Node2D>,

    #[init(load = "res://projectile.tscn")]
    projectile_scene: OnReady<Gd<PackedScene>>,
}

#[godot_api]
impl INode2D for ProjectileEnemy {
    fn ready(&mut self) {
        self.stats.health = 20;
        self.connect_aggro_area();
        let spawn_position = self.base().get_global_position();
        let limit = Vector2::new(spawn_position.x + 500.0, spawn_position.y - 200.0);
        self.movement_limit = limit;
        self.shoot_cooldown = Timer::new(2.0);

        let mut this = self.to_gd();
        let mut hitbox = self.base().get_node_as::<Area2D>("Hitbox");
        hitbox
            .signals()
            .area_entered()
            .connect(move |area| this.bind_mut().on_area_entered_hitbox(area));
    }
}

#[godot_api]
impl ProjectileEnemy {
    fn is_outside_limits(&self) -> bool {
        let position = self.base().get_global_position();
        position.x > self.movement_limit.x || position.y > self.movement_limit.y
    }

    fn on_area_entered_hitbox(&mut self, area: Gd<Area2D>) {
        println!("area entered projectile enemy");
        let damaging = DynGd::<Area2D, dyn Damaging>::from_godot(area);
        let target = self.to_gd().upcast::<Node2D>();
        let _guard = self.base_mut();
        let damageable = DynGd::<Node2D, dyn Damageable>::from_godot(target);
        dbg!(&damaging);
        let amount = damaging.dyn_bind().damage_amount();
        println!("amoutn: {amount}");
        damaging.dyn_bind().do_damage(damageable);
    }

    fn shoot_projectile(&mut self, target: Vector2) {
        let position = self.base().get_global_position();
        let mut bullet = self
            .projectile_scene
            .instantiate_as::<enemies::projectile::Projectile>();
        let target = position.direction_to(target).normalized_or_zero();
        bullet.bind_mut().velocity = target * BULLET_SPEED;
        self.base_mut()
            .call_deferred("add_child", &[bullet.to_variant()]);
    }

    fn on_aggro_area_entered(&mut self, area: Gd<Area2D>) {
        if area.is_in_group("player") {
            let target = area.get_global_position();
            godot_print!("target pos: {}", target);
            self.shoot_projectile(target);
        }
    }

    fn connect_aggro_area(&mut self) {
        let mut aggro = self.base().get_node_as::<Area2D>("AggroArea");
        let mut this = self.to_gd();

        aggro
            .signals()
            .area_entered()
            .connect(move |area| this.bind_mut().on_aggro_area_entered(area));
    }

    fn move_around(&mut self) {
        if !self.is_outside_limits() {}
    }
}

#[godot_dyn]
impl CharacterResources for ProjectileEnemy {
    fn get_health(&self) -> u32 {
        self.stats.health
    }

    fn set_health(&mut self, amount: u32) {
        self.stats.health = amount;
    }

    fn get_energy(&self) -> u32 {
        self.stats.energy
    }

    fn set_energy(&mut self, amount: u32) {
        self.stats.energy = amount;
    }

    fn get_mana(&self) -> u32 {
        self.stats.mana
    }

    fn set_mana(&mut self, amount: u32) {
        self.stats.mana = amount;
    }
}

#[godot_dyn]
impl Damageable for ProjectileEnemy {
    fn destroy(&mut self) {
        self.base_mut().queue_free();
    }
}
