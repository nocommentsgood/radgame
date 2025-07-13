use fastnoise_lite::FastNoiseLite;
use godot::{
    classes::{Camera2D, ICamera2D},
    prelude::*,
};

// Credit to Squirrel GDC presentation.
// shake = trauma.sqrt() or traume.cube()
// angle = max_angle * shake * get_random_float_negone_to_one
// x_offset = max_offset * shake * PerlinNoise[-1, 1]
// y_offset = max_offset * shake * PerlinNoise[-1, 1]

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

#[derive(GodotClass)]
#[class(base = Camera2D, init)]
pub struct ShakyPlayerCamera {
    // TODO: Finalize and remove exports.
    #[export]
    decay: f32,
    #[export]
    #[init(val = Vector2::new(100.0, 75.0))]
    max_offset: Vector2,
    #[export]
    #[init(val = 20.0)]
    max_rot: f32,
    #[init(val = FastNoiseLite::new())]
    noise: FastNoiseLite,
    trauma: f32,
    noise_y: i32,

    pub set_right: bool,
    base: Base<Camera2D>,
}

#[godot_api]
impl ICamera2D for ShakyPlayerCamera {
    fn process(&mut self, delta: f32) {
        if self.trauma > 0.0 {
            self.trauma = f32::max(self.trauma - self.decay * delta, 0.0);
            self.rust_shake();
        }

        // Lerp towards the players last movement.
        let cur_pos = self.base().get_offset();
        let target = if self.set_right {
            Vector2::new(50.0, self.base().get_offset().y)
        } else {
            Vector2::new(-50.0, self.base().get_offset().y)
        };

        let vel = cur_pos.lerp(target, 2.0 * delta);
        self.base_mut().set_offset(vel);
    }

    fn ready(&mut self) {
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
impl ShakyPlayerCamera {
    pub fn add_trauma(&mut self, amount: TraumaLevel) {
        let level = match amount {
            TraumaLevel::Low => 0.25,
            TraumaLevel::Med => 0.5,
            TraumaLevel::High => 0.7,
        };
        self.trauma = (self.trauma + level).clamp(0.0, 1.0);
    }

    fn rust_shake(&mut self) {
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

        self.base_mut().set_offset(Vector2::new(offset_x, offset_y));
    }

    // Leaving this, in the event I decide to use Godot's FastNoiseLite implementation over the
    // Rust crate.
    #[allow(dead_code, unused_variables)]
    fn godot_shake(&mut self) {
        //     let amount = self.trauma.powf(2.0);
        //     let offset_x = self.amp
        //         * amount
        //         * self
        //             .noise
        //             .get_noise_2d(self.noise.get_seed() as f32, self.noise_y);
        //     let offset_y = self.amp
        //         * amount
        //         * self
        //             .noise
        //             .get_noise_2d(self.noise.get_seed() as f32 * 2.0, self.noise_y);
        //
        //     self.base_mut().set_offset(Vector2::new(offset_x, offset_y));
    }
}
