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
    #[init(val = 5000.0)]
    max_rot: f32,
    #[export]
    #[init(val = 0.5)]
    trauma_additive: f32,
    #[init(val = FastNoiseLite::new())]
    noise: FastNoiseLite,
    trauma: f32,
    noise_y: i32,
    base: Base<Camera2D>,
}

#[godot_api]
impl ICamera2D for ShakyPlayerCamera {
    fn physics_process(&mut self, delta: f32) {
        if self.trauma > 0.0 {
            self.trauma = f32::max(self.trauma - self.decay * delta, 0.0);
            self.rust_shake();
        }
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
    pub fn add_trauma(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).clamp(0.0, 1.0);
    }

    fn rust_shake(&mut self) {
        let amount = self.trauma.powf(2.0);
        self.noise_y = self.noise_y.wrapping_add(1);
        let rotation = self.max_rot
            * amount
            * self
                .noise
                .get_noise_2d(self.noise.seed as f32, self.noise_y as f32);

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

        self.base_mut().set_global_rotation(rotation);
        self.base_mut().set_offset(Vector2::new(offset_x, offset_y));
    }

    // Leaving this, in the event I decide to use Godot's FastNoiseLite implementation over the
    // Rust crate.
    #[allow(dead_code, unused_variables)]
    fn godot_shake(&mut self) {
        let amount = self.trauma.powf(2.0);
        // let offset_x = self.amp
        //     * amount
        //     * self
        //         .noise
        //         .get_noise_2d(self.noise.get_seed() as f32, self.noise_y);
        // let offset_y = self.amp
        //     * amount
        //     * self
        //         .noise
        //         .get_noise_2d(self.noise.get_seed() as f32 * 2.0, self.noise_y);
        //
        // self.base_mut().set_offset(Vector2::new(offset_x, offset_y));
    }
}
