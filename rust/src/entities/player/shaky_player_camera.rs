use fastnoise_lite::FastNoiseLite;
use godot::{
    classes::{
        Area2D, Camera2D, IArea2D, ICamera2D, RectangleShape2D,
        tween::{EaseType, TransitionType},
    },
    prelude::*,
};

use crate::world::environment_trigger::CameraData;

#[allow(dead_code)]
pub enum TraumaLevel {
    Low,
    Med,
    High,
}

impl From<u32> for TraumaLevel {
    fn from(value: u32) -> Self {
        match value {
            30..=u32::MAX => Self::High,
            10..=29 => Self::Med,
            _ => Self::Low,
        }
    }
}

// BUG: Offset doesn't seem to reset to original position after trauma is gone.
#[derive(GodotClass)]
#[class(base = Camera2D, init)]
pub struct PlayerCamera {
    // TODO: Finalize and remove exports.
    #[export]
    decay: f32,
    max_offset: Vector2,
    #[export]
    max_rot: f32,

    #[init(val = FastNoiseLite::new())]
    noise: FastNoiseLite,
    original_offset: Vector2,
    trauma: f32,
    noise_y: i32,
    set_right: Option<bool>,
    #[init(val = true)]
    pub attached: bool,
    base: Base<Camera2D>,
}

#[godot_api]
impl ICamera2D for PlayerCamera {
    fn physics_process(&mut self, delta: f32) {
        if self.trauma > 0.0 {
            self.trauma = f32::max(self.trauma - self.decay * delta, 0.0);
            self.shake();
        }

        // Lerp towards the players last movement.
        if let Some(b) = self.set_right {
            let cur_pos = self.base().get_offset();
            let target = if b {
                Vector2::new(50.0, self.base().get_offset().y)
            } else {
                Vector2::new(-50.0, self.base().get_offset().y)
            };

            let vel = cur_pos.lerp(target, 3.0 * delta);
            self.base_mut().set_offset(vel);
        }
    }

    fn ready(&mut self) {
        self.original_offset = self.base().get_offset();
        self.noise.seed = godot::global::randi() as i32;
        // TODO: Learn more about noise generation.
        // Use wrapping math operations to prevent overflow?
        // BUG: This causes subtraction overflow
        // self.noise.frequency = 4.0;
        self.noise.octaves = 2;
        self.noise
            .set_noise_type(Some(fastnoise_lite::NoiseType::Perlin));
    }
}

#[godot_api]
impl PlayerCamera {
    #[signal]
    pub fn request_detach(pos: Vector2);

    #[signal]
    pub fn request_attach();

    pub fn add_trauma(&mut self, amount: TraumaLevel) {
        let level = match amount {
            TraumaLevel::Low => 0.25,
            TraumaLevel::Med => 0.5,
            TraumaLevel::High => 0.7,
        };
        self.trauma = (self.trauma + level).clamp(0.0, 1.0);
    }

    fn shake(&mut self) {
        let amount = self.trauma.powf(2.0);
        self.noise_y = self.noise_y.wrapping_add(1);
        let offset_x = self.max_offset.x
            * amount
            * self
                .noise
                .get_noise_2d(self.noise.seed as f32 * 2.0, self.noise_y as f32);

        let offset_y = self.max_offset.y
            * amount
            * self
                .noise
                .get_noise_2d(self.noise.seed as f32 * 3.0, self.noise_y as f32);

        let o = self.original_offset;
        self.base_mut()
            .set_offset(o + Vector2::new(offset_x, offset_y));
    }

    pub fn set_right(&mut self, value: Option<bool>) {
        self.set_right = value;

        if let Some(b) = self.set_right {
            if b {
                self.max_offset = Vector2::new(55.0, -55.0);
            } else {
                self.max_offset = Vector2::new(-55.0, -55.0);
            }
        }
    }
}

#[derive(GodotConvert, Default, PartialEq, Var, Export)]
#[godot(via = i32)]
pub enum TweenTransition {
    #[default]
    Linear,
    Sine,
    Exponential,
    Elastic,
    Cubic,
    Bounce,
    Spring,
}

impl TweenTransition {
    pub fn to_type(&self) -> TransitionType {
        match self {
            TweenTransition::Linear => TransitionType::LINEAR,
            TweenTransition::Sine => TransitionType::SINE,
            TweenTransition::Exponential => TransitionType::EXPO,
            TweenTransition::Elastic => TransitionType::ELASTIC,
            TweenTransition::Cubic => TransitionType::CUBIC,
            TweenTransition::Bounce => TransitionType::BOUNCE,
            TweenTransition::Spring => TransitionType::SPRING,
        }
    }
}

#[derive(GodotConvert, Default, PartialEq, Var, Export)]
#[godot(via = i32)]
pub enum TweenEase {
    #[default]
    In,
    Out,
    InOut,
    OutIn,
}

impl TweenEase {
    pub fn to_type(&self) -> EaseType {
        match self {
            TweenEase::In => EaseType::IN,
            TweenEase::Out => EaseType::OUT,
            TweenEase::InOut => EaseType::IN_OUT,
            TweenEase::OutIn => EaseType::OUT_IN,
        }
    }
}

#[derive(GodotConvert, Default, PartialEq, Var, Export)]
#[godot(via = i32)]
pub enum CameraFollowType {
    Simple,
    Frame,
    #[default]
    None,
}
