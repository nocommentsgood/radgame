use godot::{
    builtin::Vector2,
    classes::{CharacterBody2D, INode2D, Node2D, PhysicsBody2D, Timer},
    meta::FromGodot,
    obj::{Base, DynGd, Gd, Inherits, OnEditor, WithBaseField},
    prelude::{Export, GodotClass, GodotConvert, Var, godot_api, godot_dyn},
};

#[derive(Default, Debug, Clone, Copy, PartialEq, GodotConvert, Var, Export)]
#[godot(via = i64)]
pub enum Direction {
    #[default]
    Right,
    Left,
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Right => write!(f, "east"),
            Direction::Left => write!(f, "west"),
        }
    }
}

impl Direction {
    pub fn from_vel(velocity: &Vector2) -> Direction {
        if velocity.x < 0.0 {
            Direction::Left
        } else {
            Direction::Right
        }
    }

    pub fn to_vel(self) -> Vector2 {
        match self {
            Direction::Right => Vector2::RIGHT,
            Direction::Left => Vector2::LEFT,
        }
    }
}

pub trait Moveable {
    fn get_velocity(&self) -> Vector2;
    fn set_velocity(&mut self, velocity: Vector2);
}

/// Implements movement for CharacterBody2D's.
pub trait MoveableBody: Moveable {
    /// Calls `move_and_slide()` on the CharacterBody2D. Ensure `set_velocity` is set with desired speed
    /// prior to calling.
    fn notify_on_floor(&mut self);
    fn phy_slide(&mut self)
    where
        Self: WithBaseField<Base = CharacterBody2D>,
    {
        if !self.base().is_on_floor() {
            let v = self.get_velocity() + Vector2::DOWN;
            self.base_mut().set_velocity(v);
            self.base_mut().move_and_slide();

            let mut this = self.base_mut().clone();
            if let Some(collision) = this.get_last_slide_collision()
                && let obj = collision.get_collider()
                && let Some(c) = obj
                && c.get_class().to_string() == "TileMapLayer"
            {
                self.notify_on_floor();
            }
        } else {
            let v = self.get_velocity();
            self.base_mut().set_velocity(v);
            self.base_mut().move_and_slide();
        }
    }
}

/// Implement for nodes with no physics movement.
pub trait MoveableEntity: Moveable {
    /// Moves the entity to target position.
    /// Note: Do not provide a delta time calculation in your velocity as this internally calls
    /// velocity.
    fn node_slide(&mut self, use_physics_delta: bool)
    where
        Self: WithBaseField<Base: Inherits<Node2D>>,
    {
        let delta = if use_physics_delta {
            self.base()
                .upcast_ref::<Node2D>()
                .get_physics_process_delta_time()
        } else {
            self.base().upcast_ref::<Node2D>().get_process_delta_time()
        };
        let pos = self.base().upcast_ref::<Node2D>().get_global_position();
        let v = self.get_velocity();

        self.base_mut()
            .upcast_mut::<Node2D>()
            .set_global_position(pos + v * delta as f32);
    }
}

/// Used so both physics based nodes and non-physics based nodes can implement the
/// `enemy_state_machine_ext` trait, while implementing either `MoveableEntity` or `MoveableBody`
/// trait depending on their needs.
pub trait Move: Moveable {
    fn slide(&mut self);
}

pub trait MovementBehavior {
    fn compute_velocity(&self, cur_pos: Vector2, delta: f32) -> Vector2;
    fn set_speed(&mut self, speed: f32);
}

#[derive(GodotClass)]
#[class(init, base = Node2D)]
pub struct MoveLeft {
    #[export]
    speed: f32,
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for MoveLeft {
    fn process(&mut self, delta: f32) {
        let mut parent = self.base().get_node_as::<Node2D>("..");
        let cur_pos = parent.get_global_position();
        let new = self.compute_velocity(cur_pos, delta);
        parent.set_global_position(new);
    }
}

#[godot_dyn]
impl MovementBehavior for MoveLeft {
    fn compute_velocity(&self, cur_pos: Vector2, delta: f32) -> Vector2 {
        cur_pos + (Vector2::LEFT * self.speed) * delta
    }
    fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }
}

#[derive(GodotClass)]
#[class(init, base = Node2D)]
pub struct MoveRight {
    #[export]
    pub speed: f32,
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for MoveRight {
    fn process(&mut self, delta: f32) {
        let mut parent = self.base().get_node_as::<Node2D>("..");
        let cur_pos = parent.get_global_position();
        let new = self.compute_velocity(cur_pos, delta);
        parent.set_global_position(new);
    }
}

#[godot_dyn]
impl MovementBehavior for MoveRight {
    fn compute_velocity(&self, cur_pos: Vector2, delta: f32) -> Vector2 {
        cur_pos + Vector2::RIGHT * self.speed * delta
    }
    fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }
}

#[derive(GodotClass)]
#[class(init, base = Node2D)]
pub struct AlternatingMovement {
    /// The amount of time to spend moving. Changes direction on timeout.
    #[export]
    pub direction_timer: OnEditor<Gd<Timer>>,

    /// The amount of time to idle between direction changes.
    #[export]
    pub idle_timer: OnEditor<Gd<Timer>>,

    #[export]
    pub speed: f32,

    #[export]
    initial_direction: Direction,

    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for AlternatingMovement {
    fn ready(&mut self) {
        self.direction_timer
            .signals()
            .timeout()
            .connect_other(&self.to_gd(), Self::on_direction_timer_timeout);
        self.idle_timer
            .signals()
            .timeout()
            .connect_other(&self.to_gd(), Self::on_idle_timer_timeout);
    }

    fn process(&mut self, delta: f32) {
        if self.idle_timer.get_time_left() == 0.0 {
            let mut parent = self.base().get_node_as::<Node2D>("..");
            let cur_pos = parent.get_global_position();
            let new = self.compute_velocity(cur_pos, delta);
            parent.set_global_position(new);
        }
    }
}

#[godot_api]
impl AlternatingMovement {
    fn on_idle_timer_timeout(&mut self) {
        self.idle_timer.stop();
        self.direction_timer.start();
    }

    fn on_direction_timer_timeout(&mut self) {
        self.idle_timer.start();
        self.direction_timer.stop();
        match self.initial_direction {
            Direction::Right => self.initial_direction = Direction::Left,
            Direction::Left => self.initial_direction = Direction::Right,
        }
    }
}

#[godot_dyn]
impl MovementBehavior for AlternatingMovement {
    fn compute_velocity(&self, cur_pos: Vector2, delta: f32) -> Vector2 {
        match self.initial_direction {
            Direction::Right => cur_pos + Vector2::RIGHT * self.speed * delta,
            Direction::Left => cur_pos + Vector2::LEFT * self.speed * delta,
        }
    }
    fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }
}

/// Moves and collides a PhysicsBody2D, bouncing the body off of the collision normal.
pub fn move_bounce<T>(phys: &mut T, velocity: Vector2, speed: f32, delta: f32)
where
    T: WithBaseField<Base: Inherits<PhysicsBody2D>> + Moveable,
{
    let collision = phys
        .base_mut()
        .upcast_mut()
        .move_and_collide(velocity * delta * speed);

    if let Some(c) = collision {
        let reflect = c.get_remainder().bounce(c.get_normal());
        phys.set_velocity(velocity.bounce(c.get_normal()));
        phys.base_mut().upcast_mut().move_and_collide(reflect);
    }
}

/// Moves a Node2D inheriting node to the left.
pub fn move_left<T>(node: &mut T, speed: f32, delta: f32)
where
    T: WithBaseField<Base: Inherits<Node2D>>,
{
    let mut cur_pos = node.base().upcast_ref().get_global_position();
    cur_pos += Vector2::LEFT * speed * delta;
    node.base_mut().upcast_mut().set_global_position(cur_pos);
}

pub fn move_right<T>(node: &mut T, speed: f32, delta: f32)
where
    T: WithBaseField<Base: Inherits<Node2D>>,
{
    let mut cur_pos = node.base().upcast_ref().get_global_position();
    cur_pos += Vector2::RIGHT * speed * delta;
    node.base_mut().upcast_mut().set_global_position(cur_pos);
}

pub fn move_up<T>(node: &mut T, speed: f32, delta: f32)
where
    T: WithBaseField<Base: Inherits<Node2D>>,
{
    let mut cur_pos = node.base().upcast_ref().get_global_position();
    cur_pos += Vector2::UP * speed * delta;
    node.base_mut().upcast_mut().set_global_position(cur_pos);
}

pub fn move_down<T>(node: &mut T, speed: f32, delta: f32)
where
    T: WithBaseField<Base: Inherits<Node2D>>,
{
    let mut cur_pos = node.base().upcast_ref().get_global_position();
    cur_pos += Vector2::DOWN * speed * delta;
    node.base_mut().upcast_mut().set_global_position(cur_pos);
}

pub fn move_to<T>(node: &mut T, target: Vector2, speed: f32, delta: f32)
where
    T: WithBaseField<Base: Inherits<Node2D>>,
{
    let cur_pos = node.base().upcast_ref().get_global_position();
    node.base_mut()
        .upcast_mut()
        .set_global_position(cur_pos.move_toward(target, delta * speed));
}

/// Swaps the movement trait object of an entity to the given movement node.
pub fn swap_movement<T: MovementBehavior + Inherits<Node2D>>(
    entity: &mut Node2D,
    old_movement: &Gd<Node2D>,
    new_movement: Gd<T>,
    speed: f32,
) -> DynGd<Node2D, dyn MovementBehavior> {
    entity.remove_child(old_movement);
    entity.add_child(&new_movement.clone().upcast());
    let mut new = DynGd::<Node2D, dyn MovementBehavior>::from_godot(new_movement.upcast());
    new.dyn_bind_mut().set_speed(speed);
    new
}
