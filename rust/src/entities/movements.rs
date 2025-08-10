use godot::{
    builtin::Vector2,
    classes::{CharacterBody2D, Node2D},
    obj::WithBaseField,
};

#[derive(Default, Debug, Clone)]
pub struct SpeedComponent {
    pub attack: f32,
    pub patrol: f32,
    pub aggro: f32,
}

impl SpeedComponent {
    pub fn new(attack: u32, patrol: u32, aggro: u32) -> Self {
        Self {
            attack: attack as f32,
            patrol: patrol as f32,
            aggro: aggro as f32,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum Direction {
    #[default]
    East,
    West,
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::East => write!(f, "east"),
            Direction::West => write!(f, "west"),
        }
    }
}
impl Direction {
    pub fn from_vel(velocity: &Vector2) -> Direction {
        if velocity.x < 0.0 {
            Direction::West
        } else {
            Direction::East
        }
    }
}

pub trait Moveable {
    fn get_velocity(&self) -> Vector2;
    fn set_velocity(&mut self, velocity: Vector2);
}

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
            if let Some(collision) = this.get_last_slide_collision() {
                let obj = collision.get_collider();
                if let Some(c) = obj {
                    if c.get_class().to_string() == "TileMapLayer" {
                        self.notify_on_floor();
                    }
                }
            }
        } else {
            let v = self.get_velocity();
            self.base_mut().set_velocity(v);
            self.base_mut().move_and_slide();
        }
    }
}

pub trait MoveableEntity: Moveable {
    /// Moves the entity to target position.
    /// Note: Do not provide a delta time calculation in your velocity as this internally calls
    /// velocity.
    fn node_slide(&mut self, use_physics_delta: bool)
    where
        Self: WithBaseField<Base = Node2D>,
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

pub trait Move: Moveable {
    fn slide(&mut self);
}
