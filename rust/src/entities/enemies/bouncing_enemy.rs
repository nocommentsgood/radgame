use std::array;

use godot::{
    builtin::Vector2,
    classes::{IStaticBody2D, PackedScene, StaticBody2D, Timer},
    obj::{Base, Gd, OnReady, WithBaseField},
    prelude::{GodotClass, godot_api},
};

use crate::{
    entities::{
        damage::{AttackData, Damage, DamageType, Damageable, Element, HasHealth},
        enemies::projectile::Projectile,
        hit_reg::{Hitbox, Hurtbox},
        movements::{self},
    },
    utils::collision_layers::CollisionLayers,
};

#[derive(GodotClass)]
#[class(init, base = StaticBody2D)]
pub struct BounceEnemy {
    health: u32,
    velocity: Vector2,
    #[init(node = "Hitbox")]
    hitbox: OnReady<Gd<Hitbox>>,

    #[init(node = "ShootCooldown")]
    timer: OnReady<Gd<Timer>>,

    #[init(val = OnReady::from_loaded("uid://bh5oo6002wig6"))]
    projectile: OnReady<Gd<PackedScene>>,
    base: Base<StaticBody2D>,
}

#[godot_api]
impl IStaticBody2D for BounceEnemy {
    fn ready(&mut self) {
        self.velocity = Vector2::UP;
        self.hitbox.bind_mut().damageable_parent = Some(Box::new(self.to_gd()));

        self.timer
            .signals()
            .timeout()
            .connect_other(&self.to_gd(), Self::shoot);
    }
    fn physics_process(&mut self, delta: f32) {
        movements::move_bounce(&mut self.to_gd().upcast(), &mut self.velocity, 50.0, delta);
    }
}

#[godot_api]
impl BounceEnemy {
    fn shoot(&mut self) {
        let mut projectiles: [Gd<Projectile>; 4] =
            array::from_fn(|_| self.projectile.instantiate_as::<Projectile>());

        let directions = [Vector2::LEFT, Vector2::UP, Vector2::DOWN, Vector2::RIGHT];
        for (projectile, dir) in projectiles.iter_mut().zip(directions.iter()) {
            projectile.bind_mut().velocity = *dir * 500.0;
        }

        for projectile in &mut projectiles {
            projectile.set_global_position(self.base().get_global_position());

            let mut hurtbox = projectile.get_node_as::<Hurtbox>("Hurtbox");
            hurtbox.set_collision_layer_value(CollisionLayers::EnemyHurtbox as i32, true);
            hurtbox.set_collision_mask_value(CollisionLayers::PlayerHitbox as i32, true);
            hurtbox.bind_mut().data = Some(AttackData {
                parryable: true,
                damage: Damage {
                    raw: 5,
                    d_type: DamageType::Elemental(Element::Poison),
                },
            });
        }

        for projectile in projectiles {
            self.base_mut().add_sibling(&projectile);
        }
    }
}

impl HasHealth for Gd<BounceEnemy> {
    fn get_health(&self) -> u32 {
        self.bind().health
    }

    fn set_health(&mut self, amount: u32) {
        self.bind_mut().health = amount;
    }

    fn on_death(&mut self) {
        self.queue_free();
    }
}

impl Damageable for Gd<BounceEnemy> {
    fn handle_attack(&mut self, attack: crate::entities::damage::AttackData) {
        self.take_damage(attack.damage.raw);
    }
}
