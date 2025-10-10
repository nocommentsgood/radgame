use crate::{
    entities::{
        enemies::{
            enemy_state_machine::EnemySMType,
            physics::{Movement, Speeds},
            time::Timers,
        },
        ent_graphics::EntGraphics,
        hit_reg::HitReg,
        movements::Direction,
    },
    utils::collision_layers::CollisionLayers,
};

use godot::{
    builtin::Vector2,
    classes::{Area2D, IArea2D, Node, Node2D, RayCast2D},
    obj::{Base, Gd, WithBaseField, WithUserSignals},
    prelude::{GodotClass, godot_api},
};

#[derive(Clone)]
pub struct EnemyContext {
    pub movement: Movement,
    pub graphics: EntGraphics,
    pub sensors: EnemySensors,
    pub timers: Timers,
    pub sm: EnemySMType,
}

impl EnemyContext {
    /// Provides `Self` by obtaining the required nodes in the SceneTree at the expected path.
    pub fn default_new(
        node: &Gd<Node>,
        speeds: Speeds,
        left_patrol_target: Vector2,
        right_patrol_target: Vector2,
        timers: Timers,
        sm: EnemySMType,
    ) -> Self {
        Self {
            movement: Movement::new(
                node.clone().cast::<Node2D>().get_global_position(),
                speeds,
                left_patrol_target,
                right_patrol_target,
            ),
            graphics: EntGraphics::new(node),
            sensors: EnemySensors::default_new(node),
            timers,
            sm,
        }
    }

    pub fn update_graphics(&mut self) {
        self.graphics.update(
            self.sm.state(),
            &Direction::from_vel(&self.movement.velocity()),
        );
    }

    /// Sets the normalized velocity, applies acceleration, and moves the entity.
    pub fn update_movement(&mut self, strategy: &mut super::physics::MovementStrategy, delta: f32) {
        self.movement.update(
            strategy,
            self.sm.state(),
            self.sensors.player_detection.player_position(),
            delta,
        );
    }
}

/// Area that tracks the player's position. Enables processing when entered, disables when exited.
#[derive(GodotClass)]
#[class(base=Area2D, init)]
pub struct AggroArea {
    player_position: Option<Vector2>,
    is_tracking: bool,
    base: Base<Area2D>,
}

#[godot_api]
impl IArea2D for AggroArea {
    fn ready(&mut self) {
        self.base_mut()
            .set_collision_mask_value(CollisionLayers::PlayerHitbox as i32, true);
        self.base_mut().set_process(false);
        self.signals()
            .area_entered()
            .connect_self(Self::on_area_entered);
        self.signals()
            .area_exited()
            .connect_self(Self::on_area_exited);
    }

    fn process(&mut self, _delta: f32) {
        self.track_player_position();
    }
}

#[godot_api]
impl AggroArea {
    fn on_area_entered(&mut self, area: Gd<Area2D>) {
        self.is_tracking = true;
        self.base_mut().set_process(true);
        self.player_position = Some(area.get_global_position());
    }

    fn on_area_exited(&mut self, _area: Gd<Area2D>) {
        self.is_tracking = false;
        self.base_mut().set_process(false);
        self.player_position = None;
    }

    fn track_player_position(&mut self) {
        if self.base().has_overlapping_areas() {
            let areas = self.base().get_overlapping_areas();
            for area in areas.iter_shared() {
                let player = area
                    .get_owner()
                    .expect("AggroArea overlapping areas should be the player's hitbox. Check the physics layers bit masks.")
                    .cast::<Node2D>();
                self.player_position = Some(player.get_global_position());
            }
        }
    }
}

#[derive(Clone)]
pub struct PlayerDetection {
    aggro_area: Gd<AggroArea>,
    attack_area: Gd<Area2D>,
}

impl PlayerDetection {
    pub fn new(aggro_area: Gd<AggroArea>, mut attack_area: Gd<Area2D>) -> Self {
        attack_area.set_collision_mask_value(CollisionLayers::PlayerHitbox as i32, true);

        Self {
            aggro_area,
            attack_area,
        }
    }

    /// Connects the given callbacks:
    /// - aggro area entered/exited
    /// - attack area entered/exited
    pub fn connect_signals<A, B, C, D>(
        &mut self,
        on_aggro_area_entered: A,
        on_aggro_area_exited: B,
        on_attack_area_entered: C,
        on_attack_area_exited: D,
    ) where
        A: FnMut(Gd<Area2D>) + 'static,
        B: FnMut(Gd<Area2D>) + 'static,
        C: FnMut(Gd<Area2D>) + 'static,
        D: FnMut(Gd<Area2D>) + 'static,
    {
        self.aggro_area
            .signals()
            .area_entered()
            .connect(on_aggro_area_entered);
        self.aggro_area
            .signals()
            .area_exited()
            .connect(on_aggro_area_exited);
        self.attack_area
            .signals()
            .area_entered()
            .connect(on_attack_area_entered);
        self.attack_area
            .signals()
            .area_exited()
            .connect(on_attack_area_exited);
    }

    pub fn player_position(&self) -> Option<Vector2> {
        self.aggro_area.bind().player_position
    }

    pub fn attack_area_overlapping(&self) -> bool {
        self.attack_area.has_overlapping_areas()
    }
}

#[derive(Clone)]
pub struct EnemySensors {
    pub hit_reg: HitReg,
    pub player_detection: PlayerDetection,
    left_ground_cast: Gd<RayCast2D>,
    right_ground_cast: Gd<RayCast2D>,
    left_wall_cast: Gd<RayCast2D>,
    right_wall_cast: Gd<RayCast2D>,
}

impl EnemySensors {
    /// Provides `Self` by obtaining the required nodes at the expected path in the SceneTree.
    /// "Expected path" meaning: `EnemySensors/*`
    pub fn default_new(base_enemy: &Gd<Node>) -> Self {
        Self {
            hit_reg: HitReg::new(
                base_enemy.get_node_as("EnemySensors/Hitbox"),
                base_enemy.get_node_as("EnemySensors/Hurtboxes"),
            ),
            player_detection: PlayerDetection::new(
                base_enemy.get_node_as("EnemySensors/AggroArea"),
                base_enemy.get_node_as("EnemySensors/AttackArea"),
            ),
            left_ground_cast: base_enemy.get_node_as("EnemySensors/LeftGroundCast"),
            right_ground_cast: base_enemy.get_node_as("EnemySensors/RightGroundCast"),
            left_wall_cast: base_enemy.get_node_as("EnemySensors/LeftWallCast"),
            right_wall_cast: base_enemy.get_node_as("EnemySensors/RightWallCast"),
        }
    }

    /// Connects the expected callbacks:
    /// - hitbox entered/exited
    /// - hurtbox entered/exited
    /// - aggro area entered/exited
    /// - attack area entered/exited
    #[allow(clippy::too_many_arguments)]
    pub fn connect_signals<A, B, C, D, E, F, G, H>(
        &mut self,
        on_hitbox_entered: A,
        on_hitbox_exited: B,
        on_hurtbox_entered: C,
        on_hurtbox_exited: D,
        on_aggro_area_entered: E,
        on_aggro_area_exited: F,
        on_attack_area_entered: G,
        on_attack_area_exited: H,
    ) where
        A: FnMut(Gd<Area2D>) + 'static,
        B: FnMut(Gd<Area2D>) + 'static,
        C: FnMut(Gd<Area2D>) + 'static,
        D: FnMut(Gd<Area2D>) + 'static,
        E: FnMut(Gd<Area2D>) + 'static,
        F: FnMut(Gd<Area2D>) + 'static,
        G: FnMut(Gd<Area2D>) + 'static,
        H: FnMut(Gd<Area2D>) + 'static,
    {
        self.hit_reg.connect_signals(
            on_hitbox_entered,
            on_hitbox_exited,
            on_hurtbox_entered,
            on_hurtbox_exited,
        );
        self.player_detection.connect_signals(
            on_aggro_area_entered,
            on_aggro_area_exited,
            on_attack_area_entered,
            on_attack_area_exited,
        );
    }
    /// True if a wallcast is colliding or a groundcast is not colliding.
    pub fn are_raycasts_failing(&self) -> bool {
        self.is_wall_cast_colliding() || !self.is_groundcast_colliding()
    }

    /// If a raycast check is failing, return the type of raycast and the direction of the raycast.
    /// Note that wall raycasts failing state is colliding and ground raycast failing state is
    /// **not** colliding.
    pub fn which(&self) -> Raycasts {
        if self.is_wall_cast_colliding() {
            let dir = self.wall_collision_dir();
            Raycasts::Wall(dir)
        } else if !self.is_groundcast_colliding() {
            let dir = self.groundcast_no_collision_dir();
            Raycasts::Ground(dir)
        } else {
            unreachable!()
        }
    }

    /// Wallcasts should only collide when the entity is too close to a wall. If the entity is
    /// close to a wall, movement should stop and the case should be handled.
    pub fn is_wall_cast_colliding(&self) -> bool {
        self.left_wall_cast.is_colliding() || self.right_wall_cast.is_colliding()
    }

    /// Groundcasts should always be colliding unless the entity is at a ledge.
    fn is_groundcast_colliding(&self) -> bool {
        self.left_ground_cast.is_colliding() || self.right_wall_cast.is_colliding()
    }

    pub fn wall_collision_dir(&self) -> Direction {
        if self.is_left_wallcast_colliding() {
            Direction::Left
        } else if self.is_right_wallcast_colliding() {
            Direction::Right
        } else {
            unreachable!(
                "is_wall_cast_colliding returned true but wall_collision_dir couldn't return a direction"
            );
        }
    }

    /// The direction of the groundcast that is not colliding.
    /// Groundcast should always be colliding, else the entity is about to fall.
    pub fn groundcast_no_collision_dir(&self) -> Direction {
        if !self.left_ground_cast.is_colliding() {
            Direction::Left
        } else if !self.right_ground_cast.is_colliding() {
            Direction::Right
        } else {
            unreachable!()
        }
    }

    fn is_left_wallcast_colliding(&self) -> bool {
        self.left_wall_cast.is_colliding()
    }

    pub fn is_right_wallcast_colliding(&self) -> bool {
        self.right_wall_cast.is_colliding()
    }
}

pub enum Raycasts {
    Ground(Direction),
    Wall(Direction),
}
