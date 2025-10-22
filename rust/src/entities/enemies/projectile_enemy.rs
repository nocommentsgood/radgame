use super::{enemy_state_machine::State, projectile::Projectile};
use crate::entities::{
    damage::{
        Buff, CombatResources, Defense, Element, Health, Mana, Offense, PlayerAttacks, Resistance,
        Stamina,
    },
    enemies::{enemy_context as ctx, enemy_state_machine as esm, physics, time},
    graphics::Graphics,
    hit_reg::Hurtbox,
    movements::Direction,
};
use godot::{
    builtin::Vector2,
    classes::{Area2D, INode2D, Node2D, PackedScene},
    obj::{Base, Gd, OnReady, WithBaseField},
    prelude::{GodotClass, godot_api},
    tools::load,
};
use statig::prelude::StateMachine;

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct NewProjectileEnemy {
    #[export]
    left_target: Vector2,
    #[export]
    right_target: Vector2,

    #[init(val = OnReady::manual())]
    projectile_scene: OnReady<Gd<PackedScene>>,
    #[init(val = OnReady::manual())]
    movement: OnReady<physics::Movement>,
    #[init(val = OnReady::manual())]
    sensors: OnReady<ctx::EnemySensors>,
    #[init(val = OnReady::manual())]
    timers: OnReady<time::Timers>,
    #[init(val = OnReady::manual())]
    sm: OnReady<esm::EnemySMType>,
    #[init(val = OnReady::from_base_fn(|this|{ Graphics::new(this)}))]
    graphics: OnReady<Graphics>,

    #[init(val = OnReady::new(|| Health::new(10, 10)))]
    health: OnReady<Health>,

    #[init(val = OnReady::new(|| CombatResources::new(
        Stamina::new(20, 20), Mana::new(20, 20)
    )))]
    pub resources: OnReady<CombatResources>,

    #[init(val = OnReady::new(|| Defense::new(vec![Resistance::Physical(5)])))]
    def: OnReady<Defense>,

    #[init(val = OnReady::new(|| Offense::new(vec![Buff::Elemental(Element::Magic, 2)])))]
    pub off: OnReady<Offense>,

    node: Base<Node2D>,
}

#[godot_api]
impl INode2D for NewProjectileEnemy {
    fn ready(&mut self) {
        self.projectile_scene
            .init(load("res://world/projectile.tscn"));
        self.movement.init(physics::Movement::new(
            self.base().get_global_position(),
            physics::Speeds::new(150.0, 175.0),
            self.left_target,
            self.right_target,
        ));
        self.sensors
            .init(ctx::EnemySensors::default_new(&self.to_gd().upcast()));
        self.timers
            .init(time::Timers::default_new(&self.to_gd().upcast()));
        self.sm
            .init(esm::EnemySMType::Basic(StateMachine::default()));

        let this = self.to_gd();
        self.timers.connect_signals(
            {
                let mut this = this.clone();
                move || this.bind_mut().on_attack_timeout()
            },
            {
                let mut this = this.clone();
                move || this.bind_mut().on_patrol_timeout()
            },
            {
                let mut this = this.clone();
                move || this.bind_mut().on_idle_timeout()
            },
            {
                let mut this = this.clone();
                move || this.bind_mut().on_attack_chain_timeout()
            },
            {
                let mut this = this.clone();
                move || this.bind_mut().on_attack_anim_timeout()
            },
        );

        self.sensors.connect_signals(
            {
                let mut this = this.clone();
                move |area| this.bind_mut().on_hitbox_entered(area)
            },
            |_| (),
            |_| (),
            |_| (),
            {
                let mut this = this.clone();
                move |area| this.bind_mut().on_aggro_area_entered(area)
            },
            {
                let mut this = this.clone();
                move |area| this.bind_mut().on_aggro_area_exited(area)
            },
            {
                let mut this = this.clone();
                move |area| this.bind_mut().on_attack_area_entered(area)
            },
            |_| (),
        );

        self.timers.idle.start();
    }

    fn process(&mut self, delta: f32) {
        match self.sm.state() {
            State::ChasePlayer {}
                if self.sensors.player_detection.attack_area_overlapping()
                    && self.timers.attack.get_time_left() == 0.0 =>
            {
                self.sm.handle(&esm::EnemyEvent::InAttackRange);
            }
            State::Attack {} if self.timers.attack.get_time_left() == 0.0 => {
                self.shoot_projectile();
                self.timers.attack.start();
                self.timers.attack_anim.start();
            }
            State::Attack2 {} if self.timers.attack.get_time_left() == 0.0 => {
                self.shoot_projectile();
                self.timers.attack_chain.start();
                self.timers.attack.start();
            }
            _ => (),
        }

        let this = self.to_gd();
        self.movement.update(
            &mut physics::MovementStrategy::ManualSetPosition(this.upcast()),
            self.sm.state(),
            self.sensors.player_detection.player_position(),
            delta,
        );
        self.graphics.update(
            self.sm.state(),
            &Direction::from_vel(&self.movement.velocity()),
        );
    }
}

#[godot_api]
impl NewProjectileEnemy {
    fn on_hitbox_entered(&mut self, area: Gd<Area2D>) {
        let hurtbox = area.cast::<Hurtbox>();

        // BUG: Seems like on occasion the hurtbox sets the attack after the hitbox checks it,
        // leading to a panic upon unwrapping.
        // Could try:
        // - disabling multi-threading in Godot editor
        // - deferring the hitbox check to the next idle frame
        let attack = hurtbox.bind().attack.clone().unwrap();

        let damage = self.def.apply_resistances(attack);
        self.health.take_damage(damage);
        if self.health.is_dead() {
            self.sm.handle(&esm::EnemyEvent::Death);
            self.run_deferred(|this| this.base_mut().queue_free());
        }
    }

    pub fn on_aggro_area_entered(&mut self, _area: Gd<Area2D>) {
        self.sm.handle(&esm::EnemyEvent::FoundPlayer);
    }

    pub fn on_aggro_area_exited(&mut self, _area: Gd<Area2D>) {
        self.sm.handle(&esm::EnemyEvent::LostPlayer);
        self.timers.idle.start();
    }

    pub fn on_attack_area_entered(&mut self, _area: Gd<Area2D>) {
        self.sm.handle(&esm::EnemyEvent::InAttackRange);
    }

    pub fn on_idle_timeout(&mut self) {
        if self.sm.state() == (&State::Idle {}) {
            self.sm
                .handle(&esm::EnemyEvent::TimerElapsed(time::EnemyTimers::Idle));
            self.timers.patrol.start();
            self.movement.patrol();
        }
    }

    pub fn on_patrol_timeout(&mut self) {
        if self.sm.state() == (&State::Patrol {}) {
            self.sm
                .handle(&esm::EnemyEvent::TimerElapsed(time::EnemyTimers::Patrol));
            self.timers.idle.start();
        }
    }

    pub fn on_attack_timeout(&mut self) {
        self.sm
            .handle(&esm::EnemyEvent::TimerElapsed(time::EnemyTimers::Attack));
    }

    fn on_attack_chain_timeout(&mut self) {
        self.shoot_projectile();
        self.sm.handle(&esm::EnemyEvent::TimerElapsed(
            time::EnemyTimers::AttackChain,
        ));
    }

    fn on_attack_anim_timeout(&mut self) {
        self.sm.handle(&esm::EnemyEvent::TimerElapsed(
            time::EnemyTimers::AttackAnimation,
        ));
    }

    pub fn shoot_projectile(&mut self) {
        if let Some(player_pos) = self.sensors.player_detection.player_position() {
            let mut inst = self.projectile_scene.instantiate_as::<Projectile>();
            let target = self
                .base()
                .get_global_position()
                .direction_to(player_pos)
                .normalized_or_zero();
            let pos = self.base().get_global_position();

            if let Ok(mut attack) =
                Offense::try_attack(PlayerAttacks::FireSpell, &mut self.resources, 1)
            {
                self.off.apply_buffs(&mut attack);
                let mut hurtbox = inst.get_node_as::<Hurtbox>("Hurtbox");
                hurtbox.bind_mut().set_attack(attack);
                inst.set_global_position(pos);
                inst.bind_mut().target = target;
                // inst.bind_mut().velocity = target;
                self.base_mut().add_sibling(&inst);
            }
        }
    }
}
